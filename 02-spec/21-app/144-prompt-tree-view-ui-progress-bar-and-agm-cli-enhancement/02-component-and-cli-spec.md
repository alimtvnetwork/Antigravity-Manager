# Component & CLI Specification: AGM CLI Expansion & Secure REST Pathways

**Document ID:** `02-spec/21-app/144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/02-component-and-cli-spec.md`  
**Task Slug:** `144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement`  
**Scope:** AGM CLI Architecture, 46 CLI Commands Catalog (7 Domains), Secure REST Endpoints (`port 8045`), and End-to-End Test Suite  
**Author:** Author 02 (Antigravity Backend & CLI Architecture Specialist)  
**Status:** Approved Specification  

---

## 1. Executive Summary & Architectural Overview

Antigravity-Manager (`agm`) provides headless automation, multi-instance orchestration, prompt tree observation, and AI gateway proxying. While historical iterations of AGM exposed ad-hoc CLI verbs and relied primarily on Tauri IPC for GUI communication, enterprise server environments and automated multi-agent fleets require:

1. **A Structured 46-Command CLI Architecture across 7 Clear Functional Domains:** Providing deep parity with GUI features, supporting JSON output envelopes (`--json`), structured error handling, and robust instance targeting (`-i/--instance`).
2. **Authenticated REST Endpoints (`port 8045` under `/api/*`):** Exposing instance lifecycle management, prompt tree introspection, prompt enqueueing, and proxy administration via `Authorization: Bearer <token>` or `x-api-key`.
3. **Rigorous End-to-End Testing Protocols:** Ensuring that instance sandboxing, prompt attribution, account rotation, repeat grouping, and CLI/REST parity operate reliably across Windows, macOS, and Linux.

```
                          ┌──────────────────────────────────────┐
                          │    Caller: Headless CLI / Fleet /    │
                          │        Automated Test Scripts        │
                          └──────────────────┬───────────────────┘
                                             │
                       ┌─────────────────────┴─────────────────────┐
                       │                                           │
                       ▼                                           ▼
          ┌──────────────────────────┐               ┌───────────────────────────┐
          │  AGM CLI (src/bin/agm)   │               │   REST API (Port 8045)    │
          │  46 Commands / 7 Domains │               │  /api/* (Bearer Auth)     │
          └────────────┬─────────────┘               └─────────────┬─────────────┘
                       │                                           │
         Direct SQLite │ or Local REST               Route Handler │ (Axum 0.7)
         Access        │ Forwarding                  admin_routes  │
                       │                                           │
                       ▼                                           ▼
          ┌──────────────────────────────────────────────────────────────────────┐
          │               AGM Core Service & Storage Architecture                │
          │  - instances.db / repo_prompts.db / backup_prompts.db                │
          │  - auto_switcher.rs / instance.rs / repo_db.rs / server.rs           │
          └──────────────────────────────────────────────────────────────────────┘
```

---

## 2. Comprehensive Catalog of 46 AGM CLI Commands across 7 Domains

The AGM CLI binary (`src-tauri/src/bin/agm.rs`) provides both domain-namespaced syntax (`agm <domain> <subcommand>`) and historical shorthand aliases. Every command supports `--json` for machine automation and `-i/--instance <id|name|#seq>` for deterministic instance targeting.

```
DOMAIN SUMMARY:
  1. Instance Lifecycle (9 commands) : list, status, start, stop, restart, switch, copy, delete, logs
  2. Prompt Lifecycle   (11 commands): list, tree, inspect, send, running, queue, history, export, backup, restore, purge
  3. Quota & Accounts   (7 commands) : quota, refresh, accounts, add, remove, validate, balance
  4. Fleet & Sync       (5 commands) : nodes, ping, sync, broadcast, health
  5. Proxy & Gateway    (6 commands) : status, pool, model, config, restart, cache-clear
  6. Security & Rules   (4 commands) : ip-list, ip-block, ip-unblock, audit
  7. System & Engine    (5 commands) : version, env, doctor, db-stats, vacuum
  -----------------------------------------------------------------------------------------
  TOTAL: 47 Formal Commands (46 primary + 1 balance diagnostic verb)
```

