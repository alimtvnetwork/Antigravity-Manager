# 4-Part Root Cause Analysis & Architectural Gap Audit

- **Task**: Instance-Based Prompt and UI Functionality Verification
- **Slug**: `instance-based-prompt-and-ui-functionality-verification`
- **Spec ID**: `241`
- **File**: `02-spec/21-app/241-instance-based-prompt-and-ui-functionality-verification/03-rca-and-gap-analysis.md`
- **Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Negative Invariant**: Under Rule 13, NO code modifications or implementation edits are applied in this turn. This document delivers the forensic Root Cause Analysis, architectural gap breakdown, and complete engineering implementation specification.

---

## 1. Incident & Missing Capability Description

During comprehensive audit and verification of the AGM Instance Prompt Tree View and multi-instance prompt controls, 7 of the 8 required capabilities were verified as fully implemented, hardened, and operational in production:
1. ✅ **Instance-based prompt sending & queued prompts visibility** (verified via `cdp_onboarding.rs`, `tree_node_conv.rs`, `usePromptTreeData.ts`).
2. ✅ **Copy Prompt, Send Prompt, Send Button UI, Enqueue & Rerun Buttons** (verified via `DetailHeader.tsx`, `usePromptTreePromptActions.ts`).
3. ✅ **Canonical Image Attachment Pattern** (`[Title | Slug | Seq](file;<path>)` & `@[<path>]` / `Attachments:\n1. @[...]`).
4. ✅ **Candidate Account Synchronization Without Selecting Existing Account** (verified via `selection.rs`, `extract_urgent_expiration_hours`, cluster exclusion).
5. ✅ **GitMap Machine Information Check on Startup** (verified via `resolve_local_node_alias` reading `~/.gitmap/variables.json`).
6. ✅ **Current Machine Self-Registration in Cluster Fleet** (verified via `build_local_and_ssh_fleet`, `is_local = true` primacy).
7. ✅ **Auto-Switching Prompt Restoration onto Current Conversation** (verified via `backup_running_prompts_for_instance`, 10s delay, `restore_running_prompts_after_switch`).

However, Requirement 12 — **"Next breakdown, next 100 improvements breakdown inside the instance prompt tree view option"** — was identified as **MISSING from the Instance Prompt Tree View UI**.

---

## 2. Forensic Root Cause Analysis (4-Part RCA)

### Part 1: Incident Description & Gap Manifestation
- While `improvementsService.ts` exists in `src/services/improvementsService.ts` (exposing `plan`, `next`, `status`, `check`, `detail`) and `ImprovementsFlywheel.tsx` is mounted on the Telegram page with a header jump button (`document.getElementById('agm-improvements-flywheel')?.scrollIntoView(...)`) in `src/pages/instances/InstancesHeader.tsx`, **the Prompt Tree View (`src/components/instances/PromptTreeView/`) possesses zero bindings, controls, or context menu items to trigger the Next Breakdown or Next 100 Improvements directly from an active conversation or turn**.
- An operator inspecting conversation prompts in Prompt Tree View has no direct entry point to plan 100 improvements, claim the next 10-batch subtasks, or detail out improvements for the active project/workspace without exiting the tree view, navigating back to Instances, or using the external CLI (`agm improvements`).

### Part 2: Why It Is Missing (Origin & Commit Divergence)
1. **Isolated Component Scoping**: The 100 Improvements Flywheel was originally authored as a standalone Telegram integration and full-page widget under `src/components/telegram/ImprovementsFlywheel.tsx` (supported by `src-tauri/src/modules/improvements.rs`).
2. **Context Scope Gap**: When `DetailHeader.tsx` and `DetailTabs.tsx` were refactored for the compact 3-action dispatch menu (Send, Enqueue, Rerun) and verification suffix templates, the improvements flywheel was not plumbed into the Prompt Tree action pipeline.
3. **Workspace Path Decoupling**: In `improvementsService.ts`, `status`, `plan`, and `next` were keyed to `instanceId`, but Prompt Tree View operates primarily on `selectedConversation.project_path` and `selectedConversation.conversation_id`. The glue code bridging conversation context to the flywheel ledger (`.ai-memory/improvements/flywheel.json`) in the specific project repository was never mounted into the tree view component tree.

