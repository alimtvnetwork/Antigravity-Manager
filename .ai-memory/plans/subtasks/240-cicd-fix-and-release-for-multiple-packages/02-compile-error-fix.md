# Subtask 02 - Compile-error fix and local pre-flight

Date: 31-Oct-26
Status: in progress
Owned files (exclusive):
- src-tauri/src/modules/auto_switcher/scoring.rs
- src-tauri/src/modules/auto_switcher/selection.rs
- src-tauri/src/modules/auto_switcher/manual.rs
- src-tauri/src/modules/repo_db/agy.rs
- .ai-memory/plans/subtasks/240-cicd-fix-and-release-for-multiple-packages/02-compile-error-fix.md
Work:
- Add `rolling_4h_quota` to ProfileCandidate; populate it at all seven
  construction sites in selection.rs with the site's quota_percent value.
- Make the `active_prompt` binding mutable in agy.rs.
- Run `cargo fmt -- --check` and `cargo clippy --all-targets
  --all-features` under src-tauri; clippy must exit 0.
Evidence: local clippy exit code and the post-push CI run ID/URL.
