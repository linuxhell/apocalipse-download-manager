//! Native CLI contracts. These programs are not yt-dlp/FFmpeg command-line replacements.
use super::{AppState, MediaFormat, MediaInspection};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tauri::State;

#[path = "nibiru_adapter.rs"]
mod adapter;
pub use adapter::MediaOperation;

pub fn executable(settings: &super::UserSettings, engine: &str) -> Result<PathBuf, String> {
    if engine == "nibiru-xtreme" {
        return Ok(super::configured_xtreme(settings));
    }
    let configured = match engine {
        "nibiru-power" => &settings.power_path,
        "nibiru-colapse" => &settings.colapse_path,
        "nibiru-m3u8" => &settings.m3u8_path,
        _ => return Err("unknown_nibiru_engine".into()),
    };
    if let Some(path) = configured.as_ref().filter(|path| !path.as_os_str().is_empty()) {
        return Ok(path.clone());
    }
    let name = format!("{engine}{}", if cfg!(windows) { ".exe" } else { "" });
    Ok(super::portable_tools_directory()?.join(engine).join(name))
}

pub use super::nibiru_cli::{capabilities, http_url};

pub async fn extract(path: &Path, source: &str) -> Result<MediaInspection, String> {
    let inspection = adapter::extract(path, source, super::suggested_name).await?;
    Ok(MediaInspection {
        title: inspection.title,
        thumbnail: inspection.thumbnail,
        duration: inspection.duration,
        is_live: inspection.is_live,
        suggested_file_name: inspection.suggested_file_name,
        formats: inspection
            .formats
            .into_iter()
            .map(|format| MediaFormat {
                selection: format.selection,
                label: format.label,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn get_nibiru_capabilities(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let mut result = serde_json::Map::new();
    for engine in [
        "nibiru-xtreme",
        "nibiru-power",
        "nibiru-colapse",
        "nibiru-m3u8",
    ] {
        let path = executable(&settings, engine)?;
        result.insert(
            engine.into(),
            match capabilities(&path, engine).await {
                Ok(value) => json!({"available":true,"capabilities":value}),
                Err(error) => json!({"available":false,"error":error}),
            },
        );
    }
    Ok(Value::Object(result))
}

#[tauri::command]
pub async fn process_nibiru_media(
    state: State<'_, AppState>,
    request: MediaOperation,
) -> Result<Value, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?.clone();
    let path = executable(&settings, "nibiru-colapse")?;
    adapter::process_media(&path, &request).await
}
