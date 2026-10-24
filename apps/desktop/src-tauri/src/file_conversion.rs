//! Desktop adapter for the stream-aware converter.
use super::*;
use apocalipse_core::file_conversion::{self as converter, Inspection, Tools};

struct Session {
    id: String,
    cancel: Arc<AtomicBool>,
    info: Option<Inspection>,
    fingerprint: Option<(u64, SystemTime)>,
    running: bool,
}
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

fn tools(app: &tauri::AppHandle) -> Result<Tools, String> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    let ffmpeg = configured_tool(
        &settings.ffmpeg_path,
        if cfg!(windows) {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        },
    );
    let name = if cfg!(windows) {
        "ffprobe.exe"
    } else {
        "ffprobe"
    };
    let sibling = ffmpeg.parent().unwrap_or(Path::new(".")).join(name);
    let ffprobe = if sibling.is_file() {
        sibling
    } else {
        PathBuf::from(name)
    };
    Ok(Tools { ffmpeg, ffprobe })
}
fn fingerprint(path: &Path) -> Result<(u64, SystemTime), String> {
    let meta = fs::metadata(path).map_err(|_| "conversion_source_missing")?;
    Ok((meta.len(), meta.modified().map_err(|e| e.to_string())?))
}
#[tauri::command]
pub(super) async fn inspect_conversion_file(
    app: tauri::AppHandle,
    id: String,
    source: String,
) -> Result<Inspection, String> {
    uuid::Uuid::parse_str(&id).map_err(|_| "conversion_invalid_session")?;
    let tools = tools(&app)?;
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut slot = SESSION.lock().map_err(|e| e.to_string())?;
        if slot.as_ref().is_some_and(|s| s.running) {
            return Err("conversion_busy".into());
        }
        *slot = Some(Session {
            id: id.clone(),
            cancel: cancel.clone(),
            info: None,
            fingerprint: None,
            running: true,
        });
    }
    let initial = fingerprint(Path::new(&source));
    let result = match &initial {
        Ok(_) => converter::inspect(&tools, Path::new(&source), cancel)
            .await
            .map_err(|e| e.to_string()),
        Err(error) => Err(error.clone()),
    };
    let result = result.and_then(|info| {
        if initial.ok() != fingerprint(&info.source).ok() {
            return Err("conversion_source_changed".into());
        }
        Ok(info)
    });
    let mut slot = SESSION.lock().map_err(|e| e.to_string())?;
    if let Some(session) = slot.as_mut().filter(|s| s.id == id) {
        session.running = false;
        if let Ok(info) = &result {
            session.fingerprint = fingerprint(&info.source).ok();
            session.info = Some(info.clone());
        }
    }
    result
}
#[tauri::command]
pub(super) async fn convert_local_file(
    app: tauri::AppHandle,
    id: String,
    format: String,
    audio_track: Option<u32>,
    directory: String,
) -> Result<String, String> {
    let tools = tools(&app)?;
    let (info, cancel) = {
        let mut slot = SESSION.lock().map_err(|e| e.to_string())?;
        let s = slot
            .as_mut()
            .filter(|s| s.id == id)
            .ok_or("conversion_inspect_first")?;
        if s.running {
            return Err("conversion_busy".into());
        }
        let info = s.info.clone().ok_or("conversion_inspect_first")?;
        if s.fingerprint != fingerprint(&info.source).ok() {
            return Err("conversion_source_changed".into());
        }
        s.cancel.store(false, Ordering::Relaxed);
        s.running = true;
        (info, s.cancel.clone())
    };
    let result = converter::convert(
        &tools,
        &info,
        &format,
        audio_track,
        Path::new(&directory),
        cancel,
        |percent| {
            let _ = app.emit(
                "file-conversion-progress",
                serde_json::json!({"id": id, "percent": percent}),
            );
        },
    )
    .await
    .map_err(|e| e.to_string());
    if let Ok(mut slot) = SESSION.lock() {
        if let Some(s) = slot.as_mut().filter(|s| s.id == id) {
            s.running = false;
        }
    }
    result.map(|path| path.to_string_lossy().into_owned())
}
#[tauri::command]
pub(super) fn cancel_file_conversion(id: String) {
    if let Ok(slot) = SESSION.lock() {
        if let Some(s) = slot.as_ref().filter(|s| s.id == id) {
            s.cancel.store(true, Ordering::Relaxed);
        }
    }
}
#[tauri::command]
pub(super) fn pick_conversion_file() -> Option<String> {
    rfd::FileDialog::new()
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
}
#[tauri::command]
pub(super) fn reveal_converted_file(path: String) -> Result<(), String> {
    let slot = SESSION.lock().map_err(|e| e.to_string())?;
    if slot.as_ref().is_some_and(|s| s.running) {
        return Err("conversion_busy".into());
    }
    let path = PathBuf::from(path);
    if !path.is_file() {
        return Err("conversion_source_missing".into());
    }
    let directory = path.parent().ok_or("conversion_directory_missing")?;
    #[cfg(windows)]
    let result = Command::new("explorer.exe").arg(directory).spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(directory).spawn();
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(directory).spawn();
    result.map(|_| ()).map_err(|e| e.to_string())
}
