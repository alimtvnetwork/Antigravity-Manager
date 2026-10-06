# 23 - Switch Audit Default Key, Wrapper PID Recovery, and Session Directives (v4.124.0 to v4.125.0 follow-up)

- Written: 2026-10-06T01:45:00Z
- Session window: 2026-10-02 (v4.124.0, v4.125.0, follow-up commit `366d84e5`)
- Shipped in: first release after `366d84e5`, which is `v4.126.0` (`9b976bc9`)
- Related: `.ai-memory/issues/60-switch-drops-live-conversation-rca.md`, `.ai-memory/issues/61-running-prompt-resumption-and-conversation-continuity-rca.md`, `.ai-memory/issues/62-switch-audit-wrong-key-and-macos-wrapper-pid-rca.md`, `02-spec/21-app/100-resume-same-conversation-and-pid-cache.md`
- Task register: `.ai-memory/plans/readme.md` section "Recent Completed Tasks Register" (row 20). Read order: `.ai-memory/what-to-read.md`.

## 1. Verbatim user directives from this session

These were stated by the user (Alim) and remain binding:

1. "make this UI better in the instance section and make a minor bump and release please"
2. "respect the other PIDs when changing the account don't close other instances PID do e2e test before submitting the release please, then reelase with a minor bump please"
3. Earlier in the same session (v4.124.0 request, summarized from the transcript): fix the running prompt not being backed up, not appearing in the audit, and not resuming in the same conversation, for both Switch and fast-forward. Make the audit detail an "i" button that opens a scrollable modal with Copy and Cancel, like the email JSON view. Quota checks must be no less than 2 minutes, and the PID search runs only when credit is under the threshold. Trust the PID saved at launch when its path matches. Run a PID cache refresh every 10 minutes by default, with a minimum of 3 minutes. Follow the skill `.cursor/skills/execute-parent-task-with-n-steps-v6`, then do a minor bump and release.
4. Standing user rule: "After finishing any change, always stage only the files you changed (explicit paths, never `git add -A`), write a concise commit message, commit, and push to the current branch. Never ask for permission to commit or push. If a push is rejected because the remote is ahead, rebase onto the upstream and push again. Never force-push, never use --no-verify, and never commit unrelated files, generated caches, or secrets."

## 2. Session safety constraints (kept for every switch or instance task)

- Do not kill Cursor or `agm-alim`.
- Do not switch the default instance during tests.
- Do not delete the live sandboxes `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`.
- Do not claim a live prompt reinject was watched unless it was.
- Do not click the Google data-collection checkbox.
- Do not launch a replacement AGM GUI.
- Tag only after CI for that exact SHA is green. A cancelled macOS job is rerun with `gh run rerun <id> --failed`, never tagged over.
- PowerShell here-docs fail in this shell. Use `git commit -m "title" -m "body"`.
- Changelog attribution is only `(Thanks to @aukgit)`.

## 3. What was done (in order)

| Step | Commit / Release | Change |
|---|---|---|
| 1 | v4.124.0 | Preview fallback in `live_prompt_text`; `.antigravity_resume_task.json` keeps `session_id` and `conversation_id`; audit "i" modal (`src/pages/Audit.tsx`); 120 second Google quota floor (`MIN_GOOGLE_QUOTA_CHECK_SECONDS`); saved PID trust plus `refresh_pid_cache_if_due` (default 600, clamp 180..=1200); config field `pid_refresh_seconds`. |
| 2 | `a3fa0ee1` | Sync `readme.md` / `readme_en.md` notes to 4.124.0. Git tracks these lowercase; staging `README.md` misses them. |
| 3 | v4.125.0 | `should_spare_pid` and `other_instance_protection(except_id)` in `instance.rs`; `close_instance` and `process::get_antigravity_pids` skip other instances' PIDs; single-instance switch no longer calls `close_antigravity(None)`; `integration.rs` default switch only calls `close_instance("default")`; Instances page cards one column until `xl`, wrapping paths and action rows. |
| 4 | `366d84e5` | Follow-up from two research subagents. (a) `account.rs`: default-switch audit snapshot reads `switch_prompt_snapshot("default")` instead of the IDE flavor key. (b) `instance.rs`: `is_instance_running` searches once with `find_pids_for_data_dir` when the saved PID no longer matches, then saves the real PID via `record_instance_pid`. |