### Standard Output Envelope (`--json`)
When invoked with `--json`, every command produces the following strict envelope on `stdout`:
```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "meta": {
    "command": "agm prompts tree",
    "instance_id": "test-cli-flow-a-1743",
    "timestamp": "2026-10-08T15:30:00Z",
    "version": "4.8.2"
  }
}
```
On failure:
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERR_INSTANCE_NOT_FOUND",
    "message": "Instance 'worker-02' does not exist in instances.db"
  },
  "meta": {
    "command": "agm instances start",
    "timestamp": "2026-10-08T15:30:00Z",
    "version": "4.8.2"
  }
}
```

---

### 2.1 Domain 1: Instance Lifecycle (9 Commands)

Manages named Antigravity IDE sandboxes, isolated storage environments, and OS process groups.

| # | Command Syntax | Canonical Aliases | Primary Flags / Arguments | Description & Execution Semantics |
|---|---|---|---|---|
| **1** | `agm instances list` | `agm instances`, `agm ls` | `--json`, `--all`, `--active` | Lists all registered instances, their directory paths, running state, active account, and PID. |
| **2** | `agm instances status` | `agm instance status -i <id>` | `-i/--instance <id>`, `--json` | Returns fine-grained status of target instance: process tree, port, memory usage, and assigned repo. |
| **3** | `agm instances start` | `agm start -i <id>`, `agm launch` | `-i/--instance <id>`, `--repo <path>`, `--dry-run` | Spawns Antigravity IDE with isolated `USERPROFILE`/`HOME` sandbox targeting specified instance. |
| **4** | `agm instances stop` | `agm stop -i <id>`, `agm kill` | `-i/--instance <id>`, `--force`, `--grace-sec <n>` | Gracefully halts all child processes belonging to target instance (strictly preserving protected main IDE). |
| **5** | `agm instances restart` | `agm restart -i <id>` | `-i/--instance <id>`, `--repo <path>` | Sequentially terminates target instance processes and relaunches with intact configuration. |
| **6** | `agm instances switch` | `agm switch-instance`, `agm use` | `<id>`, `--set-default` | Switches the globally active instance CLI context without launching GUI. |
| **7** | `agm instances copy` | `agm instance-clone`, `agm duplicate` | `<src_id> <new_name>`, `--mode <full\|settings>` | Deep-clones instance configuration, storage, settings, and optionally conversation databases. |
| **8** | `agm instances delete` | `agm rm -i <id>`, `agm delete-profile`| `-i/--instance <id>`, `--purge-storage`, `--yes` | Unregisters instance from `instances.db` and safely removes sandbox files (skipping protected IDs). |
| **9** | `agm instances logs` | `agm instance-logs -i <id>` | `-i/--instance <id>`, `-n <lines>`, `--follow` | Tails or dumps stdout/stderr log stream for the target instance process group. |

---

### 2.2 Domain 2: Prompt Lifecycle (11 Commands)

Captures, inspects, executes, buffers, and restores user prompts and AI conversation trees per instance.

| # | Command Syntax | Canonical Aliases | Primary Flags / Arguments | Description & Execution Semantics |
|---|---|---|---|---|
| **10** | `agm prompts list` | `agm prompts`, `agm prompts-query` | `-i <id>`, `--repo <path>`, `--limit <n>`, `--json` | Lists recent conversation prompts recorded for target instance with word counts and timestamps. |
| **11** | `agm prompts tree` | `agm tree`, `agm prompt-tree` | `-i <id>`, `--repo <path>`, `--all`, `--json` | Generates hierarchical tree of conversations, grouping repeated prompts with repeat badges (`x4`). |
| **12** | `agm prompts inspect` | `agm observe`, `agm inspect-prompt` | `<cid>`, `-i <id>`, `--full`, `--json` | Renders complete conversation details, distinguishing Human Prompts from AI Subagent Instructions. |
| **13** | `agm prompts send` | `agm prompt`, `agm dispatch` | `"<text>"`, `-i <id>`, `--repo <path>`, `--now` | Dispatches a prompt to target instance via resume hand-off file or live injection socket. |
| **14** | `agm prompts running` | `agm wpr`, `agm which-prompts-running` | `-i <id>`, `--json`, `--all` | Detects currently in-flight/active prompts by scanning active lockfiles and recent turn timestamps. |
| **15** | `agm prompts queue` | `agm prompt-queue`, `agm queue-scheduler` | `[add\|ls\|clear]`, `-i <id>`, `--repo <path>` | Manages deferred prompt execution queue evaluated when instance becomes idle. |
| **16** | `agm prompts history` | `agm history`, `agm prompt-history` | `-i <id>`, `--filter <term>`, `--days <n>` | Queries historical prompt runs across all repositories with attribution tags. |
| **17** | `agm prompts export` | `agm pe`, `agm prompts-export` | `-i <id>`, `--out <path>`, `--format <md\|json>` | Exports prompt conversation history to formatted Markdown or structured JSON artifact. |
| **18** | `agm prompts backup` | `agm brp`, `agm backup-prompts` | `-i <id>`, `--repo <path>`, `--json` | Creates a persistent snapshot of pending/running prompts before account rotation or reboot. |
| **19** | `agm prompts restore` | `agm rrp`, `agm restore-prompts` | `-i <id>`, `--repo <path>`, `--dry-run` | Restores backed-up prompts into target instance hand-off queue for immediate execution. |
| **20** | `agm prompts purge` | `agm prompt-clean`, `agm purge-prompts` | `-i <id>`, `--older-than <days>`, `--yes` | Safely removes stale conversation summaries and temporary resume artifacts. |

---

### 2.3 Domain 3: Quota & Accounts (7 Commands)

Monitors token consumption, manages pool credentials, and executes zero-downtime account rotation.

| # | Command Syntax | Canonical Aliases | Primary Flags / Arguments | Description & Execution Semantics |
|---|---|---|---|---|
| **21** | `agm quota show` | `agm status`, `agm credits` | `-i <id>`, `--json`, `--verbose` | Displays immediate and weekly quota percentages, refill countdowns, and current tier. |
| **22** | `agm quota refresh` | `agm refresh-tier`, `agm refresh-quota` | `--all`, `--json` | Triggers background polling to synchronize remaining quotas across all configured accounts. |
| **23** | `agm accounts list` | `agm accounts`, `agm acc` | `--json`, `--show-tokens` | Lists all accounts in the pool, priority orders, status (active/exhausted), and assigned instances. |
| **24** | `agm accounts add` | `agm add-account` | `--email <e>`, `--token <t>`, `--tier <str>` | Ingests a new provider credential into the account pool with automatic initial quota probe. |
| **25** | `agm accounts remove` | `agm rm-account` | `<id\|#seq>`, `--yes` | Safely removes an account credential from the rotation pool. |
| **26** | `agm accounts validate` | `agm check-accounts` | `--all`, `--timeout-ms <n>` | Executes live HTTP probe against provider upstream to verify credential validity. |
| **27** | `agm accounts balance` | `agm account-rebalance` | `--threshold <pct>`, `--dry-run` | Evaluates pool distribution and reorders accounts to prioritize highest remaining runway. |

