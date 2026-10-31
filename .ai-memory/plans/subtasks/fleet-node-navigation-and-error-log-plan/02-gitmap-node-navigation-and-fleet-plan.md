# Subtask 02 Plan: GitMap Node Navigation, 4-Tier Fleet Mesh & Remote Instance Matrix

- **Parent Task**: `fleet-node-navigation-and-error-log-plan`
- **Subtask ID**: `Task-02`
- **Subtask Plan Path**: `.ai-memory/plans/subtasks/fleet-node-navigation-and-error-log-plan/02-gitmap-node-navigation-and-fleet-plan.md`
- **Associated Specification**: `02-spec/21-app/fleet-node-navigation-and-error-log-plan/02-gitmap-node-navigation-and-fleet-spec.md`
- **Execution Mode**: `PLAN MODE (Implementation Frozen During Ongoing Codebase Refactors)`
- **Lead Agent**: `Task Orchestrator`
- **Target Files**: Rust Backend (`src-tauri/`), React Frontend (`src/`), AGM Terminal CLI (`src-tauri/src/bin/agm/`)

---

## 1. Subtask Objective

Deliver seamless multi-machine discovery and navigation parity with GitMap across Antigravity Manager (`AGM`). This plan resolves the six architectural disconnects (GAP-01 to GAP-06) that prevent remote cluster nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) from appearing in AGM when Supabase is disabled, paused, or unreachable. 

It establishes a resilient **4-Tier Discovery Ladder**, bridges SSH Manager cluster telemetry into the Instances tab (`src/pages/Instances.tsx`), enables bidirectional GitMap configuration aliasing, and implements terminal navigation commands (`agm fleet nodes`, `agm fleet ping`, `agm fleet exec`, `agm fleet instances`).

---

## 2. File-by-File Implementation Checklist

### 2.1 Backend SSH Manager & GitMap Schema Invariant (`src-tauri/src/modules/ssh_manager/`)

- [ ] **`src-tauri/src/modules/ssh_manager/sync.rs`**:
  * **Fix Command Args at Line 173**: Replace unsupported command candidates `&["ssh", "nodes", "export-json"]` and `&["ssh", "nodes", "--json"]` with `&["nodes", "--json"]`.
  * **Windows Process Isolation**: Maintain `cmd.creation_flags(0x08000000)` (CREATE_NO_WINDOW) to prevent flashing CMD prompts during GitMap discovery.
  * **Logging Context**: Replace bare error ignores with structured tracing via `tracing::warn!(target: "ssh_manager", ...)` when GitMap CLI is not found or fails.

- [ ] **`src-tauri/src/modules/ssh_manager/types.rs`**:
  * **Serde Field Aliasing**:
    - In `SshNodeExportItem`:
      * `#[serde(alias = "host")]` on `pub ip_address: String`
      * `#[serde(alias = "user")]` on `pub username: String`
      * `#[serde(default)]` on `pub role: String`
      * `#[serde(default)]` on `pub status: String`
      * `#[serde(default)]` on `pub subsystems: Vec<String>`
      * `#[serde(default)]` on `pub is_online: bool`
    - In `SshConnectionRecord`:
      * `#[serde(alias = "host")]` on `pub ip_address: String`
      * `#[serde(alias = "user")]` on `pub username: String`
      * `#[serde(default)]` on `pub role: Option<String>`
      * `#[serde(default)]` on `pub is_online: Option<bool>`

- [ ] **`src-tauri/src/modules/ssh_manager/connections.rs`**:
  * **JSON Envelope Unpacking**: Update `decode_connections_from_json()` to gracefully deserialize raw arrays emitted by `gitmap nodes --json`.
  * **Automatic Path Normalization**: Ensure private key paths stored in `gitmap-ssh-nodes.json` (such as `C:\Users\Administrator\.ssh\id_rsa`) pass through `resolve_usable_key_path()` across Windows and Unix runners.

---

### 2.2 Backend Multi-Tier Fleet Query Engine (`src-tauri/src/modules/supabase_sync/`)

