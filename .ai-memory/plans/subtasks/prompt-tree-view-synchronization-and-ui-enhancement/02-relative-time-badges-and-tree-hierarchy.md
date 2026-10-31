# Subtask 02: Relative Time Badges and Tree Hierarchy Display

## 1. Overview
- **Subtask ID**: `02-relative-time-badges-and-tree-hierarchy`
- **Component**: `src/components/instances/PromptTreeView/`
- **Owner**: Frontend Worker
- **Status**: COMPLETED

## 2. Objectives
1. Implement `formatRelativeRecency(timestamp)` matching Antigravity IDE history:
   - `< 2m`: `now` (emerald pulsing dot)
   - `< 60m`: `${m}m` (purple dot)
   - `< 24h`: `${h}h` (purple dot)
   - `>= 24h`: `${d}d` (zinc dot)
2. Render relative recency badges on each conversation row in `PromptTreeItem.tsx`.
3. Display exact start, end, and duration in `DetailHeader.tsx` without mathematical skew.
4. Render top-level conversations as flat siblings under the project folder rather than nested sub-trees.

## 3. Verification Criteria
- [x] Visual confirmation of relative time badges (`formatRelativeRecency` in `formatters.ts` and `ConversationNode.tsx`).
- [x] Accurate start, end, and duration calculation rendered in `DetailHeader.tsx` and `PromptInstructionView.tsx`.
- [x] Flat hierarchy verified for unlinked conversations; tightened subagent classification in `tierClassification.ts`.
