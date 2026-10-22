# Subtask 01: Steps 1 & 2 - Boolean Logic Modernization & Guard Clause Inversion

- **Parent Plan**: `.ai-memory/plans/pending/177-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/177-test-case-implementation-and-task-management/spec.md#step-1-boolean-logic-modernization-in-auto-switcher-candidate-scoring`
- **Target Files**:
  - `src-tauri/src/modules/auto_switcher/scoring.rs`
  - `src-tauri/src/proxy/handlers/claude/stream_handling.rs`
- **Status**: [COMPLETED]

## Description
1. Step 1: Modernized candidate eligibility boolean logic in `src-tauri/src/modules/auto_switcher/scoring.rs`, extracting affirmative helpers:
   - `is_account_active_and_enabled(acc: &Account) -> bool`
   - `is_candidate_eligible_for_switch(acc: &Account) -> bool`
   - `is_gemini_native_model(model_name: &str) -> bool`
   Applied across `calculate_weekly_base_score`, `score_candidate_account`, and `score_candidate_account_fallback`.
2. Step 2: Inverted nested condition trees in `src-tauri/src/proxy/handlers/claude/stream_handling.rs`, extracting the empty stream check into an early-return guard clause returning `StreamOutcome::Retry`, flattening the response handling flow by 120 lines.
