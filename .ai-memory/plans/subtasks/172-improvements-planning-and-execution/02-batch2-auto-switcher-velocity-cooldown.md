# Subtask 02: Batch 2 (Imp 457–462: Auto-Switcher Cooldown, Quota Velocity & Selection Jitter)

- **Parent Plan**: `.ai-memory/plans/pending/172-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/172-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-457: Short-Term Quota Velocity Spike Dampener** (`src-tauri/src/modules/auto_switcher/velocity_spike_dampener.rs`)
   - Dampens transient 1-minute quota consumption velocity spikes to avoid premature account switching during bursty multi-agent runs.
2. **Imp-458: Stepped Exponential Cooldown Decay** (`src-tauri/src/modules/auto_switcher/cooldown_decay_stepper.rs`)
   - Reduces cooldown durations step-wise for accounts exhibiting healthy quota resets, enabling faster candidate re-entry.
3. **Imp-459: Non-Deterministic Candidate Selection Jitter** (`src-tauri/src/modules/auto_switcher/selection_jitter_allocator.rs`)
   - Injects subtle proportional score jitter (+/-0.5%) into identical candidate scores to eliminate deterministic rotation loops.
4. **Imp-460: Multi-Instance Lease Conflict Claim Coordinator** (`src-tauri/src/modules/auto_switcher/instance_lease_claim.rs`)
   - Coordinates profile lease acquisition with atomic reservation checks to prevent concurrent sibling instances from double-booking.
5. **Imp-461: Account Model Capability Tier Filter** (`src-tauri/src/modules/auto_switcher/account_tier_filter.rs`)
   - Filters candidate accounts by required model capabilities (e.g. Claude 3.5 Sonnet, Gemini 2.0 Flash) before score evaluation.
6. **Imp-462: Post-Switch Request Health Tracker** (`src-tauri/src/modules/auto_switcher/switch_recovery_tracker.rs`)
   - Monitors the first 3 prompt dispatches to a newly switched instance, verifying zero transport errors or quota rejections.
