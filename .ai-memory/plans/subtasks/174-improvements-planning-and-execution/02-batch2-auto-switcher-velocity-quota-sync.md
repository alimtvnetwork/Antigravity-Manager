# Subtask 02: Batch 2 (Imp 517–522: Auto-Switcher Candidate Scoring, Velocity Dampening & Quota Alignment)

- **Parent Plan**: `.ai-memory/plans/pending/174-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/174-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-517: Quota Reset Epoch Replenishment Tracker** (`src-tauri/src/modules/auto_switcher/quota_reset_epoch_tracker.rs`)
   - Records historical quota reset timestamps and computes rolling replenishment intervals per account.
2. **Imp-518: Consecutive Switch Cooldown Penalty Ladder** (`src-tauri/src/modules/auto_switcher/consecutive_switch_cooldown.rs`)
   - Tracks consecutive switches per instance profile and enforces progressive cooldown penalties to suppress rapid cycling.
3. **Imp-519: Inactive Account Staleness Detector** (`src-tauri/src/modules/auto_switcher/account_staleness_detector.rs`)
   - Detects stale or inactive accounts based on last successful turn timestamps and flags them for quota re-probing.
4. **Imp-520: Candidate Score Percentile Normalizer** (`src-tauri/src/modules/auto_switcher/score_percentile_normalizer.rs`)
   - Normalizes candidate algorithm scores into percentile bands (0–1000) for consistent scoreboard visualization.
5. **Imp-521: Post-Switch Settling Delay Guard** (`src-tauri/src/modules/auto_switcher/switch_settling_timer.rs`)
   - Enforces an async non-blocking 10-second settling delay following credential switches before allowing prompt injection.
6. **Imp-522: Sibling Instance Switch Readiness Barrier** (`src-tauri/src/modules/auto_switcher/instance_readiness_barrier.rs`)
   - Coordinates multi-instance switch readiness barriers, preventing concurrent rotations on sibling profiles.
