# Subtask 02: Batch 2 (Imp 397–402: Auto-Switcher Cooldown, Quota & Rotation Guards)

- **Parent Plan**: `.ai-memory/plans/pending/170-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/170-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-397: Sliding Window Quota Velocity Guard** (`src-tauri/src/modules/auto_switcher/sliding_velocity_guard.rs`)
   - Calculates rolling 5-minute quota consumption velocity with burst dampening.
2. **Imp-398: Consecutive Failure Cooldown Multiplier** (`src-tauri/src/modules/auto_switcher/consecutive_failure_multiplier.rs`)
   - Applies progressive geometric backoff multipliers (1x, 2x, 4x, 8x) to cooldown timers for accounts with repeated failures.
3. **Imp-399: Multi-Instance Binding Conflict Detector** (`src-tauri/src/modules/auto_switcher/binding_conflict_detector.rs`)
   - Checks across active instance profile records to detect if two separate IDE instances are concurrently bound to the same target account.
4. **Imp-400: Candidate Rotation Cooldown Sweeper** (`src-tauri/src/modules/auto_switcher/candidate_cooldown_sweeper.rs`)
   - Sweeps expired account cooldown records and returns refreshed candidate lists ready for rotation scoring.
5. **Imp-401: Account Tier Priority Weight Normalizer** (`src-tauri/src/modules/auto_switcher/tier_priority_normalizer.rs`)
   - Normalizes account tier weights (Ultra, Pro, Free, Sandbox) into calibrated scoring multipliers.
6. **Imp-402: Instance Relaunch Readiness Evaluator** (`src-tauri/src/modules/auto_switcher/relaunch_readiness_evaluator.rs`)
   - Evaluates whether an instance that underwent account switching has fully settled before prompt re-injection.
