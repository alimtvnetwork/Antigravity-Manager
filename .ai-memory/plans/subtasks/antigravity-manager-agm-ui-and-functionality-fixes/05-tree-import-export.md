# Task-05: Tree import/export (SQLite + JSON)

## Status
PENDING (needs Task-01 gap report)

## Objective
Add copy-all, export, and import to the prompt tree view.

## Requirements
- "Copy all tree prompts" action on the tree view (right-hand side).
- Export section with:
  - SQLite: single `.db` file containing everything (prompts, structure,
    metadata).
  - JSON: full tree as JSON.
- Import section with:
  - Import from SQLite DB file.
  - Import from JSON file.
- One import/export section hosting all four operations (don't scatter).
- Import must validate the file and report real errors (no silent
  failures); export must confirm what was written and where.

## Acceptance
- [ ] Copy-all works.
- [ ] SQLite export/import round-trips without data loss.
- [ ] JSON export/import round-trips without data loss.
- [ ] `cargo fmt` + `clippy` clean; `npm run build` passes.

## Constraints
- GitMap search only. Relative paths only.
