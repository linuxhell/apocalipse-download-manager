# Apocalipse Download Manager 0.8.0

- Add stream-aware local file conversion in Portuguese, English and Simplified Chinese, accessible from Downloads, Add download and completed-file actions.
- Detect real image/audio/video content with FFprobe, ignore attached cover art, and fully check video before treating all-black video with audio as audio-only. No audio-only targets for silent video.
- Offer installed FFmpeg-supported image, audio and video outputs, including lossless options, audio-track selection, progress and cancellation. Preserve originals and never overwrite existing outputs.
- Expand the download dialog to the main-window area, use two columns where possible and keep advanced options visible.
- Keep extension thumbnails during media inspection and retry the YouTube fallback after decoding failures. Chromium extension 0.3.207 retains remote thumbnail URLs when portable images exceed the bridge limit. Firefox retains the signed 0.3.206 package.
- FFmpeg with FFprobe is required for conversion; install it in Settings → Tools. Xtreme remains pinned at 0.2.1.
