# Subtask 02: Batch 2 (Imp 487–492: Auto-Switcher Adaptive Dampeners & Quota Boundary Synchronization)

- **Parent Plan**: `.ai-memory/plans/pending/173-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/173-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-487: 5-Hour Rolling Quota Boundary Synchronizer** (`src-tauri/src/modules/auto_switcher/quota_boundary_synchronizer.rs`)
   - Aligns rotation evaluation timers with rolling 5-hour quota reset boundaries to prevent switching right before an account recovers.
2. **Imp-488: Consecutive Switch Velocity Throttle** (`src-tauri/src/modules/auto_switcher/consecutive_switch_throttle.rs`)
   - Enforces a hard throttle limiting instance account switches to at most 3 operations within any 60-second sliding window.
3. **Imp-489: LRU Account-Instance Binding History Cache** (`src-tauri/src/modules/auto_switcher/binding_history_cache.rs`)
   - Maintains in-memory LRU tracking of recent account-to-instance bindings to preserve profile continuity and affinity.
4. **Imp-490: Idle Candidate Score Time-Decay Curve** (`src-tauri/src/modules/auto_switcher/candidate_score_decay.rs`)
   - Applies a progressive time-decay curve to candidate scores for accounts that remain idle, balancing load across peers.
5. **Imp-491: High-Speed Split-DB Switch Audit Logger** (`src-tauri/src/modules/auto_switcher/switch_audit_logger.rs`)
   - Logs structured switch execution audits directly into `task_history.db` with rationale codes and settling latencies.
6. **Imp-492: Active Instance Readiness Prober** (`src-tauri/src/modules/auto_switcher/active_instance_prober.rs`)
   - Performs non-blocking HTTP and CDP health probes to confirm an IDE instance is ready to receive prompt injections.
