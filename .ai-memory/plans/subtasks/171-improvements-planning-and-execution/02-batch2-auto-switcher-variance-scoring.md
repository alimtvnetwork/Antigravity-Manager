# Subtask 02: Batch 2 (Imp 427–432: Auto-Switcher Variance Scoring & Cooldown Telemetry)

- **Parent Plan**: `.ai-memory/plans/pending/171-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/171-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-427: Candidate Score Variance & Outlier Dampener** (`src-tauri/src/modules/auto_switcher/score_variance_guard.rs`)
   - Analyzes candidate score distribution across account pools and dampens transient metric spikes to prevent switch flapping.
2. **Imp-428: Stale Profile Lease Reclaim Watchdog** (`src-tauri/src/modules/auto_switcher/lease_reclaim_watchdog.rs`)
   - Sweeps and forcefully reclaims orphaned workspace leases held by terminated or non-responsive IDE instances past the TTL threshold.
3. **Imp-429: Cooldown Event & Rotation Telemetry Reporter** (`src-tauri/src/modules/auto_switcher/cooldown_telemetry_reporter.rs`)
   - Emits structured telemetry records capturing cooldown reasons, durations, and account transitions for audit and fleet sync.
4. **Imp-430: Account Priority Tier Fallback Ladder** (`src-tauri/src/modules/auto_switcher/priority_tier_ladder.rs`)
   - Provides a deterministic fallback ladder (Ultra -> Pro -> Sandbox -> Free) when primary tier accounts enter active cooldown.
5. **Imp-431: Candidate Exclusion Registry & Manual Bypass Cache** (`src-tauri/src/modules/auto_switcher/candidate_exclusion_registry.rs`)
   - Maintains in-memory thread-safe registries of temporarily excluded accounts (manual bypass, user disabled, quota exhausted).
6. **Imp-432: Account Switch Settling Barrier & State Prober** (`src-tauri/src/modules/auto_switcher/switch_settling_barrier.rs`)
   - Coordinates a non-blocking settling delay barrier with progressive health probes before enabling prompt injection on switched profiles.
