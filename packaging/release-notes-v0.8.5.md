# Apocalipse Download Manager 0.8.5

- Tool updates retain the original executable names and paths. Validation and old-file deletion must succeed; failure restores the previous set.
- Retry blocked staging cleanup briefly on Windows and close updater handles before cleanup.
- Clean abandoned recognized tool candidates older than one hour after validating the installed executables. Active locked updates, recovery backups, symlinks and unknown content are preserved.
- Includes the unchanged Mozilla-signed Firefox 0.3.207 extension and Xtreme 0.2.2.
