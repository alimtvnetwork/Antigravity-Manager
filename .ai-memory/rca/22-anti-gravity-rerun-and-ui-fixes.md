# RCA-22: Anti-gravity Rerun Target ID, Sequential Enqueue, and Tree View Compaction

- **Date:** 2026-10-27
- **Severity:** High
- **Scope:** Prompt Queue, Rerun Dispatch, Tree View UI, and GitMap Native CLI
- **Affected Systems:** `src-tauri/src/modules/repo_db/`, `src/components/instances/PromptTreeView/`, `.agents/skills/`

---

## 1. Incident Summary & User Impact

1. **Rerun Target ID Loss:** When rerunning conversations, the target conversation UUID was lost or degraded to `'current'`, causing prompts to enqueue into the wrong conversation session.
2. **Sequential Enqueue Dropped After First Time:** When a prompt was enqueued, subsequent clicks on "Enqueue" (or multi-round reruns) were silently dropped or failed to show up in the tree view.
3. **Unwanted UI Refresh Bar on Enqueue:** Clicking "Enqueue" triggered a top 2px indeterminate progress bar and spinning refresh icon, breaking the seamless 0ms UI flow.
4. **Right-Hand Tree Filter Clipping:** Row 2 filters (`[ 🚫 Hide Auto ]`, `[ 📦 Archived (0) ]`) required 451px and overflowed standard 328px sidebars, clipping the right-hand controls.
5. **GitMap Autonomous CLI Violation:** The agent executed ambient `python -c 'import sqlite3...'` strings in Windows PowerShell, which failed with exit code 1 due to quote escaping instead of using native compiled Go commands (`gitmap agent task init`).

---

## 2. Root Cause Analysis (4-Part RCA)

### Root Cause 1: Backend Live-Status Dedup in `save_or_requeue_prompt` Swallowed Subsequent Enqueues
- **Location:** `src-tauri/src/modules/repo_db/prompts_crud.rs`
- **Mechanism:** When a prompt was enqueued the first time, its status was quickly updated to `dispatched` or `running`. When the user clicked "Enqueue" a second time, `save_or_requeue_prompt` searched `active_prompts` by `prompt_content` in an `.or_else()` fallback. Because the first prompt was `dispatched` or `running`, `live_status` evaluated to active, and the function logged `"Preserving live prompt ... skipping overwrite with incoming 'queued'"` and returned `Ok(())` without executing `INSERT OR REPLACE INTO active_prompts`.
- **Impact:** Any second enqueue or multi-round rerun with identical content while a prompt was in-flight was silently dropped from SQLite.

### Root Cause 2: Tree Node Converter Collapsed Queued Prompts into Historical Turns
- **Location:** `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- **Mechanism:** In `build_active_prompt_conv_nodes`, the converter queried `target_conv.prompts.iter_mut().find(|p| p.prompt_text.trim() == ap.prompt_content.trim())`. If the conversation already had a turn with that text (e.g. from `transcript.jsonl` or a previous turn), it mutated the existing turn in place and never appended a new turn (`target_conv.prompts.push(ap_turn)`).
- **Impact:** In the UI, the tree view never rendered a new queued turn when enqueuing existing prompts or second rounds; subsequent enqueued items appeared to vanish.

### Root Cause 3: Frontend Missing Dispatch Props & Target ID Degradation
- **Location:** `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, `PromptInstructionView.tsx`
- **Mechanism:** `<PromptInstructionView />` was not receiving `conversationTargetMode`, `newConversationTitle`, `confirmationSuffix`, `onRerun`, `onEnqueue`, or `onResend` props from `PromptTreeViewModal`. When falling back to internal handlers, `selectedProject?.repo_path` defaulted to empty string and conversation IDs dropped to `'current'`.
- **Impact:** Rerun failed to target the authoritative UUID, and dispatch mode state was unsynchronized between header and body.

### Root Cause 4: Invasive SWR Invalidation Triggered Skeleton / Progress Bar
- **Location:** `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`, `DetailHeader.tsx`
- **Mechanism:** `handleEnqueuePrompt`, `handleResendPrompt`, and `handleRerunConversationAction` called `loadTree(true, true)` with `isInitialLoad = true`. This activated `setIsLoading(true)`, mounting the top 2px blue progress bar and full skeleton loader.
- **Impact:** Annoying visual stutter and progress bar flash on every enqueue action.

