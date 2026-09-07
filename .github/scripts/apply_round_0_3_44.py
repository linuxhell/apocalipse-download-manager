from pathlib import Path

# This test-only patch is applied after apply_fb_library_test.py.
# It keeps the Rapidgator CDP transport intact while hardening the shared blob bridge,
# adds real pause/resume for active blob streams, and keeps a single Facebook Reels overlay.

# --- Extension background: smaller/retriable blob chunks + pause-aware streams.
p = Path('browser-extension/background.js')
s = p.read_text(encoding='utf-8')

anchor = '''async function rapidgatorBridgeHealth() {
'''
helper = '''async function rapidgatorBlobChunk(uploadId, bytes) {
  // Keep JSON comfortably below the desktop bridge request/chunk limits.
  // Retry only HTTP 400: a 400 means the desktop rejected the chunk before accepting it.
  let lastError = null;
  for (let attempt = 0; attempt < 3; attempt += 1) {
    try {
      return await rapidgatorBridgePost("/v1/blob/chunk", { uploadId, data: hex(bytes) });
    } catch (error) {
      lastError = error;
      if (!String(error).includes("bridge_http_400")) throw error;
      await new Promise((resolve) => setTimeout(resolve, 100 * (attempt + 1)));
    }
  }
  throw lastError || new Error("blob_chunk_failed");
}

async function waitForBlobResume(uploadId) {
  while (true) {
    const status = await rapidgatorBridgePost("/v1/blob/status", { uploadId });
    if (!status?.paused) return status;
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
}

'''
if 'async function rapidgatorBlobChunk' not in s:
    if anchor not in s: raise SystemExit('background health anchor missing')
    s = s.replace(anchor, helper + anchor, 1)

