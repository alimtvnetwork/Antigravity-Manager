# Subtask 05: Log Attributes, Audit Copy, Process Exit & Release Ceremony

Owner: Lead / Both Workers
Target Files:
- `src/components/audit/AuditPanel.tsx`
- `src-tauri/src/modules/logger.rs`
- `src-tauri/src/modules/process/`
- `src-tauri/src/modules/account/switch.rs`
- `changelog.md`, `changelog_en.md`, `readme.md`, `readme_en.md`

## Requirements
1. **Button Telemetry & Log Section Attributes**:
   - Add `data-ui-name` and `data-ui-id` attributes to buttons in UI logs and action bars.
   - Display friendly names in audit logs instead of internal codes.
   - Add a `📋 Copy Summary` button to the audit summary.
2. **System Log Filtering in AGM Log**:
   - Filter noisy system log entries (`HEARTBEAT`, `TELEMETRY`, `TOKEN_USAGE`) by default with an optional toggle.
3. **Process Clean Termination on Application Close**:
   - Ensure closing Antigravity Manager triggers `exit_app_fully`, terminating all spawned child processes and daemons.
4. **Account Switch 15% Threshold Verification**:
   - Ensure automatic account switching evaluates candidate credit thresholds at 15% remaining quota.
5. **Release Notes & Attribution**:
   - Strict User Directive: "In the release note, remove the segment thanking aukgit."
   - Verify all links and paths are strictly relative Git paths.
   - Run full pre-flight quality gates (`cargo fmt`, `cargo clippy`, `npm run build`).

## Status: COMPLETED
- Added `data-ui-name` and `data-ui-id` telemetry attributes to interaction recording.
- Enhanced AuditPanel with friendly action names and '📋 Copy Summary' clipboard button.
- Implemented system telemetry filtering in logger and log bridge.
- Confirmed full process termination via `exit_app_fully` hooks.
- Prepared release notes adhering to the directive to omit the thanking segment.
