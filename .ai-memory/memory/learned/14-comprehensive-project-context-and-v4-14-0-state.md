# Comprehensive Project Context & v4.14.0 Ingestion Memory

> **Path:** `.ai-memory/memory/learned/14-comprehensive-project-context-and-v4-14-0-state.md`
> **Topic:** Ingestion of repository identity, last 10 git commits, CODE RED rules, coding guidelines, error philosophy, active plans, and runtime compatibility
> **Date:** 2026-09-17
> **Status:** Active

---

## 1. Project Identity & Architecture

- **Project:** Antigravity Tools / Antigravity-Manager (`alimtvnetwork/Antigravity-Manager`)
- **Version:** `4.14.0` (governed by `version.json` at root as single canonical source of truth)
- **Primary Function:** Enterprise-grade AI account management and high-performance protocol proxy gateway for Antigravity (Google DeepMind agentic AI assistant / IDE).
- **Core Technology Stack:**
  - **Backend (Rust):** Tauri v2 (`src-tauri/`), Tokio runtime, Axum HTTP proxy engine (`axum 0.7`), Hyper, Reqwest, Rusqlite (`rusqlite 0.32` with WAL mode), Tracing subscriber.
  - **Frontend (TypeScript / React):** React 19, TypeScript 5.8, Vite 7, TailwindCSS 3.4, DaisyUI 5, Zustand 5, Lucide icons, i18next.
  - **Automation & Quality Gates (Python):** 34 specialized scripts under `03-ai-scripts/` powered by `02-shared-engine.py` (27 quality gates passing in `06-cicd-local-runner.py`).

---

## 2. Recent Git History & Architectural Intent (Last 10 Commits)

1. `7a0365d` - `test(proxy): increase security db benchmark assertions to 120s for runner virtualized I/O tolerance`:
   - *Files Changed:* `src-tauri/src/proxy/tests/security_ip_tests.rs` (+2, -2)
   - *Architectural Intent:* Broadens benchmark timeout thresholds to prevent test flakiness under shared CI virtual runner I/O latency.
2. `8b04e85` - `docs(plans): document completed plan 21 for v4.14.0 release`:
   - *Files Changed:* `.ai-memory/plans/01-index.md` (+1), `.ai-memory/plans/completed/21-ubuntu-ide-diagnostics-and-multi-instance-cloning.md` (+75)
   - *Architectural Intent:* Formally records completion of Plan 21 covering Ubuntu IDE discovery diagnostics and multi-instance cloning.
3. `42d40fc` - `fix(compiler): resolve str::contains args and AppError string conversion in backend`:
   - *Files Changed:* `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/process.rs`, `src-tauri/src/proxy/server.rs` (+6, -4)
   - *Architectural Intent:* Resolves Rust 2021 compiler type inference issues on `str::contains` string slice arguments and explicit `AppError` conversions.
4. `df1cb70` - `style: apply rustfmt formatting across rust modules`:
   - *Files Changed:* `src-tauri/src/commands/mod.rs`, `src-tauri/src/error.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/process.rs` (+41, -42)
   - *Architectural Intent:* Enforces uniform Rust style conventions via `cargo fmt`.
5. `ffab2a4` - `chore(release): normalize release notes line endings to UNIX LF`:
   - *Files Changed:* `03-ai-scripts/29-release-orchestrator.py` (+1, -1)
   - *Architectural Intent:* Normalizes generated release notes to strict Unix LF to pass encoding linters.
6. `c97dea8` - `release: v4.14.0 Ubuntu IDE discovery diagnostics, multi-instance isolation, and executable cloning`:
   - *Files Changed:* 29 files (+706, -161)
   - *Architectural Intent:* Release v4.14.0 introducing Ubuntu IDE detection, backtrace capture, multi-instance isolation, executable cloning, and synchronized version metadata.
7. `62a3bbc` - `fix(ci): synchronize version.json, exclude target dir in file size guard, and add v4.13.0 release notes`:
   - *Files Changed:* 5 files (+89, -9)
   - *Architectural Intent:* Resolves CI check failures by excluding `target/` from `03-ai-scripts/02-shared-engine.py` and syncing `version.json`.
8. `b494887` - `docs(plans): consolidate plan 20 subtasks into completed archive`:
   - *Files Changed:* 5 files (+61, -181)
   - *Architectural Intent:* Archives completed subtasks of Plan 20 into `.ai-memory/plans/completed/`.
9. `8233868` - `feat(release): v4.13.0 - Ubuntu IDE discovery, storage auto-healing, and English README overhaul`:
   - *Files Changed:* 16 files (+819, -417)
   - *Architectural Intent:* Ships release v4.13.0 with automated storage.json repair and full English README overhaul.
10. `47c8802` - `docs(cicd): add RCA for TS2503 JSX namespace and TS6133 unused variables`:
    - *Files Changed:* `.ai-memory/cicd-issues/09-ts-jsx-namespace-and-unused-vars-rca.md` (+42)
    - *Architectural Intent:* Documents Root Cause Analysis for TypeScript JSX namespace compilation issues.

---

