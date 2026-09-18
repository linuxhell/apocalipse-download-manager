use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::{Header, Response, Server};

#[derive(Clone)]
pub struct OverlayState {
    pub presenter_name: String,
    pub author: String,
    pub message: String,
    pub show_message: bool,
    pub theme: String,
    pub viewers: u64,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            presenter_name: "Apocalipse Stream".into(),
            author: "Rafael Souza".into(),
            message: "Mensagem destacada".into(),
            show_message: false,
            theme: "Eclipse".into(),
            viewers: 0,
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render(o: &OverlayState) -> String {
    let accent = match o.theme.as_str() {
        "Abismo" => "#813dff",
        "Obsidiana" => "#c8c8c8",
        "Cavaleiro Vermelho" => "#ff2525",
        "Minimal Apocalypse" => "#ffffff",
        "Profecia" => "#ff4a24",
        _ => "#ff731e",
    };

    let body = if o.show_message {
        format!(
            "<div class='card'><b>{}</b><div>{}</div></div>",
            esc(&o.author),
            esc(&o.message)
        )
    } else {
        format!(
            "<div class='card name'>◉ {}</div>",
            esc(&o.presenter_name)
        )
    };

    format!(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta http-equiv="refresh" content="0.35">
<style>
html,body{{margin:0;width:100%;height:100%;background:transparent;overflow:hidden;font-family:Segoe UI,Arial,sans-serif}}
.wrap{{position:absolute;left:44px;bottom:44px;max-width:760px}}
.card{{color:#fff;background:linear-gradient(100deg,rgba(8,8,9,.96),rgba(30,10,8,.90));border-left:6px solid {accent};border-radius:10px;padding:18px 24px;box-shadow:0 0 28px rgba(255,50,20,.22);font-size:28px}}
.card b{{display:block;color:{accent};margin-bottom:6px}}
.name{{font-weight:800}}
.viewer{{position:absolute;right:30px;top:24px;color:#eee;background:rgba(0,0,0,.55);padding:8px 12px;border-radius:8px;font-size:18px}}
</style>
</head>
<body>
<div class="viewer">👁 {viewers}</div>
<div class="wrap">{body}</div>
</body>
</html>"#,
        accent = accent,
        viewers = o.viewers,
        body = body
    )
}

pub fn start_server(state: Arc<Mutex<OverlayState>>) {
    thread::spawn(move || {
        let Ok(server) = Server::http("127.0.0.1:8765") else {
            return;
        };
        for req in server.incoming_requests() {
            if req.url() == "/health" {
                let _ = req.respond(Response::from_string("ok"));
                continue;
            }
            let html = state.lock().map(|o| render(&o)).unwrap_or_default();
            let mut resp = Response::from_string(html);
            if let Ok(header) = Header::from_bytes(
                &b"Content-Type"[..],
                &b"text/html; charset=utf-8"[..],
            ) {
                resp = resp.with_header(header);
            }
            let _ = req.respond(resp);
        }
    });
}
