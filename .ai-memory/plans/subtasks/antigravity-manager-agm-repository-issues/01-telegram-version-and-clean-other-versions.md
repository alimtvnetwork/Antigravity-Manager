# Subtask 01: Telegram Version Investigation & Clean Other Versions CLI

Owner: Worker 01 / Backend
Target Files:
- `src-tauri/src/modules/telegram_inbound/`
- `src-tauri/src/bin/agm/`
- `src-tauri/src/modules/updater/`

## Requirements
1. **Telegram Version Root Cause Investigation**:
   - Investigate why Telegram worker reports `v1.9.0` or links to an older version.
   - Audit `src-tauri/src/modules/telegram_inbound/formatting.rs` and handlers. Ensure version reporting uses host `crate::VERSION`.
   - Remove unnecessary `Id` tag appended to prompt dispatches.
   - Fix `/shot` / `/screenshot` capturing and sending.
   - Implement configurable prompt summary word count (100 or 200 words) in settings and CLI (`agm telegram config --summary-words <100|200>`).
   - Format previous prompts as short 100/200 word snippets with one-tap resend buttons.
2. **Clean Other Versions CLI (`agm clean-other-versions` / `agm ov`)**:
   - Implement command `agm clean-other-versions` (alias: `agm ov`, `agm versions clean`).
   - Scan system paths for older/alternate installs of Antigravity Tools / Antigravity Manager (`%LOCALAPPDATA%\\Programs\\antigravity-tools`, legacy directories) and remove them.
   - Crucial Invariant: Strictly retain account and session information inside the SQLite database (`%APPDATA%\\antigravity_tools\\`, `~/.antigravity_tools/`).

## Status: COMPLETED
- Telegram dynamic versioning fixed to `crate::VERSION`.
- Internal ID wrappers stripped from prompts.
- Configurable prompt summary words (100/200) added with inline resend buttons.
- `clean-other-versions` CLI command implemented and verified (reclaimed 179 MB, preserved DBs).
