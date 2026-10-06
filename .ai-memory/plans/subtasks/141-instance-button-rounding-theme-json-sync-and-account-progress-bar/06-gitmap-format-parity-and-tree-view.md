# Subtask 06: GitMap Format Parity & Tree View Visualization

## Objective
Ensure project lists, pinned projects, and conversations strictly adhere to GitMap format (`[AGM:P001 | GM:#1]`, `[AGM:C001 | GM:<cid>]`) across GUI tree views and JSON serialization.

## Target Files
- `src/components/instances/PromptTreeViewModal.tsx`

## Verification
- Project badges render `[AGM:P001 | GM:#1]`.
- Conversation badges render `[AGM:C001 | GM:<cid>]`.
- Segmented buttons in `PromptTreeViewModal.tsx` toolbar adhere to the 4px outer rounding rule.
