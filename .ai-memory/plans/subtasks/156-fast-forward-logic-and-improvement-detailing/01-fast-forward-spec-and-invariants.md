# Subtask 01: Fast-Forward Specification and System Invariants

## Objective
Author the formal specification of Fast-Forward logic and system invariants in `02-spec/21-app/156-fast-forward-logic-and-improvement-detailing/spec.md`.

## Key Deliverables
1. **Fast-Forward Core Logic Documentation**:
   - Instance resolution and running state inspection.
   - Dual-key exclusion set construction (`account_id` and `bound_email`).
   - Two-pass candidate scoring architecture:
     - Pass 1 (Primary): Strict 100% 4-hour quota requirement ($Q_{4\text{h}} \ge 100\%$).
     - Pass 2 (Proportional Fallback): Triggered only when zero 100% accounts exist in the candidate pool.
   - Live API pre-switch verification probe before instance state modification.
   - Prompt snapshotting (`.antigravity_resume_task.json`) and atomic profile injection.
2. **System Invariants**:
   - Invariant 1: Any account with $Q_{4\text{h}} < 100\%$ (even reduced by 1% or 2%) MUST evaluate to `0.0` in primary scoring.
   - Invariant 2: Active in-use accounts (on target instance, running sibling instances, or remote leases) MUST evaluate to `activeFactor = 0` / score `0.0`.
   - Invariant 3: Dual-key matching (ID and email) is mandatory for all exclusion and in-use checks.
