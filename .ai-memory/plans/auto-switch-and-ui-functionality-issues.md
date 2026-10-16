# Master Plan: Auto-Switch and UI Functionality Issues

- **Slug**: `auto-switch-and-ui-functionality-issues`
- **Specification Directory**: `02-spec/21-app/auto-switch-and-ui-functionality-issues/`
- **Subtasks Directory**: `.ai-memory/plans/subtasks/auto-switch-and-ui-functionality-issues/`
- **Benchmark Branch**: `backup/auto-switch-and-ui-functionality-issues-benchmark`
- **Target Version**: `4.208.0` (Minor Bump)

---

## 1. High-Level Objectives

1. **Auto-Switch & Quota Synchronization Hardening**:
   - Repair `fetch_quota_summary` in `fetch_more.rs` to retry without `project_id` on 403, preventing false fallback to 100%.
   - Implement active 30s–50s background quota polling for the running instance's bound account in `status.rs` daemon loop.
   - Emit `accounts://refreshed` event in `daemon.rs` when quotas update so the frontend Zustand store reflects live numbers immediately.
   - Include account quota refresh in `useInstancePage.ts` polling cycle.
2. **Fast-Forward Manual Trigger Repair**:
   - Resolve blocking timeouts in `switch_account_to_instance` and unblock candidate selection from false-positive active lease exclusions.
3. **Subprocess Window Suppression (`CREATE_NO_WINDOW`)**:
   - Apply `CREATE_NO_WINDOW` (`0x08000000`) across all Windows subprocess spawn sites in `ssh_manager`, `telegram_inbound`, and `repo_db/tree_format.rs`.
4. **Interaction Audit Log Copy & Metadata Repair**:
   - Implement true clipboard copying in `AuditPanel.tsx` for `JSON` and `JSONL` with visual `Copied!` feedback.
   - Add `#[serde(rename_all = "camelCase")]` to `InteractionEntry` in `src-tauri/src/modules/audit/types.rs`.
   - Fix IPC parameter key contract `{ input: ... }` in `audit-client.ts`.
   - Enhance `useAuditRecorder.ts` to climb to parent interactive elements, capture caller function names, and record stack traces.
   - Record auto-switch evaluation decisions and rotation results into the audit database via `audit::record_if_active`.
5. **Prompt Backup Cap (Default 100) & Settings Control**:
   - Implement `enforce_max_backup_prompts_cap` in `backup_prompts_db` using FIFO order (`ORDER BY created_at DESC, id DESC LIMIT ?1`).
   - Add `max_backup_prompts: usize` (default 100) to `AppConfig`.
   - Add a numeric configuration input in Settings UI (`AdvancedMaintenanceSection.tsx`).
6. **Strict Email Masking**:
   - Eliminate hover raw email leaks in `InstanceCard.tsx`.
   - Standardize `maskEmail` to `ris***@...` format in `maskEmail.ts`.
   - Enforce backend `mask_email` in `tracing` loggers and Telegram alerts.
7. **Updater Polish**:
   - Replace `group-hover:animate-bounce` in `NavMenu.tsx` with smooth micro-scale transition.
   - Set 18pt font and enable Virtual Terminal Processing in `delegate_updater/run.rs`.
8. **E2E Testing & Release Ceremony**:
   - Author and run automated integration tests verifying quota refresh, auto-switch trigger, audit copy, and backup cap.
   - Pre-flight checks (`cargo fmt`, `cargo check`, `npm run build`), bump minor version `v4.208.0`, synchronize changelogs and READMEs with strictly `@aukgit` attribution, and commit/push via `gitmap cpf`.

---

## 2. Decomposed Subtasks

- `01-quota-sync-and-auto-switch-fix.md`
- `02-process-window-suppression-create-no-window.md`
- `03-audit-log-json-copy-and-callstack-enrichment.md`
- `04-max-backup-prompts-cap-and-settings.md`
- `05-strict-email-masking-and-updater-polish.md`
- `06-e2e-verification-bump-and-release.md`
