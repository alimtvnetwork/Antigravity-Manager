# Account Rotation API Endpoint & Split Repo DB Specification

> **Document ID:** `02-spec/21-app/15-account-rotation-api-endpoint.md`
> **Status:** Approved Architectural Specification
> **Target Platforms:** Linux (Ubuntu/Debian), Windows 10/11, macOS

---

## 1. Overview & Capabilities

This specification details the Account Rotation HTTP API endpoints and the Split Repo Database architecture.

External automation scripts, CLI utilities, IDE extensions, or scheduled cron daemons can trigger automatic account and profile rotation via HTTP endpoints exposed on the Antigravity Manager Admin Proxy Server (`127.0.0.1:8045` by default).

### Available Endpoints

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `POST` | `/api/accounts/rotate` | Triggers immediate rotation to the next best healthy account in the pool |
| `POST` | `/api/auto-switcher/rotate` | Alias for account rotation via auto-switcher subsystem |
| `GET` | `/api/auto-switcher/status` | Queries live status of auto-switcher daemon, active account, and quota |
| `GET` | `/api/repo-db/projects` | Queries all detected running projects across instances |
| `GET` | `/api/repo-db/prompts` | Queries all backed up prompts staged in repo_prompts.db |
| `POST` | `/api/accounts/switch` | Explicitly switches to a specific `account_id` payload |

---

## 2. API Contract & Schemas

### 2.1 POST `/api/accounts/rotate` & `/api/auto-switcher/rotate`

Triggers immediate evaluation of candidate profiles in the account pool, selects the profile with the highest remaining quota, backs up all running project prompts into the dedicated SQLite state database (`repo_prompts.db`), switches instance credentials, and directly dispatches running prompts upon completion without queuing.

#### Request Headers

```http
POST /api/accounts/rotate HTTP/1.1
Host: 127.0.0.1:8045
Content-Type: application/json
Authorization: Bearer <admin_token_if_configured>
```

#### Request Body

No request body is required (`{}` or empty).

#### Response (200 OK)

```json
{
  "is_success": true,
  "message": "Successfully rotated to profile 'default' with account 'dev@example.com'",
  "status": {
    "is_running": true,
    "active_instance_id": "default",
    "active_account_email": "dev@example.com",
    "current_quota_percent": 88.5,
    "last_check_timestamp": 1726671000,
    "last_switch_timestamp": 1726671015,
    "last_switch_reason": "Manual rotation triggered by user"
  }
}
```

#### Error Response (409 Conflict)

Returned if another switch or rotation operation is currently in progress:

```json
{
  "error": "Another switch or rotation operation is already in progress"
}
```

#### Error Response (500 Internal Server Error)

Returned if no alternative healthy profile exists in the pool:

```json
{
  "error": "No alternative healthy profile found in pool"
}
```

---

### 2.2 GET `/api/auto-switcher/status`

#### Request Headers

```http
GET /api/auto-switcher/status HTTP/1.1
Host: 127.0.0.1:8045
```

#### Response (200 OK)

```json
{
  "is_running": true,
  "active_instance_id": "default",
  "active_account_email": "dev@example.com",
  "current_quota_percent": 88.5,
  "last_check_timestamp": 1726671000,
  "last_switch_timestamp": 1726671015,
  "last_switch_reason": "Manual rotation triggered by user"
}
```

---

## 3. Split Repo DB & Running Prompts Lifecycle

Before an account rotation or workspace instance switch takes place, active prompts across all running projects are preserved:

```
  ┌──────────────────────────────────────────────────────────────┐
  │                    Active Workspace Instances                │
  └──────────────────────────────┬───────────────────────────────┘
                                 │
                                 ▼
  ┌──────────────────────────────────────────────────────────────┐
  │ 1. Scan Running Projects via repo_db::detect_running_projects│
  │    - Reads workspaceStorage/*/workspace.json                 │
  │    - Matches active instance PIDs (sysinfo)                  │
  └──────────────────────────────┬───────────────────────────────┘
                                 │
                                 ▼
  ┌──────────────────────────────────────────────────────────────┐
  │ 2. Backup Running Prompts via repo_db::backup_running_prompts│
  │    - Extracts active chat / prompt records from state.vscdb  │
  │    - Writes to repo_prompts.db with status = 'backed_up'     │
  └──────────────────────────────┬───────────────────────────────┘
                                 │
                                 ▼
  ┌──────────────────────────────────────────────────────────────┐
  │ 3. Execute Profile Rotation (Token Injection & Window Settle)│
  │    - Synchronous close with SIGTERM -> wait -> SIGKILL       │
  │    - Inject fresh OAuth tokens into isolated state.vscdb     │
  │    - Launch target instance executable                       │
  └──────────────────────────────┬───────────────────────────────┘
                                 │
                                 ▼
  ┌──────────────────────────────────────────────────────────────┐
  │ 4. Direct Dispatch via repo_db::dispatch_running_prompts     │
  │    - Queries all 'backed_up' prompts for running projects    │
  │    - Immediately dispatches prompts directly (NO QUEUING)    │
  │    - Updates prompt status to 'dispatched'                   │
  └──────────────────────────────────────────────────────────────┘
```

---

## 4. Ubuntu / Linux Process Management Standard

On Linux distributions (Ubuntu 20.04/22.04/24.04), process lifecycle during rotation must adhere to:

1. **Synchronous Exit Polling**: `close_instance` polls PID existence every 100ms up to 3000ms.
2. **SIGKILL Fallback**: If processes remain alive after 3000ms, SIGKILL (`kill -9`) is issued to prevent lingering instances holding write locks on `state.vscdb`.
3. **AppImage Sanitization**: Spawning instances removes `LD_LIBRARY_PATH`, `APPIMAGE`, and `APPDIR` while preserving standard `DISPLAY`, `WAYLAND_DISPLAY`, and `XAUTHORITY`.
