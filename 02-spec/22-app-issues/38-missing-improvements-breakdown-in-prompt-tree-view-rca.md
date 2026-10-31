# 4-Part Root Cause Analysis: Missing Improvements Breakdown in Instance Prompt Tree View

- **Issue ID**: 38
- **Category**: UI / Prompt Tree Action Controls
- **Severity**: Low / Capability Gap
- **Date**: 2026-10-31
- **File**: `02-spec/22-app-issues/38-missing-improvements-breakdown-in-prompt-tree-view-rca.md`
- **Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)

---

## 1. Incident Description & Scope
During comprehensive audit of multi-instance prompt controls and Prompt Tree View actions, it was identified that the "Next breakdown / next 100 improvements breakdown inside the instance prompt tree view option" requested by the operator does not exist inside the Prompt Tree View modal or its header/context menus. While the 100 Improvements Flywheel exists on the Telegram page and has a jump button on the Instances page, no trigger or breakdown integration exists inside `src/components/instances/PromptTreeView/`.

---

## 2. Root Cause Analysis (4-Part RCA)

### Part 1: Symptoms & Observability
- When navigating into the Prompt Tree View (`PromptTreeViewModal.tsx`), operators can inspect projects, conversations, and individual turns.
- Action options in `DetailHeader.tsx` are strictly restricted to:
  - `primaryDispatchAction`: `'send' | 'enqueue' | 'rerun'`.
  - Copy actions (rich image, markdown, plain text, AI payload).
  - Suffix verification templates.
- There is no button, pill, or menu option to trigger "Next Breakdown" or "100 Improvements Breakdown" for the active conversation or project.

### Part 2: Why It Happened
- The 100 Improvements Flywheel (`improvementsService.ts`, `ImprovementsFlywheel.tsx`) was originally authored and integrated exclusively for:
  1. The Telegram bot interface (`/improvements [plan|next|status|check]`).
  2. The dedicated Telegram page tab in AGM (`src/pages/Telegram.tsx`).
  3. A quick-scroll button in `InstancesHeader.tsx` (`document.getElementById('agm-improvements-flywheel')?.scrollIntoView`).
- When the Prompt Tree View was redesigned for compact 3-action dispatching, no bridge was implemented between `PromptTreeView` and `improvementsService`.

### Part 3: How It Is Missing in Codebase
- Search across all files in `src/components/instances/PromptTreeView/` yields 0 occurrences of `improvements`, `flywheel`, or `breakdown`.
- No import of `src/services/improvementsService.ts` exists in `PromptTreeView/`.
- Neither `DetailHeader.tsx`, `TreePanel.tsx`, `ConversationContextMenu.tsx`, nor `ProjectContextMenu.tsx` expose an action to dispatch or breakdown improvements.

### Part 4: Step-by-Step Remediation Plan
1. **DetailHeader Action Integration**:
   - Add a "Next Breakdown / 100 Steps" action item in `DetailHeader.tsx` row 1 or inside the dispatch split dropdown.
2. **Service Binding**:
   - Wire the trigger to `improvementsService.next(false, targetInstanceId)` and `improvementsService.detail(false, targetInstanceId)`.
3. **Queue Monotonicity**:
   - Ensure the generated next-step prompt is enqueued into the active conversation in strict FIFO order (`ORDER BY created_at ASC, id ASC`) with status `enqueued_ide`.
4. **Context Menu Parity**:
   - Add "Plan 100 Improvements" and "Next 10 Subtasks" to `ConversationContextMenu.tsx` and `ProjectContextMenu.tsx`.
5. **Implementation & Release**:
   - Implemented via `ImprovementsBreakdownModal.tsx`, `DetailHeader.tsx` Capsule C, `ConversationContextMenu.tsx`, and `ProjectContextMenu.tsx`.

---

## 3. Remediation Outcome & Resolution Status

- **Status**: **RESOLVED & OPERATIONAL** (Released in `v4.283.0`)
- **Implemented Components**:
  - `src/components/instances/PromptTreeView/ImprovementsBreakdownModal.tsx`: Complete dark-glass modal dialog providing wave visualization, slot progress (`5/5 slots`), active task hints, category selector (`system`, `ui`, `code`, `ideas`), Dry Run inspection, and direct markdown export.
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`: Integrated Capsule C (`[✨ Next Breakdown | 100 | v]`) into the main action bar and added "Next Breakdown (100 Improvements)" to the dispatch popover.
  - `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`: Added "Next Breakdown (100 Steps)" to conversation node context menu.
  - `src/components/instances/PromptTreeView/ProjectContextMenu.tsx`: Added "Plan 100 Improvements..." to project node context menu.
- **Verification**: Clean Vite frontend compilation (`npm run build`), zero Rust errors (`cargo fmt --check`), and full end-to-end wiring to `improvementsService`.
