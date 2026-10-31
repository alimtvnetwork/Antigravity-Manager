---
plan: ui-and-functionality-enhancement-instructions
subtask: "02"
title: SSH Node Connectivity, GitMap Parity, and Shared Cluster Restoration
domain: rust-backend-and-frontend
depends_on: "01"
citations:
  component_spec: 02-spec/21-app/ui-and-functionality-enhancement-instructions/02-component-spec.md
target_files:
  - src-tauri/src/modules/ssh_manager/exec.rs
  - src-tauri/src/modules/ssh_manager/paths.rs
  - src-tauri/src/modules/ssh_manager/sync.rs
  - src-tauri/src/commands/ssh.rs
  - src-tauri/src/modules/supabase_sync/query.rs
  - src/components/instances/fleet/useFleetMachines.ts
  - src/components/instances/FleetMachinesTable.tsx
  - src/components/instances/fleet/FleetMachineRow.tsx
status: pending
---

# Subtask 02: SSH Node Connectivity, GitMap Parity, and Shared Cluster Restoration

## 1. Context & Scope

This subtask implements the backend and frontend components specified in `02-spec/21-app/ui-and-functionality-enhancement-instructions/02-component-spec.md`:
1. **SSH Node Connectivity & GitMap Parity**:
   - Resolve remote node authentication failures (specifically node `w2` at `192.168.1.7:22`) by integrating GitMap's Go in-process SSH fallback (`golang.org/x/crypto/ssh`).
   - Support encrypted vault credentials (`winPass: salt:9f8b2c4e:UypnMmpIN3BMLg==` $\rightarrow$ `rtyrty123@` for Windows targets) from `repo-secrets/01-gitmap/06-vmpass.json` and `gitmap.db`.
   - Add automated public key injection and auth repair through `gitmap ssh fix-auth <target>` inside `src-tauri/src/modules/ssh_manager/exec.rs` and `src-tauri/src/commands/ssh.rs`.
   - Expand candidate seed paths in `src-tauri/src/modules/ssh_manager/paths.rs` to ingest `04-ssh-nodes.json`.
2. **Shared Cluster Section Restoration**:
   - Refactor `src-tauri/src/modules/supabase_sync/query.rs` to aggregate registered local SSH nodes (`ssh_nodes.json`, `gitmap.db`) alongside Supabase cloud sync nodes.
   - Refactor `src/components/instances/fleet/useFleetMachines.ts` to eliminate unconditional `setMachines([])` aborts when Supabase sync is disabled.
   - Update `src/components/instances/FleetMachinesTable.tsx` and `FleetMachineRow.tsx` to display registered SSH fleet machines in "SSH Mesh Mode" when Supabase is off, ensuring continuous multi-machine visibility.

---

## 2. Target Files & Symbols Breakdown

| Target File | Target Function / Symbol | Line Range (Approx) | Modification Action |
| :--- | :--- | :--- | :--- |
| `src-tauri/src/modules/ssh_manager/paths.rs` | `candidate_seed_node_paths` | Lines 43–56 | Add `04-ssh-nodes.json` and repo-secrets candidates for comprehensive node seeding. |
| `src-tauri/src/modules/ssh_manager/exec.rs` | `exec_ssh_command` | Lines 197–288 | Implement multi-stage execution ladder: Native OpenSSH $\rightarrow$ GitMap CLI exec fallback $\rightarrow$ GitMap fix-auth repair. |
| `src-tauri/src/modules/ssh_manager/exec.rs` | `fix_node_auth_via_gitmap` | New function | Helper to invoke `gitmap ssh fix-auth <target>` to inject public keys using vaulted password. |
| `src-tauri/src/commands/ssh.rs` | `test_ssh_node_connection` | Lines 49–61 | Update connection testing to utilize the GitMap fallback ladder and return diagnostic details. |
| `src-tauri/src/modules/supabase_sync/query.rs` | `query_fleet_machines` | Lines 9–370 | Aggregate local SSH connections from `load_ssh_connections()` when Supabase is disabled, and merge deduplicated nodes when enabled. |
| `src/components/instances/fleet/useFleetMachines.ts` | `useFleetMachines` | Lines 56–94, 97–143 | Remove `if (!syncActive) { setMachines([]); return; }`; query API unconditionally. |
| `src/components/instances/FleetMachinesTable.tsx` | `FleetMachinesTable` | Lines 27–37, 49–59 | Render table if `machines.length > 0` even when `isSyncEnabled == false`; update subtitle to reflect hybrid mode. |
| `src/components/instances/fleet/FleetMachineRow.tsx` | `FleetMachineRow` | Lines 40–80 | Add visual badge distinguishing `Supabase Cloud` vs `SSH Mesh / GitMap` node origin. |