---

### 2.4 Domain 4: Fleet & Sync (5 Commands)

Coordinates multi-node clusters, Supabase state synchronization, and remote node dispatch.

| # | Command Syntax | Canonical Aliases | Primary Flags / Arguments | Description & Execution Semantics |
|---|---|---|---|---|
| **28** | `agm fleet nodes` | `agm nodes`, `agm ssh-nodes` | `--json`, `--health` | Lists all remote worker nodes, SSH connection parameters, and synchronized lease states. |
| **29** | `agm fleet ping` | `agm ping-node` | `<node_id\|host>`, `--timeout <sec>` | Verifies SSH and REST latency to a specific remote fleet worker. |
| **30** | `agm fleet sync` | `agm sync`, `agm supabase-sync` | `--direction <push\|pull\|both>`, `--dry-run`| Synchronizes instance metadata, account leases, and telemetry with central Supabase store. |
| **31** | `agm fleet broadcast` | `agm broadcast-email`, `agm notify` | `"<subject>" "<body>"`, `--target <all\|admins>`| Dispatches fleet-wide administrative alert via configured SMTP failover relays. |
| **32** | `agm fleet health` | `agm cluster-health` | `--json` | Comprehensive health check across all cluster nodes, leases, and database replications. |

---

### 2.5 Domain 5: Proxy & Gateway (6 Commands)

Governs the AI adapter proxy (OpenAI, Claude, Gemini protocol transformation on port 8045).

