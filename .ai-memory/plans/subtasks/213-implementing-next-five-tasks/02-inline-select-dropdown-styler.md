# Subtask 02: Inline Select Dropdown Styler (`Imp-691`)
Sequence: 213
Step: 277
Status: COMPLETED
Module: `src-tauri/src/modules/instance/inline_select_dropdown_styler.rs`
Test Suite: `src-tauri/src/modules/instance/tests_inline_select_dropdown_styler.rs`

## Description
Enforces AGENTS.md rule: "Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects." Provides styling verification, AST/class pattern linting, and canonical class generator for segmented capsule dropdowns.

## Verification Details
- Defined canonical constants `CANONICAL_INLINE_SELECT_CLASSES` and `CANONICAL_INLINE_OPTION_CLASSES`.
- Implemented `DropdownStylingReport` tracking compliance, transparent background, zero border, zero focus ring, zero outline, forbidden classes found, and violation messages.
- Implemented `validate_select_classes` identifying missing mandatory tokens and detecting forbidden opaque backgrounds, nested pills, visible borders, rings, and shadows.
- Implemented `normalize_select_classes` sanitizing arbitrary class strings and returning guaranteed compliant classes.
- Implemented `format_capsule_select_html` producing safe, escaped `<select>` and `<option>` markup.
- Registered and re-exported in `src-tauri/src/modules/instance.rs`.
- Comprehensive unit test suite in `src-tauri/src/modules/instance/tests_inline_select_dropdown_styler.rs`.