---

## 3. Concrete Implementation Steps

### Step 1: Update Candidate Seed Paths in `paths.rs`

**File:** `src-tauri/src/modules/ssh_manager/paths.rs`

Expand `candidate_seed_node_paths()` to discover all variant filenames in `repo-secrets`:
```rust
pub(crate) fn candidate_seed_node_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("gitmap-ssh-nodes.json"),
        PathBuf::from("04-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/04-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/01-gitmap/gitmap-ssh-nodes.json"),
        PathBuf::from("../repo-secrets/01-gitmap/04-ssh-nodes.json"),
        PathBuf::from("../../repo-secrets/gitmap-ssh-nodes.json"),
        PathBuf::from("../../repo-secrets/01-gitmap/04-ssh-nodes.json"),
    ];
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".gitmap").join("gitmap-ssh-nodes.json"));
        paths.push(home.join(".gitmap").join("04-ssh-nodes.json"));
        paths.push(home.join(".gitmap").join("ssh-nodes.json"));
        paths.push(home.join("gitmap-ssh-nodes.json"));
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        paths.push(
            PathBuf::from(local_app_data)
                .join("gitmap-cli")
                .join("data")
                .join("gitmap-ssh-nodes.json"),
        );
    }
    paths
}
```

### Step 2: Implement GitMap Delegation & Fallback in `exec.rs`

**File:** `src-tauri/src/modules/ssh_manager/exec.rs`

1. Add `fix_node_auth_via_gitmap(target: &str) -> Result<String, String>`:
   - Probes `resolve_gitmap_bin()`.
   - Runs `gitmap ssh fix-auth <target>` with creation flag `0x08000000` on Windows.
   - Captures output and logs success/failure.
2. In `exec_ssh_command()`:
   - When native `ssh` returns an exit code != 0 or matches `Permission denied` / code 255:
     * Check if `gitmap` binary is available.
     * Execute `gitmap ssh exec <alias> "<cmd>"`.
     * If `gitmap ssh exec` succeeds: return `SshExecNodeResult` marked as successful via GitMap fallback.
     * If `gitmap ssh exec` indicates authorization failure: invoke `fix_node_auth_via_gitmap(alias)` once, then retry native OpenSSH execution.

### Step 3: Update `test_ssh_node_connection` in `commands/ssh.rs`

**File:** `src-tauri/src/commands/ssh.rs`

Update `test_ssh_node_connection`:
```rust
#[tauri::command]
pub async fn test_ssh_node_connection(id_or_alias: String) -> Result<SshExecNodeResult, String> {
    tokio::task::spawn_blocking(move || {
        let clean = id_or_alias.trim();
        let cmd = vec!["echo AGM_SSH_OK".to_string()];
        let results = ssh_manager::exec_ssh_command(clean, &cmd, None, None, None)?;
        let first = results
            .into_iter()
            .next()
            .ok_or_else(|| format!("No SSH node matched '{}'", clean))?;

        if !first.success && first.stderr.to_lowercase().contains("permission denied") {
            // Attempt auto-repair via GitMap fix-auth
            if let Ok(fix_msg) = ssh_manager::fix_node_auth_via_gitmap(clean) {
                tracing::info!(target: "ssh_manager", "Auto-repaired auth for {clean}: {fix_msg}");
                let retried = ssh_manager::exec_ssh_command(clean, &cmd, None, None, None)?;
                if let Some(res) = retried.into_iter().next() {
                    return Ok(res);
                }
            }
        }

        Ok(first)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
```