Verification for `366d84e5`: `cargo fmt -- --check` clean; `cargo clippy --all-targets --all-features` zero errors and no warnings in touched lines; targeted tests passed: `saved_pid_identity_and_refresh_floor`, `switch_spares_another_instances_pid`, `switch_payload_keeps_from_to_reason_and_reinject`. Live account switch e2e was NOT run (protecting the user's running IDE).

## 4. Root causes learned (one sentence each)

- Audit snapshot empty: `switch_account` passed `target_ide` (an IDE flavor such as `"ide"`) to `switch_prompt_snapshot`, but prompts are keyed by instance id, so the default instance must be read with `"default"`.
- Instance shown stopped on macOS: the launcher records the PID of the short-lived `open` wrapper, which exits after spawning the IDE, so a saved-PID-only check returns false for a live instance.

## 5. Learned patterns

- Instance identity and IDE flavor are different keys. `target_ide` names the IDE flavor; per-instance data (prompts, PIDs, leases) is keyed by instance id (`"default"` or the instance uuid).
- A saved PID is a hint, not proof. Keep the cheap identity check first, and keep an explicit fallback search that rewrites the saved PID. This follows the AGENTS.md rule "Keep an explicit fallback to the verified path whenever you replace it".
- Same-conversation resume depends on `session_id` and `conversation_id` in `.antigravity_resume_task.json`. `agy -p` starts a new CLI prompt and does not resume a conversation.
- Cargo builds on this host: `$env:CARGO_TARGET_DIR="D:\work\antigravity-manager\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"`, run `cargo fmt` from `src-tauri`.

## 6. State observed on 2026-10-06 (git audit of last 30 commits)

- HEAD is `ccf336b1` (v4.158.0 line). 119 commits landed after `366d84e5`. Both fixes are still present: `account.rs:1743` reads `switch_prompt_snapshot("default")`; `is_instance_running` at `instance.rs:2584` still recovers and records the real PID.
- Changed after this session: commits `a0c5cc18` and `3581207d` (v4.155.0) made the saved-PID branch also call `find_pids_for_data_dir` and require the saved PID to be in that list. That removes the "trust the saved PID without a scan" fast path the user asked for. Logged as open ambiguity `.ai-memory/ambiguous-questions/01-new-ambiguity/02-is-instance-running-saved-pid-fast-path.md`. Do not revert it without the user's answer.
- Last 30 commits trajectory (v4.151.0 to v4.158.0): instance fleet deploy and default theme presets (`6d41d931`), AccountRow JSX and rustfmt CI fixes (`8ce647c7`, `e0ce4f7c`, `a19644a7`, `76493525`, `5de41444`, `d56a384e`, `65800177`), split restart button and running projects detection (`2a5cd064`, `3581207d`, `2289730e`, `4c28ecf8`), seven AGM skills (`2274f100`), accounts cyan borders and email cooldown (`fbd5e819`), prompt tree from summaries DB and neon progress glow (`ece936ab`, `8406677f`, `17cc2a50`, `337ed8f2`), weekly scoring iterations (`45e2c69f`, `1796e21e`, `c7b11628`), two-phase 4 hour gate (`c7b11628`, `b7e5744c`), delimiter and rustfmt fixes (`01b48a06`, `5bb32c51`, `6228e437`, `ccf336b1`).
- Recurring lesson from that history: 12 of the last 30 commits are rustfmt or compile fixes after a feature commit. Run `cargo fmt -- --check` and `cargo clippy --all-targets --all-features` before every push, not after CI fails.

## 7. Still pending from this session

Tracked in `.ai-memory/plans/pending/139-switch-followups-pid-refresh-parity-and-live-e2e.md`:

1. `pid_refresh_seconds` exists only in `src-tauri/src/models/config.rs`; no Settings UI control and no CLI flag (Headless and CLI parity rule).
2. Passing `session_id` to the `agy` CLI. No supported flag is known; do not invent one.
3. Live account switch e2e that watches the prompt reinject into the same conversation, run on a sandbox instance only.
4. Answer to the open ambiguity in section 6.
