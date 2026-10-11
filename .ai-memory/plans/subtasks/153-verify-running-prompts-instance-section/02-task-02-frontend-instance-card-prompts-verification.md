---
plan: 153-verify-running-prompts-instance-section
subtask: "02"
title: Frontend Instance Card Prompts Badging and Projects Section Verification
domain: frontend-react
status: verified
evidence:
  - "isNodeOwnedByInstance verified to strictly scope projects to matching instance configs"
  - "hasActiveTask verified in useInstanceCardData and useInstancePage with genuine turn validation"
  - "InstanceCard header verified to render pulsing emerald [● Prompt] button when hasActiveTask is true"
  - "InstanceCardProjects verified to sort running projects to the top with [● RUNNING] pill badge"
  - "23 automated test assertions passing in src/pages/instances/__tests__/instancerunningprompts.test.ts"
verified_invariants:
  - "Invariant I5 (Instance Ownership Scoping): Default aliases match default config only; clones match strictly by ID/seq_num"
  - "Invariant I6 (Card Header Active Task Badging): [● Prompt] badge renders if and only if instance is running and has genuine active turn"
  - "Invariant I7 (Running Project Elevation): Actively running projects sort strictly to index 0 in the projects section"
  - "Invariant I8 (Ghost Filtering Parity): Untitled conversations with 0 words never trigger hasActiveTask or running pill"
citations:
  architecture_spec: 02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md
  root_cause_analysis: 02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md
target_files:
  - src/pages/instances/instancecard.tsx
  - src/pages/instances/instancecardactions.tsx
  - src/pages/instances/instancecardprojects.tsx
  - src/pages/instances/instancecarddata.ts
  - src/pages/instances/useinstancepage.ts
  - src/pages/instances/instancepageutils.ts
  - src/pages/instances/__tests__/instancerunningprompts.test.ts
---

# Subtask 02: Frontend Instance Card Prompts Badging and Projects Section Verification

## 1. Context & Objectives

The instance management page serves as the mission control dashboard for multi-instance Antigravity operations. When prompts are actively running inside an IDE instance, the corresponding instance card must clearly communicate this state to the user without cross-profile bleed or false positives.

Key verification objectives:
1. Verify `isNodeOwnedByInstance` correctly associates projects and conversation nodes strictly to the owning instance configuration.
2. Verify `hasActiveTask` calculation in `useInstanceCardData` requiring both an active host process and a genuine running turn.
3. Verify visual rendering of the pulsing emerald `[● Prompt]` button in the `InstanceCard` header.
4. Verify `InstanceCardProjects` sorting behavior, elevating active running projects to the top and displaying the green `[● RUNNING]` pill badge.
5. Verify automated test execution and regression coverage in `src/pages/instances/__tests__/instancerunningprompts.test.ts`.

---

## 2. Verified Architectural Invariants

### Invariant I5: Per-Instance Ownership Scoping
- **Rule**: A project tree node must only be attributed to an instance if its `instance_id` matches the instance configuration.
- **Implementation**:
  - In `src/pages/instances/instancepageutils.ts` (lines 47-80):
    ```typescript
    export const isNodeOwnedByInstance = (
        node: AgmProjectTreeNode,
        instConfig: { id: string; is_default?: boolean; seq_num?: number }
    ): boolean => {
        if (!node.instance_id || !node.instance_id.trim()) return false;
        if (instConfig.is_default) {
            return isDefaultOwned(node.instance_id, instConfig.id);
        }
        if (node.instance_id === instConfig.id) return true;
        if (instConfig.seq_num && node.instance_seq_num === instConfig.seq_num) return true;
        return false;
    };
    ```
  - Cloned instances (e.g. `default-copy-8159`) reject default aliases (`"default"`, `"__default__"`), preventing default profile prompts from appearing in cloned cards.

