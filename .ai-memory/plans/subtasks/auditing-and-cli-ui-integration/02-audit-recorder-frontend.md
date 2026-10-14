# Task-02: Interaction audit recorder — frontend

## Status
PENDING

## Objective
Wire the recorder into the React frontend: capture interactions, show
recording state, and display/export the audit trail.

## Requirements
- A global interaction hook/listener that captures clicks, navigation,
  inputs, and scrolls while recording is active, reporting
  `method_name` (the handler invoked), `ui_target` (route + component +
  element id), and `trigger` to the backend commands from Task-01.
- Recording indicator in the UI (follow AGENTS.md UI conventions:
  contextual placement, pill/badge styling — discover existing patterns
  via GitMap search, don't invent new visual language).
- Audit panel/modal listing entries (filterable by actor/result), with
  JSON/JSONL export for bug reports.
- "No Silent Errors": frontend failures go through `src/stores/error-store.ts`
  (`captureError`/`trackWarning`) — never bare `catch {}`.
- Follow the 4-tier prompt origin conventions where relevant; reuse
  existing design tokens.

## Acceptance
- [ ] Interactions captured with all fields while recording.
- [ ] Indicator visible; panel lists/filters/exports entries.
- [ ] No unhandled promise rejections; errors tracked.

## Constraints
- GitMap search only. Relative paths only.
- Disjoint from Task-01's files (backend module is Worker 01's).
