# Subtask 02: GitMap SUG Watch Loop, Auto-Clear on Shutdown & Real-Time Web UI

Traceability ID: Task-03
Spec Reference: [02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md](../../../02-spec/21-app/67-gitmap-sug-overhaul-and-repo-secrets-consolidation.md)
Target Files: D:/work/gitmap/cli/cmdagy/agy_sug.go, D:/work/gitmap/cli/cmdagy/agy_sug_executor.go, D:/work/gitmap/cli/cmdagy/agy_sug_ui.go
Action:
- Add `watch` command alias for `run` with explicit terminal status readout and interval timer.
- Add setting `AutoClearOnShutdown` (persisted in `sug_watch_list.json` and configurable via CLI / root config).
- Clear the watch list automatically when system shutdown is triggered or initiated.
- Implement `gitmap sug ui` / `gitmap sug watch ui` launching an embedded HTTP server and opening the browser.
- Build the web dashboard with real-time SSE / polling showing watch list projects, CI/CD pipeline status, countdown timers, project dropdown picker for 1-click add, and drag-and-drop support.

Acceptance Criteria:
- `gitmap sug watch` runs the monitoring loop with clear status logging.
- `gitmap sug ui` starts the dashboard and serves the web UI on localhost.
- Watch list is cleared upon shutdown when `AutoClearOnShutdown` is true.

Targeted Verification:
- Run `gitmap sug ui --dry-run` or check local server endpoint responds with 200 OK.
