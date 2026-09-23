# AGM Native Terminal CLI Expanded Commands Suite

Spec Reference: 02-spec/21-app/20-agm-cli-expanded-commands.md
Parent Index: [02-spec/21-app/01-index.md](01-index.md)
Status: ACTIVE
Version: 1.0.0

## 1. Executive Summary

This specification expands the native `agm` terminal CLI engine in Antigravity-Manager with high-utility developer and operator commands modeled after GitMap's autonomous command suite:
- `agm doctor` / `agm check`: Comprehensive system health diagnostic.
- `agm accounts` / `agm acc`: Listing and inspection of registered accounts and quotas.
- `agm switch <email>`: Direct terminal account switching.
- `agm prompts` / `agm prompts ls`: Prompt template and active prompt inventory inspection.
- `agm proxy` / `agm proxy test`: Proxy server status, health check, and route telemetry.
- `agm sync`: Cross-node / local storage synchronization.
- `agm pull`: Repository branch update and synchronization.
- `agm clean` / `agm purge`: Safe cache and temporary artifact pruning.
- `agm logs`: Real-time and recent log viewing with tailing.

---

## 2. Command Architecture & Syntax Specifications

### 2.1 `agm doctor` (alias `agm check`)
Performs pre-flight health diagnostic:
1. **Database Vault Integrity**: Validates presence and readability of:
   - `accounts.json`
   - `email_vault.db`
   - `repo_prompts.db`
   - `security.db`
   - `thinking_store.db`
2. **Runtime Process Inspection**: Checks if Antigravity IDE processes (`antigravity.exe` or `code`) are active.
3. **Proxy Gateway Check**: Verifies if the local Axum reverse proxy port (default: 8045) is actively listening.
4. **Environment & PATH**: Confirms `agm.exe` registration in User PATH and PowerShell `$PROFILE`.
5. **Output**: Clean ANSI color-coded summary table with overall health verdict (`HEALTHY`, `DEGRADED`, or `UNHEALTHY`).

### 2.2 `agm accounts` (alias `agm acc`)
Inspects configured accounts:
- Lists all registered accounts in a formatted table:
  - `INDEX`, `EMAIL`, `TIER`, `STATUS` (`ACTIVE` / `STANDBY`), `WEEKLY QUOTA`, `UPDATED`.
- Flags:
  - `--active`: Filters output to only the currently active account.
  - `--json`: Outputs raw JSON array for machine parsing.

### 2.3 `agm switch <account-or-email>`
Switches the active account directly from the terminal without launching the GUI:
1. Accepts full email address or unique prefix (case-insensitive).
2. Sets `current_account_id` in `accounts.json`.
3. Triggers account switch hooks and outputs confirmation with previous and newly activated account details.

### 2.4 `agm prompts` (alias `agm prompts ls`)
Inspects saved and active prompts from `repo_prompts.db`:
- Lists prompts with `ID`, `PROJECT / WORKSPACE`, `STATUS`, and `PROMPT SNIPPET`.
- Flags:
  - `--running`: Filters to prompts with status `'running'`.

### 2.5 `agm proxy` [test | status]
Queries the local Axum reverse proxy engine:
1. `agm proxy status`: Displays proxy port, upstream routing protocols (Claude `/v1/messages`, OpenAI `/v1/chat/completions`, Gemini `/v1beta/models/*`), active token stats, and session count.
2. `agm proxy test`: Sends a lightweight loopback HTTP request to `127.0.0.1:8045/health` or `/accounts/current` to verify latency and responsiveness.

### 2.6 `agm sync`
Triggers synchronization of local account records, instance profiles, and prompt states with local persistence and Supabase (if configured).

### 2.7 `agm pull`
Executes `git pull origin main` in the repository root and reports commit synchronization status.

### 2.8 `agm clean` (alias `agm purge`)
Performs safe disk hygiene:
- Cleans build artifacts (`target/debug/incremental`), temporary log exports, and caches.
- **Safety Invariant**: Strictly protects `email_vault.db`, `email_passwords.db`, `accounts.json`, and all credentials from deletion.

### 2.9 `agm logs` (alias `agm log`)
Reads and outputs recent application and proxy logs:
- Flags:
  - `--tail <N>`: Displays the last N lines (default: 25).
  - `--filter <text>`: Filters log lines containing the specified substring.

---

## 3. Remote Email Inbound Integration

All newly added commands are directly mapped to the inbound email remote control parser in `src-tauri/src/modules/email_inbound.rs`:
- `sub: [worker-name|ip] | agm doctor`
- `sub: [worker-name|ip] | agm accounts`
- `sub: [worker-name|ip] | agm prompts`
- `sub: [worker-name|ip] | agm proxy`
- `sub: [worker-name|ip] | agm clean`
- `sub: [worker-name|ip] | agm switch | <email>`

The inbound watcher executes the requested command on the host, formats the output in clean plaintext tables, and replies via Phase 2 Result receipt.
