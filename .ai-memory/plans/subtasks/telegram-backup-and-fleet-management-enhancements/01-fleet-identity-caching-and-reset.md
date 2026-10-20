# Subtask Plan 01: Fleet Machine Identity, Caching, Cluster Reset, and Cross-Machine Dispatch

- **Subtask Name:** Fleet Machine Identity, Caching, Cluster Reset, and Cross-Machine Dispatch
- **Task Slug:** `telegram-backup-and-fleet-management-enhancements`
- **File:** `.ai-memory/plans/subtasks/telegram-backup-and-fleet-management-enhancements/01-fleet-identity-caching-and-reset.md`
- **Parent Spec:** `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/01-architecture-spec.md`
- **Target Release:** `v4.161.0`
- **Status:** READY FOR IMPLEMENTATION

---

## 1. Objectives

1. **Machine Identity & 5-Node Collision Fix**:
   - Reconcile `192.168.1.3` strictly to `w1` (local lead) and `192.168.1.20` strictly to `main` via Cluster-IP primacy in `src-tauri/src/modules/supabase_sync/node.rs`.
   - Fix deduplication logic in `query_fleet_machines` (`src-tauri/src/modules/supabase_sync/query.rs`) so that `w1` and `main` never drop each other due to alias/IP cross-matching.
   - Fix `SshNodeItem` in `src/components/instances/fleet/useFleetMachines.ts` to include `ip_address: string`, preventing `undefined` IP values when mapping from Tauri IPC `list_ssh_nodes`.

2. **30-Second TTL Caching & Frontend SWR Decoupling**:
   - Add `CLUSTER_BALANCES_CACHE` with 30s TTL in `src-tauri/src/modules/ssh_manager/exec.rs` to fast-fail/fast-return cluster instance and quota balance telemetry.
   - Decouple `enrichFleetMachines()` in `src/components/instances/fleet/useFleetMachines.ts` from the initial table rendering path, adopting a non-blocking Stale-While-Revalidate (SWR) architecture to eliminate UI freezing.

3. **SSH Cluster Reset Subsystem**:
   - Update `reset_ssh_cluster_nodes()` in `src-tauri/src/modules/ssh_manager/sync.rs` to reload configurations from candidate `gitmap-ssh-nodes.json` and existing `ssh_nodes.json`, preserving existing passwords and key paths instead of blanking them out.
   - Clear `SSH_LIVENESS_CACHE` and `CLUSTER_BALANCES_CACHE` on reset to force immediate re-verification of TCP liveness and instance states.
   - Refresh the UI table immediately upon reset completion.

4. **Cross-Machine Prompt Dispatch via Base64**:
   - Encode remote prompt payloads as RFC 4648 Base64 strings in `src-tauri/src/modules/telegram_inbound/injection.rs` (`execute_remote_prompt_injection`) and `src-tauri/src/bin/agm/prompt_dispatch.rs` (`cmd_prompt_dispatch`).
   - Add `--base64` / `-b64` / `--b64` decoding support to `cmd_prompt_dispatch` in `src-tauri/src/bin/agm/prompt_dispatch.rs` to ensure multi-line prompts, code blocks, quotes, and shell variables survive remote execution over SSH without escaping errors.

---

## 2. Target Files & Edit Scope

| Layer | Relative File Path | Component / Function | Scope of Edit |
|:---|:---|:---|:---|
| **Rust Backend** | `src-tauri/src/modules/supabase_sync/node.rs` | `resolve_local_node_alias()` | Enforce Cluster-IP primacy: prioritize `GITMAP_CLUSTER_NODES` matching `192.168.1.3` -> `w1` and `192.168.1.20` -> `main` before checking local SSH connection files. |
| **Rust Backend** | `src-tauri/src/modules/supabase_sync/query.rs` | `build_local_and_ssh_fleet()`, `query_fleet_machines()` | Fix peer deduplication logic: ensure local machine matching requires IP identity, and prevent `w1` and `main` from dropping each other during deduplication loops. |
| **Rust Backend** | `src-tauri/src/modules/ssh_manager/exec.rs` | `CLUSTER_BALANCES_CACHE`, `get_cluster_instances_and_balances()` | Add 30s TTL in-memory cache `CLUSTER_BALANCES_CACHE` and expose `invalidate_cluster_balances_cache()`. Return cached balances in <1ms. |
| **Rust Backend** | `src-tauri/src/modules/ssh_manager/sync.rs` | `reset_ssh_cluster_nodes()` | Read existing credentials from `gitmap-ssh-nodes.json` / `ssh_nodes.json` to preserve passwords and key paths. Invalidate `SSH_LIVENESS_CACHE` and `CLUSTER_BALANCES_CACHE`. |
| **Rust Backend** | `src-tauri/src/modules/telegram_inbound/injection.rs` | `execute_remote_prompt_injection()` | Encode remote prompt payload into Base64 before executing `agm prompt ... --base64 <b64>`. |
| **Rust Backend** | `src-tauri/src/bin/agm/prompt_dispatch.rs` | `cmd_prompt_dispatch()` | Add `--base64` flag parser and decoder; use Base64 encoding when delegating via `--node <alias>`. |
| **TypeScript Frontend** | `src/components/instances/fleet/useFleetMachines.ts` | `SshNodeItem`, `mapSshNodesToFleet()`, `fetchMachines()` | Update `SshNodeItem` interface to include `ip_address?: string`. Decouple `enrichFleetMachines()` into a non-blocking background promise. |

