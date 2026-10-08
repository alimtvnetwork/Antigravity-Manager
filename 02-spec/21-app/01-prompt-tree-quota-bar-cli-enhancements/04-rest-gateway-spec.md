# Secure REST Endpoints Gateway Architecture & Specification

## 1. Architectural Overview & Security Invariants

The AGM REST API Gateway provides a high-performance, asynchronous HTTP interface for headless agents, external automations, and developer companion tools to query instance telemetry, quota states, prompt histories, and trigger dispatch operations.

### Core Security Invariants
1. **Localhost Binding by Default**:
   - The HTTP listener strictly binds to `127.0.0.1:8045` by default (`allow_lan_access = false`).
   - Binds to `0.0.0.0:8045` only when explicitly configured by the user via GUI or CLI flag (`--allow-lan`).
2. **Mandatory Bearer Authentication Elevation**:
   - When `allow_lan_access = true`, the security middleware automatically elevates authentication mode from `Off` to `AllExceptHealth` (or `StrictAll`), rejecting unauthenticated LAN requests.
   - All management and mutation endpoints (`/api/v1/instance/*`, `/api/v1/prompts/*`) require a valid `Authorization: Bearer <token>` or `x-api-key` header.
3. **Admin Token Separation**:
   - Destructive operations (process termination, profile deletion, database clearing) require an admin-privileged token recorded in `security.db`.
4. **IP Filtering & Rate-Limiting**:
   - Connection attempts pass through `ip_filter.rs` enforcing CIDR whitelisting and dynamic rate-limiting (e.g. 120 req/min per IP with temporary TTL ban for offenders).

---

## 2. API Endpoint Taxonomy

### A. Health & Diagnostics (Public on Localhost)
- `GET /health` / `GET /healthz` / `GET /api/health`:
  - Returns `200 OK` with JSON payload `{ "status": "UP", "version": "4.158.0", "gateway": "online" }`.
  - Always exempt from authentication in `AllExceptHealth` mode for Docker/k8s liveness probes.
- `GET /api/v1/doctor`:
  - Returns structured health report across all 5 SQLite databases and instance processes. Requires standard token.

### B. Quota & Account Management
- `GET /api/v1/quotas`:
  - Returns active account inventory with rolling 4H quotas, weekly quotas, reset timestamps, and health status.
- `POST /api/v1/instances/:id/fast-forward`:
  - Triggers fast-forward credential switch to the healthiest account for instance `:id`.

### C. Prompt Tree & Conversation History
- `GET /api/v1/prompts/tree`:
  - Returns full hierarchical project and conversation tree across all instances.
  - Supports query parameters: `?instance_id=<id>&only_running=true&max_words=2000`.
- `GET /api/v1/prompts/:id`:
  - Returns full prompt payload, markdown preview, word counts, and referenced asset links.
- `POST /api/v1/prompts/dispatch`:
  - Enqueues or dispatches prompt to target workspace IDE. Payload: `{ "instance_id": "...", "project_path": "...", "prompt": "...", "enqueue": true }`.
- `POST /api/v1/prompts/backup`:
  - Triggers snapshot backup of active prompts to `repo_prompts.db`.
- `POST /api/v1/prompts/restore`:
  - Restores prompts from backup database.

### D. Instance Lifecycle & Configuration
- `GET /api/v1/instances`:
  - Returns list of instance profiles, active PIDs, and running status.
- `POST /api/v1/instances/create`:
  - Provisions a new isolated profile.
- `POST /api/v1/instances/:id/switch`:
  - Updates bound email credentials.
- `POST /api/v1/instances/:id/launch`:
  - Starts instance IDE process.
- `POST /api/v1/instances/:id/stop`:
  - Stops instance IDE process.

---

## 3. Middleware Pipeline

```mermaid
flowchart LR
    Req[Incoming HTTP Request] --> IPFilter[IP Filter & Rate Limiter]
    IPFilter --> LANCheck[LAN Security Elevation]
    LANCheck --> AuthMiddleware[Bearer / API-Key Auth]
    AuthMiddleware --> RoleGate[Admin / User Scope Gate]
    RoleGate --> Handler[Axum Endpoint Handler]
    Handler --> SplitDB[Split SQLite Storage]
```

1. **IP Filter (`ip_filter.rs`)**: Drops blacklisted IPs immediately without reading body.
2. **Auth Layer (`auth.rs`)**: Validates token hash against `security.db` with sub-millisecond in-memory cache.
3. **Audit Log**: Every authenticated mutating request writes an immutable audit record into `task_history.db`.