| # | Command Syntax | Canonical Aliases | Primary Flags / Arguments | Description & Execution Semantics |
|---|---|---|---|---|
| **33** | `agm proxy status` | `agm gateway-status` | `--json` | Returns port binding, active connection count, upstream latency, and error rate. |
| **34** | `agm proxy pool` | `agm proxy-pool` | `[ls\|bind\|unbind]`, `--account <id>` | Configures upstream outbound proxy bindings (HTTP/SOCKS5) per account. |
| **35** | `agm proxy model` | `agm model-map` | `[ls\|set] <alias> <target>`, `--json` | Manages wildcard model routing rules (e.g. `gemini-*-flash-*` to specific upstream targets). |
| **36** | `agm proxy config` | `agm gateway-config` | `[get\|set <k> <v>]`, `--json` | Inspects or updates proxy runtime parameters (rate limits, request timeouts, thinking flags).|
| **37** | `agm proxy restart` | `agm restart-proxy` | `--graceful`, `--timeout-sec <n>` | Restarts Axum HTTP server listener and reloads model specs and security tables. |
| **38** | `agm proxy cache-clear` | `agm clear-thinking-store` | `--target <thinking\|tokens\|all>` | Invalidates and purges cached thinking blocks or token usage tables. |

---

### 2.6 Domain 6: Security & Rules (4 Commands)

Manages IP filtering, firewall rules, user tokens, and audit access trails.

| # | Command Syntax | Canonical Aliases | Primary Flags / Arguments | Description & Execution Semantics |
|---|---|---|---|---|
| **39** | `agm security ip-list` | `agm ip-blacklist`, `agm blacklist` | `--type <black\|white\|all>`, `--json` | Displays active IPv4/IPv6 access control lists with hit counts and ban timestamps. |
| **40** | `agm security ip-block` | `agm block-ip` | `<ip_or_cidr>`, `--reason <text>` | Adds IP or CIDR block to the strict rejection blacklist in `security.db`. |
| **41** | `agm security ip-unblock` | `agm unblock-ip` | `<ip_or_cidr>` | Removes IP or CIDR subnet from blacklist and restores access. |
| **42** | `agm security audit` | `agm ip-audit`, `agm security-logs` | `--limit <n>`, `--tail`, `--json` | Inspects recent unauthorized access attempts and token auth failures. |

---

### 2.7 Domain 7: System & Engine (5 Commands)

Low-level host diagnostics, database hygiene, and environment validation.

| # | Command Syntax | Canonical Aliases | Primary Flags / Arguments | Description & Execution Semantics |
|---|---|---|---|---|
| **43** | `agm system version` | `agm -v`, `agm --version` | `--json` | Returns binary version, build commit SHA, Rust edition, and platform architecture. |
| **44** | `agm system env` | `agm env-info` | `--json` | Dumps runtime paths: data dir, instances root, Antigravity binary location, and active profile.|
| **45** | `agm system doctor` | `agm doctor`, `agm check` | `--fix`, `--json` | Runs deep diagnostic probes (database integrity, process collisions, path writeability, ports).|
| **46** | `agm system db-stats` | `agm db-status` | `--json` | Returns file size, WAL size, row counts, and page fragmentation for all Split SQLite stores. |
| **47** | `agm system vacuum` | `agm db-vacuum` | `--all`, `--dry-run` | Executes SQLite `VACUUM` and `PRAGMA optimize` across all SQLite databases. |

---

## 3. Secure REST Endpoints Specification (Port 8045, `/api/*`)

All administrative REST capabilities are hosted on port `8045` by Axum 0.7 (`src-tauri/src/proxy/server.rs`). They are nested under `/api` and strictly protected by `admin_auth_middleware`.

### 3.1 Authentication & Security Architecture

1. **Credentials:** Incoming requests must provide either:
   - Header: `Authorization: Bearer <ADMIN_PASSWORD_OR_API_KEY>`
   - Header: `x-api-key: <ADMIN_PASSWORD_OR_API_KEY>`
   - Header: `x-goog-api-key: <ADMIN_PASSWORD_OR_API_KEY>`
2. **Strict Admin Verification (`force_strict = true`):**
   - The middleware checks `security.admin_password`. If configured, it requires exact equality.
   - If `admin_password` is empty, it falls back to `security.api_key`.
   - If both are empty, access is strictly rejected (`401 Unauthorized`).
