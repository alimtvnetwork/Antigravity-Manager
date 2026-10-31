# Sequence 206: E2E Master Plan (Steps 241–245)

## Overview
This master verification plan outlines the scope, test files, and behavioral gates for Batch 206 (`Imp-655` through `Imp-659`).

## Step-by-Step Task Mapping
| Step | Improvement ID | Module | Target File | Verification Test Suite |
| :--- | :--- | :--- | :--- | :--- |
| Step 241 | `Imp-655` | `modules::instance` | `src-tauri/src/modules/instance/side_effect_order_enforcer.rs` | `src-tauri/src/modules/instance/tests_side_effect_order_enforcer.rs` |
| Step 242 | `Imp-656` | `modules::release_gate` | `src-tauri/src/modules/release_gate/branch_release_channel_gate.rs` | `src-tauri/src/modules/release_gate/tests_branch_release_channel_gate.rs` |
| Step 243 | `Imp-657` | `modules::release_gate` | `src-tauri/src/modules/release_gate/release_notes_relative_paths_linter.rs` | `src-tauri/src/modules/release_gate/tests_release_notes_relative_paths_linter.rs` |
| Step 244 | `Imp-658` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/thinking_budget_proportional_allocator.rs` | `src-tauri/src/proxy/handlers/common/tests_thinking_budget_proportional_allocator.rs` |
| Step 245 | `Imp-659` | `modules::db_orm` | `src-tauri/src/modules/db_orm/fifo_prompt_restore_transaction_guard.rs` | `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_transaction_guard.rs` |

## Operational Guidelines
- Enforce strict relative Git paths in all documentation and assertions.
- No build / no test execution during authoring.
- Follow AGENTS.md error propagation and FIFO constraints.
