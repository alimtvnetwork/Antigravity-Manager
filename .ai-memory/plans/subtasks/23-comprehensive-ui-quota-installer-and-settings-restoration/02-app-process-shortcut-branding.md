# Subtask 02: App, Process & Shortcut Branding Alignment
Traceability ID: Task-03
Spec Reference: [02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md](../../../02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md)
Target Files: src-tauri/tauri.conf.json, src-tauri/Cargo.toml, src-tauri/hooks.nsh, install.ps1, install.sh
Action:
- Set Tauri `productName` to "Antigravity Manager Tools by Alim" or "Antigravity Tools Manager".
- Keep version in Windows Add/Remove DisplayName (`Antigravity Manager Tools 4.65.3`).
- Remove version number from Start Menu / Taskbar shortcut creation so it pins cleanly.
- Ensure Task Manager shows descriptive title instead of raw `agm-alim`.
Acceptance Criteria:
- Task Manager and shortcut titles reflect clean branding.
Targeted Verification: git diff verification.
