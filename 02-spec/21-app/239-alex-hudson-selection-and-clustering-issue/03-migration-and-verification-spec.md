# Migration & Verification Specification: Alex Hudson Selection and Clustering

## 1. Migration Strategy

- **No Schema Changes Required**: Uses existing SQLite schemas (`accounts`, `instances`, `ssh_nodes.json`, `gui_config.json`).
- **Backward Compatibility**: All existing CLI flags (`agm ff`, `agm auto-switch`, `agm fleet machines`) remain 100% backward compatible while gaining cluster-aware deduplication and transparent decision telemetry.

## 2. Verification Test Plan

### Test 1: Node Alias Resolution Parity
- **Input**: Local machine with IP `192.168.1.7`.
- **Expected Outcome**: `resolve_local_node_alias()` returns `"w2"` (or configured alias), NOT hardcoded `"w1"`.
- **Verification**: `query_fleet_machines()` returns 6 distinct nodes: `w1` at `192.168.1.3`, `w2` at `192.168.1.7` marked `is_local = true`, `w3` at `192.168.1.12`, `w4` at `192.168.1.13`, `u1` at `192.168.1.22`, and `main` at `192.168.1.20`.

### Test 2: Cross-Machine Account Exclusion
- **Setup**: Account `alex.hudson.riseup@gmail.com` marked active on remote node `w1`.
- **Action**: Trigger Fast-Forward rotation on local instance `#2 lara-pub`.
- **Expected Outcome**: `alex.hudson.riseup@gmail.com` is strictly excluded from candidates. Another eligible healthy candidate (e.g. `david.copper.riseup@gmail.com` or `rokixshohag1@gmail.com`) is selected.
- **Verification**: Telemetry logs show `alex.hudson.riseup@gmail.com` under `fleet_excluded_accounts`.

### Test 3: Frontend & Backend Pre-flight Gates
- Run `npm run build` or `vite build` without errors.
- Verify TypeScript compilation via `tsc --noEmit`.
- Run frontend unit tests via `node scripts/run-frontend-tests.mjs`.
