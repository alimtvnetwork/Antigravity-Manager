# Subtask 02: Segmented Capsule Toggle Switch Styler & Linter (`Imp-711`)
Sequence: 217
Step: 297
Status: COMPLETED
Module: `src-tauri/src/modules/instance/segmented_capsule_toggle_switch_styler.rs`

## Description
Implement the Segmented Capsule Toggle Switch Styler & Linter adhering to specification `02-spec/21-app/217-implementing-next-five-tasks/spec.md`:
1. Enforce AGENTS.md Minimalist & Contextual UI:
   "When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons. Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects."
2. Implement `CapsuleSegmentOption`, `CapsuleStyleConfig`, `SegmentedCapsuleFormatting`, and `CapsuleAuditReport`.
3. Provide `format_segmented_capsule_toggle` and `audit_capsule_markup_compliance`.
4. Create companion unit tests in `src-tauri/src/modules/instance/tests_segmented_capsule_toggle_switch_styler.rs`.

## Verification & Outcomes
- Implemented `CapsuleSegmentOption`, `CapsuleStyleConfig`, and `SegmentedCapsuleFormatting`.
- Implemented `CapsuleAuditFinding` and `CapsuleAuditReport` with detailed recommendations.
- Implemented `format_segmented_capsule_toggle` generating contiguous dark-glass segmented pill capsules (`rounded-full`, subtle divider lines, active state styling).
- Implemented `audit_capsule_markup_compliance` detecting disjoint button layouts, missing `rounded-full` boundaries, and non-borderless `<select>` dropdowns.
- Unit tests written in `tests_segmented_capsule_toggle_switch_styler.rs` verifying capsule generation and markup auditing.
