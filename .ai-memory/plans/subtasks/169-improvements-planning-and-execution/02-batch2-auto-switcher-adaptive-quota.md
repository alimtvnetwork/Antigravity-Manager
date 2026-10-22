# Subtask 02: Batch 2 (Imp 367–372: Auto-Switcher Adaptive Quota & Headroom)

- **Parent Plan**: `.ai-memory/plans/pending/169-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/169-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-367: Dynamic Quota Floor Guard** (`src-tauri/src/modules/auto_switcher/dynamic_quota_floor.rs`)
   - Calculates dynamic quota safety margins based on target model context window and average prompt size.
2. **Imp-368: Entropy Protected Candidate Shuffler** (`src-tauri/src/modules/auto_switcher/entropy_candidate_shuffler.rs`)
   - Uses crypto-seeded pseudorandom shuffling for candidate accounts with tied scores to ensure even distribution.
3. **Imp-369: Quota Consumption Acceleration Detector** (`src-tauri/src/modules/auto_switcher/quota_acceleration_detector.rs`)
   - Computes the rate of change of quota velocity (second derivative) to detect runaway prompt consumption loops early.
4. **Imp-370: Machine Lease Pre-Release Coordinator** (`src-tauri/src/modules/auto_switcher/lease_prerelease_coordinator.rs`)
   - Handles orderly lease hand-off and pre-release before host sleep, hibernation, or application shutdown.
5. **Imp-371: Cooldown Recovery Curve Calculator** (`src-tauri/src/modules/auto_switcher/cooldown_recovery_curve.rs`)
   - Calculates smooth non-linear sigmoid recovery curves for accounts emerging from rate-limit cooldown.
6. **Imp-372: Account Session Affinity Tracker** (`src-tauri/src/modules/auto_switcher/session_affinity_tracker.rs`)
   - Maintains conversation-to-account stickiness windows to prevent unnecessary switches during active multi-turn interactions.