- [ ] **`src-tauri/src/modules/supabase_sync/query.rs`**:
  * **4-Tier Discovery Resolution**: Refactor `query_fleet_machines()` to aggregate nodes across all four tiers:
    ```rust
    pub async fn query_fleet_machines() -> Result<Vec<FleetMachineInfo>, AppError> {
        // Tier 1: Query Supabase if enabled and configured
        let (supabase_machines, supabase_ok) = query_tier1_supabase().await;
        
        // Tier 2: Query Local GitMap SSH Mesh concurrently
        let gitmap_machines = query_tier2_gitmap_ssh().await;
        
        // Tier 3: Query Decentralized SQLite Fleet Cache (fleet_sync.db)
        let sqlite_machines = query_tier3_sqlite_fleet();
        
        // Tier 4: Query Local Machine Telemetry (Baseline Fallback)
        let local_machine = build_local_fallback_machine();
        
        // Merge with precedence & deduplicate by node_id / IP address
        Ok(merge_fleet_nodes(supabase_machines, gitmap_machines, sqlite_machines, local_machine))
    }
    ```
  * **Remove Hard PostgREST Exit**: Eliminate early returns that exit before probing Tier 2 GitMap SSH nodes.
  * **Latency Scoring**: Store TCP ping round-trip latency (`latency_ms`) in `FleetMachineInfo`.

- [ ] **`src-tauri/src/modules/supabase_sync/migration.rs`**:
  * **Enrich Local Fallback**: Update `build_local_fallback_machine()` to include loaded SSH nodes from `ssh_manager::load_ssh_connections()` so that headless and offline runs retain cluster node awareness.

- [ ] **`src-tauri/src/modules/supabase_sync/types.rs`**:
  * **Model Extension**:
    * Add `pub latency_ms: Option<u64>` to `FleetMachineInfo`.
    * Add `pub role: String` to `FleetMachineInfo`.
    * Ensure `source` field is properly serialized as `"supabase"`, `"gitmap-ssh"`, `"hybrid"`, or `"local"`.

---

### 2.3 AGM CLI Navigation Suite (`src-tauri/src/bin/agm/fleet_cmds.rs`)

- [ ] **`cmd_fleet_nodes(args: &[String])`**:
  * Replace the single-node local stub with a call to the 4-Tier discovery resolver (`supabase_sync::query_fleet_machines()`).
  * Render an aligned ASCII table displaying: `ALIAS`, `ROLE`, `HOST (IP:PORT)`, `USER`, `STATUS`, `SOURCE`, `LATENCY`, and `INSTANCES`.
  * Format status with colored badges: `● online` (green) and `○ offline` (slate/dim).
  * Handle `--json` flag to print structured `CliEnvelope::ok("fleet nodes", ...)`.

- [ ] **`cmd_fleet_ping(args: &[String])`**:
  * Implement target argument parsing: `agm fleet ping [alias|all]`.
  * Execute concurrent TCP socket probes (timeout: `1500ms`) using a bounded thread pool or tokio async tasks.
  * Print formatted ping results with round-trip latency in milliseconds.
  * Emit machine-readable output when `--json` is supplied.

- [ ] **`cmd_fleet_exec(args: &[String])`**:
  * Implement command dispatch: `agm fleet exec <alias|all> "<cmd>"`.
  * Dispatch via `ssh_manager::exec::execute_remote_ssh_command()`.
  * Aggregate standard output, stderr, and exit codes per node.

- [ ] **`cmd_fleet_instances(args: &[String])`**:
  * Implement remote instance inspection: `agm fleet instances <alias>`.
  * Connect to remote node over SSH and invoke `agm.exe instances --json` (or inspect `state.vscdb` / `instances.json`).
  * Tabulate remote instance IDs, names, PIDs, active models, and prompt statuses.

---

### 2.4 Frontend Stores & Hooks (`src/components/instances/fleet/`)

- [ ] **`src/components/instances/fleet/useFleetMachines.ts`**:
  * **Remove Premature Exit**: Delete `if (!syncActive) { setMachines([]); return; }`.
  * **Decouple Sync State**: Treat `isSyncEnabled` as true if either Supabase Sync is active OR at least one GitMap SSH node is configured in the cluster.
  * **Fix Local ID Filter**: Update `filtered = data.filter((m) => m.node_id !== effectiveLocalId)`. If `data` contains remote nodes (from GitMap SSH or Supabase), retain those nodes and strip only the local node. If only the local node is present, handle gracefully without masking errors.
  * **Dynamic Refresh Interval**: Maintain the 15-second polling countdown with automatic pause when the browser tab is hidden (`document.visibilityState === 'hidden'`).

