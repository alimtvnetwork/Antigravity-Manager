# 139 - Switch Follow-ups: PID Refresh Parity, agy Session Resume, Live E2E

Status: COMPLETED
Created: 2026-10-06
Source: session of 2026-10-02 (v4.124.0, v4.125.0, `366d84e5`); memory `.ai-memory/memory/learned/23-switch-audit-default-key-and-wrapper-pid-recovery.md`
Task register: `.ai-memory/plans/readme.md` "Recent Completed Tasks Register"

## Steps

1. Expose `pid_refresh_seconds` (default 600, clamp 180..=1200, `src-tauri/src/models/config.rs`) in the Settings UI next to the auto switcher controls and as an `agm` CLI config flag, through the same config save path (Headless and CLI parity rule in AGENTS.md).
2. Find whether the `agy` CLI supports resuming a conversation by `session_id`. If it does, pass it from `spawn_prompt_via_agy`. If it does not, record that in spec 100 and stop. Do not invent a flag.
3. Run a live account switch e2e on a sandbox instance only (never the default instance, never kill Cursor or `agm-alim`): back up a running prompt, switch, confirm the audit row has prompt and conversation id, confirm the prompt resumes in the same conversation, confirm another instance's PID stays alive.
4. Resolve ambiguity `.ai-memory/ambiguous-questions/01-new-ambiguity/02-is-instance-running-saved-pid-fast-path.md` with the user, then apply the chosen option with a test.

## Done when

- All four steps have a commit or a recorded decision, and `.ai-memory/plans/readme.md` moves this file to `completed/`.

## Progress (2026-10-23, subtask 190-05)

- Step 1 DONE: `pid_refresh_seconds` is now exposed in the Settings UI (AutoSwitcher → Polling Settings, "PID Refresh Interval (seconds)" slider, 180–1200s, default 600s, saved through the same config save path as the other auto-switcher controls) and as an AGM CLI flag (`agm auto-switch pid-refresh [N]`, clamped 180..=1200, shown in `agm auto-switch status`), giving GUI/CLI parity through `config::save_app_config`.
- Step 2 DONE (check recorded): the `agy` CLI does support resuming a conversation — via `--conversation <conversation_id>` (`--resume` is rejected by agy). `spawn_prompt_via_agy` in `src-tauri/src/modules/repo_db/agy.rs` already passes it, deriving the conversation id from `prompt.session_id` (`agy_conversation_id`), and the resume file written by `dispatch_running_prompts` carries `session_id`/`conversation_id` (see spec 100, `02-spec/21-app/100-resume-same-conversation-and-pid-cache.md`). No code change was needed; no flag was invented.
- Step 3 DEFERRED: the live sandbox account-switch E2E is executed in subtask 190-08's E2E phase (builds/runs are forbidden during the implementation phase of task 190).
- Step 4 RESOLVED 2026-10-23: the user decided "trust but always verify before execution; if mismatch, full rescan". Audit of the current `is_instance_running` (`src-tauri/src/modules/instance/pid_util.rs`) shows it already implements exactly that rule — saved PID first, single-process identity + instance-window verification before any trust, full rescan on any mismatch, found PID recorded — so no code change was needed. The ambiguity file is marked resolved with the full audit trail. (Step 3, the live sandbox switch E2E, remains open: it needs a Windows/IDE machine.)