## 3. CODE RED Rules & Strict Avoidances (Justifying Files)

1. **No Explicit True Checks (TOTAL BAN):** Never evaluate booleans against `true` (`if isReady == true`). Always evaluate implicitly (`if isReady`). Justified in `AGENTS.md` (Rule 1) and `.ai-memory/strictly-avoid.md`.
2. **No Mixed Polarity (TOTAL BAN):** Never combine positive and negative checks in the same condition (`if isA && !isB`). Split into separate guard clauses. Justified in `AGENTS.md` (Rule 1) and `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/04-parameters-and-conditions.md`.
3. **Never Disable CI/CD (TOTAL BAN):** Never bypass, comment out, or disable CI/CD steps or linters. Justified in `AGENTS.md` (Rule 2) and `.ai-memory/strictly-avoid.md`.
4. **Strict Lowercase File Naming (TOTAL BAN on uppercase):** All files and scripts must use strictly lowercase naming (e.g. `readme.md`). Justified in `AGENTS.md` (Rule 4) and `.ai-memory/strictly-avoid.md`.
5. **Strict Relative Git Paths Mandate (TOTAL BAN on Absolute Paths / `file:///` URIs):** Never write absolute paths or `file:///` URIs in any repository files, plans, or documentation. Justified in `AGENTS.md` (Rule 5) and `.ai-memory/strictly-avoid.md`.
6. **Go Error Handling Standard (`*appfault.AppError`):** Functions returning structured error metadata in Go must use `*appfault.AppError`. Justified in `AGENTS.md` (Rule 6), `.ai-memory/strictly-avoid.md`, and `.ai-memory/memory/learned/12-go-cli-apperror-and-dry-help-handling.md`.
7. **`readme.txt` Timestamp Generator (TOTAL BAN):** Never build, suggest, or discuss timestamp generators targeting `readme.txt`. Justified in `.ai-memory/strictly-avoid.md`.
8. **No Commit of Generated Test Artifacts or Binaries:** `.gitignore` must exclude `.test-report.*`, `.exe`, `.dll`, `tmp/`. Justified in `.ai-memory/strictly-avoid.md`.
9. **No Release on Every Commit:** Only perform releases when the user explicitly commands it. Justified in `.ai-memory/strictly-avoid.md`.
10. **Spec/19 Implementation (TOTAL BAN):** Repo is spec-only for `02-spec/19-main-worker-service/`. Justified in `.ai-memory/strictly-avoid.md`.

---

## 4. Coding & Database Conventions

- **Boolean Identifiers:** Strictly prefixed with `is` or `has` (`isActive`, `hasPermission`). Negative prefixes (`isNot`, `no`, `without`) and auxiliary verbs (`can`, `should`, `was`, `did`) are prohibited.
- **Acronym Casing:** PascalCase `Id`, `Ip`, `Url` (never `ID`, `IP`, `URL`).
- **Database Schema Conventions:**
  - Database engine: SQLite via `rusqlite` with WAL mode.
  - Connection pragmas: `PRAGMA journal_mode = WAL;`, `PRAGMA busy_timeout = 5000;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA foreign_keys = ON;`.
  - Table naming: PascalCase or snake_case matching schema spec (`request_logs`, `ip_access_logs`, `user_tokens`).
  - Columns: Primary keys named `id` or `{Table}Id`.
- **Universal Response Envelope:** `{ data, errors[], meta }` / `{ Status, Attributes, Results }`.

---

## 5. Active Plans & Pending Tasks

### Pending Plans (`.ai-memory/plans/pending/`):
1. `04-guideline-prompt-and-installer-upgrade.md`: Guideline prompt and installer enhancements (subtasks in `subtasks/03-guideline-prompt-and-installer-upgrade/`).
2. `09-update-prompts-and-release.md`: Update prompts from meta-repo and release (deferred under WOR policy).
3. `11-code-red-refactor-remediation.md`: Remediate Code Red enum, boolean, and query wrapper violations.

### Active Subtask Batches (`.ai-memory/plans/subtasks/`):
- `03-guideline-prompt-and-installer-upgrade/`: 4 subtasks (`01-format-overview-prompt.md`, `02-upgrade-installer-scripts.md` [done], `03-generate-50-improvements.md`, `04-release.md`).
- `04-reverse-engineering/`: 3 subtasks (`01-proxy-core-and-handlers.md`, `02-modules-storage-and-security.md`, `03-frontend-ui-and-state.md`).
- `17-cicd-automation/`: 3 subtasks (`01-clean-stale-tasks-and-links.md`, `02-align-quality-gates-matrix.md`, `03-verify-local-runner-and-git-commit.md`).

### Issues & Ambiguities:
- `issues/`: 0 active bugs logged.
- `cicd-issues/`: 10 resolved RCAs (RCAs 01 through 10 fully analyzed and documented).
- `ambiguous-questions/01-new-ambiguity/`: 1 open ambiguity (`01-pluggable-logger-backend-and-uber-zap-migration.md`).
- `ambiguous-questions/02-ambiguity-resolved/`: 0 answered questions on file.