### Step 4: Multi-Source Hybrid Cluster Aggregation in `supabase_sync/query.rs`

**File:** `src-tauri/src/modules/supabase_sync/query.rs`

1. Refactor `query_fleet_machines()`:
   - When `!config.is_sync_enabled`:
     * Call `build_local_fallback_machine()`.
     * Call `crate::modules::ssh_manager::load_ssh_connections()`.
     * For each connection, run TCP probe (`check_tcp_liveness(&conn.ip_address, conn.port, 1500)`).
     * Construct a `FleetMachineInfo` for each node with `source: "ssh-mesh"`, `is_online`, `bound_emails`, and default active instance.
     * Return local machine + SSH nodes.
   - When `config.is_sync_enabled`:
     * Execute standard Supabase query across `nodes`, `instance_profiles`, `workspace_leases`.
     * Load SSH connections and merge nodes not present in Supabase (keyed by `ip_address` or `alias`).
     * Tag Supabase nodes with `source: "supabase"` and SSH-only nodes with `source: "ssh-mesh"`.
     * Sort with local first, online first, then alphabetical by alias.

### Step 5: Non-Blocking Cluster Hook in `useFleetMachines.ts`

**File:** `src/components/instances/fleet/useFleetMachines.ts`

1. In `fetchMachines`:
   - Keep `setIsSyncEnabled(syncActive)`.
   - Remove:
     ```typescript
     if (!syncActive) {
         setMachines([]);
         return;
     }
     ```
   - Always proceed to execute `fetchFleetMachinesApi()`.
   - Filter `data.filter((m) => m.node_id !== effectiveLocalId)`.
   - Update `setMachines(filtered)`.
2. In initial `useEffect`:
   - Always invoke `fetchFleetMachinesApi()` regardless of `syncActive`.

### Step 6: UI Visibility & Hybrid Mode Support in `FleetMachinesTable.tsx` and `FleetMachineRow.tsx`

**File:** `src/components/instances/FleetMachinesTable.tsx`

1. Update empty state logic:
   ```tsx
   // Only render disabled state if sync is disabled AND no local cluster/SSH nodes exist
   if (!isLoading && !isSyncEnabled && machines.length === 0) {
       return <FleetDisabledState />;
   }
   ```
2. Update header title and subtitle dynamically:
   - Subtitle:
     ```tsx
     <p className="text-[11px] text-slate-400">
         {isSyncEnabled
             ? 'Real-time multi-machine cluster state synced via Supabase & SSH Mesh'
             : 'Registered SSH Fleet Nodes (Local Mesh Mode)'}
     </p>
     ```
   - Add status pill if running without Supabase:
     ```tsx
     {!isSyncEnabled && (
         <span className="px-2 py-0.5 rounded-full text-[10px] font-medium bg-amber-500/20 text-amber-300 border border-amber-500/30">
             Local Mesh
         </span>
     )}
     ```

**File:** `src/components/instances/fleet/FleetMachineRow.tsx`

1. Render a source pill beside the machine status:
   - `machine.source === 'supabase'`: Sky blue badge (`Cloud`).
   - `machine.source === 'ssh-mesh'`: Purple badge (`SSH Mesh`).

---

## 4. Verification & Testing Checklist

- [ ] **Node W2 Reachability Verification**:
  - Run `gitmap ssh check w2` and verify network reachability.
  - Execute `test_ssh_node_connection("w2")` from AGM and confirm GitMap fallback / auto-repair functions correctly.
- [ ] **Shared Cluster Restoration Verification**:
  - Disable Supabase Sync in Settings $\rightarrow$ Cloud Sync.
  - Navigate to Instances screen $\rightarrow$ Shared Cluster section.
  - Verify that registered nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) render in the table with accurate IP, status, and instances.
  - Re-enable Supabase Sync and verify deduplicated hybrid aggregation.
- [ ] **Code Quality Gates**:
  - `cd src-tauri && cargo fmt -- --check`
  - `cd src-tauri && cargo clippy --all-targets --all-features`
  - `npm run build`
