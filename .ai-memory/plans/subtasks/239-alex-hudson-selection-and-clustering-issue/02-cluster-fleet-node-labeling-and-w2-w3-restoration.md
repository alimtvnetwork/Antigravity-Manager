# Subtask 02: Cluster Fleet Node Labeling & W2/W3 Parity

- **Slug**: `alex-hudson-selection-and-clustering-issue - Task 02`
- **Spec**: `02-spec/21-app/239-alex-hudson-selection-and-clustering-issue/01-architecture-spec.md`
- **Status**: `COMPLETED`

## Checklist
- [x] Inspect and eliminate all hardcoded `"w1"` overrides in `src-tauri/src/modules/supabase_sync/query.rs`
- [x] Update `src-tauri/src/modules/supabase_sync/node.rs` (`resolve_local_node_alias`) to inspect GitMap `variables.json` (`machine.alias`), Cluster-IP matching, and config alias
- [x] Ensure `is_local` determination only checks true local IP and local node ID, never hardcoded IP `192.168.1.3`
- [x] Ensure all 6 canonical cluster nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) are preserved in `build_local_and_ssh_fleet()` and `query_fleet_machines()` without dropping `w2` or `w1`
