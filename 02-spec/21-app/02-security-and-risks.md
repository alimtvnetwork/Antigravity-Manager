# Security Audit, Vulnerability Analysis, and Risk Assessment

> **Specification:** `02-spec/21-app/02-security-and-risks.md`
> **Status:** Production-Ready
> **Audit Date:** 2026-09-10
> **Overall Risk Level:** **MEDIUM**

---

## 1. Executive Summary

A comprehensive automated and manual security inspection of the Antigravity-Manager codebase was conducted across its Rust backend (`src-tauri/`), React frontend (`src/`), and build toolchain.

Antigravity-Manager operates as a high-privilege local proxy service and account manager handling sensitive OAuth access tokens, Google Cloud session state, local IDE database files, and network sockets. While the core architecture leverages safe Rust memory semantics and parameterized SQL queries, specific design trade-offs regarding OAuth secrets, local API key enforcement, and cross-origin access present actionable security risks.

---

## 2. Threat Modeling & Vulnerability Findings

### 2.1 Hardcoded Secrets & Embedded Credentials
- **Finding SEC-001 (High Sensitivity):** Hardcoded Google OAuth Client Secret
  - **Location:** `src-tauri/src/modules/oauth.rs:L4-L5`
  - **Code:**
    ```rust
    const CLIENT_ID: &str = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
    const CLIENT_SECRET: &str = "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";
    ```
  - **Analysis:** The Google OAuth Client Secret is embedded in plaintext in source code and compiled into release binaries. While Google treats desktop/native OAuth clients as public clients, exposing client secrets in public source repositories enables third parties to impersonate the application's OAuth client identity.
  - **Remediation:** Migrate to pure OAuth 2.0 Authorization Code Flow with PKCE (RFC 7636) without client secrets, or inject credentials at compile time via build-time environment variables.

### 2.2 Unauthenticated Local Proxy & CORS Posture
- **Finding SEC-002 (Medium Risk):** Permissive CORS Combined with Optional API Key Authentication
  - **Location:** `src-tauri/src/proxy/server.rs`
  - **Analysis:**
    - The proxy HTTP listener binds by default to `127.0.0.1:8045`.
    - Permissive CORS headers allow all origins (`CorsLayer::new().allow_origin(Any).allow_headers(Any)`).
    - API Key authentication is optional and disabled by default for ease of local IDE configuration.
    - **Exploit Vector:** A malicious website visited in any web browser on the host machine could issue JavaScript `fetch()` calls to `http://127.0.0.1:8045/v1/chat/completions`, abusing the user's upstream quota, consuming account balances, or extracting active model configurations.
  - **Remediation:**
    1. Require an auto-generated Master API Key by default for all external proxy endpoints.
    2. Restrict CORS origins to approved IDE webview origins or reject browser requests that include an untrusted `Origin` header.

### 2.3 Account Credential Persistence on Disk
- **Finding SEC-003 (Medium Risk):** Plaintext OAuth Refresh Token Storage
  - **Location:** `src-tauri/src/modules/account.rs`
  - **Analysis:** User accounts, including active Google OAuth `refresh_token` and `access_token` strings, are persisted in local JSON files under `%APPDATA%/antigravity-tools/` (or OS user data directories). Other non-elevated user-space processes running under the same user profile can read these credentials directly.
  - **Remediation:** Encrypt stored tokens using operating system credential storage mechanisms:
    - Windows: Windows Data Protection API (DPAPI) via `CryptProtectData`.
    - macOS: Apple Keychain Services (`security` CLI / Keychain API).
    - Linux: Freedesktop Secret Service (`secret-tool` / libsecret).

### 2.4 SQL Injection & Database Posture
- **Finding SEC-004 (Medium Risk):** Dynamic SQL Formatting in Security DB
  - **Location:** `src-tauri/src/modules/security_db.rs:215-222` (and lines 233-248)
  - **Code:**
    ```rust
    format!(
        "SELECT id, client_ip, timestamp, method, path, user_agent, status, duration, api_key_hash, blocked, block_reason, username
         FROM ip_access_logs
         WHERE blocked = 1 AND client_ip LIKE '%{}%'
         ORDER BY timestamp DESC
         LIMIT {} OFFSET {}",
        ip, limit, offset
    )
    ```
  - **Analysis:** Dynamic raw string interpolation is used inside `get_ip_access_logs` instead of SQLite parameterized queries. The `ip` filter input is concatenated directly into the query string (`client_ip LIKE '%{}%'`). While other database modules (`proxy_db.rs`, `user_token_db.rs`) leverage `rusqlite::params![]` or prepared statements, this pattern introduces an active SQL injection vulnerability if an unescaped or malicious IP string containing single quotes or SQLite operators is passed through the IPC interface.
  - **Remediation:** Refactor `get_ip_access_logs` to use positional SQLite parameter bindings (`WHERE client_ip LIKE '%' || ? || '%'`) and pass `ip`, `limit`, and `offset` as parameters to `conn.prepare()` or `query_map()`.

