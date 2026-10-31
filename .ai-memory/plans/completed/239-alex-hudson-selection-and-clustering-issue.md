# Plan: Alex Hudson Selection and Clustering Issue

## Task Identity
- **Task**: Alex Hudson Selection and Clustering Issue
- **Root Slug**: `alex-hudson-selection-and-clustering-issue`
- **Spec Directory**: `02-spec/21-app/239-alex-hudson-selection-and-clustering-issue/`
- **Status**: `COMPLETED`

## Subtask Breakdown

### Task-01: Spec Authoring & 4-Part Root Cause Analysis
- **Slug**: `alex-hudson-selection-and-clustering-issue - Task 01`
- **File**: `.ai-memory/plans/subtasks/239-alex-hudson-selection-and-clustering-issue/01-spec-and-cluster-rca.md`
- **Objective**: Author architectural spec, component spec, verification plan, and 4-part RCA in `02-spec/21-app/239-alex-hudson-selection-and-clustering-issue/`.
- **Status**: `COMPLETED`

### Task-02: Cluster Fleet Node Labeling & W2/W3 Parity
- **Slug**: `alex-hudson-selection-and-clustering-issue - Task 02`
- **File**: `.ai-memory/plans/subtasks/239-alex-hudson-selection-and-clustering-issue/02-cluster-fleet-node-labeling-and-w2-w3-restoration.md`
- **Objective**: Eliminate hardcoded `"w1"` overrides in `query.rs`. Authoritatively resolve local node identity using `resolve_local_node_alias()` (Cluster-IP primacy against `GITMAP_CLUSTER_NODES`, GitMap `variables.json`, and configured `node_alias`). Preserve all 6 canonical nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) without dropping `w2` or `w1`.
- **Status**: `COMPLETED`

### Task-03: Cluster-Wide Cross-Instance / Cross-Machine Account Exclusion
- **Slug**: `alex-hudson-selection-and-clustering-issue - Task 03`
- **File**: `.ai-memory/plans/subtasks/239-alex-hudson-selection-and-clustering-issue/03-cross-instance-cross-machine-account-exclusion.md`
- **Objective**: Extend `get_active_in_use_account_ids()` in `instances.rs`, `select_candidate_profiles()` in `selection.rs`, `manual.rs`, and frontend `smart.ts` to aggregate active accounts from `query_fleet_machines()` and cached fleet state, strictly excluding accounts active on any instance on any machine in the cluster.
- **Status**: `COMPLETED`

### Task-04: Fast-Forward & Smart-Switch Transparency Telemetry ("Let It Show")
- **Slug**: `alex-hudson-selection-and-clustering-issue - Task 04`
- **File**: `.ai-memory/plans/subtasks/239-alex-hudson-selection-and-clustering-issue/04-fast-forward-smart-switch-transparency-and-telemetry.md`
- **Objective**: Expose the selection algorithm rationale, candidate score evaluation, selection tier, and cluster-excluded accounts in rotation telemetry, audit logs, and UI notifications.
- **Status**: `COMPLETED`

### Task-05: Preflight Checks, E2E Verification & Minor Bump Release
- **Slug**: `alex-hudson-selection-and-clustering-issue - Task 05`
- **File**: `.ai-memory/plans/subtasks/239-alex-hudson-selection-and-clustering-issue/05-preflight-e2e-and-minor-bump-release.md`
- **Objective**: Execute pre-flight checks (`cargo fmt`, frontend build, TypeScript checks), bump minor version (`v4.281.0`), synchronize READMEs & CHANGELOGs with sole attribution to `@aukgit`, commit, tag, and push.
- **Status**: `COMPLETED`