---

## 3. Step-by-Step Implementation Instructions

### Step 1: Cluster-IP Primacy in `supabase_sync/node.rs`
- In `src-tauri/src/modules/supabase_sync/node.rs`:
  - Locate `pub fn resolve_local_node_alias() -> String`.
  - Reorder the evaluation ladder:
    1. Retrieve `local_ip = get_local_ip()`.
    2. Check `GITMAP_CLUSTER_NODES` **FIRST**:
       ```rust
       for &(alias, ip, _, _) in crate::modules::ssh_manager::GITMAP_CLUSTER_NODES {
           if ip == local_ip.trim() {
               return alias.to_string();
           }
       }
       ```
    3. Check `load_ssh_connections()` for IP match only after the canonical cluster check.
    4. Follow existing fallbacks for email watcher machine name, Supabase config, and final `"w1"`.
  - Invariant: A host with local IP `192.168.1.3` MUST deterministically resolve to alias `"w1"`. A host with `192.168.1.20` MUST deterministically resolve to `"main"`.

### Step 2: Fix Deduplication in `supabase_sync/query.rs`
- In `src-tauri/src/modules/supabase_sync/query.rs`:
  - In `build_local_and_ssh_fleet()`:
    - Replace loose OR check:
      ```rust
      // Instead of:
      // if conn.ip_address == local_ip || conn.alias.eq_ignore_ascii_case(&local_alias)
      // Use exact identity matching:
      let is_self = conn.ip_address == local_ip 
          || (conn.alias.eq_ignore_ascii_case(&local_alias) && (local_ip.is_empty() || conn.ip_address == local_ip));
      if is_self {
          if let Some(local_machine) = machines.first_mut() {
              local_machine.node_alias = conn.alias.clone();
              local_machine.alias = conn.alias;
          }
          continue;
      }
      ```
  - In `query_fleet_machines()`:
    - Replace the deduplication loop over `all_conns` to ensure canonical cluster nodes are not dropped against each other:
      ```rust
      for conn in all_conns {
          let is_dup = machines.iter().any(|m| {
              m.ip_address == conn.ip_address
                  || (m.node_alias.eq_ignore_ascii_case(&conn.alias) && m.ip_address == conn.ip_address)
          });
          if !is_dup {
              machines.push(build_ssh_mesh_machine_info(&conn, now));
          }
      }
      ```

### Step 3: Frontend Type Parity & Fallback Fix in `useFleetMachines.ts`
- In `src/components/instances/fleet/useFleetMachines.ts`:
  - Update `interface SshNodeItem`:
    ```typescript
    interface SshNodeItem {
        alias: string;
        ip_address?: string;
        ip?: string;
        status?: string;
        port?: number;
        os?: string;
    }
    ```
  - In `mapSshNodesToFleet(sshNodes)`:
    ```typescript
    let alias = n.alias.trim();
    let ip = (n.ip_address || n.ip || '').trim();
    ```
  - This ensures that when `list_ssh_nodes` returns `SshNodeRecord` objects with `ip_address`, the IP is never blank.

### Step 4: Add `CLUSTER_BALANCES_CACHE` in `ssh_manager/exec.rs`
- In `src-tauri/src/modules/ssh_manager/exec.rs`:
  - Declare static cache:
    ```rust
    pub static CLUSTER_BALANCES_CACHE: Lazy<RwLock<Option<(Vec<ClusterInstanceBalanceItem>, Instant)>>> =
        Lazy::new(|| RwLock::new(None));

    pub fn invalidate_cluster_balances_cache() {
        if let Ok(mut cache) = CLUSTER_BALANCES_CACHE.write() {
            *cache = None;
        }
    }
    ```
  - In `get_cluster_instances_and_balances()`:
    - At start: check read lock. If elapsed time < 30s, return cached items.
    - At end before returning: write items and `Instant::now()` into `CLUSTER_BALANCES_CACHE`.

### Step 5: Decouple `enrichFleetMachines()` in `useFleetMachines.ts`
- In `src/components/instances/fleet/useFleetMachines.ts`:
  - In `fetchMachines`:
    - Immediately call `setMachines(sortFleetMachines(effectiveMachines))` as soon as baseline data is fetched.
    - Trigger `enrichFleetMachines(effectiveMachines)` in the background without `await`:
      ```typescript
      enrichFleetMachines(effectiveMachines)
          .then((enriched) => {
              setIsSyncEnabled(syncActive || enriched.length > 0);
              setMachines(sortFleetMachines(enriched));
          })
          .catch((e) => {
              useErrorStore.getState().trackWarning(e, {
                  source: 'useFleetMachines.asyncEnrich',
                  triggerAction: 'enrich_fleet_machines',
              });
          });
      ```
    - Apply identical decoupled handling to the `init()` function inside `useEffect`.

