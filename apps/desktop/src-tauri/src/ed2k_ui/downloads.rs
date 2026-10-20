use super::{save_file, Ed2kState};
use nibiru_xtreme::ed2k::{self, FileLink, TransferStage};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Manager, State};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Download {
    pub id: String,
    pub file: FileLink,
    pub directory: PathBuf,
    pub status: String,
    pub verified: u64,
    pub peer: Option<String>,
    pub queue_rank: Option<u32>,
    pub error: Option<String>,
}
struct Inner {
    path: PathBuf,
    tasks: Mutex<BTreeMap<String, Download>>,
    active: Mutex<BTreeMap<String, Arc<AtomicBool>>>,
    slots: Arc<tokio::sync::Semaphore>,
}
#[derive(Clone)]
pub struct Downloads(Arc<Inner>);
impl Downloads {
    pub fn empty(path: PathBuf) -> Self {
        Self(Arc::new(Inner {
            path,
            tasks: Mutex::new(BTreeMap::new()),
            active: Mutex::new(BTreeMap::new()),
            slots: Arc::new(tokio::sync::Semaphore::new(2)),
        }))
    }
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let queue = Self::empty(path);
        match std::fs::metadata(&queue.0.path) {
            Ok(metadata) => {
                if metadata.len() > 1024 * 1024 {
                    return Err("ed2k_download_queue_too_large".into());
                }
                let bytes = std::fs::read(&queue.0.path).map_err(|e| e.to_string())?;
                let tasks: Vec<Download> =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                if tasks.len() > 200 {
                    return Err("ed2k_download_queue_full".into());
                }
                let mut stored = queue.0.tasks.lock().map_err(|e| e.to_string())?;
                for mut task in tasks {
                    uuid::Uuid::parse_str(&task.id).map_err(|e| e.to_string())?;
                    if FileLink::parse(&task.file.link()).map_err(|e| e.to_string())? != task.file
                        || !task.directory.is_absolute()
                        || task.verified > task.file.size
                    {
                        return Err("invalid_ed2k_download_record".into());
                    }
                    if task.status != "completed" {
                        task.status = "paused".into();
                    }
                    task.peer = None;
                    task.queue_rank = None;
                    if stored.insert(task.id.clone(), task).is_some() {
                        return Err("duplicate_ed2k_task".into());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        Ok(queue)
    }
    pub fn list(&self) -> Result<Vec<Download>, String> {
        Ok(self
            .0
            .tasks
            .lock()
            .map_err(|e| e.to_string())?
            .values()
            .cloned()
            .collect())
    }
    fn persist(&self, tasks: &BTreeMap<String, Download>) -> Result<(), String> {
        let records: Vec<_> = tasks.values().collect();
        save_file(
            &self.0.path,
            &serde_json::to_vec(&records).map_err(|e| e.to_string())?,
        )
    }
    fn update(&self, id: &str, edit: impl FnOnce(&mut Download)) -> Result<(), String> {
        let mut tasks = self.0.tasks.lock().map_err(|e| e.to_string())?;
        let task = tasks.get_mut(id).ok_or("ed2k_task_not_found")?;
        edit(task);
        self.persist(&tasks)
    }
    fn start(&self, app: tauri::AppHandle, id: String) -> Result<(), String> {
        let mut active = self.0.active.lock().map_err(|e| e.to_string())?;
        if active.contains_key(&id) {
            return Err("ed2k_task_still_running".into());
        }
        let task = self
            .0
            .tasks
            .lock()
            .map_err(|e| e.to_string())?
            .get(&id)
            .cloned()
            .ok_or("ed2k_task_not_found")?;
        if task.status == "completed" {
            return Err("ed2k_task_already_completed".into());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        self.update(&id, |task| {
            task.status = "queued".into();
            task.error = None;
            task.peer = None;
            task.queue_rank = None;
        })?;
        active.insert(id.clone(), cancel.clone());
        drop(active);
        let queue = self.clone();
        tauri::async_runtime::spawn(async move {
            let result:Result<(),String>=async {
                let _permit=tokio::select! {
                    result=queue.0.slots.clone().acquire_owned()=>result.map_err(|e|e.to_string())?,
                    _=async{while !cancel.load(Ordering::Acquire){tokio::time::sleep(std::time::Duration::from_millis(50)).await;}}=>return Err("cancelled".into()),
                };
                if cancel.load(Ordering::Acquire){return Err("cancelled".into());}
                let state=app.state::<Ed2kState>();
                let connection=state.server.read().await.as_ref().cloned().ok_or("ed2k_not_connected")?;
                queue.update(&id,|task|task.status="connecting".into())?;
                let hash=task.file.hash_hex();
                let mut sources=None;
                let mut error="ed2k_no_sources".to_owned();
                for attempt in 0..3 {
                    if cancel.load(Ordering::Acquire){return Err("cancelled".into());}
                    let found = tokio::select! {
                        result=connection.sources(&hash)=>result,
                        _=async{while !cancel.load(Ordering::Acquire){tokio::time::sleep(std::time::Duration::from_millis(50)).await;}}=>return Err("cancelled".into()),
                    };
                    match found {
                        Ok(found)=>{sources=Some(found);break;},
                        Err(reason)=>error=reason.to_string(),
                    }
                    if attempt<2 {tokio::time::sleep(std::time::Duration::from_secs(1)).await;}
                }
                let sources=sources.ok_or(error)?;
                tokio::fs::create_dir_all(&task.directory).await.map_err(|e|e.to_string())?;
                let callbacks=if connection.is_low_id(){None}else{state.incoming.lock().await.as_ref().map(|listener|ed2k::CallbackClient::new(connection.clone(),listener.callbacks()))};
                let options=ed2k::TransferOptions{identity:state.user_hash,client_id:connection.client_id,callbacks};
                ed2k::download_with_options(&task.file,&task.directory,&sources,options,cancel.clone(),|progress|{
                    let _=queue.update(&id,|task|{
                        task.status=match progress.stage{TransferStage::Connecting=>"connecting",TransferStage::Queued=>"waiting_peer",TransferStage::Checking=>"checking",TransferStage::Downloading=>"downloading",TransferStage::Completed=>"completed"}.into();
                        task.verified=progress.verified;task.peer=progress.peer;task.queue_rank=progress.queue_rank;
                    });
                }).await.map_err(|e|format!("{e:#}"))?;
                Ok(())
            }.await;
            let _ = queue.update(&id, |task| {
                task.peer = None;
                task.queue_rank = None;
                match result {
                    Ok(()) => {
                        task.status = "completed".into();
                        task.verified = task.file.size;
                        task.error = None;
                    }
                    Err(_) if cancel.load(Ordering::Acquire) => {
                        task.status = "paused".into();
                        task.error = None;
                    }
                    Err(error) => {
                        task.status = "error".into();
                        task.error = Some(error);
                    }
                }
            });
            if let Ok(mut active) = queue.0.active.lock() {
                active.remove(&id);
            }
        });
        Ok(())
    }
}

#[tauri::command]
pub async fn ed2k_add_download(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    link: String,
) -> Result<Option<String>, String> {
    state.ready()?;
    let file = FileLink::parse(link.trim()).map_err(|e| e.to_string())?;
    // Use the directory already chosen by the user in ADM preferences.
    let directory =
        super::super::configured_download_directory(&app, &app.state::<super::super::AppState>())?;
    let id = uuid::Uuid::new_v4().to_string();
    {
        let mut tasks = state.downloads.0.tasks.lock().map_err(|e| e.to_string())?;
        if tasks.len() >= 200 {
            return Err("ed2k_download_queue_full".into());
        }
        if tasks
            .values()
            .any(|task| task.file.hash == file.hash && task.directory == directory)
        {
            return Err("ed2k_download_already_added".into());
        }
        if directory.join(&file.name).exists() {
            return Err("destination_already_exists".into());
        }
        tasks.insert(
            id.clone(),
            Download {
                id: id.clone(),
                file,
                directory,
                status: "paused".into(),
                verified: 0,
                peer: None,
                queue_rank: None,
                error: None,
            },
        );
        if let Err(error) = state.downloads.persist(&tasks) {
            tasks.remove(&id);
            return Err(error);
        }
    }
    // The record is durable before network work starts. If disconnected, retain
    // a paused record so users can connect and resume without losing the link.
    if state
        .server
        .read()
        .await
        .as_ref()
        .is_some_and(|s| s.is_connected())
    {
        state.downloads.start(app, id.clone())?;
    }
    Ok(Some(id))
}
#[tauri::command]
pub fn ed2k_download_action(
    app: tauri::AppHandle,
    state: State<'_, Ed2kState>,
    id: String,
    action: String,
) -> Result<(), String> {
    state.ready()?;
    match action.as_str() {
        "resume" => state.downloads.start(app, id),
        "pause" => {
            let active = state.downloads.0.active.lock().map_err(|e| e.to_string())?;
            if let Some(cancel) = active.get(&id) {
                cancel.store(true, Ordering::Release);
                state
                    .downloads
                    .update(&id, |task| task.status = "pausing".into())
            } else {
                state.downloads.update(&id, |task| {
                    if task.status != "completed" {
                        task.status = "paused".into();
                    }
                })
            }
        }
        "remove" => {
            let active = state.downloads.0.active.lock().map_err(|e| e.to_string())?;
            if active.contains_key(&id) {
                return Err("pause_ed2k_task_before_removing".into());
            }
            let mut tasks = state.downloads.0.tasks.lock().map_err(|e| e.to_string())?;
            let task = tasks.remove(&id).ok_or("ed2k_task_not_found")?;
            if let Err(error) = state.downloads.persist(&tasks) {
                tasks.insert(id, task);
                return Err(error);
            }
            Ok(())
        }
        _ => Err("invalid_ed2k_download_action".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_restores_unfinished_tasks_paused_and_keeps_verified_counts() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("downloads.json");
        let queue = Downloads::empty(path.clone());
        let id = uuid::Uuid::new_v4().to_string();
        let file = FileLink::parse("ed2k://|file|test.bin|123|0123456789abcdef0123456789abcdef|/")
            .unwrap();
        let task = Download {
            id: id.clone(),
            file,
            directory: root.path().to_owned(),
            status: "downloading".into(),
            verified: 12,
            peer: Some("1.2.3.4".into()),
            queue_rank: None,
            error: None,
        };
        {
            let mut tasks = queue.0.tasks.lock().unwrap();
            tasks.insert(id.clone(), task);
            queue.persist(&tasks).unwrap();
        }
        let loaded = Downloads::load(path).unwrap();
        let tasks = loaded.list().unwrap();
        assert_eq!(tasks[0].status, "paused");
        assert_eq!(tasks[0].verified, 12);
        assert!(tasks[0].peer.is_none());
    }
}
