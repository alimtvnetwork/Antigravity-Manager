# Subtask 03: Token Budget Decider & Thinking Allocator
Status: COMPLETED
Sequence: 197
Owner: Subagent-3
Target Module: `src-tauri/src/proxy/handlers/common/token_budget_decider.rs`
Test Suite: `src-tauri/src/proxy/handlers/common/tests_token_budget_decider.rs`
Objective: Dynamic headroom calculation between prompt tokens, thinking/reasoning budget, and max_output_tokens, ensuring max_output_tokens strictly exceeds thinking budget.

Evidence:
- Module `src-tauri/src/proxy/handlers/common/token_budget_decider.rs` created with `ModelThinkingCapability`, `TokenAllocation`, `decide_token_budget`, and `resolve_model_thinking_capability`.
- Test suite `src-tauri/src/proxy/handlers/common/tests_token_budget_decider.rs` created with 16 unit tests covering thinking budget filtering, backfilling, ceiling clamping, and output headroom validation.
- Deferred E2E Master Plan `02-spec/21-app/197-implementing-next-five-tasks/e2e_master_plan.md` created covering scenarios E2E-197-01 through E2E-197-05.
