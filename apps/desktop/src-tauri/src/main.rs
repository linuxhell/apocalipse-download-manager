#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apocalipse_core::{
    classify_url, plan_download, Capabilities, DownloadEngine, DownloadEvent, DownloadId,
    DownloadKind, DownloadRequest, DownloadState, DownloadTask,
};
use serde::Serialize;
use std::{fs, path::{Path, PathBuf}, sync::Mutex};
use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, State};
use tokio::sync::mpsc;

struct AppState {
    queue: Mutex<Vec<DownloadTask>>,
    queue_path: PathBuf,
}

#[derive(Serialize)]
struct PlanResponse {
    primary: String,
    fallbacks: Vec<String>,
    reason: String,
}

#[tauri::command]
fn inspect_url(url: String) -> Result<PlanResponse, String> {
    let capabilities = Capabilities {
        aria2: true,
        yt_dlp: true,
        n_m3u8dl_re: true,
        torrent: true,
        amule: false,
    };
    let plan = plan_download(&url, capabilities).ok_or_else(|| "unsupported_url".to_owned())?;
    Ok(PlanResponse {
        primary: format!("{:?}", plan.primary),
        fallbacks: plan.fallbacks.iter().map(|engine| format!("{engine:?}")).collect(),
        reason: plan.reason.to_owned(),
    })
}

fn load_queue(path: &Path) -> Vec<DownloadTask> {
    fs::read(path).ok().and_then(|data| serde_json::from_slice(&data).ok()).unwrap_or_default()
}

fn save_queue(state: &AppState, queue: &[DownloadTask]) -> Result<(), String> {
    if let Some(parent) = state.queue_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let data = serde_json::to_vec_pretty(queue).map_err(|error| error.to_string())?;
    fs::write(&state.queue_path, data).map_err(|error| error.to_string())
}

fn update_task(app: &tauri::AppHandle, id: DownloadId, persist: bool, update: impl FnOnce(&mut DownloadTask)) {
    let state = app.state::<AppState>();
    if let Ok(mut queue) = state.queue.lock() {
        if let Some(task) = queue.iter_mut().find(|task| task.id == id) {
            update(task);
            if persist {
                let _ = save_queue(&state, &queue);
            }
        }
    }
}

async fn run_download(app: tauri::AppHandle, id: DownloadId, request: DownloadRequest) {
    update_task(&app, id, true, |task| task.state = DownloadState::Inspecting);
    let engine = match DownloadEngine::new() {
        Ok(engine) => engine,
        Err(error) => {
            update_task(&app, id, true, |task| task.state = DownloadState::Failed { message: error.to_string() });
            return;
        }
    };
    let (events, mut receiver) = mpsc::channel(64);
    let mut download = Box::pin(engine.download(request, events));
    loop {
        tokio::select! {
            result = &mut download => {
                match result {
                    Ok(()) => update_task(&app, id, true, |task| task.state = DownloadState::Completed),
                    Err(error) => update_task(&app, id, true, |task| task.state = DownloadState::Failed { message: error.to_string() }),
                }
                break;
            }
            event = receiver.recv() => match event {
                Some(DownloadEvent::Started { resumed_at, total }) => update_task(&app, id, true, |task| {
                    task.state = DownloadState::Downloading;
                    task.received = resumed_at;
                    task.total = total;
                }),
                Some(DownloadEvent::Progress { received, total }) => update_task(&app, id, false, |task| {
                    task.received = received;
                    task.total = total;
                }),
                Some(DownloadEvent::Completed { bytes }) => update_task(&app, id, true, |task| {
                    task.received = bytes;
                    task.total = Some(bytes);
                    task.state = DownloadState::Completed;
                }),
                None => break,
            }
        }
    }
}

fn suggested_name(source: &str) -> String {
    source.split(['/', '\\']).next_back().and_then(|part| part.split(['?', '#']).next())
        .filter(|part| !part.is_empty()).unwrap_or("download").chars()
        .map(|character| if "<>:\"/\\|?*".contains(character) { '_' } else { character }).collect()
}

#[tauri::command]
fn list_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadTask>, String> {
    state.queue.lock().map(|queue| queue.clone()).map_err(|error| error.to_string())
}

#[tauri::command]
fn enqueue_download(app: tauri::AppHandle, state: State<'_, AppState>, url: String) -> Result<DownloadTask, String> {
    inspect_url(url.clone())?;
    if classify_url(&url) != Some(DownloadKind::Http) {
        return Err("selected_engine_not_implemented".to_owned());
    }
    let download_dir = app.path().download_dir().map_err(|error| error.to_string())?;
    let task = DownloadTask::new(&url, download_dir.join(suggested_name(&url)));
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.push(task.clone());
    save_queue(&state, &queue)?;
    drop(queue);
    let request = DownloadRequest { url, destination: task.destination.clone(), overwrite: false };
    let task_id = task.id;
    tauri::async_runtime::spawn(run_download(app, task_id, request));
    Ok(task)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let queue_path = app.path().app_data_dir()?.join("queue.json");
            app.manage(AppState { queue: Mutex::new(load_queue(&queue_path)), queue_path });
            let show = MenuItem::with_id(app, "show", "Show Apocalipse", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let icon = app.default_window_icon().cloned().expect("application icon is required");
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Apocalipse Download Manager")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![inspect_url, list_downloads, enqueue_download])
        .run(tauri::generate_context!())
        .expect("failed to run Apocalipse Download Manager");
}
