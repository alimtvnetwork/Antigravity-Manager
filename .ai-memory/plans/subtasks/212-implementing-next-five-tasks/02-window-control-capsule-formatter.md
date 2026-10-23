# Subtask 02: Window Control Capsule Formatter (`Imp-686`)
Status: COMPLETED
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Minimalist & Contextual UI:
"When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons."

Implemented `src-tauri/src/modules/instance/window_control_capsule_formatter.rs` and unit tests in `src-tauri/src/modules/instance/tests_window_control_capsule_formatter.rs`.
Registered in `src-tauri/src/modules/instance.rs`.

## Verification Evidence
- Defined `CANONICAL_CAPSULE_CONTAINER_CLASSES` (`rounded-full`, shared border, dark-glass backdrop), `CANONICAL_CAPSULE_BUTTON_CLASSES`, and `CANONICAL_CAPSULE_DIVIDER_CLASSES`.
- Implemented `format_window_control_capsule` generating structured 3-button window actions (Minimize, Maximize/Restore, Close).
- Added dynamic state toggle between floating ("Maximize") and maximized ("Restore Down") window states.
- Added comprehensive unit tests covering button ordering, destructive styling on close, tooltip switching, and class validation.
- Deferred E2E command documented in `02-spec/21-app/212-implementing-next-five-tasks/e2e_master_plan.md`.
