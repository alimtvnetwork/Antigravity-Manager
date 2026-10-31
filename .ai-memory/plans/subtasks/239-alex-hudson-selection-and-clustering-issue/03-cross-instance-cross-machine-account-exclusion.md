# Subtask 03: Cluster-Wide Cross-Instance / Cross-Machine Account Exclusion

- **Slug**: `alex-hudson-selection-and-clustering-issue - Task 03`
- **Spec**: `02-spec/21-app/239-alex-hudson-selection-and-clustering-issue/02-component-and-algorithm-spec.md`
- **Status**: `COMPLETED`

## Checklist
- [x] Extend `get_active_in_use_account_ids()` in `src-tauri/src/modules/auto_switcher/instances.rs` to aggregate active accounts from `query_fleet_machines()` / cached fleet state
- [x] Update `select_candidate_profiles()` in `src-tauri/src/modules/auto_switcher/selection.rs` to exclude fleet-wide active accounts
- [x] Update `trigger_manual_rotation_for_instance()` in `src-tauri/src/modules/auto_switcher/manual.rs` to exclude fleet-wide active accounts
- [x] Update `src/stores/useInstanceStore/smart.ts` to enrich `activeInUseAccountIds` with cached fleet machine active accounts
- [x] Verify that `alex.hudson.riseup@gmail.com` is strictly excluded from candidate selection when active on another node