### 2.5 Subprocess Command Execution
- **Finding SEC-005 (Low Risk):** Process Management & System Tool Invocations
  - **Location:** `src-tauri/src/modules/process.rs`, `patch.rs`, `cloudflared.rs`
  - **Analysis:** System commands (`taskkill`, `kill`, `codesign`, `open`, `explorer`) are invoked using `std::process::Command::new` with explicit argument slices rather than shell interpolation (`sh -c`). Paths are resolved from known system directories.
  - **Rating:** PASSED (Standard desktop process control).

---

## 3. Dependency Vulnerability & Toolchain Audit

- **Cargo Manifest (`src-tauri/Cargo.toml`):**
  - Uses modern, actively maintained crates: `tauri 2.2.5`, `tokio 1.x`, `axum 0.7`, `hyper 1.x`, `reqwest 0.12`, `rusqlite 0.32`.
  - TLS implemented via `rustls-tls` (avoiding legacy OpenSSL linking vulnerabilities).
- **Node Manifest (`package.json`):**
  - Modern frontend stack: React 19, TypeScript 5.8, Vite 7.
  - Development tools isolated in `devDependencies`.

---

## 4. Risk Rating & Actionable Recommendations

| Risk Category | Severity | Current Mitigation | Actionable Recommendation |
|---|:---:|---|---|
| Hardcoded OAuth Client Secret | MEDIUM | Public client scope | Transition to pure PKCE flow or environment injection |
| Browser CORS Drive-by Execution | MEDIUM | Localhost binding | Enforce Master API Key or filter incoming browser Origin headers |
| Token Storage in Plaintext | MEDIUM | User data directory ACL | Encrypt credentials via DPAPI / OS Keychain |
| SQL Injection | MEDIUM | Parameterized queries in proxy_db; dynamic string formatting in security_db | Refactor `security_db.rs:215-222` to use `rusqlite::params![]` with positional bindings |
| Subprocess Injection | LOW | Vectorized arguments | Maintain explicit argument vectors |

**Overall Project Risk Classification:** **MEDIUM**
The project is structurally secure against remote network attacks when bound to `127.0.0.1`, with primary residual risks concentrated around local workstation credential storage, dynamic SQL formatting in security logs, and cross-origin browser requests.

---

## 5. Verification & Acceptance Criteria

### AC-SEC-001: SQL Injection Parameterization
- **Executable Test:** `tests::test_security_db_parameterized_queries`
- **Given** an IP filter string containing SQL metacharacters (e.g. `' OR '1'='1' --`) supplied to `get_ip_access_logs`.
- **When** the query executes against SQLite database `security.db`.
- **Then** the database engine executes the query using positional parameter bindings without SQL syntax errors or logic alteration, treating the filter strictly as literal string content.

### AC-SEC-002: CORS Origin Validation & API Key Guard
- **Executable Test:** `tests::test_proxy_cors_and_api_key_guard`
- **Given** an incoming HTTP request to the local reverse proxy (`http://127.0.0.1:8045/v1/*`) containing an external or untrusted browser `Origin` header.
- **When** no valid authorization header or master API key is supplied.
- **Then** the proxy server rejects the request with HTTP 401 Unauthorized / 403 Forbidden, preventing unauthorized cross-origin quota consumption.

### AC-SEC-003: Credential Storage Protection
- **Executable Test:** `tests::test_credential_storage_dpapi`
- **Given** user OAuth tokens (`access_token`, `refresh_token`) and session secrets persisted to disk.
- **When** written to the local configuration directory.
- **Then** credentials must not be stored in unencrypted cleartext JSON; storage must utilize platform-native protection (DPAPI on Windows, Keychain on macOS, Secret Service on Linux) with user-restricted filesystem ACLs.

