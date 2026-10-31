# 4-Part Root Cause Analysis (RCA): Alex Hudson Selection and Clustering Issue

## 1. Symptom

1. **Cross-Machine Account Collision**:
   - In instance management, instance `#2 lara-pub` (PID 7404, ID `lara-pub-6067`) selected account `alex.hudson.riseup@gmail.com` [PRO] despite the same account already being bound and active on another instance / machine in the shared cluster.
2. **Missing Node W2 and Local Machine Mislabeled as W1**:
   - In the "Shared Cluster / Fleet Nodes" view, the local machine (IP `192.168.1.7`) was labeled `w1 [Lead] [Local] [ONLINE]`.
   - Node `w2` was missing from the table entirely.
   - Remote worker node `w3` (and others) were omitted or displaced.

## 2. Root Cause

1. **Local-Only Account Exclusion in Auto-Switcher**:
   - In `src-tauri/src/modules/auto_switcher/instances.rs` (`get_active_in_use_account_ids`) and `selection.rs` (`select_candidate_profiles`), candidate exclusion only inspected local running instances from `list_running_or_active_instances()`.
   - The selection logic never queried `query_fleet_machines()` or cached cluster state to identify accounts currently occupied by remote fleet nodes.
   - In the frontend (`src/stores/useInstanceStore/smart.ts`), `activeInUseAccountIds` only filtered `get().instances` (local instances).
   - Consequently, when Fast-Forward or Smart-Switch evaluated candidates, accounts active on remote machines (such as `alex.hudson.riseup@gmail.com`) appeared completely free and available.
2. **Hardcoded `"w1"` in Fleet Aggregation Query**:
   - In `src-tauri/src/modules/supabase_sync/query.rs` (lines 169-170, 192-193, 490, 496, 620-621), the local machine alias was hardcoded:
     `local_machine.node_alias = "w1".to_string();`
     `local_machine.alias = "w1".to_string();`
   - In line 479, `ip_address == "192.168.1.3"` was unconditionally hardcoded as `is_local = true`.
   - When the local machine (IP `192.168.1.7`) ran `query_fleet_machines()`, it was forced to adopt alias `"w1"`.
   - When merging peer connections and `GITMAP_CLUSTER_NODES`:
     - Node `w2` (IP `192.168.1.7`) was dropped because its IP matched the local machine's IP.
     - Node `w1` (IP `192.168.1.3`) was dropped because its alias matched the local machine's overridden alias `"w1"`.
     - Node `w2` completely disappeared, and the real `w1` was masked by the local machine.

## 3. Systematic Fix

1. **Cluster-Aware Account Exclusion**:
   - Extend `get_active_in_use_account_ids()` in `instances.rs` and `select_candidate_profiles()` in `selection.rs` to extract all active accounts from `query_fleet_machines()` / cached fleet state.
   - In frontend `smart.ts`, enrich `activeInUseAccountIds` with accounts reported by `useFleetMachines` / cached fleet nodes.
   - Ensure accounts occupied by ANY instance on ANY machine in the cluster are strictly excluded from candidate selection.
2. **Dynamic Machine Alias Resolution & Node Preservation**:
   - Eliminate all hardcoded `"w1"` assignments in `src-tauri/src/modules/supabase_sync/query.rs`.
   - Use `resolve_local_node_alias()` to dynamically determine the local machine's alias based on Cluster-IP primacy (`192.168.1.7` $\rightarrow$ `w2`, `192.168.1.12` $\rightarrow$ `w3`, `192.168.1.3` $\rightarrow$ `w1`), GitMap `variables.json`, and configured `node_alias`.
   - Preserve all 6 canonical nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) without dropping any node due to false deduplication.
3. **Transparent Algorithm Rationale ("Let It Show")**:
   - Return and log the selection tier, score breakdown, and exclusion reasons in rotation telemetry.

## 4. Prevention & Guardrails

- Enforce invariant in CI and unit tests that `query_fleet_machines()` must always return all 6 canonical cluster nodes.
- Add regression test verifying that an account active on any remote fleet node is strictly excluded from Fast-Forward and Smart-Switch candidate selection.
- Ban hardcoded node aliases (`"w1"`) across the codebase.
