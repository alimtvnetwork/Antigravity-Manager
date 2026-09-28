# Subtask 03: Post-Switch Re-Injection, Liveness Verification & Multi-Channel Telemetry

Traceability ID: Task-04, Task-06
Spec Reference: [02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md](../../../02-spec/21-app/68-account-switch-98pct-e2e-parallel-prompt-backup-and-instance-verification.md)
Target Files: src-tauri/src/services/auto_switch_engine.rs, src-tauri/src/services/telegram_bot.rs, src-tauri/src/services/email_service.rs, src-tauri/src/commands/agm_cli.rs
Action:
- After Fast-Forward account rotation, automatically re-inject restored prompts into target workspaces.
- Check execution state using prompt runner watcher (`agm prompts status`) to ensure tasks are actively running.
- If prompt execution does not resume, broadcast an emergency failure alert via Telegram and Email.
- Send step-by-step telemetry over Telegram and Email:
  - Step 1: Pre-switch backup notification listing project names (e.g. `Backing up 2 projects: gitmap, Antigravity-Manager`).
  - Step 2: Account rotation notification with refreshed quota balance.
  - Step 3: Re-injection status and confirmation that prompts are running.
- Enrich AGM CLI help menus with clear banners, flag descriptions, and copy-pasteable examples.

Acceptance Criteria:
- Prompts resume execution post-rotation with zero manual intervention.
- Liveness watcher triggers alert if prompts stall.
- Telegram and Email messages enumerate project names clearly without raw IDs.
- CLI help output displays clean formatting and examples.

Targeted Verification:
- Cargo clippy check across all modules: PASSED (0 errors).
- Email dispatch tested via `agm email help`: PASSED (dispatched successfully to devorg.bd@gmail.com).
- Post-switch prompt re-injection and liveness telemetry wired in `notification_hub.rs` (`notify_post_switch_prompt_status`): VERIFIED.
- Human-readable project names used throughout telemetry cards without raw IDs: VERIFIED.
- Help text polished across all CLI commands (`agm prompts help`, `agm running-prompts help`, `agm running-projects help`, `agm instances help`, `agm ff help`, `agm auto-switch help`, `agm sync --help`, `agm pull --help`, `agm install --help`, `agm email help`, `agm telegram help`, `agm ssh help`, `agm backup help`, `agm restore help`): VERIFIED.

Status: Completed
Verification Date: 2026-09-28
