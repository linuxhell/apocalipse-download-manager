use super::*;
use std::sync::Mutex;
use tokio::task::JoinHandle;
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
fn request(url: &str) -> MediaPreviewRequest {
    MediaPreviewRequest { url: url.into(), user_agent: Some("SyntheticBrowser/1.0 Test".into()), referer: Some("https://www.tiktok.com/".into()), cookie_header: Some("session=synthetic-secret".into()), content_type: Some("video/mp4".into()) }
}
fn client() -> Client { Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).build().unwrap() }
struct Fixture { url: String, received: Arc<Mutex<Vec<String>>>, task: JoinHandle<()> }
impl Drop for Fixture { fn drop(&mut self) { self.task.abort(); } }
async fn fixture(responses: Vec<String>) -> Fixture {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let received = Arc::new(Mutex::new(Vec::new())); let log = received.clone();
    let task = tokio::spawn(async move {
        for response in responses {
            let (mut stream, _) = listener.accept().await.unwrap(); let mut bytes = Vec::new();
            loop {
                let mut chunk = [0; 2048]; let count = stream.read(&mut chunk).await.unwrap();
                if count == 0 { break; } bytes.extend_from_slice(&chunk[..count]);
                if bytes.windows(4).any(|s| s == b"\r\n\r\n") { break; }
            }
            log.lock().unwrap().push(String::from_utf8(bytes).unwrap());
            let _ = stream.write_all(response.as_bytes()).await; let _ = stream.shutdown().await;
        }
    });
    Fixture { url, received, task }
}
fn response(status: &str, headers: &str, body: &str) -> String {
    format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}", body.len())
}
struct Running { target: String, stop: oneshot::Sender<()>, task: JoinHandle<()>, logs: Arc<Mutex<Vec<String>>> }
fn running(request: MediaPreviewRequest) -> Running {
    let relay = Relay::bind(request, Client::builder().no_proxy()).unwrap(); let target = relay.target.clone();
    let (stop, receiver) = oneshot::channel(); let logs = Arc::new(Mutex::new(Vec::new())); let sink = logs.clone();
    let reporter: Reporter = Arc::new(move |event, detail| sink.lock().unwrap().push(format!("{event} {detail}")));
    let task = tokio::spawn(relay.run(receiver, reporter, Duration::from_secs(5), Duration::from_secs(15)));
    Running { target, stop, task, logs }
}
async fn stop(server: Running) {
    let _ = server.stop.send(()); tokio::time::timeout(Duration::from_secs(2), server.task).await.unwrap().unwrap();
}
#[test]
fn only_direct_tiktok_media_uses_the_new_transport() {
    let mut capture = request("https://v16-webapp-prime.tiktok.com/video/tos/synthetic/?mime_type=video_mp4");
    assert!(is_candidate(&capture));
    capture.url = "https://v16.tiktokcdn.com/clip.mp4?signature=synthetic".into(); assert!(is_candidate(&capture));
    capture.url = "https://unrelated.example/clip.mp4".into(); assert!(!is_candidate(&capture));
    capture.url = "https://tiktok.com.unrelated.example/clip.mp4".into(); assert!(!is_candidate(&capture));
    capture.url = "https://v16.tiktok.com/master.m3u8".into(); assert!(!is_candidate(&capture));
    capture.url = "https://v16.tiktok.com/opaque".into(); capture.content_type = Some("application/dash+xml".into()); assert!(!is_candidate(&capture));
    capture.url = "https://www.tiktok.com/@creator/video/123".into(); capture.content_type = None; assert!(!is_candidate(&capture));
    capture.url = "magnet:?xt=urn:btih:synthetic".into(); assert!(!is_candidate(&capture));
}
#[tokio::test]
async fn signed_url_headers_and_seek_ranges_are_preserved_exactly() {
    let _serial = SERIAL.lock().await;
    let origin = fixture(vec![response("206 Partial Content", "Content-Type: video/mp4\r\nContent-Range: bytes 2-5/8\r\nAccept-Ranges: bytes\r\n", "cdef")]).await;
    let path = "/video/tos/synthetic/?a=1988&&mime_type=video_mp4&signature=a%2Bb%3D&space=%20&plus=a+b";
    let server = running(request(&format!("{}{path}", origin.url)));
    let answer = client().get(&server.target).header("Range", "bytes=2-5").send().await.unwrap();
    assert_eq!(answer.status(), StatusCode::PARTIAL_CONTENT); assert_eq!(answer.headers()["content-range"], "bytes 2-5/8");
    assert_eq!(answer.headers()["content-length"], "4"); assert_eq!(answer.bytes().await.unwrap().as_ref(), b"cdef");
    let got = origin.received.lock().unwrap()[0].clone();
    assert!(got.starts_with(&format!("GET {path} HTTP/1.1\r\n")), "{got}");
    assert!(got.contains("user-agent: SyntheticBrowser/1.0 Test\r\n")); assert!(got.contains("referer: https://www.tiktok.com/\r\n"));
    assert!(got.contains("cookie: session=synthetic-secret\r\n")); assert!(got.contains("range: bytes=2-5\r\n"));
    assert!(!server.target.contains("signature"));
    let logs = server.logs.lock().unwrap().join("\n"); assert!(logs.contains("media.preview_streaming"));
    assert!(!logs.contains("synthetic-secret") && !logs.contains("signature=")); stop(server).await;
}
#[tokio::test]
async fn redirects_do_not_receive_cookies_or_expose_location_to_player() {
    let _serial = SERIAL.lock().await;
    let destination = fixture(vec![response("200 OK", "Content-Type: video/mp4\r\nSet-Cookie: other=secret\r\n", "movie")]).await;
    let origin = fixture(vec![response("302 Found", &format!("Location: {}/next?signature=secret\r\n", destination.url), "")]).await;
    let server = running(request(&format!("{}/original", origin.url)));
    let answer = client().get(&server.target).send().await.unwrap(); assert_eq!(answer.status(), StatusCode::OK);
    assert!(answer.headers().get("location").is_none()); assert!(answer.headers().get("set-cookie").is_none()); assert_eq!(answer.text().await.unwrap(), "movie");
    assert!(origin.received.lock().unwrap()[0].contains("cookie:")); assert!(!destination.received.lock().unwrap()[0].contains("cookie:")); stop(server).await;
}
#[tokio::test]
async fn untrusted_local_requests_never_contact_the_source() {
    let _serial = SERIAL.lock().await;
    let origin = fixture(vec![response("200 OK", "Content-Type: video/mp4\r\n", "movie")]).await;
    let server = running(request(&origin.url));
    for builder in [client().get(format!("{}-wrong", server.target)), client().get(&server.target).header("Origin", "https://untrusted.example"), client().get(&server.target).header("Host", "untrusted.example"), client().post(&server.target), client().get(&server.target).header("Range", "bytes=0-1,3-4")] {
        assert_eq!(builder.send().await.unwrap().status(), StatusCode::FORBIDDEN);
    }
    assert!(origin.received.lock().unwrap().is_empty()); stop(server).await;
}
#[tokio::test]
async fn upstream_failures_are_reported_without_private_error_bodies() {
    let _serial = SERIAL.lock().await;
    let origin = fixture(vec![response("403 Forbidden", "Content-Type: text/html\r\n", "private URL and account info")]).await;
    let server = running(request(&origin.url));
    let answer = client().get(&server.target).send().await.unwrap(); assert_eq!(answer.status(), StatusCode::BAD_GATEWAY); assert!(answer.bytes().await.unwrap().is_empty());
    let log = server.logs.lock().unwrap().join("\n"); assert!(log.contains("preview_upstream_http_403")); assert!(!log.contains("account info") && !log.contains("synthetic-secret")); stop(server).await;
}
#[tokio::test]
async fn head_fallback_preserves_auth_and_invalid_ranges_have_no_body() {
    let _serial = SERIAL.lock().await;
    let origin = fixture(vec![response("405 Method Not Allowed", "", ""), response("200 OK", "Content-Type: video/mp4\r\n", "movie"), response("416 Range Not Satisfiable", "Content-Range: bytes */5\r\n", "private error body")]).await;
    let server = running(request(&origin.url));
    let head = client().head(&server.target).send().await.unwrap(); assert_eq!(head.status(), StatusCode::OK); assert_eq!(head.headers()["content-length"], "5"); assert!(head.bytes().await.unwrap().is_empty());
    let answer = client().get(&server.target).header("Range", "bytes=999-").send().await.unwrap(); assert_eq!(answer.status(), StatusCode::RANGE_NOT_SATISFIABLE); assert_eq!(answer.headers()["content-range"], "bytes */5"); assert_eq!(answer.headers()["content-length"], "0"); assert!(answer.bytes().await.unwrap().is_empty());
    let requests = origin.received.lock().unwrap().clone(); assert!(requests[0].starts_with("HEAD ")); assert!(requests[1].starts_with("GET ") && requests[1].contains("cookie: session=synthetic-secret")); stop(server).await;
}
#[tokio::test]
async fn overlapping_previews_keep_separate_sources_credentials_and_bytes() {
    let _serial = SERIAL.lock().await;
    let a = fixture(vec![response("200 OK", "Content-Type: video/mp4\r\n", "video-a")]).await;
    let b = fixture(vec![response("200 OK", "Content-Type: video/webm\r\n", "video-b")]).await;
    let mut second = request(&b.url); second.cookie_header = Some("session=other".into());
    let sa = running(request(&a.url)); let sb = running(second); assert_ne!(sa.target, sb.target);
    let fetch = async |url: &str| client().get(url).send().await.unwrap().text().await.unwrap();
    let (ra, rb) = tokio::join!(fetch(&sa.target), fetch(&sb.target)); assert_eq!(ra, "video-a"); assert_eq!(rb, "video-b");
    assert!(!a.received.lock().unwrap()[0].contains("session=other")); assert!(!b.received.lock().unwrap()[0].contains("synthetic-secret")); stop(sa).await; stop(sb).await;
}
#[tokio::test]
async fn unused_previews_expire_and_close_the_port() {
    let _serial = SERIAL.lock().await;
    let relay = Relay::bind(request("https://media.example/video"), Client::builder().no_proxy()).unwrap(); let target = relay.target.clone();
    let (_stop, receiver) = oneshot::channel(); let reporter: Reporter = Arc::new(|_, _| {});
    tokio::time::timeout(Duration::from_secs(2), relay.run(receiver, reporter, Duration::from_millis(20), Duration::from_millis(100))).await.unwrap(); assert!(client().get(target).send().await.is_err());
}
#[test]
fn rejects_query_only_urls_and_header_injection() {
    for url in ["?a=1988", "file:///etc/passwd", "https://name:secret@media.example/v", "https://media.example/v\r\n--flag"] { assert!(validate(&request(url)).is_err()); }
    let mut bad = request("https://media.example/v"); bad.cookie_header = Some("sid=one\r\nHost: evil".into()); assert!(validate(&bad).is_err());
    bad.cookie_header = None; bad.referer = Some("javascript:alert(1)".into()); assert!(validate(&bad).is_err());
}
#[test]
fn portable_player_path_and_local_url_are_separate_arguments() {
    let root = std::env::temp_dir().join(format!("adm vlc test {}", uuid::Uuid::new_v4())); let player = root.join("App/vlc/vlc.exe");
    std::fs::create_dir_all(player.parent().unwrap()).unwrap(); std::fs::write(&player, b"test fixture").unwrap();
    assert_eq!(effective_player(&root.join("VLCPortable.exe")), player);
    let target = "http://127.0.0.1:1234/random/media"; let command = player_command(&player, target);
    assert_eq!(command.get_program(), player.as_os_str()); assert_eq!(command.get_current_dir(), player.parent());
    let args = command.get_args().map(|s| s.to_string_lossy().into_owned()).collect::<Vec<_>>(); assert!(args.iter().any(|arg| arg == target));
    if cfg!(windows) { assert!(args.iter().any(|arg| arg == "--no-one-instance")); }
    assert!(!format!("{command:?}").contains("signature=")); std::fs::remove_dir_all(root).unwrap();
}
