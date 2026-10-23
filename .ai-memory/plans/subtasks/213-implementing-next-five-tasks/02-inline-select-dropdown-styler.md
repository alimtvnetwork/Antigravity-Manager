# Subtask 02: Inline Select Dropdown Styler (`Imp-691`)
Sequence: 213
Step: 277
Status: IN_PROGRESS
Module: `src-tauri/src/modules/instance/inline_select_dropdown_styler.rs`
Test Suite: `src-tauri/src/modules/instance/tests_inline_select_dropdown_styler.rs`

## Description
Enforces AGENTS.md rule: "Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects." Provides styling verification, AST/class pattern linting, and canonical class generator for segmented capsule dropdowns.
