# ADM 0.4.80 / extension 0.3.190 — Cinema Premium test candidate

- Replaces the theme selector with a searchable 28-theme gallery, live preview, Apply, Restore default and Cancel. Draft appearance is reverted when leaving Themes; only Apply persists it.
- Ships local high-definition wallpapers at their original resolution (1672 × 941 and above), plus separate lightweight gallery thumbnails. Cyberpunk uses a reconstructed clean background from the maintainer's robotic-alien reference; Samurai uses the supplied image; Fantasy shows the elf with feet in water and a fairy on her extended hand.
- Adds glass intensity, background brightness, Aero blur, smooth animations, corner radius and interface size. Scenery opacity stays separate from text opacity. Dark and light palettes retain different contrast backing limits.
- Applies scenery and glass to the titlebar, sidebar, downloads, settings, tools and standalone Link presentation. Download queue, progress, torrent actions, recording actions, diagnostics and existing controls remain connected to the existing backend.
- Adds native main-window minimize, maximize/restore, close and drag actions. Window buttons are always black with white symbols; closing retains the existing close-to-tray behavior.
- Synchronizes all 28 palette names with the extension, which retains solid opaque panels. Firefox test packaging remains unsigned unless a signed 0.3.190 XPI is supplied.

## Validation

- 326 Node regression tests passed locally.
- Full frontend initialized in a DOM harness; gallery, draft/apply/cancel, navigation rollback and native-action dispatch passed.
- Wallpaper/thumbnail containers and dimensions checked; all 28 assets are present.
- Test CI includes browser screenshots and small-window checks as well as Rust formatting, workspace tests and desktop builds. Native compilation and visual browser QA still need the GitHub test workflows to run. Local Chromium download was unavailable in this environment.
- This is a build-only candidate. No release tag or main merge is requested by this change.