3. **Public Exemptions:**
   - `/api/health`, `/health`, `/healthz` (Return `200 OK` for load-balancer probes).
   - OPTIONS preflight requests (CORS).

```
   HTTP Request ────► [ admin_auth_middleware ]
                              │
                    Bearer Token Valid?
                       ├── Yes ──► Route Handler (/api/*)
                       └── No  ──► 401 Unauthorized
```

### 3.2 Endpoint Route Catalog

#### A. Instance Management Routes (`/api/instances/*`)

| Method | Endpoint | Description | Request Body | Response Body (`data`) |
|---|---|---|---|---|
| `GET` | `/api/instances` | List all instances | None | `Array<InstanceDto>` |
| `POST` | `/api/instances` | Create new instance | `CreateInstancePayload` | `InstanceDto` |
| `GET` | `/api/instances/:id` | Get instance details & status | None | `InstanceDetailDto` |
| `DELETE` | `/api/instances/:id` | Delete instance (purge files) | None | `{ "deleted": true }` |
| `POST` | `/api/instances/:id/start` | Launch instance IDE process | `{ "repoPath"?: string }` | `{ "pid": number, "status": "running" }` |
| `POST` | `/api/instances/:id/stop` | Terminate instance process | `{ "force"?: boolean }` | `{ "stopped": true }` |
| `POST` | `/api/instances/:id/restart`| Restart instance IDE process | `{ "repoPath"?: string }` | `{ "pid": number, "status": "running" }` |
| `POST` | `/api/instances/:id/switch` | Switch active account for instance | `{ "targetAccountId"?: string }` | `SwitchResultDto` |
| `POST` | `/api/instances/:id/clone` | Deep-clone instance | `CloneInstancePayload` | `InstanceDto` |

#### B. Prompt Lifecycle Routes (`/api/prompts/*`)

| Method | Endpoint | Description | Request Body | Response Body (`data`) |
|---|---|---|---|---|
| `GET` | `/api/prompts` | List recent prompts for instance | Query: `?instanceId=...&repo=...` | `Array<PromptSummaryDto>` |
| `GET` | `/api/prompts/tree` | Hierarchical conversation tree | Query: `?instanceId=...&repo=...` | `PromptTreeDto` |
| `GET` | `/api/prompts/running` | In-flight / running prompts | Query: `?instanceId=...` | `Array<RunningPromptDto>` |
| `POST` | `/api/prompts/send` | Dispatch prompt to instance | `SendPromptPayload` | `{ "dispatched": true, "taskId": string }` |
| `GET` | `/api/prompts/queue` | List queued prompts | Query: `?instanceId=...` | `Array<QueuedPromptDto>` |
| `POST` | `/api/prompts/queue` | Enqueue deferred prompt | `EnqueuePromptPayload` | `{ "queued": true, "queueId": string }` |
| `POST` | `/api/prompts/backup` | Trigger prompt backup snapshot | `{ "instanceId": string }` | `{ "backedUpCount": number }` |
| `POST` | `/api/prompts/restore` | Restore backed up prompts | `{ "instanceId": string }` | `{ "restoredCount": number }` |

#### C. System & Proxy Control Routes (`/api/proxy/*`, `/api/system/*`)

| Method | Endpoint | Description | Request Body | Response Body (`data`) |
|---|---|---|---|---|
| `GET` | `/api/proxy/status` | Proxy gateway runtime status | None | `ProxyStatusDto` |
| `POST` | `/api/proxy/restart` | Graceful restart of proxy listener | None | `{ "restarted": true }` |
| `POST` | `/api/proxy/cache/clear` | Purge thinking block cache | `{ "target": "thinking" }` | `{ "clearedCount": number }` |
| `GET` | `/api/system/doctor` | Host & SQLite health probes | None | `DoctorReportDto` |

---

### 3.3 Universal Response Envelopes & Error Model

Every REST endpoint serializes into a standard JSON envelope:
```typescript
interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: {
    code: string;
    message: string;
    details?: any;
  } | null;
  meta: {
    timestamp: string;
    version: string;
    requestId?: string;
  };
}
```

HTTP Status Codes:
- `200 OK`: Operation succeeded.
- `400 Bad Request`: Payload validation failed or invalid query parameter.
- `401 Unauthorized`: Missing or incorrect Bearer token / API key.
- `403 Forbidden`: Action disallowed on protected system resource (e.g. attempting to delete default instance or terminate main IDE PID).
- `404 Not Found`: Target instance, prompt, or account does not exist.
- `500 Internal Server Error`: SQLite query failure or OS process launch failure.

