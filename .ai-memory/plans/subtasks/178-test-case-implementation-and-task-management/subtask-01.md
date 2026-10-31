# Subtask 01: Steps 1 & 2 - Boolean Logic Modernization & Guard Clause Inversion

- **Parent Plan**: `.ai-memory/plans/pending/178-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/178-test-case-implementation-and-task-management/spec.md#step-1-boolean-logic-modernization-in-telegram-inbound-processing`
- **Target Files**:
  - `src-tauri/src/modules/telegram_inbound/processing.rs`
  - `src-tauri/src/modules/email_inbound/dispatcher.rs`
- **Status**: [COMPLETED]

## Description
1. Step 1: Modernized node identifier logic in `src-tauri/src/modules/telegram_inbound/processing.rs`, extracting affirmative helpers:
   - `is_valid_node_identifier(prefix_candidate: &str, cmd_candidate: &str) -> bool`
   - `is_command_target_local(target_node: &str) -> bool`
   - `extract_node_targeted_command<'a>(trimmed: &'a str, lower_full: &str) -> Option<(&'a str, &'a str)>`
   Eliminated compound negative chains and flattened the command routing loop.
2. Step 2: Extracted `is_heavy_action_type(action: &InboundAction) -> bool` in `src-tauri/src/modules/email_inbound/dispatcher.rs` and inverted nested rate-limiting conditionals into early-return guard clauses.
