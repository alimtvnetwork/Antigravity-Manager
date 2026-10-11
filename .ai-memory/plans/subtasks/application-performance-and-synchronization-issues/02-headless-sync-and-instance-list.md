# Subtask 02: Headless Machine Sync & Instance List Executable Path Display

Spec Reference: [02-headless-machine-sync-spec.md](../../../../02-spec/21-app/application-performance-and-synchronization-issues/02-headless-machine-sync-spec.md)

## Objective
Implement UI executable path display in the instance list table using `formatShortPath` and `inst.config.executable_path` (with fallback to default detected IDE), and implement headless multi-machine fleet synchronization over Email / Telegram relays with JSON schema `agm-fleet-sync-v1.0`, SQLite tables (`FleetNodes`, `FleetInstances`, `FleetWorkspaces`), and top-level summary view `FleetSummaryView`.

---

## Target Files & Code Locations

1. `src/components/instances/InstanceTable.tsx` (Lines 88–97, 120–146)
2. `src/components/instances/table/InstanceRowCells.tsx` (Lines 270–300)
3. `src/components/instances/table/instanceTableTypes.ts`
4. `src/components/instances/table/instanceTableUtils.ts`
5. `src/components/instances/fleet/fleetTypes.ts`
6. `src/components/instances/fleet/FleetSummaryView.tsx` (New Component)
7. `src-tauri/src/modules/fleet_db.rs` (New Split-DB Module)
8. `src-tauri/src/modules/email_inbound/dispatcher.rs` & `src-tauri/src/modules/telegram_inbound/`

---

## Actionable Implementation Steps

### Step 1: Implement Executable Path Display in Instance Table
- In `src/components/instances/table/InstanceRowCells.tsx`:
  - Export interface `ExecutableCellProps`:
    ```typescript
    export interface ExecutableCellProps {
        inst: InstanceStatus;
        isCopied: boolean;
        onCopy: () => void;
    }
    ```
  - Implement `ExecutableCell({ inst, isCopied, onCopy }: ExecutableCellProps)`:
    - Extract `const rawExePath = inst.config.executable_path || '';`
    - Format short path using `formatShortPath(rawExePath)` if set; otherwise display `"Default IDE"`.
    - Render folder/terminal icon, monospace text, hover tooltip with full path, and copy-to-clipboard button.
    - If `rawExePath` is present, display subtle badge `"CUSTOM"` (`bg-amber-500/10 text-amber-500 border border-amber-500/20`).
- In `src/components/instances/InstanceTable.tsx`:
  - Add state for tracking copied executable path: `const [copiedExeId, setCopiedExeId] = useState<string | null>(null);`
  - In `<thead>`, insert column header:
    ```html
    <th className="px-2 py-1.5 min-w-[110px] max-w-[130px]">IDE Executable</th>
    ```
  - In `<tbody>`, mount `<ExecutableCell />` adjacent to `<PathCell />`.

### Step 2: Implement SQLite Table Migrations (`fleet_db.rs`)
- In `src-tauri/src/modules/fleet_db.rs`:
  - Implement schema migration function `init_fleet_tables(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error>`.
  - Create tables with idempotent DDL:
    - `FleetNodes (node_id PRIMARY KEY, alias, ip_address, public_ip, os_info, transport_channel, last_heartbeat_at, uptime_seconds, status, tool_version, raw_telemetry, created_at, updated_at)`
    - `FleetInstances (id PRIMARY KEY, node_id, instance_id, name, is_running, pid, executable_path, data_dir, bound_account_email, quota_percent, quota_weekly_percent, last_used, updated_at)`
    - `FleetWorkspaces (id PRIMARY KEY, node_id, instance_id, workspace_path, workspace_name, has_running_prompt, prompt_preview, turn_status, updated_at)`
  - Provide CRUD helper functions:
    - `upsert_fleet_node_telemetry(payload: &FleetSyncPayload) -> AppResult<()>`
    - `get_fleet_summary() -> AppResult<FleetSummaryData>`
    - `mark_stale_nodes_offline(stale_threshold_secs: i64) -> AppResult<usize>`

### Step 3: Implement JSON Schema `agm-fleet-sync-v1.0` Models
- In `src-tauri/src/modules/fleet_types.rs` and `src/components/instances/fleet/fleetTypes.ts`:
  - Define serializable Rust and TypeScript data structures:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FleetSyncPayload {
        pub version: String,
        pub schema: String, // "agm-fleet-sync-v1.0"
        pub msg_id: String,
        pub msg_type: String, // "HEARTBEAT" | "FULL_STATE"
        pub node_id: String,
        pub node_alias: String,
        pub timestamp: i64,
        pub transport_channel: String,
        pub node_telemetry: FleetNodeTelemetry,
        pub instances: Vec<FleetInstancePayload>,
        pub workspaces: Vec<FleetWorkspacePayload>,
        pub auth_signature: String,
    }
    ```

### Step 4: Integrate Message Handlers in Email & Telegram Inbound Daemons
- In `src-tauri/src/modules/email_inbound/dispatcher.rs`:
  - Intercept emails with subject prefix `[AGM-FLEET-SYNC]`.
  - Parse JSON envelope, verify HMAC signature with `crate::modules::security::verify_fleet_hmac`.
  - Route payload to `fleet_db::upsert_fleet_node_telemetry`.
- In `src-tauri/src/modules/telegram_inbound/`:
  - Handle command `/fleet_sync <json_or_doc>`.
  - Ingest payload and update local fleet database.

### Step 5: Author Top-Level `FleetSummaryView.tsx` Component
- In `src/components/instances/fleet/FleetSummaryView.tsx`:
  - Implement top-level container:
    - Summary metrics bar: Total Nodes Online/Offline, Total Instances, Total In-Flight Tasks.
    - Transport toggle capsule (`[ All Transports | 📧 Email | 📱 Telegram ]`).
    - Manual refresh button with spinning state and auto-polling countdown.
  - Implement Node Cards Grid:
    - Card header: Node alias, online pulse indicator, OS icon/label, IP address, uptime badge.
    - Embedded compact table: List of profiles on that node with PID, account email, quota bar, and executable path.
    - Running workspace drawer: Displays active prompts and turn statuses.
- Export and integrate `FleetSummaryView` into `InstancesPage` or tabbed section in `src/pages/Instances.tsx`.

---

## Verification & Preflight Commands

Execute following verification commands locally:
```bash
# 1. Rust code formatting check
cd src-tauri && cargo fmt -- --check

# 2. Rust clippy gate (all features and targets)
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Frontend compilation check
npm run build
```