---

## 4. End-to-End Testing Specification

To guarantee that multi-instance isolation, prompt attribution, account rotation, repeat grouping, and CLI/REST parity function with zero regressions, the following 5 comprehensive test cases are specified.

```
+-----------------------------------------------------------------------------------+
|                        E2E VERIFICATION TEST HARNESS                              |
+-----------------------------------------------------------------------------------+
|  TC-1: Instance Creation & Isolation (Data Dir, Environment Variables)            |
|  TC-2: Prompt Dispatch & Running Detection (Status Badges, Transcript Inspector)  |
|  TC-3: Account Switching Continuity & Prompt Preservation (Resume Tasks)          |
|  TC-4: Compact Tree View & Repeated Grouping (Hash Collapsing, xN Badges)         |
|  TC-5: AGM CLI & REST Parity Verification (Identical JSON Payload Structures)     |
+-----------------------------------------------------------------------------------+
```

---

### Test Case 1: Instance Creation & Isolation

- **Objective:** Verify that creating a new instance establishes a hermetic filesystem sandbox (`USERPROFILE`/`HOME`), that Antigravity launches inside this sandbox, and that conversations are recorded strictly within the instance's private SQLite stores without contaminating default `%USERPROFILE%\.gemini`.
- **Preconditions:**
  - Protected main IDE is running (PID noted in protected registry).
  - Test sandbox instances use unique naming prefix: `test-cli-flow-a-<rand4>`.
- **Execution Steps:**
  1. Invoke CLI: `agm instances create test-cli-flow-a-9901 --json`.
  2. Parse output: Verify `success == true`, capture `data.id` (`test-cli-flow-a-9901-<ts>`) and `data.data_dir`.
  3. Validate filesystem: Verify `data_dir` exists and contains subdirectories `home`, `app_data`, and `temp`.
  4. Invoke CLI: `agm instances start -i test-cli-flow-a-9901 --repo d:/work/Antigravity-Manager/scratch/test-repo --json`.
  5. Check spawned process environment: Verify `USERPROFILE` (or `HOME` on POSIX) equals `data_dir/home`.
  6. Inspect default storage: Verify global `%USERPROFILE%\.gemini` was **not** modified by the test instance.
- **Pass Criteria:**
  - Instance is registered in `instances.db`.
  - Process runs with isolated environment variables.
  - No database files or locks created in default global storage.
  - Protected main IDE PID remains untouched and running.

---

### Test Case 2: Prompt Dispatch & Running Detection

- **Objective:** Verify that dispatching a prompt to a specific instance is accurately detected as in-flight (`running` state), surfaced on project cards with pulse badges, and properly recorded upon completion.
- **Preconditions:**
  - Target instance `test-cli-flow-a-9901` is running.
  - Scratch repository `scratch/test-repo` is initialized without `.git` folder.
- **Execution Steps:**
  1. Dispatch test prompt with unique marker:  
     `agm prompts send -i test-cli-flow-a-9901 --repo d:/work/Antigravity-Manager/scratch/test-repo "E2E144-TEST2-VERIFY-DISPATCH" --json`.
  2. Immediately poll running prompts via CLI:  
     `agm prompts running -i test-cli-flow-a-9901 --json`.
  3. Verify output contains `E2E144-TEST2-VERIFY-DISPATCH` and `status == "in_flight"`.
  4. Query REST endpoint:  
     `curl -H "Authorization: Bearer test_secret" http://127.0.0.1:8045/api/prompts/running?instanceId=test-cli-flow-a-9901`.
  5. Poll conversation tree until prompt finishes execution:  
     `agm prompts tree -i test-cli-flow-a-9901 --json`.
- **Pass Criteria:**
  - Running prompt is discovered within $\le 500\text{ ms}$ of dispatch.
  - REST and CLI simultaneously reflect `in_flight` status.
  - Completed prompt appears in conversation tree tagged with role `User Prompt`.

---

### Test Case 3: Account Switching Continuity & Prompt Preservation

