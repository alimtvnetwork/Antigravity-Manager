# E2E run index — `20261019a`

Task: `antigravity-manager-package-repository-update-and-prompt-functionality-testing` (spec 154).
Evidence contract: spec §3 (dual-Id: `#P<seq>` + `E2E154-<runid>-<instance>-<caseNN>`).

`<instance>` is the test-instance slug from `agm instances test-create --json`
on the execution machine; substitute it into every evidence Id at run time.

## Verdict table

| Case | Evidence Id | Folder | Prompt Ids | Verdict | Verification class |
|------|-------------|--------|------------|---------|--------------------|
| E2E-154-01 | `E2E154-20261019a-<instance>-01` | `01-backup-multi-image/` | `#P001` | INCOMPLETE — no display / toolchain / binary on this VM | CI-verified-by-review + machine-verified (pending) |
| E2E-154-02 | `E2E154-20261019a-<instance>-02` | `02-restore-byte-identical/` | `#P001` | INCOMPLETE — same | CI-verified-by-review + machine-verified (pending) |
| E2E-154-03 | `E2E154-20261019a-<instance>-03` | `03-panel-running-first-next4/` | `#P010`–`#P016` | INCOMPLETE — panel rendering needs a running app | CI-verified-by-review + machine-verified (pending) |
| E2E-154-04 | `E2E154-20261019a-<instance>-04` | `04-rerun-with-image/` | `#P002` → `#P002-R1` | INCOMPLETE — dispatch needs a running app | CI-verified-by-review + machine-verified (pending) |
| E2E-154-05 | `E2E154-20261019a-<instance>-05` | `05-resend-cli-ui-parity/` | `#P003` | INCOMPLETE — UI + binary needed | CI-verified-by-review + machine-verified (pending) |
| E2E-154-06 | `E2E154-20261019a-<instance>-06` | `06-queue-overflow/` | `#P020`–`#P027` | INCOMPLETE — panel rendering needs a running app | CI-verified-by-review + machine-verified (pending) |
| E2E-154-07 | `E2E154-20261019a-<instance>-07` | `07-empty-queue/` | (none queued) | INCOMPLETE — rendering needs a running app | CI-verified-by-review + machine-verified (pending) |

No case is marked PASS: per the evidence contract, a case without all five
bundle parts is INCOMPLETE, and no part was executed on this VM.

## Statically verified on this VM (CI-verified-by-review)

- **SV-1 — IPC contract shape** (`src-tauri/src/commands/cmd_running_up_next.rs`):
  `get_running_and_up_next(instance_id: Option<String>)` (line 130) returns
  `RunningAndUpNext { now_running, up_next, up_next_total, instance_id, as_of_ms }`
  (lines 32–44); `PromptQueueRow { id, instance_id, status, origin_tier, created_at, has_images }`
  (lines 19–29). Matches spec 01 §6.1 exactly. Registered in
  `src-tauri/src/lib/run.rs:351`.
- **SV-2 — FIFO ordering**: panel queries `ORDER BY created_at ASC, id ASC`
  (`cmd_running_up_next.rs:79,98`); backup reads same order (`backup.rs:285,290,294`);
  restore reads `ORDER BY created_at ASC, sequence_id ASC` (`restore.rs:58,69,80`);
  `VwQueuedPromptsFifo` orders `CreatedAt ASC, ActivePromptId ASC` (`views.rs:142`).
  All `DESC` orderings audited — listings/history/retention-prune only, none in
  dispatch logic. No LIFO violation.
- **SV-3 — hook polling/TTL** (`src/components/instances/PromptTreeView/useRunningAndUpNext.ts`):
  3s poll while mounted; 30s TTL → stale, 90s → drop, never on a single missed
  poll; no re-sort (backend order verbatim); manual errors → `captureError`,
  background → `trackWarning`, last-good retained; immediate refresh on
  `prompt://dispatched` / `prompt://resumed`; `{ instanceId }` arg naming matches
  the working repo convention (not a bug).
- **Bugs found by static review: none.** No src/ changes made (per subtask boundary).

## Machine-run checklist (shared pre/post flight)

```sh
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cargo build --bin agm
agm instances test-create --json        # -> <instance>
# ... per-case steps in each folder's notes.md ...
agm instances test-clean
```
