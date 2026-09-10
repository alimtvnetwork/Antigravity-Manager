# Session: 2026-04-19 — Performance, Boolean Naming, Schema Descriptive Columns

**Status:** ✅ Done
**Recorded:** 2026-04-19

---

## Highlights

This was a long, single-day session that produced multiple linked deliverables across `spec/` and `linters-cicd/`. The work centered on three themes: **CI/CD performance & UX flags**, **boolean column naming overhaul**, and **mandatory descriptive columns**.

### linters-cicd evolution

- **v3.11.x → v3.12.0** — Performance spec implementation: middle-out walker, `--jobs N|auto`, `--check-timeout SECONDS` (default 20s), and `TOOL-TIMEOUT` synthetic SARIF result on overrun.
- **v3.12.0 → v3.13.0** — `--version` flag added to every check script via shared `_lib/cli.py` (uses `inspect.stack()` to derive `<rule-slug>` from caller's parent directory).
- **v3.13.0 → v3.14.0** — New `BOOL-NEG-001` check at `checks/boolean-column-negative/sql.py`. Flags `^(Is|Has)(Not|No)[A-Z]` column names inside `CREATE TABLE` bodies. Allow-list expanded over the day from 4 → 10 names: `IsDisabled`, `IsInvalid`, `IsIncomplete`, `IsUnavailable`, `IsUnread`, `IsHidden`, `IsBroken`, `IsLocked`, `IsUnpublished`, `IsUnverified`.

### Codegen tool (new)

- Created `linters-cicd/codegen/` implementing Rule 9's auto-inverted-field contract.
- Per-language regex parsers: Go (`db:""`), PHP (`#[Db('…')]`), TypeScript (`@DbField('…')`).
- Per-language emitters: Go methods, PHP traits, TS abstract-class getter mixins.
- Symmetric inversion table: `invert(invert(x)) == x`. Domain-specific pairs (`HasAccess` ↔ `IsUnauthorized`, `HasChildren` ↔ `IsSingle`) preferred over `Has-No-*` fallbacks.
- Outputs go to `<basename>.generated.<ext>` companion files; source never modified.

### Spec changes — `02-spec/04-database-conventions/03-naming-conventions.md`

- **v3.2.0 → v3.3.0** — Rule 2 clarification (single-negative roots like `IsDisabled` are NOT forbidden, only explicit `Not`/`No` prefixes are). Added Rule 9 (Auto-Generated Inverted Computed Fields) with code-generation contract.
- **v3.3.0 → v3.4.0** — Rule 8 restructured into a **three-bucket table** (Forbidden / Positive / 🟦 Approved Inverse of Positive). Reframed `HasNoAccess` → `IsUnauthorized`, `HasNoChildren` → `IsSingle`, `HasNoLicense` → `HasInvalidLicense`.
- **v3.4.0 → v3.5.0** — Added Rules 10/11/12 for descriptive free-text columns (`Description` on entity/reference tables; `Notes`+`Comments` on transactional tables; both nullable). Updated summary table and the `AgentSite`/`Transaction` SQL example.

### Spec changes — schema design + cross-links

- `02-spec/04-database-conventions/03-schema-design.md` v3.3.0 — added §6 "Mandatory Descriptive Columns" with table-category classification matrix.
- `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/01-index.md` — added "Database ↔ Code Inverse Pattern (Rule 9)" callout linking to the canonical Rule 9 spec, the codegen tool, and BOOL-NEG-001.
- `02-spec/02-coding-guidelines/01-cross-language/12-no-negatives.md` v2.2.0 — added "Database-Backed Inverses (Rule 9)" subsection.

### CI/CD spec

- Created `02-spec/02-coding-guidelines/06-cicd-integration/99-troubleshooting.md` covering python3 detection, tree-sitter install failures, SARIF size limits, false-positive triage, and TOML parse errors. Updated `00-overview.md` document inventory.

### Earlier in the same session

- Implemented FAQ features in code: inline `codeguidelines:disable=` parsing, `--baseline`, `--refresh-baseline`, `--rules`, `--exclude-rules`, `.codeguidelines.toml` config loading.
- Added `STYLE-099 SuppressionWithoutReason` synthetic finding for suppressions missing the em-dash reason.

---

## Key Verifications Performed

- BOOL-NEG-001 smoke test against an SQL fixture with 6 forbidden + 4 allow-listed names → 4 findings, exit 1, allow-list silently passed.
- Codegen smoke test against fixtures in `/tmp/cg-fix/` for all three languages → 2 Go types / 6 fields, 1 PHP class / 3 fields, 1 TS class / 3 fields. Symmetric inversion verified (e.g., `IsDisabled` → `IsEnabled`, `IsUnverified` → `IsVerified`).
- `--version` smoke test → `coding-guidelines/<rule-slug> 3.13.0` (and later `3.14.0`).
- Spec tree regenerated after every spec file change via `node scripts/sync-spec-tree.mjs` (568 files, 79 folders, 26 top-level entries).

---

## Files Touched (Highlights)

### Created

- `linters-cicd/checks/boolean-column-negative/sql.py`
- `linters-cicd/codegen/inverted_fields.py`
- `linters-cicd/codegen/inversion_table.py`
- `linters-cicd/codegen/parsers/{__init__,go_parser,php_parser,ts_parser}.py`
- `linters-cicd/codegen/emitters/{__init__,go_emitter,php_emitter,ts_emitter}.py`
- `linters-cicd/codegen/readme.md`
- `linters-cicd/scripts/emit-timeout.py`
- `02-spec/02-coding-guidelines/06-cicd-integration/99-troubleshooting.md`
- `01-prompts/03-write-prompt.md`
- `.lovable/memory/sessions/2026-04-19-perf-boolean-naming-schema.md` (this file)
- `.lovable/memory/avoid/01-avoid-per-task-folders.md`

### Modified

- `linters-cicd/VERSION` (3.11.x → 3.14.0)
- `linters-cicd/run-all.sh`
- `linters-cicd/checks/_lib/walker.py`
- `linters-cicd/checks/_lib/cli.py`
- `linters-cicd/checks/registry.json`
- `02-spec/04-database-conventions/03-naming-conventions.md` (v3.2.0 → v3.5.0)
- `02-spec/04-database-conventions/03-schema-design.md` (v3.2.0 → v3.3.0)
- `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/01-index.md`
- `02-spec/02-coding-guidelines/01-cross-language/12-no-negatives.md` (v2.1.0 → v2.2.0)
- `src/data/specTree.json` (regenerated multiple times)
- `.lovable/29-plan.md`, `.lovable/suggestions.md`, `.lovable/strictly-avoid.md`, `.lovable/memory/01-index.md`

### Removed (folder restructure)

- `.lovable/completed-tasks/`
- `.lovable/pending-tasks/`
- `.lovable/strictly-avoid/`
- `.lovable/suggestions/`

---

## Lessons / Patterns Captured

1. **Single-negative roots are not forbidden, double negatives are.** The user's mental model is: "`IsDisabled` is a real state, `IsNotEnabled` is a sentence." Reflect this in every linter and spec.
2. **Store one canonical column, derive the inverse in code.** Codegen makes this ergonomic without adding DB columns.
3. **Domain-specific inverses beat mechanical ones.** Prefer `IsUnauthorized` over `HasNoAccess`, `IsSingle` over `HasNoChildren`.
4. **Every entity table needs `Description`; every transactional table needs `Notes` + `Comments`.** All nullable. This absorbs unforeseen business needs without schema migrations.
5. **Always update both naming and schema-design specs together** when a column-level rule changes — the user reads naming-conventions first.

---

## What Next AI Session Should Pick Up

The pending plan items from `.lovable/29-plan.md` Active Work section are the natural next steps. The most impactful single task is **Wire codegen into CI** (`run-all.sh` + `git diff --exit-code`) so developers can never forget to regenerate companion files. Right after that: **Add unit tests for both BOOL-NEG-001 and the codegen inversion table**.
