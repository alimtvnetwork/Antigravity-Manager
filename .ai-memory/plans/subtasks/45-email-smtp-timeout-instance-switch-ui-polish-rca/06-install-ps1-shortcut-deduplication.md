# Subtask: Desktop Shortcut De-Duplication & Legacy Removal

**Target Files:**
- `install.ps1`

**Action:**
1. In `install.ps1`:
   - Set `$AppName = "AGM by Alim"` (line 54) to match `tauri.conf.json` product name.
   - In `Remove-LegacyUpstreamInstallation`, add `"Anti-Gravity Tools by Alim.lnk"` to `$legacyShortcuts` so old shortcuts from previous versions are cleanly unlinked.
   - Before creating Desktop and Start Menu shortcuts, check if the shortcut already exists and verifies its `TargetPath` equals `$ExePath`. If it already points to the correct executable, skip duplicate creation.
   - Clean up any stray `"Anti-Gravity Tools by Alim.lnk"` found on Desktop or Start Menu prior to shortcut verification.

**Constraints:**
- No duplicate desktop icons.
- Safe idempotent installer execution.
