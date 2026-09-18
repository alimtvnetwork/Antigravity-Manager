# Antigravity-Manager Workspace Onboarding & Architecture Memory

> **Path:** `.ai-memory/memory/learned/11-antigravity-manager-workspace-onboarding.md`
> **Topic:** Repository identity, Tauri v2 desktop architecture, proxy engine, SQLite schemas, and recent git history
> **Ingested:** 2026-09-10
> **Status:** Active

---

## 1. Project Identity & Architecture

- **Project:** Antigravity-Manager (`lbjlaq/Antigravity-Manager`)
- **Version:** `4.7.0` (synced across `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `Casks/antigravity-tools.rb`, `readme.md`)
- **Primary Function:** Cross-platform desktop application and reverse proxy / account manager for Antigravity (Google DeepMind agentic AI coding assistant).
- **Core Technology Stack:**
  - **Backend (Rust):** Tauri v2 (`src-tauri/`), Tokio asynchronous runtime, Axum HTTP proxy engine (`axum 0.7`), Hyper, Reqwest, Rusqlite (`rusqlite 0.32` with WAL mode), Tracing subscriber.
  - **Frontend (TypeScript / React):** React 19, TypeScript 5.8, Vite 7, TailwindCSS 3.4, Zustand 5, Lucide icons, i18next multi-language support.

---

## 2. Polyglot Metrics & Topology

- **Rust:** 154 source files (`src-tauri/src/`)
- **TypeScript:** 93 source files (`src/`)
- **Markdown:** 30 documentation files (`docs/`, root docs)
- **HTML:** 3 files
- **JavaScript:** 2 manifest/config files
- **CSS:** 1 file
- **Python:** 1 utility script
- **Total Local Project Source Files:** ~284 files

---

## 3. SQLite Database Schemas & Storage Engines

The application leverages local SQLite databases via `rusqlite` with WAL (`Write-Ahead Logging`) mode and 5000ms busy timeout:

1. **Proxy Logs (`proxy_logs.db`):**
   - `request_logs`: Primary monitoring table tracking `id`, `timestamp`, `method`, `url`, `status`, `duration`, `model`, `error`, `request_body`, `response_body`, `input_tokens`, `output_tokens`, `cached_tokens`, `account_name`, `provider`.
2. **Security & Access Control (`security_db.rs`):**
   - `ip_access_logs`: Tracking inbound client requests, IP address, timestamp, method, path, user-agent, status, duration, API key hash, blocked status, and reason.
   - `ip_blacklist`: Banned IP patterns with expiration timestamps and hit counts.
   - `ip_whitelist`: Approved IP patterns with descriptions.
3. **Token & Quota Statistics (`token_stats.rs`, `user_token_db.rs`):**
   - Per-account token tracking, 5-hour quota windows, weekly quota buckets, and circuit breaker rate limiting.

---

## 4. Recent Git History & Architectural Intent (Last 10 Commits)

1. `85fb4fe`: docs: collapse troubleshooting, advanced image generation, contributors, and special thanks in READMEs.
2. `9f3a93f`: chore(release): synchronize version 4.7.0 in READMEs, Casks, and frontend fallbacks.
3. `8e1bb30`: fix(proxy): expose `Retry-After` header on temporary 503 responses (fixes #3414).
4. `953d3a8`: chore(release): bump version to 4.7.0 and update changelog.
5. `e4b7e87`: Merge PR #3415: fix(proxy): scope `sessionId` per conversation and bump on upstream 1M accumulation.
6. `6753072`: Merge PR #3413: feat(circuit-breaker): add lock on zero quota option and respect configured max backoff steps.
7. `8e570df`: Merge PR #3412: fix(i18n): detect the system language for new configurations.
8. `a97d412`: fix(proxy): scope `sessionId` per conversation and bump on upstream 1M accumulation (root cause: upstream sessionId was derived solely from email FNV-1a, causing all conversations to share one session until 1M token ceiling caused 400 error; fixed by fingerprinting first user message + generation counter).
9. `ade8f6c`: feat(circuit-breaker): add `lock_on_zero_quota` toggle in settings to lock accounts until reset time when 5h or weekly bucket quota is 0%, cap retry lockout dynamically to max configured backoff steps.
10. `086b6a1`: fix(i18n): detect system language on fresh configuration startup.

---

## 5. Non-Negotiable Coding & Behavior Rules

1. **Root Readme Lowercase:** Strictly named `readme.md`.
2. **Artifact Exclusions:** `.gitignore` excludes test reports, coverage, binary files (`.exe`, `.dll`), and temporary cache (`tmp/`).
3. **No CI/CD Bypass:** Never disable, bypass, or comment out automated checks.
4. **Implicit Boolean Evaluation:** Never compare `== true`. All positive booleans must evaluate implicitly (`if isValid`).
5. **Strict Positive Prefixes:** Booleans prefixed with `is` or `has` only (`isActive`, `hasPermission`). Words like `can`, `should`, `was` are banned.
6. **Error Management:** Universal envelopes `{ data, errors[], meta }`, zero swallowed errors, and typed error wrappers.
