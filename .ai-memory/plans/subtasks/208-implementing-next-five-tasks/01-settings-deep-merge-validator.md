# Subtask 01: Headless & CLI Settings Deep-Merge Validator (`Imp-665`)
Status: COMPLETED
Parent Plan: 208-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Headless & CLI Parity:
"Any profile/instance duplication or settings synchronization must pass through identical underlying service routes (`copy_instance_with_options`), preserving project continuity (`workspaceStorage`) and allowing deep-merge settings synchronization from both GUI and CLI."

Implement `src-tauri/src/modules/instance/settings_deep_merge_validator.rs` and unit tests in `src-tauri/src/modules/instance/tests_settings_deep_merge_validator.rs`.

## Requirements
- Define JSON value recursive deep-merge algorithm preserving nested objects and array elements.
- Validate that synchronization preserves existing developer settings without clobbering orthogonal keys.
- Reject shallow overwrite attempts where partial updates delete existing configurations.
- Provide structured report tracking merged keys, conflict resolutions, and preserved properties.
