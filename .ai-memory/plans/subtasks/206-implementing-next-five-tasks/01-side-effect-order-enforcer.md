# Subtask 01: Side Effect Ordering & Pre-Point-of-No-Return Enforcer (`Imp-655`)
Status: COMPLETED
Parent Plan: 206-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Risky Path Replacement:
"Order side effects to fail before the point of no return: write credentials before killing a process, validate before deleting. Keep an explicit fallback to the verified path whenever you replace it. Detect broadly, act narrowly: a matcher may recognize a whole class of problems, while its effect stays inside the intended data — not across line breaks, tags, or other clauses. Bound every wait with a timeout."

Implemented `src-tauri/src/modules/instance/side_effect_order_enforcer.rs` and unit tests in `src-tauri/src/modules/instance/tests_side_effect_order_enforcer.rs`.

## Requirements
- Defined `SideEffectKind` (`Validation`, `CredentialPersist`, `ProcessTermination`, `StorageDeletion`) with predicate helpers.
- Defined `SideEffectStage` with builder methods, timeout bounds, and verified fallback tracking.
- Implemented `validate_side_effect_order` to verify that irreversible actions strictly follow successful credentials persistence and pre-validation.
- Implemented unit tests covering valid orders, inverted orders, and timeout bounds.
- Registered in `src-tauri/src/modules/instance.rs`.
