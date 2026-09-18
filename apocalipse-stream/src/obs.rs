use crate::config::Settings;
use anyhow::{anyhow, Context, Result};
use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{net::TcpStream, path::Path, process::{Child, Command}};
use tungstenite::{connect, stream::MaybeTlsStream, Message, WebSocket};

type Ws = WebSocket<MaybeTlsStream<TcpStream>>;

pub fn launch_hidden(path: &str) -> Result<Child> {
    let exe = Path::new(path);
    if !exe.is_file() {
        return Err(anyhow!("obs64.exe não encontrado"));
    }
    let mut cmd = Command::new(exe);
    cmd.args(["--portable", "--minimize-to-tray", "--disable-shutdown-check"]);
    if let Some(dir) = exe.parent() {
        cmd.current_dir(dir);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    Ok(cmd.spawn().context("não foi possível iniciar o OBS")?)
}

fn read_json(ws: &mut Ws) -> Result<Value> {
    loop {
        let msg = ws.read()?;
        if let Message::Text(text) = msg {
            return Ok(serde_json::from_str(text.as_ref())?);
        }
    }
}

pub fn connect_obs(cfg: &Settings) -> Result<Ws> {
    let url = format!("ws://127.0.0.1:{}", cfg.ws_port);
    let (mut ws, _) = connect(url.as_str()).context("obs-websocket não respondeu")?;
    let hello = read_json(&mut ws)?;
    let d = &hello["d"];
    let rpc = d["rpcVersion"].as_u64().unwrap_or(1);
    let mut identify = json!({"rpcVersion": rpc, "eventSubscriptions": 0});

    if let Some(auth) = d.get("authentication") {
        let challenge = auth["challenge"].as_str().unwrap_or("");
        let salt = auth["salt"].as_str().unwrap_or("");
        let secret = base64::engine::general_purpose::STANDARD
            .encode(Sha256::digest(format!("{}{}", cfg.ws_password, salt).as_bytes()));
        let response = base64::engine::general_purpose::STANDARD
            .encode(Sha256::digest(format!("{}{}", secret, challenge).as_bytes()));
        identify["authentication"] = Value::String(response);
    }

    ws.send(Message::Text(
        json!({"op": 1, "d": identify}).to_string().into(),
    ))?;

    let identified = read_json(&mut ws)?;
    if identified["op"].as_u64() != Some(2) {
        return Err(anyhow!("OBS recusou a autenticação WebSocket"));
    }
    Ok(ws)
}

pub fn request(ws: &mut Ws, request_type: &str, data: Value) -> Result<Value> {
    let request_id = format!(
        "apoc-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    );
    ws.send(Message::Text(
        json!({
            "op": 6,
            "d": {
                "requestType": request_type,
                "requestId": request_id,
                "requestData": data
            }
        })
        .to_string()
        .into(),
    ))?;

    loop {
        let value = read_json(ws)?;
        if value["op"].as_u64() == Some(7)
            && value["d"]["requestId"].as_str() == Some(request_id.as_str())
        {
            let ok = value["d"]["requestStatus"]["result"]
                .as_bool()
                .unwrap_or(false);
            if !ok {
                let comment = value["d"]["requestStatus"]["comment"]
                    .as_str()
                    .unwrap_or("falha no OBS");
                return Err(anyhow!(comment.to_string()));
            }
            return Ok(value["d"]["responseData"].clone());
        }
    }
}

pub fn test(cfg: &Settings) -> Result<String> {
    let mut ws = connect_obs(cfg)?;
    let data = request(&mut ws, "GetVersion", json!({}))?;
    Ok(format!(
        "OBS conectado: {}",
        data.get("obsVersion")
            .and_then(Value::as_str)
            .unwrap_or("OK")
    ))
}

pub fn set_video(cfg: &Settings) -> Result<String> {
    let (w, h) = cfg.preset.dimensions();
    let mut ws = connect_obs(cfg)?;
    request(
        &mut ws,
        "SetVideoSettings",
        json!({
            "fpsNumerator": cfg.fps,
            "fpsDenominator": 1,
            "baseWidth": w,
            "baseHeight": h,
            "outputWidth": w,
            "outputHeight": h
        }),
    )?;
    Ok(format!("Vídeo configurado: {}x{} @ {} FPS", w, h, cfg.fps))
}

pub fn start_record(cfg: &Settings) -> Result<String> {
    std::fs::create_dir_all(&cfg.record_dir).ok();
    let mut ws = connect_obs(cfg)?;
    let _ = request(
        &mut ws,
        "SetRecordDirectory",
        json!({"recordDirectory": cfg.record_dir}),
    );
    request(&mut ws, "StartRecord", json!({}))?;
    Ok("GRAVANDO".into())
}

pub fn stop_record(cfg: &Settings) -> Result<String> {
    let mut ws = connect_obs(cfg)?;
    let data = request(&mut ws, "StopRecord", json!({}))?;
    Ok(data
        .get("outputPath")
        .and_then(Value::as_str)
        .map(|p| format!("Gravação finalizada: {p}"))
        .unwrap_or_else(|| "Gravação finalizada.".into()))
}

pub fn start_stream(cfg: &Settings) -> Result<String> {
    let mut ws = connect_obs(cfg)?;
    if !cfg.stream_key.trim().is_empty() {
        request(
            &mut ws,
            "SetStreamServiceSettings",
            json!({
                "streamServiceType": "rtmp_custom",
                "streamServiceSettings": {
                    "server": cfg.stream_server,
                    "key": cfg.stream_key,
                    "use_auth": false
                }
            }),
        )?;
    }
    request(&mut ws, "StartStream", json!({}))?;
    Ok("AO VIVO".into())
}

pub fn stop_stream(cfg: &Settings) -> Result<String> {
    let mut ws = connect_obs(cfg)?;
    request(&mut ws, "StopStream", json!({}))?;
    Ok("Transmissão encerrada.".into())
}

pub fn add_overlay(cfg: &Settings) -> Result<String> {
    let mut ws = connect_obs(cfg)?;
    let scene = request(&mut ws, "GetCurrentProgramScene", json!({}))?
        .get("currentProgramSceneName")
        .and_then(Value::as_str)
        .unwrap_or("Scene")
        .to_string();

    let result = request(
        &mut ws,
        "CreateInput",
        json!({
            "sceneName": scene,
            "inputName": "Apocalipse Stream Overlay",
            "inputKind": "browser_source",
            "inputSettings": {
                "url": "http://127.0.0.1:8765/overlay",
                "width": 1920,
                "height": 1080,
                "fps": 60
            },
            "sceneItemEnabled": true
        }),
    );

    match result {
        Ok(_) => Ok("Overlay adicionado à cena atual.".into()),
        Err(e) if e.to_string().to_lowercase().contains("already") => {
            Ok("Overlay já existe na cena.".into())
        }
        Err(e) => Err(e),
    }
}

pub fn shutdown_outputs(cfg: &Settings) {
    if let Ok(mut ws) = connect_obs(cfg) {
        if let Ok(v) = request(&mut ws, "GetStreamStatus", json!({})) {
            if v.get("outputActive").and_then(Value::as_bool).unwrap_or(false) {
                let _ = request(&mut ws, "StopStream", json!({}));
            }
        }
        if let Ok(v) = request(&mut ws, "GetRecordStatus", json!({})) {
            if v.get("outputActive").and_then(Value::as_bool).unwrap_or(false) {
                let _ = request(&mut ws, "StopRecord", json!({}));
            }
        }
    }
}
