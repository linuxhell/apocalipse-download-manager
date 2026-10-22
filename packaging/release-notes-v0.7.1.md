# Apocalipse Download Manager 0.7.1

- Download thread slider and per-site checkbox are visible outside Advanced options, in English, Portuguese and Simplified Chinese.
- Checking saves the current count immediately; changing 1 → 4 → 8 replaces the saved count; unchecking removes the site's thread preference without removing credentials or other settings. Rapid changes are serialized and stale lookups cannot overwrite the user's selection.
- The slider is always sent explicitly to the engine and takes priority over site defaults and automatic tuning. Servers and small files can use fewer connections, never more than the chosen limit.
- Includes pinned Xtreme 0.2.1: HTTPS also trusts the OS certificate store and reports certificate failures safely. The three reported NirSoft ZIP URLs and the additionally reported ESOUI ZIP downloaded successfully with ZIP integrity verification.
- Includes the ED2K preview, lock cleanup and cache fixes from the 0.7.0 preparation. Browser extension remains 0.3.206.
