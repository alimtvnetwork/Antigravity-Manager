# Learned Memory: v4.17.0 Architecture, SQLite Tool Signatures, Test Resilience, and .ai-memory Ingestion

> **Path:** `.ai-memory/memory/learned/15-v4-17-0-architecture-sqlite-tool-signatures-and-test-resilience.md`
> **Topic:** Ingestion of repository identity, v4.17.0 release state, SQLite L2 tool signatures, 17 CI/CD RCAs, last 10 git commits, and CODE RED rules
> **Date:** 2026-09-18
> **Status:** Active

---

## 1. Project Identity & Architecture

- **Project:** Antigravity Tools / Antigravity-Manager (`alimtvnetwork/Antigravity-Manager`)
- **Version:** `4.17.0` (governed by `version.json` at root as single canonical source of truth)
- **Primary Function:** Enterprise-grade AI account management and high-performance protocol proxy gateway for Antigravity (Google DeepMind agentic AI assistant / IDE).
- **Core Technology Stack:**
  - **Backend (Rust):** Tauri v2 (`src-tauri/`), Tokio runtime, Axum HTTP proxy engine (`axum 0.7`), Hyper, Reqwest, Rusqlite (`rusqlite 0.32` with WAL mode), Tracing subscriber.
  - **Frontend (TypeScript / React):** React 19, TypeScript 5.8, Vite 7, TailwindCSS 3.4, DaisyUI 5, Zustand 5, Lucide icons, i18next.
  - **Automation & Quality Gates (Python):** 34 specialized scripts under `03-ai-scripts/` powered by `02-shared-engine.py` (27 quality gates passing in `06-cicd-local-runner.py`).

---

## 2. Recent Git History & Architectural Intent (Last 10 Commits)

1. `6caad74` - `refactor(spec): update relative spec cross-references and script links`:
   - *Files Changed:* 403 files (+2587, -2587)
   - *Architectural Intent:* Updates relative markdown links and cross-references across all specs and scripts following the `.lovable` to `.ai-memory` structural reorganization.
2. `31daf04` - `docs(release): preserve v4.16.0 in readme changelog and update releaseDate to 2026-09-18`:
   - *Files Changed:* `readme.md`, `version.json` (+7, -2)
   - *Architectural Intent:* Preserves changelog history and aligns release dates.
3. `e3cd2b4` - `release: v4.17.0 Test Suite Resilience, SQLite L2 Tool Signatures, and Gemini 3.7 Flash Routing`:
   - *Files Changed:* 19 files (+107, -47)
   - *Architectural Intent:* Formal release of v4.17.0 bringing test resilience, signature caching, and model routing.
4. `4d5bca5` - `fix(tests): preserve sample_log byte size and adjust prompt_log test payload to 2048`:
   - *Files Changed:* `src-tauri/src/proxy/monitor.rs` (+6, -6)
   - *Architectural Intent:* Prevents response body truncation under simple storage mode while preserving sample byte allocations for retention tests.
5. `e49d6a1` - `fix(tests): isolate signature cache sqlite persistence, size prompt log response, and fix 3.7 flash canonical resolution`:
   - *Files Changed:* 6 files (+79, -8)
   - *Architectural Intent:* Implements `clear_tool_signatures()` and ensures canonical `gemini-3.7-flash` model resolution.
6. `d94291c` - `style(tests): format security test setup return types for rustfmt compliance`:
   - *Files Changed:* 4 files (+71, -3)
   - *Architectural Intent:* Enforces strict `cargo fmt` formatting on security test return types.
7. `ce5f2ab` - `fix(tests): resolve test data dir concurrency, token manager isolation, and canonical DTO alignment`:
   - *Files Changed:* 10 files (+203, -13)
   - *Architectural Intent:* Hardens test concurrency isolation using `TestDataDir::new()` and `TEST_TOKEN_DB_MUTEX`.
8. `ce2d319` - `chore(format): align account.rs test mutex formatting with rustfmt standard`:
   - *Files Changed:* `src-tauri/src/modules/account.rs` (+3, -1)
   - *Architectural Intent:* Rustfmt line formatting alignment.
9. `30c0aa5` - `fix(tests): resolve test isolation, dynamic variant mapping, and image defense assertions`:
   - *Files Changed:* 6 files (+89, -64)
   - *Architectural Intent:* Fixes test isolation and assertions across proxy handlers and mappers.
10. `afd7aa6` - `fix(proxy): fix user_enabled_thinking in openai mapper and harden linux dependency installation`:
    - *Files Changed:* 4 files (+16, -11)
    - *Architectural Intent:* Resolves `user_enabled_thinking` field handling in OpenAI mapper and CI workflow hardening.

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
  - Tables: `request_logs` (proxy_logs.db), `ip_access_logs`, `ip_blacklist`, `ip_whitelist` (security.db), `user_tokens`, `token_ip_bindings`, `token_usage_logs` (user_tokens.db), `tool_signatures` (proxy_db.rs).
- **Universal Response Envelope:** `{ data, errors[], meta }` / `{ Status, Attributes, Results }`.

---

## 5. Active Plans & Pending Tasks

### Pending Plans (`.ai-memory/plans/pending/`):
1. `04-guideline-prompt-and-installer-upgrade.md`: Guideline prompt and installer enhancements (subtasks in `subtasks/03-guideline-prompt-and-installer-upgrade/`).
2. `09-update-prompts-and-release.md`: Update prompts from meta-repo and release (deferred under WOR policy).
3. `11-code-red-refactor-remediation.md`: Remediate Code Red enum, boolean, and query wrapper violations.

### Active Subtasks (`.ai-memory/plans/subtasks/`):
- `03-guideline-prompt-and-installer-upgrade/`: 4 subtasks (`01-format-overview-prompt.md`, `02-upgrade-installer-scripts.md` [done], `03-generate-50-improvements.md`, `04-release.md`).
- `04-reverse-engineering/`: 3 subtasks (`01-proxy-core-and-handlers.md`, `02-modules-storage-and-security.md`, `03-frontend-ui-and-state.md`).
- `17-cicd-automation/`: 3 subtasks (`01-clean-stale-tasks-and-links.md`, `02-align-quality-gates-matrix.md`, `03-verify-local-runner-and-git-commit.md`).

### Issues & Ambiguities:
- `issues/`: 0 active bugs logged.
- `cicd-issues/`: 17 resolved RCAs (RCAs 01 through 17 fully analyzed and documented).
- `ambiguous-questions/01-new-ambiguity/`: 1 open ambiguity (`01-pluggable-logger-backend-and-uber-zap-migration.md`).
- `ambiguous-questions/02-ambiguity-resolved/`: 0 answered questions on file.
