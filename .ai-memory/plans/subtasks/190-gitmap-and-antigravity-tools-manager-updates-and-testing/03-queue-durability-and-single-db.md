# Subtask 03 — Queue advancement + persisted resume + single backup DB (RC-4, RC-7, RC-8)

Repo: Antigravity-Manager. Spec: 01-agm-prompt-restore-spec.md
Status: PENDING. Owner: Worker wave 1.

## Goal
Restored queues always drain, pending resumes survive an app restart, and GUI/CLI share one backup database.

## Owned files
- src-tauri/src/modules/repo_db/resend.rs
- src-tauri/src/modules/instance/post_switch_resume.rs
- src-tauri/src/modules/backup_prompts_db/db.rs

## Steps
1. resend.rs: after a dispatch pass, if prompts were re-queued in place for a workspace (one-per-repo rule), loop additional passes while progress is being made (bounded, e.g. max 10 passes), instead of leaving the remainder to agy-exit drain alone. Preserve FIFO and the one-per-repo-per-pass rule inside each pass.
2. post_switch_resume.rs: persist scheduled resume tasks (target instance, backed_up count, deadline) to the app config/DB on schedule; on app start, re-arm any persisted task whose deadline has not passed and whose restore has not completed; clear the persisted record when the resume completes or is cancelled by a superseding switch.
3. backup_prompts_db/db.rs `get_backup_prompts_db_path`: remove the CWD-relative `data/backup-prompts/SQL.db` branch; always resolve the app-data-dir database so GUI and CLI share one store. If an existing CWD-relative DB is found, prefer migrating/opening the app-data one and log the choice via tracing (no silent divergence).

## Verification (deferred to subtask 08)
- Code re-read; E2E scenarios 2 and 4 (restart survival) in spec 03.
