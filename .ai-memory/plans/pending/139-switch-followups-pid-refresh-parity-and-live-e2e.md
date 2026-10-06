# 139 - Switch Follow-ups: PID Refresh Parity, agy Session Resume, Live E2E

Status: pending
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