### Invariant I6: Card Header Active Task Badging
- **Rule**: The pulsing emerald `[● Prompt]` button appears in the card header if and only if `hasActiveTask` is `true`.
- **Implementation**:
  - In `src/pages/instances/instancecarddata.ts` (lines 76-87):
    ```typescript
    const hasActiveTask =
        Boolean(inst.is_running) &&
        runningTreeNodes.some((node) => {
            const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
            const hasGenuineRunningConv = (node.conversations || []).some(
                (c) =>
                    Boolean(c.is_running) &&
                    !isGhostConversation(c) &&
                    !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            return isInstanceMatch && Boolean(node.is_running) && hasGenuineRunningConv;
        });
    ```
  - In `src/pages/instances/instancecard.tsx` (lines 123-133): Renders:
    ```tsx
    {hasActiveTask && (
        <button
            type="button"
            onClick={() => openPromptTree(inst.config.id)}
            className="inline-flex items-center gap-1 px-1.5 h-5 rounded-[5px] text-[9px] font-bold bg-[#1af18d]/15 hover:bg-[#1af18d]/25 text-emerald-700 dark:text-[#1af18d] border border-[#1af18d]/30 shrink-0 cursor-pointer transition-colors shadow-2xs"
            title="Active prompt/task running - Click to open Prompt Tree"
        >
            <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] shadow-[0_0_4px_rgba(26,241,141,0.9)] animate-pulse shrink-0" />
            <span>Prompt</span>
        </button>
    )}
    ```

### Invariant I7: Running Project Elevation in Projects Section
- **Rule**: Actively running projects are sorted to index 0 of the "Active / Recent Projects" list.
- **Implementation**:
  - In `src/pages/instances/instancecardprojects.tsx` (lines 14-23):
    ```typescript
    const sortedProjects = [...instanceProjects].sort((a, b) => {
        const isARunning = Boolean(inst.is_running) && Boolean(a.is_running);
        const isBRunning = Boolean(inst.is_running) && Boolean(b.is_running);
        if (isARunning && !isBRunning) return -1;
        if (isBRunning && !isARunning) return 1;
        const aLatest = Math.max(0, ...(a.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        const bLatest = Math.max(0, ...(b.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        if (bLatest !== aLatest) return bLatest - aLatest;
        return a.repo_name.localeCompare(b.repo_name);
    });
    ```
  - Renders the pulsing green `[● RUNNING]` pill badge alongside turn counts (lines 72-77).

### Invariant I8: Ghost Conversation Filter
- **Rule**: Untitled conversations with 0 prompt words and empty preview do not trigger `hasActiveTask` or elevate a project.
- **Implementation**:
  - Validated by `isGhostConversation` in `src/components/instances/prompttreeview/conversationguards.ts`.

---

## 3. Test Suite Verification & Results

Automated unit tests were executed via the project test runner:
```bash
node scripts/run-frontend-tests.mjs instanceRunningPrompts
```

**Results Summary**:
- **Test File**: `src/pages/instances/__tests__/instancerunningprompts.test.ts`
- **Total Tests**: 23 test cases
- **Passed**: 23
- **Failed**: 0
- **Duration**: ~200ms
- **Outcome**: 100% Passing Gate Guarantee

The tests empirically verify:
1. `isDefaultOwned` and `isNodeOwnedByInstance` correctly handle default profiles, cloned profiles, custom UUIDs, and sequential fallbacks.
2. `sortInstanceProjects` places running projects first when the instance is running, but safely respects dead-process gating when the instance is stopped.
3. `checkHasActiveTask` strictly requires all three conditions: live instance process, node instance ownership, and genuine active conversation.
4. `isGhostConversation` and `isStaleOrEmptyConversation` accurately classify untitled empty conversations while exempting active turns.

---

## 4. Target Files & Relative References

- `src/pages/instances/instancecard.tsx` — Instance card component rendering header badging and section layout.
- `src/pages/instances/instancecardactions.tsx` — Action buttons toolbar including the Prompt Tree launcher.
- `src/pages/instances/instancecardprojects.tsx` — Active / Recent Projects list with running badge and turn counts.
- `src/pages/instances/instancecarddata.ts` — Card state derivation hook computing `hasActiveTask`.
- `src/pages/instances/useinstancepage.ts` — Instance dashboard page state manager.
- `src/pages/instances/instancepageutils.ts` — Instance ownership predicates (`isNodeOwnedByInstance`, `isDefaultOwned`).
- `src/pages/instances/__tests__/instancerunningprompts.test.ts` — Automated test suite verifying instance running prompts logic.
- `02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md` — Specification reference.
- `02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md` — Root cause analysis and matrix.
