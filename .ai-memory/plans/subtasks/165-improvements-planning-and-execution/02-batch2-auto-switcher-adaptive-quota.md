# Subtask 02: Batch 2 (Imp 247–252: Auto-Switcher Adaptive Quota Scoring & Recovery Metrics)

- **Parent Plan**: `.ai-memory/plans/pending/165-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/165-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Targeted Improvements
- [x] **Imp-247: Quota Velocity Exponential Moving Average Smoother** (`src-tauri/src/modules/auto_switcher/velocity_smoother.rs`)
- [x] **Imp-248: Multi-Tier Account Failover Ladder** (`src-tauri/src/modules/auto_switcher/failover_ladder.rs`)
- [x] **Imp-249: Hourly Switch Entropy & Oscillation Guard** (`src-tauri/src/modules/auto_switcher/hourly_entropy_guard.rs`)
- [x] **Imp-250: Per-Instance Account Cooldown Interval Map** (`src-tauri/src/modules/auto_switcher/instance_cooldown_map.rs`)
- [x] **Imp-251: Comprehensive Account Readiness Evaluator** (`src-tauri/src/modules/auto_switcher/readiness_evaluator.rs`)
- [x] **Imp-252: Decorrelated Jitter Backoff Calculator** (`src-tauri/src/modules/auto_switcher/jittered_backoff.rs`)

## Verification Summary
- `velocity_smoother.rs`: Configurable EMA token burn velocity filter to eliminate false spikes from turn bursts.
- `failover_ladder.rs`: Deterministic account priority categorization (`Primary` -> `Secondary` -> `Sandbox` -> `EmergencyQuota`).
- `hourly_entropy_guard.rs`: Rolling Shannon entropy calculator over switch targets to detect and prevent runaway oscillations.
- `instance_cooldown_map.rs`: Thread-safe per-instance isolated cooldown expiration map with auto-sweeper.
- `readiness_evaluator.rs`: Consolidated readiness validation across tokens, cluster lease, 4h rolling quota, and cooldown.
- `jittered_backoff.rs`: Full and decorrelated backoff algorithms to prevent thundering herd upon quota resets.