### Step 6: Update `reset_ssh_cluster_nodes()` in `ssh_manager/sync.rs`
- In `src-tauri/src/modules/ssh_manager/sync.rs`:
  - In `reset_ssh_cluster_nodes()`:
    1. Scan candidate paths (`candidate_seed_node_paths()`) and load existing `ssh_nodes.json`.
    2. Build a map of existing credentials: `HashMap<String, (String, String, String)>` (mapping `alias.to_lowercase()` and `ip_address` to `(encrypted_password, encryption_algo, key_path)`).
    3. Clear `node_display_cache.db`.
    4. Call `invalidate_ssh_liveness_cache(None, None)`.
    5. Call `invalidate_cluster_balances_cache()`.
    6. Construct canonical records for `GITMAP_CLUSTER_NODES`, injecting the preserved credentials if present.
    7. Execute `sync_ssh_connections_locally(&canonical_records)`.
    8. Execute `update_ssh_config(false)`.

### Step 7: Base64 Remote Prompt Dispatch & CLI Handler
- In `src-tauri/src/modules/telegram_inbound/injection.rs`:
  - In `execute_remote_prompt_injection`:
    - Use `base64::engine::general_purpose::STANDARD.encode(prompt_text.as_bytes())`.
    - Format command:
      ```rust
      let b64_prompt = base64::engine::general_purpose::STANDARD.encode(prompt_text.as_bytes());
      let remote_cmd = format!("{agm_bin} prompt {target_proj}{inst_flag} --base64 {b64_prompt}");
      ```
- In `src-tauri/src/bin/agm/prompt_dispatch.rs`:
  - In `cmd_prompt_dispatch(args)`:
    - Add parser for `--base64` / `-b64` / `--b64`:
      ```rust
      } else if arg == "--base64" || arg == "-b64" || arg == "--b64" {
          if i + 1 < args.len() {
              if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(args[i + 1].trim()) {
                  if let Ok(decoded_text) = String::from_utf8(bytes) {
                      text_parts.push(decoded_text);
                  }
              }
              i += 2;
              continue;
          }
      }
      ```
    - When delegating to remote node via `explicit_node`:
      ```rust
      let b64 = base64::engine::general_purpose::STANDARD.encode(final_prompt.as_bytes());
      let remote_cmd = format!("agm prompt {}{} --base64 {}", target_arg, inst_arg, b64);
      ```

---

## 4. Edge Cases & Safeguards

1. **Stale / Corrupted Disk Caches**:
   If an operator manually edited `ssh_nodes.json` or display caches with conflicting aliases, Step 1 guarantees that physical IP `192.168.1.3` always yields `w1` and `192.168.1.20` always yields `main`.
2. **Missing `gitmap-ssh-nodes.json` during Reset**:
   If seed files are absent on a clean install, `reset_ssh_cluster_nodes()` smoothly defaults to empty passwords/keys without throwing errors, preserving canonical node records.
3. **Invalid or Corrupted Base64 Strings**:
   If an external caller passes malformed base64, `cmd_prompt_dispatch` handles the `Err` gracefully without panicking or creating empty prompts.
4. **Offline Nodes during Balance Scan**:
   `check_tcp_liveness` fast-fails offline nodes within 10ms via `SSH_LIVENESS_CACHE`, preventing blocked thread pools.
5. **No Silent Errors Rule**:
   All fallible operations (`fs::remove_file`, `std::fs::write`, `Connection::open`) must propagate via `AppError` or log through `crate::error::record_ignored` with explicit justification comments.

---

## 5. Verification Steps

1. **Compilation & Formatting Gate**:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
2. **Machine Identity & 6-Node Invariant Verification**:
   - Run `agm fleet machines` or query `get_fleet_machines` via Tauri IPC.
   - Confirm that the output lists all 6 nodes (`w1`, `main`, `u1`, `w2`, `w3`, `w4`).
   - Confirm `w1` has IP `192.168.1.3` and `main` has IP `192.168.1.20`. Neither node is dropped or collapsed.
3. **Caching & Latency Benchmark**:
   - Measure execution time of subsequent calls to `get_cluster_instances_and_balances`. Confirm response returns in <5ms on cache hit.
   - Check frontend navigation: verify that the Fleet table renders immediately without freezing while quota badges stream in.
4. **Cluster Reset Verification**:
   - Add a test credential to `gitmap-ssh-nodes.json`.
   - Call `reset_ssh_cluster`.
   - Inspect `ssh_nodes.json` and verify the password/key path was retained for the matching node.
   - Verify `SSH_LIVENESS_CACHE` was cleared.
5. **Base64 Prompt Dispatch Verification**:
   - Dispatch a multiline test prompt containing quotes, markdown tables, and `$PATH` via `agm prompt default --base64 <b64>`.
   - Verify that `active_prompts` in SQLite contains the exact decoded text character-for-character.
