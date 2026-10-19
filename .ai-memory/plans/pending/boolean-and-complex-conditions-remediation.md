# Execution Plan: Boolean & Complex Condition Remediation

## User Request (Verbatim)
So many complex ifs didn't fix as per @[.agents/skills/cg-execute] @[.agents/skills/booleans-and-complex-conditions]

## Executive Summary
Audit and eliminate multi-clause compound conditional expressions, mixed polarity checks, double negatives, and nested ternary/logic chains across touched modules in `src-tauri/src/modules/repo_db/`, `src-tauri/src/modules/telegram_inbound/`, `src-tauri/src/modules/workspace_lease_manager/`, and `src/components/instances/PromptTreeView/`. Enforce affirmative predicate helper functions, positive boolean naming (`is*`, `has*`), and guard clauses with early exits as required by `02-spec/02-coding-guidelines/02-booleans-and-complex-conditions.md`.

## Task Inventory
1. **Task-01 (`Worker 01`):** Refactor `src-tauri/src/modules/repo_db/tree_collect.rs`, `tree_node_conv.rs`, and `tree_phases.rs`.
   - Extract `should_retain_unrunning_conversation`, `is_vacant_ghost_conversation`, `is_instance_compatible`, and `is_default_instance_match`.
   - Eliminate 3+ condition chains and mixed polarity expressions (`if is_a && !is_b`).
2. **Task-02 (`Worker 02`):** Refactor `src-tauri/src/modules/repo_db/transcript.rs`, `src-tauri/src/modules/telegram_inbound/send_flow.rs`, and `src-tauri/src/modules/workspace_lease_manager/acquire.rs`.
   - Eliminate compound negative checks in tail-line seekers, subagent prompt category decoders, and lease acquisition conflict guards.
   - Extract affirmative helpers `is_lease_held_by_other_node`, `is_valid_send_token`, etc.
3. **Task-03 (`Worker 03`):** Refactor `src/components/instances/PromptTreeView/usePromptTreeData.ts` and `tierClassification.ts`.
   - Extract `isDefaultInstanceAlias`, simplify hierarchy assembly and status sort comparison predicates.

## Verification Contract
- `cd src-tauri && cargo fmt -- --check` (exit 0)
- `cd src-tauri && cargo clippy --all-targets --all-features` (exit 0)
- `cargo test --lib modules::repo_db` (exit 0)
- `npm run build` (exit 0)
