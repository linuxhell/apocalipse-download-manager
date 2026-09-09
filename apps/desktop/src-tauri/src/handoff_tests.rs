use super::*;
use std::sync::Barrier;

struct TestDirectory(PathBuf);
impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("adm-handoff-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for TestDirectory {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

#[test]
fn reserves_batch_names_before_any_file_exists() {
    let dir = TestDirectory::new();
    let mut queue = Vec::new();
    for index in 0..3 {
        let url = format!("https://example.test/video/{index}/?mime_type=video_mp4");
        let name = append_source_extension(suggested_name(&url), &url, DownloadKind::Http);
        let mut task = DownloadTask::new(&url, dir.0.join(name));
        reserve_queued_task(&mut queue, &mut task, true).unwrap();
        let expected = if index == 0 { "download.mp4".into() } else { format!("download ({index}).mp4") };
        assert_eq!(task.destination, dir.0.join(expected));
        assert_eq!(task.source, url);
        assert!(!task.destination.exists());
        assert!(!partial_path(&task.destination).exists());
    }
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
}

#[test]
fn concurrent_producers_keep_all_payloads_and_ids_separate() {
    let dir = TestDirectory::new();
    let queue = Mutex::new(Vec::new());
    let barrier = Barrier::new(32);
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for index in 0..32 {
            let queue = &queue;
            let barrier = &barrier;
            let directory = &dir.0;
            handles.push(scope.spawn(move || {
                let mut task = DownloadTask::new(format!("https://example.test/{index}"), directory.join("video.mp4"));
                barrier.wait();
                { reserve_queued_task(&mut queue.lock().unwrap(), &mut task, true).unwrap(); }
                barrier.wait();
                let payload = vec![index as u8; 4096 + index * 37];
                fs::write(&task.destination, &payload).unwrap();
                (task, payload)
            }));
        }
        let completed = handles.into_iter().map(|handle| handle.join().unwrap()).collect::<Vec<_>>();
        assert_eq!(completed.iter().map(|(task, _)| &task.destination).collect::<HashSet<_>>().len(), 32);
        assert_eq!(completed.iter().map(|(task, _)| task.id).collect::<HashSet<_>>().len(), 32);
        for (task, payload) in completed { assert_eq!(fs::read(task.destination).unwrap(), payload); }
    });
    assert_eq!(queue.lock().unwrap().len(), 32);
}

#[test]
fn simultaneous_duplicate_urls_are_rejected_atomically() {
    let dir = TestDirectory::new();
    let queue = Mutex::new(Vec::new());
    let barrier = Barrier::new(16);
    let successes = std::thread::scope(|scope| {
        let handles = (0..16).map(|_| {
            let queue = &queue;
            let barrier = &barrier;
            let path = dir.0.join("same.mp4");
            scope.spawn(move || {
                let mut task = DownloadTask::new("https://example.test/same", path);
                barrier.wait();
                reserve_queued_task(&mut queue.lock().unwrap(), &mut task, true).is_ok()
            })
        }).collect::<Vec<_>>();
        handles.into_iter().map(|handle| handle.join().unwrap()).filter(|success| *success).count()
    });
    assert_eq!(successes, 1);
    assert_eq!(queue.lock().unwrap().len(), 1);
}

#[test]
fn paused_failed_completed_and_queued_tasks_keep_their_destinations() {
    let dir = TestDirectory::new();
    let mut queue = Vec::new();
    for (index, state) in [DownloadState::Paused, DownloadState::Failed { message: "test".into() }, DownloadState::Completed, DownloadState::Queued].into_iter().enumerate() {
        let mut task = DownloadTask::new(format!("https://example.test/{index}"), dir.0.join("video.mp4"));
        task.state = state;
        reserve_queued_task(&mut queue, &mut task, false).unwrap();
    }
    assert_eq!(unique_destination_with_queue(&dir.0, "video.mp4", &queue).unwrap(), dir.0.join("video (4).mp4"));
}

#[test]
fn existing_files_partials_and_chunks_are_never_reused() {
    let dir = TestDirectory::new();
    fs::write(dir.0.join("video.mp4"), b"existing").unwrap();
    fs::write(partial_path(&dir.0.join("video (1).mp4")), b"partial").unwrap();
    fs::create_dir_all(apocalipse_core::chunk_directory(&dir.0.join("video (2).mp4"))).unwrap();
    assert_eq!(unique_destination(&dir.0, "video.mp4").unwrap(), dir.0.join("video (3).mp4"));
    assert_eq!(fs::read(dir.0.join("video.mp4")).unwrap(), b"existing");
    assert_eq!(fs::read(partial_path(&dir.0.join("video (1).mp4"))).unwrap(), b"partial");
}

#[test]
fn partial_aliases_and_normalized_paths_are_reserved() {
    let dir = TestDirectory::new();
    let mut queue = vec![DownloadTask::new("https://example.test/a", dir.0.join("./video.mp4"))];
    let mut task = DownloadTask::new("https://example.test/b", dir.0.join("video.mp4.part"));
    reserve_queued_task(&mut queue, &mut task, true).unwrap();
    assert_ne!(task.destination, dir.0.join("video.mp4.part"));
    assert_eq!(unique_destination_with_queue(&dir.0, "video.mp4", &queue).unwrap(), dir.0.join("video (1).mp4"));
    if cfg!(any(windows, target_os = "macos")) {
        assert_eq!(unique_destination_with_queue(&dir.0, "VIDEO.MP4", &queue).unwrap(), dir.0.join("VIDEO (1).MP4"));
    }
}

#[test]
fn explicit_redownload_allows_same_url_but_not_same_destination() {
    let dir = TestDirectory::new();
    let mut queue = Vec::new();
    for _ in 0..3 {
        let mut task = DownloadTask::new("https://example.test/same", dir.0.join("video.mp4"));
        reserve_queued_task(&mut queue, &mut task, false).unwrap();
    }
    assert_eq!(queue.iter().map(|task| &task.destination).collect::<HashSet<_>>().len(), 3);
}

#[test]
fn exhausted_names_fail_instead_of_using_an_unchecked_fallback() {
    let dir = TestDirectory::new();
    let queue = (0..10_000).map(|index| {
        let name = if index == 0 { "video.mp4".into() } else { format!("video ({index}).mp4") };
        DownloadTask::new("https://example.test/same", dir.0.join(name))
    }).collect::<Vec<_>>();
    assert_eq!(unique_destination_with_queue(&dir.0, "video.mp4", &queue), Err("download_destination_names_exhausted".into()));
}

#[test]
fn mime_query_adds_the_correct_extension_without_rewriting_urls() {
    for (mime, extension) in [("video_mp4", "mp4"), ("video%2Fwebm", "webm"), ("audio_mp4", "m4a"), ("audio%2Fmpeg", "mp3")] {
        let source = format!("https://example.test/resource/?mime_type={mime}&signature=a%2Fb%2Bc");
        let original = source.clone();
        assert_eq!(append_source_extension("download".into(), &source, DownloadKind::Http), format!("download.{extension}"));
        assert_eq!(append_source_extension("chosen.mkv".into(), &source, DownloadKind::Http), "chosen.mkv");
        assert_eq!(source, original);
    }
    assert_eq!(append_source_extension("download".into(), "https://example.test/?mime_type=text_html", DownloadKind::Http), "download");
}
