#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use apocalipse_core::{plan_download, Capabilities, DownloadTask};
use serde::Serialize;
use std::{fs, path::{Path, PathBuf}, sync::Mutex};
use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder, Manager, State};

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
    let download_dir = app.path().download_dir().map_err(|error| error.to_string())?;
    let task = DownloadTask::new(&url, download_dir.join(suggested_name(&url)));
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
    queue.push(task.clone());
    save_queue(&state, &queue)?;
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