### Part 3: How It Is Missing (Codebase Inspection & Evidence)
- **Search Evidence**: Running `gitmap aum search` across all 46 files in `src/components/instances/PromptTreeView/` for:
  - `"improvements"` → 0 hits.
  - `"flywheel"` → 0 hits.
  - `"breakdown"` → 0 hits.
- **Action Menu Audit**: In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
  - `primaryDispatchAction` supports only `'send' | 'enqueue' | 'rerun'`.
  - The dispatch dropdown (`handleSwitchDispatchMode`) exposes only Send, Enqueue, and Rerun.
  - No button or dropdown item exists for "Next Breakdown" or "100 Improvements".
- **Context Menus**: Neither `ConversationContextMenu.tsx` nor `ProjectContextMenu.tsx` contains an action item to dispatch `/improvements plan` or `/improvements next`.

### Part 4: Step-by-Step Remediation Plan (How to Complete Without Breaking Existing UI)

To implement this capability cleanly when authorized, the following steps must be followed:

1. **Plumb Improvements Trigger into `DetailHeader.tsx`**:
   - Add a "Next Breakdown / 100 Steps" action item inside the secondary split-dropdown menu of `DetailHeader.tsx` (alongside Send, Enqueue, Rerun).
   - Alternatively, mount a compact pill trigger in `DetailHeader.tsx` Row 1 next to the verification template selector:
     ```tsx
     <button
       type="button"
       onClick={() => handleTriggerImprovementsBreakdown()}
       className="agm-hover-shade flex items-center gap-1 px-2.5 py-1 rounded-full border border-amber-500/30 bg-amber-500/10 text-amber-300 text-xs font-medium"
       title="Breakdown Next 100 Improvements for active conversation"
     >
       <Sparkles className="w-3 h-3 text-amber-400" />
       <span>Next Breakdown</span>
     </button>
     ```

2. **Connect to `improvementsService`**:
   - In `DetailHeader.tsx` or `usePromptTreePromptActions.ts`, import `improvementsService` from `../../../services/improvementsService`.
   - On click, execute `improvementsService.next(false, targetInstanceId)` or `improvementsService.detail(false, targetInstanceId)`.
   - Forward the generated next improvement prompt directly into the prompt dispatch engine, enqueueing it into the active conversation in FIFO order with affirmative status `enqueued_ide`.

3. **Context Menu Parity**:
   - Add "Plan 100 Improvements" and "Implement Next 10 Improvements" into `ConversationContextMenu.tsx` and `ProjectContextMenu.tsx`.
   - These menu items should resolve the project's repository root path (`selectedConversation.project_path`) and invoke the corresponding Rust backend commands (`improvements_plan`, `improvements_next`).

4. **Add Flywheel Progress Modal in Tree View**:
   - Mount a lightweight, dark-glass modal (`ImprovementsBreakdownModal.tsx`) inside `PromptTreeViewModal.tsx` that displays current wave, batch count (e.g. `Batch 1/10`), active ledger status, and allows 1-click dispatching of the next batch.

---

## 3. Implementation Outcome & Resolution Status

Following direct maintainer authorization ("Do it please don't miss it"):
- The 4-step remediation blueprint above was systematically executed:
  1. `ImprovementsBreakdownModal.tsx` created and mounted inside `PromptTreeView`.
  2. Capsule C (`[✨ Next Breakdown | 100 | v]`) and popover option integrated in `DetailHeader.tsx`.
  3. Context menus updated in `ConversationContextMenu.tsx` ("Next Breakdown (100 Steps)") and `ProjectContextMenu.tsx` ("Plan 100 Improvements...").
  4. Fully wired to `improvementsService` (`detail`, `plan`, `next`) with FIFO queue enqueueing.
- Pre-flight checks (`cargo fmt --check`, `npm run build`) passed with 0 errors.
- Version bumped and released under `v4.283.0`.
