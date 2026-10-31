# Parent Plan: 154-antigravity-manager-package-repository-update-and-prompt-functionality-testing

- Repo: `alimtvnetwork/Antigravity-Manager` (Tauri desktop app, "Antigravity Manager Tools")
- Baseline: `main` @ `c7efb910` (pulled 2026-10-19; tree clean at plan time)
- Spec: `02-spec/21-app/154-antigravity-manager-package-repository-update-and-prompt-functionality-testing/` (`01-architecture-spec.md`, `02-component-and-e2e-spec.md`)
- Subtasks: `.ai-memory/plans/subtasks/154-antigravity-manager-package-repository-update-and-prompt-functionality-testing/` (01–04)
- Run ledger: `.ai-memory/temp-agents/antigravity-manager-package-repository-update-and-prompt-functionality-testing/ledger.md`
- Task DB: `task-20261018184744-antigravity-manager-package-repository-update-and-prompt-functionality-testing` (budget 300)
- Status: Phase 1 COMPLETE (research + specs) → Phase 2 execution

## Goal (user's non-negotiable task)
1. Latest package repo — DONE via `git pull` (de835afe → c7efb910).
2. Prompts backup & restore with images works perfectly — verify e2e, fix defects.
3. Prompt tree view shows the currently-running first prompt + next four queued prompts.
4. Rerun / re-send the same prompt with an image.
5. Every test carries its prompt `Id` + screenshot/image evidence.
6. One atomic commit + push at the end (`gitmap cpf`).

## Key findings driving the plan
- "Now Running + Up Next" panel does NOT exist — new UI (spec §1).
- `extract_image_payload_or_path` captures only the FIRST `data:image/` URI — prime backend fix (subtask 01).
- Rerun/resend image attachment on the rerun path is unverified — must prove re-injection (subtask 04, E2E-154-04).
- Ambiguities A1–A10 resolved in specs; A4 resolved as dual-Id contract (`#P<seq>` prompt Id + `E2E154-<runid>-<instance>-<caseNN>` evidence Id).
- House rules: FIFO `ORDER BY created_at ASC, id ASC`; 4-tier badges; no silent errors; relative paths; pre-flight (cargo fmt/clippy, npm build).

## Subtasks & ownership (disjoint)
| Code | Subtask file | Owner | Owned files |
|---|---|---|---|
| Task-01 | 01-backend-queue-backup-restore.md | Worker 1 | `src-tauri/src/modules/repo_db/{detection,queue,agy,backup,schema}.rs`, `src-tauri/src/modules/backup_prompts_db/*`, `src-tauri/src/commands/cmd_queue_resend.rs` (+ new IPC file) |
| Task-02 | 02-panel-data-contracts.md | Worker 2 | `src/components/instances/PromptTreeView/useRunningAndUpNext.ts` (+ optional `src/stores/up-next-store.ts`); defines strip props contract |
| Task-03 | 03-tree-panel-ui.md | Worker 2 | `src/components/instances/PromptTreeView/NowRunningUpNext.tsx` (new), edits to `PromptTreeViewModal.tsx`, `DetailHeader.tsx` (rerun flyout), `ResendAsModal.tsx` |
| Task-04 | 04-e2e-test-execution.md | Wave 2 | `02-spec/21-app/154-…/assets/e2e-<runid>/` only; execution, no src/ changes; bugs recorded not fixed |

Lead owns: specs, this plan, indexes (`02-spec/21-app/readme.md`, `.ai-memory/plans/readme.md`), ledger, final `gitmap cpf` + push. Workers NEVER run git commands.

## Waves
- Wave 1: Worker 1 → Task-01; Worker 2 → Task-02 + Task-03 (contract between 02/03 fixed in specs; parallel safe).
- Wave 2: Task-04 e2e execution (after 01–03 done).
- Phase 3: consolidate, push-gate checks (targeted linters, secrets gate, .gitignore hygiene), one atomic `gitmap cpf "prompt-tree - <summary>"` + push.

## Acceptance (top-level)
- Panel shows running-first + next-4 FIFO with Ids + 4-tier badges; empty/overflow/stale states correct.
- Backup→restore image round-trip byte-identical (SHA256 per image); multi-image prompts fully captured.
- Rerun / re-send with image re-injects the image (proven, not assumed).
- All 7 E2E cases have complete 5-part evidence bundles with canonical Ids.
- Pre-flight gates pass; tree pushed clean.

## Completion log (2026-10-19)
- Task-01 (backend): DONE — multi-image extraction fix (`extract_image_uris_or_paths`), session_id preservation, RESEND_IN_PROGRESS double-dispatch guard + unit tests, `get_running_and_up_next` IPC (`cmd_running_up_next.rs`); lead registered the IPC in `commands/mod.rs` + `lib/run.rs`. FIFO audit clean.
- Task-02 (data hook): DONE — `useRunningAndUpNext.ts` (3s poll, TTL 30s stale / 90s drop, `prompt://dispatched|resumed` event refresh, error-store routing).
- Task-03 (strip UI): DONE — `NowRunningUpNext.tsx` (running-first, next-4 + `+N more`, 4-tier badges, Id per row, a11y), modal mount, DetailHeader rerun flyout attach-image, ResendAsModal include-image toggle. `npm run build` exit 0.
- Task-04 (e2e): DONE — evidence tree `02-spec/21-app/154-…/assets/e2e-20261019a/` (7 cases, index.md); all cases INCOMPLETE-or-static (no display / Rust toolchain / agm binary on build VM); exact machine commands recorded; nothing faked.
- Design deviation (logged): rerun dispatches validated file path, not base64 — follow-up: `write_image_attachment` IPC.
- Lead reversions: `package-lock.json` regenerated churn reverted (out of scope).
- Verified: diffstat matches worker reports; secrets gate clean (1 pre-existing `<redacted>` placeholder); no absolute paths in new files; node_modules/dist gitignored.
