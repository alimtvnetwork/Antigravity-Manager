# Subtask 02: Window Control Capsule Formatter (`Imp-686`)
Status: IN_PROGRESS
Parent Plan: 212-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Minimalist & Contextual UI:
"When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons."

Implement `src-tauri/src/modules/instance/window_control_capsule_formatter.rs` and unit tests in `src-tauri/src/modules/instance/tests_window_control_capsule_formatter.rs`.

## Requirements
- Format window controls (Minimize, Maximize/Restore, Close) into contiguous 3-segment pill capsules.
- Ensure unified classes (`rounded-full`, shared border, dark-glass backdrop, subtle dividers).
- Comprehensive unit tests covering 3-button capsule generation, window state toggling, and divider placement.
