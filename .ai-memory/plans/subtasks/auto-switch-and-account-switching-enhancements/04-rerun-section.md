# Task-04: Rerun section in prompts tree

## Status
PENDING (needs Task-01)

## Objective
Add a rerun section in the prompts tree send area.

## Requirements
- Rerun count selector (user picks N, e.g. 2–3).
- Message options: "is it done?", "is it published?", or randomized
  messages from a small built-in set (clearly labeled as randomized in
  the UI).
- Dispatches the chosen message N times through the existing dispatch
  path (no forked logic).
- Each rerun is audit-logged; failures are visible (no silent errors).
- UI follows AGENTS.md conventions (professional, consistent tokens).

## Acceptance
- [ ] Count selector + message options present and functional.
- [ ] N dispatches occur via the canonical path.
- [ ] `npm run build` passes.

## Constraints
- GitMap search only. Relative paths only.
