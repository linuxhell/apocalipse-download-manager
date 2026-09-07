from pathlib import Path

p = Path('apps/desktop/src-tauri/src/main.rs')
s = p.read_text(encoding='utf-8')
start = s.index('} else if first.starts_with("POST /v1/blob/status ") {')
end = s.index('} else if first.starts_with("POST /v1/blob/end ") {', start)
block = s[start:end]
block = block.replace('.and_then(|request| recording_stop_requested(app, &request))', '.and_then(|request| blob_status_requested(app, &request))')
block = block.replace('Ok(stop) => bridge_response(', 'Ok((stop, paused)) => bridge_response(')
block = block.replace('&format!("{\\\"stop\\\":{stop}}"),', '&format!("{\\\"stop\\\":{stop},\\\"paused\\\":{paused}}"),')
if 'recording_stop_requested' in block or 'Ok((stop, paused))' not in block or 'paused' not in block:
    raise SystemExit('blob status route patch failed')
s = s[:start] + block + s[end:]
p.write_text(s, encoding='utf-8')
