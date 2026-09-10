# Specification Remediation Plan — Audit v2

> **Source Audit:** `02-spec/25-app-spec-audit/02-audit-2026-09-10-v2.md`
> **Plan Version:** 1.0.0
> **Status:** COMPLETED
> **Budget:** N = 200

---

## 1:1 Remediation Ledger

- [x] Finding [F-001]: File `02-spec/21-app/06-api-contracts-and-ipc-registry.md`, Issue: 101 Tauri IPC commands collapsed into summary rows without argument/return contracts, Remedy: Expand IPC registry with typed Serde JSON argument and return signatures for all registered domains.
- [x] Finding [F-002]: File `02-spec/21-app/01-index.md`, Issue: Only 3 of 27 mandatory coding guideline checklist topics bound in §6, Remedy: Author full 27-row normative binding table in `01-index.md` §6 linking to `02-spec/02-coding-guidelines/`.
- [x] Finding [F-003]: File `02-spec/23-app-db/01-index.md`, Issue: SQLite PRAGMAs (WAL mode, busy_timeout=5000, foreign_keys=ON) missing or inconsistent, Remedy: Standardize SQLite PRAGMAs across all three application database specifications.
- [x] Finding [F-004]: File `02-spec/21-app/07-testing-and-verification.md`, Issue: Cites invalid cargo package `antigravity-manager`, nonexistent `src-tauri/tests/`, and nonexistent cargo test target, Remedy: Correct cargo test command to `antigravity-tools`, align test paths with existing unit tests in `src-tauri/src/`, and add frontend testing specifications.
- [x] Finding [F-005]: File `02-spec/21-app/` and `02-spec/24-app-ui-design-system/01-index.md`, Issue: All 22 acceptance criteria written in abstract Gherkin without naming executable test function identifiers; invalid `npm run test` cited, Remedy: Bind criteria to concrete executable test function names and valid npm verification commands (`npm run build`, `tsc --noEmit`).
- [x] Finding [F-006]: File `.github/workflows/ci.yml` and `02-spec/12-cicd-pipeline-workflows/02-ci-pipeline.md`, Issue: CI pipeline omits `cargo test --workspace`; local runner lacks Rust/TS gates, Remedy: Add `cargo test --workspace` to CI and update `02-ci-pipeline.md` to reflect Tauri/Rust.
- [x] Finding [F-007]: File `02-spec/17-consolidated-guidelines/` (`16-app.md`, `19-app-design-system-and-ui.md`, `25-app-database.md`, `06-error-management.md`), Issue: Severe architectural drift in consolidated mirrors, Remedy: Synchronize consolidated mirrors with authoritative Antigravity-Manager v4.7.0 specifications.
- [x] Finding [F-008]: File `02-spec/21-app/05-frontend-ui-and-state-management.md`, Issue: Dual-mode request bridge lacks explicit mapping matrix for 127 Web vs 26 Tauri commands, Remedy: Document runtime capability matrix in `05-frontend-ui-and-state-management.md`.
- [x] Finding [F-009]: File `02-spec/21-app/05-frontend-ui-and-state-management.md`, Issue: Anti-garbage naming violation in form state (`tempApiKey`, `tempAdminPassword`), Remedy: Rename to `draftApiKey` and `draftAdminPassword`.
- [x] Finding [F-010]: File `02-spec/21-app/01-index.md`, Issue: Uppercase path casing in link to `../../AGENTS.md`, Remedy: Update link to lowercase strictly following lowercase rules.

---

## Subtasks Decomposition

1. `.lovable/plans/subtasks/13-spec-fix/01-fix-spec-index-and-guidelines.md` (F-002, F-003, F-010)
2. `.lovable/plans/subtasks/13-spec-fix/02-fix-ipc-registry-and-frontend.md` (F-001, F-008, F-009)
3. `.lovable/plans/subtasks/13-spec-fix/03-fix-testing-and-acceptance-criteria.md` (F-004, F-005)
4. `.lovable/plans/subtasks/13-spec-fix/04-fix-cicd-and-consolidated-mirrors.md` (F-006, F-007)
