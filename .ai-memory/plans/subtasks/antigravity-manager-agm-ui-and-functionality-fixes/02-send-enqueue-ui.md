# Task-02: Send/Enqueue UI — dropdown + polish

## Status
PENDING (needs Task-01 gap report)

## Objective
Implement the Send-button dropdown with the Enqueue option and polish the
prompt-tree action area to a professional standard.

## Requirements
- Send button gets a dropdown; Enqueue lives inside it as a distinct action.
  (Check Task-01: build only what's missing.)
- Professional polish of the action area — reviewer's discretion, following
  AGENTS.md UI conventions (segmented pill capsules, dark-glass styling,
  existing design tokens).
- **DO NOT touch the progress bar.** Explicit user order.
- Every button must give real feedback (no silent no-ops) — errors via
  the error store.

## Acceptance
- [ ] Dropdown present and functional; Enqueue distinct from Send.
- [ ] UI looks professional; progress bar untouched.
- [ ] `npm run build` passes (frontend changes).

## Constraints
- GitMap search only. Relative paths only.
- Owns PromptTreeView UI files — coordinate with Task-04 (lead arbitrates).
