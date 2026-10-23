# Subtask 02: Capsule Segmented Control Styling Validator (`Imp-676`)
Status: IN_PROGRESS
Parent Plan: 210-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Minimalist & Contextual UI:
"When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons. Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects."

Implement `src-tauri/src/modules/instance/capsule_segmented_control_validator.rs` and unit tests in `src-tauri/src/modules/instance/tests_capsule_segmented_control_validator.rs`.

## Requirements
- Scan UI component markup for adjacent window control buttons or toolbar buttons.
- Verify container elements wrap buttons in contiguous capsules with `rounded-full` and subtle dividers.
- Detect nested pill-in-pill visual defects where an inline `<select>` lacks transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`).
- Provide structured report tracking audited containers, compliant elements, and styling violations.