- [ ] **`src/components/instances/fleet/fleetTypes.ts`**:
  * Synchronize TypeScript definitions with Rust backend:
    ```typescript
    export interface FleetMachineInfo {
        node_id: string;
        node_alias: string;
        alias?: string;
        role?: string;
        os_info?: string;
        ip_address: string;
        is_online: boolean;
        last_heartbeat_timestamp: number;
        uptime_seconds: number;
        in_flight_prompts_count: number;
        active_instances: FleetInstanceSummary[];
        bound_emails: string[];
        source?: 'supabase' | 'gitmap-ssh' | 'hybrid' | 'local';
        latency_ms?: number | null;
        status?: string;
    }
    ```

---

### 2.5 UI Presentation Layer (`src/components/instances/`)

- [ ] **`src/components/instances/FleetMachinesTable.tsx`**:
  * **Table Headers**: Add `Source`, `Host (IP:Port)`, `Latency`, and `Actions` columns.
  * **Cluster State Header**: Update descriptive subtitle from *"Real-time multi-machine cluster state synced via Supabase"* to *"Multi-machine cluster topology aggregated via GitMap SSH Mesh & Supabase Sync"*.
  * **Graceful State Handling**: Replace `<FleetDisabledState />` with a subtle inline warning banner if Supabase is disabled while GitMap SSH nodes are actively displayed.

- [ ] **`src/components/instances/fleet/FleetMachineRow.tsx`**:
  * **Source Badge**:
    * Render `GitMap SSH` pill (`bg-cyan-500/15 text-cyan-400 border border-cyan-500/30 font-mono text-[10px]`).
    * Render `Supabase` pill (`bg-purple-500/15 text-purple-400 border border-purple-500/30 font-mono text-[10px]`).
    * Render `Hybrid` pill (`bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 font-mono text-[10px]`).
  * **Latency Pill**: Display live round-trip latency next to the IP address (e.g., `2ms` in emerald, `120ms` in amber).
  * **Action Controls (Contiguous Segmented Pill Capsule)**:
    * `[Ping]`: Calls backend TCP probe and updates row latency in real-time.
    * `[SSH]`: Copies SSH terminal connection string or triggers local terminal launch.
    * `[Inspect]`: Opens remote instance drawer showing active profiles and running prompt counts.

- [ ] **`src/components/instances/fleet/FleetTableStates.tsx`**:
  * Update `FleetEmptyState` with actionable buttons:
    * `[Import from GitMap]`: Triggers `import_ssh_nodes_from_gitmap`.
    * `[Configure SSH Nodes]`: Navigates to `/settings` SSH section.

---

## 3. Test Suites & Quality Verification

### 3.1 Backend Rust Tests
- [ ] **`src-tauri/src/modules/ssh_manager/tests.rs`**:
  * `test_gitmap_nodes_json_deserialization()`: Assert that raw JSON output from `gitmap nodes --json` (with `host`, `user`, and `status` keys) successfully deserializes into `Vec<SshConnectionRecord>`.
  * `test_candidate_seed_paths_discovery()`: Verify that `gitmap-ssh-nodes.json` in the root workspace is prioritized and discovered.

- [ ] **`src-tauri/src/modules/supabase_sync/tests.rs`**:
  * `test_4tier_fleet_resolution_without_supabase()`: Mock Supabase as disabled/unreachable, verify `query_fleet_machines()` returns GitMap SSH nodes (`w1`, `w2`, etc.).
  * `test_node_deduplication_and_merge()`: Assert that duplicate node entries across Supabase and GitMap SSH are merged into a single record with `source = "hybrid"`.

### 3.2 Frontend React Tests
- [ ] **`src/components/instances/__tests__/fleetMachinesTable.test.ts`**:
  * Test that remote machines are displayed in the table when `is_sync_enabled = false` but `source = 'gitmap-ssh'`.
  * Test that latency pills and source badges render appropriate color classes.

---

## 4. Verification Procedures & Execution Gates

Before marking implementation complete, the future AI agent must execute:

1. **Rust Code Formatting Gate**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
2. **Rust Linter & Compilation Gate**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
3. **Frontend Compilation Gate**:
   ```bash
   npm run build
   ```
4. **CLI Manual Verification**:
   ```bash
   cargo run --bin agm -- fleet nodes
   cargo run --bin agm -- fleet ping w1
   ```
   *Expected Outcome*: Both `main` and `w1` are listed with status `● online`, latency is reported accurately, and no panics or unhandled errors occur.
