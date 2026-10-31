# Master Plan: Anti-gravity Rerun and UI Fixes

## 1. Metadata
- **Task ID**: `231-anti-gravity-rerun-and-ui-fixes`
- **Parent Task**: `parent-231-anti-gravity-rerun-and-ui-fixes`
- **Slug**: `anti-gravity-rerun-and-ui-fixes`
- **Sequence**: `231`
- **Current Version**: `4.271.0`
- **Target Release Version**: `4.272.0` (Minor Bump)
- **Status**: `ACTIVE`
- **Attribution Invariant**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Path Hygiene**: Strictly repository-relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`)

---

## 2. Problem Statement & Root Cause Synthesis
1. **Rerun Target ID Loss & Deduplication Dropping**:
   - In `prompts_crud.rs`, a query `SELECT status FROM active_prompts WHERE (repo_path = ?1 ...) AND prompt_content = ?4` was matching previous completed prompts and silently dropping new rerun/enqueue requests.
   - In `PromptInstructionView.tsx`, `repoPath` fell back to empty string when `selectedProject` was null, and `PromptTreeViewModal.tsx` failed to pass `onRerun`/`onEnqueue` props, causing isolated un-synced execution.
2. **Immediate Multi-Round Rerun Queueing**:
   - Rerun multi-round batch (e.g. 4 rounds) must enqueue all rounds immediately into the SQLite queue and optimistically display all 4 queued rounds in the chat/tree without waiting 30 seconds or artificial backoff.
3. **Enqueue Button Refresh Bar Defect**:
   - `handleEnqueuePrompt`, `handleResendPrompt`, and `handleRerun` invoked `loadTree(true, true)` with `isInitialLoad = true`, mounting the 2px blue progress bar and spinning the header icon.
   - Remediation: Call `loadTree(false, true)` for seamless silent background sync.
4. **Tree View Filter Row Clipping**:
   - Row 2 required 451px, exceeding the default 340px / min 280px panel width, cutting off the rightmost controls (`Hide Auto` and the `<select>` dropdown).
   - Remediation: Compact left status capsule to icon-first with concise text (`Run`, `Queue`, `Pin`, Lucide `Image` icon) and right capsule to sleek icon-only buttons (`EyeOff`/`Eye`, `Archive` toggle button), reducing required width to ~190px (zero clipping).
5. **Enqueued Item Tree Visibility**:
   - Decouple `is_queued` from `is_inst_alive` in `tree_node_conv.rs` so queued prompts are never demoted to `COMPLETED`.
   - Update `list_all_prompts` in `project_queries.rs` to prioritize active/queued rows before old completed ones.
   - Inject optimistic turns into `treeData` on enqueue/rerun for 0ms immediate rendering.
6. **Tree View Design System in Coding Guidelines**:
   - Document full canonical design specification in `02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md` and link in `02-spec/24-app-ui-design-system/01-index.md`.

---

## 3. Subtask Decomposition Matrix
| Subtask ID | Code | Slug | Title | Assigned Agent | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `subtask-231-01` | `SUB-01` | `rerun-target-id-immediate-queue` | Rerun Target ID Routing & Immediate FIFO Queue Injection | Worker 01 | PENDING |
| `subtask-231-02` | `SUB-02` | `enqueue-refresh-bar-tree-visibility` | Enqueue Refresh Bar Suppression & Immediate Tree Visibility | Worker 02 | PENDING |
| `subtask-231-03` | `SUB-03` | `tree-filter-compaction-design-system` | Tree View Filter Row Compaction & Design System Guideline | Worker 03 | PENDING |

---

## 4. Verification & Release Protocol
1. **Pre-flight Checks**:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
2. **Automated E2E Test Suite**:
   - Run `powershell -ExecutionPolicy Bypass -File scripts/test-anti-gravity-rerun-and-ui-fixes-e2e.ps1`.
3. **Documentation & Release Ceremony**:
   - 4-Part RCA in `.ai-memory/rca/22-anti-gravity-rerun-and-ui-fixes.md` and `.ai-memory/issues/65-anti-gravity-rerun-and-ui-fixes.md`.
   - `npm run bump minor` -> `v4.272.0`.
   - Update `CHANGELOG.md` and `CHANGELOG_EN.md` (sole attribution to `@aukgit`).
   - Synchronize `README.md` and `README_EN.md` under `## 📝 Changelog`.
   - Update `.ai-memory/what-to-read.md`.
   - Move plan to `.ai-memory/plans/completed/`.
   - Atomic commit & push via `gitmap cpr` and tag `v4.272.0`.