### Root Cause 5: Horizontal Layout Overflow (451px in 328px Envelope)
- **Location:** `src/components/instances/PromptTreeView/TreePanel.tsx`
- **Mechanism:** Filter Row 2 used verbose text labels (`All Running`, `Queued`, `Pinned`, `Attached Images`) and a bulky `<select>` dropdown (`[ 🚫 Hide Automated ]`), requiring 451px. In a 280px minimum container, this created a 171px deficit, clipping right-hand buttons.
- **Impact:** Filter controls were visually cut off and inaccessible on compact screens.

---

## 3. Surgical Remedies & Architectural Invariants

| Component | Remedy Applied | Invariant Enforced |
| :--- | :--- | :--- |
| **`prompts_crud.rs`** | Exempted queued prompts: `&& prompt.status != "queued"`. Queued prompts are distinct append actions with unique IDs and are never dropped. | **Sequential Enqueue Invariant:** `status == 'queued'` is never dropped by in-flight status matching. |
| **`tree_node_conv.rs`** | Gated `existing_turn` search to `if is_ap_run && !is_queued`. Queued prompts always append a fresh `AgmPromptTurnNode` with `status: "QUEUED"`. | **Queued Turn Append Invariant:** Every queued row in SQLite represents a distinct queued turn in the conversation tree. |
| **`PromptInstructionView.tsx`** | Added 0ms optimistic turn injection into `executeDirectEnqueue` and wired all dispatch props (`conversationTargetMode`, `newConversationTitle`, `confirmationSuffix`). | **Optimistic UI Invariant:** Enqueued items appear immediately at 0ms latency in the tree view. |
| **`usePromptTreePromptActions.ts`** | Replaced `loadTree(true, true)` with `loadTree(false, true)`. Silent background revalidation without `isLoading = true`. | **Silent SWR Invariant:** Background tree synchronization never mounts top indeterminate progress bars. |
| **`TreePanel.tsx`** | Compacted Row 2 into two segmented capsules using icon-first labels (`Run`, `Queue`, `Pin`, Lucide `Image`) and icon toggles (`EyeOff`/`Eye`, Lucide `Archive` with count badge). Total width: 194px. | **Zero-Clipping Invariant:** Filter controls strictly consume $\le 195\text{ px}$, maintaining $\ge 70\text{ px}$ margin at $280\text{ px}$. |
| **GitMap Agent Task CLI** | Banned ambient `python -c "import sqlite3..."`. Switched to native compiled Go command `gitmap agent task init --name <slug> --budget 400` (<16ms). | **GitMap Command Replacement Matrix:** Strict adherence to rows 28, 41, 42 of `.agents/skills/gitmap/skill.md`. |

---

## 4. Verification Evidence & Test Proof

Executed dedicated automated test suite `scripts/test-anti-gravity-rerun-and-ui-fixes-e2e.ps1`:
- **Stage 0:** Pre-flight checks passed; Caller IDE session (`v3-7391`, PID 3496) preserved with immunity.
- **Stage 1:** Transient sandbox created and isolated (`test-agm-test-rerun-231-*`).
- **Stage 2:** Rust architectural invariants verified (`prompts_crud.rs`, `tree_node_conv.rs`, `project_queries.rs`).
- **Stage 3:** Frontend compilation verified with 0 TypeScript errors (`dist/index.html` built via Vite in 18.53s).
- **Stage 4:** Row 2 layout budget calculated at 194px ($\le 195\text{ px}$), with $+70\text{ px}$ safety margin at $280\text{ px}$ width (zero clipping).
- **Stage 5:** Target conversation UUID preserved ($018d9b56... \ne \text{'current'}$); 4 rounds enqueued in 139ms (<500ms limit, 0ms throttle); strict FIFO ordering verified; sequential enqueue while in-flight preserved (`count = 2`).
- **Stage 6:** Enqueue refresh bar suppressed via `loadTree(false, true)`; 0ms optimistic turn injection active; 60ms delay eliminated.
- **Stage 7:** Zero-orphan teardown complete; caller IDE untouched.
- **Result:** `Total: 28 | Passed: 28 | Failed: 0` (100% Green).