# ChatGPT Library: smaller pieces and pause-aware sending.
old = '''        for (let offset = 0; offset < value.length; offset += 64 * 1024) {
          const slice = value.subarray(offset, Math.min(value.length, offset + 64 * 1024));
          await rapidgatorBridgePost("/v1/blob/chunk", { uploadId, data: hex(slice) });
          state.bytes += slice.length;
          chunks += 1;
        }
'''
new = '''        for (let offset = 0; offset < value.length; offset += 16 * 1024) {
          const slice = value.subarray(offset, Math.min(value.length, offset + 16 * 1024));
          await waitForBlobResume(uploadId);
          await rapidgatorBlobChunk(uploadId, slice);
          state.bytes += slice.length;
          chunks += 1;
        }
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'await rapidgatorBlobChunk(uploadId, slice);' not in s:
    raise SystemExit('chatgpt chunk loop missing')

# Rapidgator CDP: request smaller IO chunks and honor desktop pause before committing each chunk.
s = s.replace('''const chunk = await debuggerCommand(debuggee, "IO.read", { handle: body.stream, size: 64 * 1024 });''',
              '''const chunk = await debuggerCommand(debuggee, "IO.read", { handle: body.stream, size: 16 * 1024 });''', 1)
old = '''      if (bytes.length) {
        await rapidgatorBridgePost("/v1/blob/chunk", { uploadId, data: hex(bytes) });
        state.bytes += bytes.length;
'''
new = '''      if (bytes.length) {
        await waitForBlobResume(uploadId);
        await rapidgatorBlobChunk(uploadId, bytes);
        state.bytes += bytes.length;
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'await rapidgatorBlobChunk(uploadId, bytes);' not in s:
    raise SystemExit('rapidgator chunk send missing')

p.write_text(s, encoding='utf-8')

# --- Facebook Reels: only one overlay pair follows the currently visible reel.
p = Path('browser-extension/content.js')
s = p.read_text(encoding='utf-8')
anchor = '''    document.querySelectorAll("video,audio").forEach((element) => {
'''
active = '''    const isFacebookReelsPage = /(^|\\.)facebook\\.com$/i.test(location.hostname)
      && /(?:^|\\/)reels?(?:\\/|$)/i.test(location.pathname);
    let activeFacebookReel = null;
    if (isFacebookReelsPage) {
      const viewportCenter = innerHeight / 2;
      const candidates = [...document.querySelectorAll("video")]
        .map((video) => ({ video, rect: video.getBoundingClientRect() }))
        .filter(({ rect }) => rect.width >= 100 && rect.height >= 55 && rect.bottom > 0 && rect.top < innerHeight)
        .sort((left, right) => {
          const leftCenter = Math.abs((left.rect.top + left.rect.bottom) / 2 - viewportCenter);
          const rightCenter = Math.abs((right.rect.top + right.rect.bottom) / 2 - viewportCenter);
          return leftCenter - rightCenter;
        });
      activeFacebookReel = candidates[0]?.video || null;
      for (const overlay of [...activeOverlays.values()]) {
        if (overlay.element?.tagName === "VIDEO" && overlay.element !== activeFacebookReel) overlay.cleanup();
      }
    }

'''
if 'const isFacebookReelsPage =' not in s:
    if anchor not in s: raise SystemExit('overlay iterator anchor missing')
    s = s.replace(anchor, active + anchor, 1)

old = '''      const isFacebookVideo = element.tagName === "VIDEO" && /(^|\\.)facebook\\.com$/i.test(location.hostname);
      const tikTokUrl = element.tagName === "VIDEO" ? tikTokUrlFor(element) : null;
'''
new = '''      const isFacebookVideo = element.tagName === "VIDEO" && /(^|\\.)facebook\\.com$/i.test(location.hostname);
      if (isFacebookReelsPage && isFacebookVideo && element !== activeFacebookReel) return;
      const tikTokUrl = element.tagName === "VIDEO" ? tikTokUrlFor(element) : null;
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'element !== activeFacebookReel' not in s:
    raise SystemExit('facebook overlay guard anchor missing')
p.write_text(s, encoding='utf-8')

# --- Desktop: widen safe bridge limits and make blob tasks pause/resume aware.
p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')

s = s.replace('const MAX_REQUEST_SIZE: usize = 262_144;', 'const MAX_REQUEST_SIZE: usize = 524_288;', 1)
s = s.replace('if value.len() > 131_072 || !value.len().is_multiple_of(2) {',
              'if value.len() > 262_144 || !value.len().is_multiple_of(2) {', 1)

old = '''fn recording_stop_requested(app: &tauri::AppHandle, request: &BlobFinish) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let task_id = state
        .blob_uploads
        .lock()
        .map_err(|error| error.to_string())?
        .get(&request.upload_id)
        .map(|upload| upload.task_id)
        .ok_or_else(|| "blob_upload_not_found".to_owned())?;
    let stop = state
        .recording_stops
        .lock()
        .map_err(|error| error.to_string())?
        .contains(&task_id);
    Ok(stop)
}
'''
new = '''fn blob_status_requested(app: &tauri::AppHandle, request: &BlobFinish) -> Result<(bool, bool), String> {
    let state = app.state::<AppState>();
    let task_id = state
        .blob_uploads
        .lock()
        .map_err(|error| error.to_string())?
        .get(&request.upload_id)
        .map(|upload| upload.task_id)
        .ok_or_else(|| "blob_upload_not_found".to_owned())?;
    let stop = state
        .recording_stops
        .lock()
        .map_err(|error| error.to_string())?
        .contains(&task_id);
    let paused = state
        .queue
        .lock()
        .map_err(|error| error.to_string())?
        .iter()
        .find(|task| task.id == task_id)
        .is_some_and(|task| task.state == DownloadState::Paused);
    Ok((stop, paused))
}
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'fn blob_status_requested' not in s:
    raise SystemExit('blob status function missing')

old = '''        match serde_json::from_str::<BlobFinish>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| recording_stop_requested(app, &request))
        {
            Ok(stop) => bridge_response(
                &mut stream,
                "200 OK",
                origin,
                &format!("{\\\"stop\\\":{stop}}"),
            ),
'''
new = '''        match serde_json::from_str::<BlobFinish>(body)
            .map_err(|error| error.to_string())
            .and_then(|request| blob_status_requested(app, &request))
        {
            Ok((stop, paused)) => bridge_response(
                &mut stream,
                "200 OK",
                origin,
                &format!("{\\\"stop\\\":{stop},\\\"paused\\\":{paused}}"),
            ),
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'Ok((stop, paused))' not in s:
    raise SystemExit('blob status route missing')

old = '''    let cancel = state
        .workers
        .lock()
        .map_err(|error| error.to_string())?
        .remove(&id)
        .ok_or_else(|| "download_not_running".to_owned())?;
    let _ = cancel.send(());
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
'''
new = '''    let cancel = state
        .workers
        .lock()
        .map_err(|error| error.to_string())?
        .remove(&id);
    if let Some(cancel) = cancel {
        let _ = cancel.send(());
    } else {
        let active_blob = state
            .blob_uploads
            .lock()
            .map_err(|error| error.to_string())?
            .values()
            .any(|upload| upload.task_id == id);
        if !active_blob {
            return Err("download_not_running".to_owned());
        }
    }
    let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'let active_blob = state' not in s:
    raise SystemExit('pause block missing')

old = '''fn resume_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: DownloadId,
) -> Result<(), String> {
    let task = {
'''
new = '''fn resume_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: DownloadId,
) -> Result<(), String> {
    let active_blob = state
        .blob_uploads
        .lock()
        .map_err(|error| error.to_string())?
        .values()
        .any(|upload| upload.task_id == id);
    if active_blob {
        let mut queue = state.queue.lock().map_err(|error| error.to_string())?;
        let task = queue
            .iter_mut()
            .find(|task| task.id == id)
            .ok_or_else(|| "download_not_found".to_owned())?;
        if task.state != DownloadState::Paused {
            return Err("download_not_resumable".to_owned());
        }
        task.state = DownloadState::Downloading;
        save_queue(&state, &queue)?;
        diagnostic_log(&state, "INFO", "task.resumed", &format!("task={id} engine=blob"));
        return Ok(());
    }
    let task = {
'''
if old in s:
    s = s.replace(old, new, 1)
elif 'engine=blob' not in s:
    raise SystemExit('resume function anchor missing')

p.write_text(s, encoding='utf-8')

# --- Version marker for this build.
p = Path('browser-extension/manifest.json')
s = p.read_text(encoding='utf-8')
s = s.replace('"version": "0.3.43"', '"version": "0.3.44"', 1)
p.write_text(s, encoding='utf-8')
