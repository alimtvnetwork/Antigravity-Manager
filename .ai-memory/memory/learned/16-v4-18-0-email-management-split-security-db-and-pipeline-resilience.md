# Learned Memory: v4.18.0 Email Management, Split Security Vault DB, and Pipeline Resilience

> **Path:** `.ai-memory/memory/learned/16-v4-18-0-email-management-split-security-db-and-pipeline-resilience.md`
> **Topic:** Ingestion of repository identity, v4.18.0 architecture, last 10 git commits, email subsystem, split security DB, CODE RED rules, and CI/CD quality resilience
> **Date:** 2026-09-19
> **Status:** Active

---

## 1. Project Identity & Architecture

- **Project:** Antigravity Tools (AGM by Alim) / Antigravity-Manager (`alimtvnetwork/Antigravity-Manager`)
- **Version:** `4.18.0` (governed strictly by `version.json` at root as single canonical source of truth)
- **Primary Function:** Enterprise-grade AI account management, multi-instance isolation, and high-performance protocol proxy gateway for Antigravity (Google DeepMind agentic AI assistant / IDE).
- **Core Technology Stack:**
  - **Backend (Rust):** Tauri v2 (`src-tauri/`), Tokio runtime, Axum HTTP proxy engine (`axum 0.7`), Hyper, Reqwest, Rusqlite (`rusqlite 0.32` with WAL mode), Tracing subscriber, Lettre SMTP / IMAP inbound parsers, AES-256-GCM encryption.
  - **Frontend (TypeScript / React):** React 19, TypeScript 5.8, Vite 7, TailwindCSS 3.4, DaisyUI 5, Zustand 5, Lucide icons, i18next.
  - **Automation & Quality Gates (Python):** 34 specialized scripts under `03-ai-scripts/` powered by `02-shared-engine.py` (27 quality gates passing in `06-cicd-local-runner.py`).

---

## 2. Recent Git History & Architectural Intent (Last 10 Commits)

1. `b3327d3` - `fix(ci): align ModalDialog props and resolve rust match delimiter syntax error`:
   - *Files Changed:* `.ai-memory/cicd-issues/19-modal-props-and-rust-delimiter-syntax-error-rca.md`, `src-tauri/src/modules/email_inbound.rs`, `src/components/settings/EmailNotificationSettings.tsx` (+83, -43)
   - *Architectural Intent:* Aligns `<ModalDialog>` props to standard contract (`type="confirm"`, `onConfirm`, `onCancel`), cleans unused imports, and restores missing closing brace in Rust `email_inbound.rs` match arm.
2. `5663615` - `fix(email): enforce affirmative boolean naming and flatten conditionals`:
   - *Files Changed:* `.ai-memory/temp/recent-file-changes.json`, `src-tauri/src/modules/email_inbound.rs`, `src-tauri/src/modules/email_io.rs`, `src-tauri/src/modules/email_watcher.rs` (+51, -55)
   - *Architectural Intent:* Enforces repository affirmative boolean naming (`is*`, `has*`) and flattens nested conditionals to comply with Rule 1 of `AGENTS.md`.
3. `82a3fbc` - `feat(email): implement named prompt search and dispatch for inbound mailbox`:
   - *Files Changed:* `.agents/skills/email-management/skill.md`, `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`, `src-tauri/src/modules/email_inbound.rs`, `src-tauri/src/modules/email_sender.rs`, `src-tauri/src/modules/repo_db.rs` (+191, -0)
   - *Architectural Intent:* Introduces search and direct injection of named prompts from running workspace repos upon receiving inbound email commands.
4. `efad89f` - `docs(spec): index email dispatch spec in 21-app and enqueue in what-to-read`:
   - *Files Changed:* `.ai-memory/what-to-read.md`, `02-spec/21-app/01-index.md` (+3, -0)
   - *Architectural Intent:* Formally registers Spec 16 (`email-dispatch-mailbox-remote-management-and-split-security-db.md`) into reading priority orders.
5. `ef39656` - `fix(auto_switcher): dispatch email notice just before workspace switch`:
   - *Files Changed:* `src-tauri/src/modules/auto_switcher.rs` (+4, -4)
   - *Architectural Intent:* Ensures outbound notification dispatch occurs immediately prior to initiating the automated profile transition.
6. `3f57851` - `fix(email): enhance backup/restore to include split passwords db and clarify ui text`:
   - *Files Changed:* `src-tauri/src/modules/email_io.rs`, `src/components/settings/EmailNotificationSettings.tsx` (+27, -5)
   - *Architectural Intent:* Guarantees that split security credential vault (`email_passwords.db`) is included during full configuration backup and restore.
7. `f4ebdb8` - `docs(ai-memory): add recent-file-changes, test-inventory manifest, runner-eta, and update plan 25 loop header`:
   - *Files Changed:* `.agents/skills/cg-enum-standards/skill.md`, `.ai-memory/plans/completed/25-email-management-split-security-db-and-remote-control.md`, `.ai-memory/temp/*`, `.ai-memory/test-inventory.json` (+33, -3)
   - *Architectural Intent:* Records test inventory and runner telemetry tracking structures.
8. `433c7dd` - `docs(plan-25): clean verbatim prompt markdown formatting`:
   - *Files Changed:* `.ai-memory/plans/completed/25-email-management-split-security-db-and-remote-control.md` (+4, -4)
   - *Architectural Intent:* Markdown formatting cleanups.
9. `7a0a5e4` - `feat(email): complete split passwords db, excel import, bidirectional receipts, and consolidate subtasks (plan 25)`:
   - *Files Changed:* 18 files (+856, -341)
   - *Architectural Intent:* Completes Plan 25 delivery, including Excel/CSV import/export, bidirectional HTML receipts, and subtask consolidation.
10. `d5a3a31` - `feat: implement email dispatch, mailbox remote management, split security vault db, and remote control (plan 25)`:
    - *Files Changed:* 27 files (+4105, -2)
    - *Architectural Intent:* Core architectural implementation of the email notification and remote control subsystem.

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
  - SQLite engines configured with mandatory pragmas: `PRAGMA journal_mode = WAL;`, `PRAGMA busy_timeout = 5000;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA foreign_keys = ON;`.
  - Active Databases:
    1. `proxy_logs.db` (`request_logs`): Inbound request telemetry, token usage, latency.
    2. `security.db` (`ip_access_logs`, `ip_blacklist`, `ip_whitelist`): IP filtering, CIDR rules, access logs.
    3. `user_tokens.db` (`user_tokens`, `token_ip_bindings`, `token_usage_logs`): Multi-user authentication & curfew.
    4. `email_vault.db` (`email_accounts`, `notify_recipients`, `email_notification_settings`, `email_inbound_audit_log`): Mailbox configuration and audit telemetry.
    5. `email_passwords.db` (`email_credentials`): Isolated AES-256-GCM encrypted passwords and OpenSSH RSA identities.
    6. `repo_prompts.db` (`running_prompts`): Workspace prompts backup and dispatch cache.
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
- `cicd-issues/`: 20 resolved RCAs (RCAs 01 through 20 fully analyzed and documented).
- `ambiguous-questions/01-new-ambiguity/`: 1 open ambiguity (`01-pluggable-logger-backend-and-uber-zap-migration.md`).
- `ambiguous-questions/02-ambiguity-resolved/`: 0 answered questions on file.
