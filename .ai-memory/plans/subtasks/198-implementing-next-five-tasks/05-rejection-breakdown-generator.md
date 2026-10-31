# Subtask 05: Rejection File-by-File Breakdown Generator
Status: COMPLETED
Sequence: 198
Owner: Subagent-3
Target Module: `src-tauri/src/modules/release_gate/rejection_breakdown_generator.rs`
Test Suite: `src-tauri/src/modules/release_gate/tests_rejection_breakdown_generator.rs`
Objective: Enforce AGENTS.md rule requiring a concrete file-by-file accept/drop breakdown and actionable forward guidance on any rejection.

Evidence:
- Module `src-tauri/src/modules/release_gate/rejection_breakdown_generator.rs` created with `FileActionDisposition`, `FileBreakdownItem`, `RejectionBreakdownReport`, and `generate_rejection_breakdown`.
- Test suite `src-tauri/src/modules/release_gate/tests_rejection_breakdown_generator.rs` created covering relative Git path validation, missing actionable guidance detection, and markdown table rendering.
- Registered and re-exported in `src-tauri/src/modules/release_gate/mod.rs`.