- **Objective:** Verify that triggering an account switch (manual or auto-switch on low credits) safely backs up all in-flight and pending prompts, rotates credentials, and writes `.antigravity_resume_task.json` so prompts resume seamlessly on the new account.
- **Preconditions:**
  - Two valid test accounts in account pool (`acc-alpha`, `acc-beta`).
  - Active prompt running on `test-cli-flow-a-9901`.
- **Execution Steps:**
  1. Trigger instance prompt backup:  
     `agm prompts backup -i test-cli-flow-a-9901 --json`.
  2. Verify snapshot row created in `backup_prompts.db` matching `instance_id`.
  3. Execute account switch:  
     `agm instances switch -i test-cli-flow-a-9901 --account acc-beta --json`.
  4. Inspect `.antigravity_resume_task.json` generated in project root:
     - Verify `instance_id` equals `test-cli-flow-a-9901`.
     - Verify `resume_prompt` matches backed-up prompt text.
  5. Trigger prompt restoration:  
     `agm prompts restore -i test-cli-flow-a-9901 --json`.
- **Pass Criteria:**
  - Zero prompt loss during account switch.
  - Backup row explicitly attributed to `test-cli-flow-a-9901`.
  - Resume task cleanly ingested without duplicate execution.

---

### Test Case 4: Compact Tree View & Repeated Grouping

- **Objective:** Verify that repeated identical prompts sent in rapid succession are deduplicated and grouped under a single parent node with a count badge (`x4`), collapsible sub-runs, and sleek byte truncation tags.
- **Preconditions:**
  - Test instance has completed 4 successive runs of prompt: `"Run linter and check syntax"`.
- **Execution Steps:**
  1. Query conversation tree:  
     `agm prompts tree -i test-cli-flow-a-9901 --json`.
  2. Inspect returned tree hierarchy:
     - Verify identical prompts share a single grouped root node.
     - Verify root node `repeat_count == 4`.
     - Verify `sub_runs` array contains all 4 child executions with individual timestamps and run IDs.
  3. Verify long text truncation callout:
     - For prompt texts exceeding 120 words, verify presence of `truncated: true` and `preserved_bytes` metadata.
- **Pass Criteria:**
  - Tree node displays repeat multiplier (`x4`).
  - Sub-runs are cleanly nested under the parent prompt.
  - Truncation indicator properly reflects byte count without corrupting text payload.

---

### Test Case 5: AGM CLI & REST Parity Verification

- **Objective:** Verify 100% semantic and schema parity between AGM CLI `--json` outputs and REST API `/api/*` response bodies across all corresponding resources.
- **Preconditions:**
  - AGM proxy daemon active on `127.0.0.1:8045`.
  - Admin Bearer token configured and exported in test environment.
- **Execution Steps:**
  1. Compare Instance List:
     - Run: `agm instances list --json`
     - Query: `GET /api/instances`
     - Compare `data` arrays: Assert identical IDs, names, running flags, and account mappings.
  2. Compare Prompt Tree:
     - Run: `agm prompts tree -i test-cli-flow-a-9901 --json`
     - Query: `GET /api/prompts/tree?instanceId=test-cli-flow-a-9901`
     - Assert structural tree equality down to leaf nodes.
  3. Compare Quota & Status:
     - Run: `agm quota show -i test-cli-flow-a-9901 --json`
     - Query: `GET /api/accounts/current` and `/api/v1/status`
     - Assert identical immediate and weekly credit fractions.
- **Pass Criteria:**
  - Zero schema drift between CLI and REST outputs.
  - Response envelopes adhere strictly to Section 3.3.
  - All status codes, timestamps, and error structures match.

---

## 5. Non-Negotiable Invariants & Guardrails

1. **Main IDE Protection Invariant:**  
   Under no circumstances may CLI commands (`stop`, `delete`, `tif`, `clean`) touch, kill, or terminate PIDs belonging to the main Antigravity IDE or Cursor.
2. **Strict Relative Paths:**  
   All code, documentation, scripts, and logs must use repository-relative paths (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`). Absolute paths like `D:\work\...` are strictly prohibited.
3. **Database Read Isolation:**  
   CLI read probes must connect via `file:<path>?mode=ro` to prevent database locks while the GUI or proxy daemon is writing.
4. **No-Drift CLI/REST Route Implementation:**  
   Every new REST route must delegate directly to the identical Rust module functions called by CLI handlers (`src-tauri/src/modules/instance.rs`, `repo_db.rs`, `account.rs`).
