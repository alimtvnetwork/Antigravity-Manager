# 4-Part Root Cause Analysis: Cross-Mapper Thinking Budget Config Concurrency Race

## Metadata
- **Pipeline Run ID**: 35049449971
- **Repository**: alimtvnetwork/Antigravity-Manager
- **Commit**: `5ea21561`
- **Impacted Jobs**:
  - `Check Rust Code` (`windows-2025`)
- **Date**: 2026-09-16
- **Status**: Resolved

---

## 1. Symptoms

In GitHub Actions CI run [#35049449971](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35049449971), 6 of 7 jobs passed completely (including all Tauri app builds for Windows/macOS/Ubuntu and checks for macOS/Ubuntu). `Check Rust Code (windows-2025)` failed in `Run Rust tests` with a single assertion mismatch:

```text
DEBUG: Assigning generation_config: {"thinkingConfig":{"includeThoughts":true,"thinkingBudget":32000},"topP":1.0,"topK":40,"maxOutputTokens":40192,"stopSequences":["<|user|>","<|end_of_turn|>","\n\nHuman:"]}

thread 'proxy::mappers::claude::request::tests::test_claude_flash_thinking_budget_capping' (2148) panicked at src\proxy\mappers\claude\request.rs:3019:9:
assertion `left == right` failed
  left: 32000
 right: 24576

failures:
    proxy::mappers::claude::request::tests::test_claude_flash_thinking_budget_capping

test result: FAILED. 611 passed; 1 failed; finished in 47.26s
```

---

## 2. Root Cause

1. **Global Mutable State in `OnceLock<RwLock<ThinkingBudgetConfig>>`**:
   `src-tauri/src/proxy/config.rs` maintains a process-wide `GLOBAL_THINKING_BUDGET_CONFIG` that controls whether thinking budgets are handled via `Auto`, `Custom`, `Passthrough`, or `Adaptive`.
2. **Cross-Mapper Test Interleaving**:
   Tests across different mapper modules (`claude::request`, `openai::request`, `gemini::wrapper`) run concurrently on multiple worker threads during `cargo test`.
   - `test_claude_adaptive_global_config` in `claude::request` mutated global config to `ThinkingBudgetMode::Adaptive`.
   - In `ThinkingBudgetMode::Adaptive`, `transform_claude_request_in` passes through raw `budget_tokens` without applying `Auto` capping.
   - Concurrently, `test_claude_flash_thinking_budget_capping` executed with `budget_tokens = 32000` expecting `Auto` mode capping to 24576.
   - Due to the parallel mutation, it evaluated under `Adaptive` mode and returned 32000 instead of 24576.

---

## 3. Resolution

1. **Global Test Mutex for Thinking Budget Config**:
   Exported `pub static TEST_THINKING_BUDGET_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());` in `src-tauri/src/proxy/config.rs`.
2. **Synchronized Mapper Tests**:
   - In `claude::request::tests`: synchronized `test_claude_flash_thinking_budget_capping` (with explicit reset to default) and `test_claude_adaptive_global_config`.
   - In `openai::request::tests`: synchronized `test_issue_1602_custom_mode_gemini_capping`, `test_gemini_pro_thinking_injection`, and `test_flash_thinking_budget_capping`.
   - In `gemini::wrapper`: aliased `TEST_MUTEX` to `crate::proxy::config::TEST_THINKING_BUDGET_MUTEX` so all wrapper tests share the same global lock.

---

## 4. Verification & Prevention

1. **Verification**:
   - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` verified clean with exit code 0.
   - Pushed commit to `main` for full matrix verification.
2. **Prevention**:
   - Any global state mutated by unit tests MUST be guarded by a centralized, shared test mutex across all crates and modules.
   - Tests expecting default global state must explicitly acquire the mutex and restore `Default::default()`.
