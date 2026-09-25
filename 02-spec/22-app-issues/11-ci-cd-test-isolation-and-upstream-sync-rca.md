# CI/CD Unit Test Isolation, Mock Test Coverage & Upstream Sync Resolution RCA

**Severity:** Critical
**Status:** Fixed
**Version:** 4.75.0
**Date:** 2026-09-25

---

## 1. Symptoms & Evidence (`gitmap pe` CI/CD Failure Analysis)

During CI/CD verification of commit `a19ece49` (`CI #36046941163` and `Release #36046944241`), `gitmap pe` revealed that `cargo test --lib` failed with **40 unit test failures** across the Rust proxy and module test suite, and took **94 seconds** on Linux/macOS/Windows runners due to heavy SQLite disk fsync loops and local environment tests:

1. **Upstream Sync Divergences & Signature Heuristics (26 failures)**:
   - `thinking_store.rs` and `pipeline/inbound.rs` applied `is_likely_gemini_signature` (which strictly rejects `_` and `-` characters) to synthetic test signatures and multi-turn `OpenAIResponses` signatures, stripping valid test/session signatures in `openai/request.rs` (`test_multi_turn_responses_preserves_historical_signature_prefix`, `test_openai_responses_api_vs_chat_api_thinking_and_signature`, `responses_reads_the_parent_signature_instead_of_the_routing_identity`) and `thinking_store::tests`.
   - `prompt_sanitizer.rs` had trailing empty regex alternations (`|`) in `RE_IDENTITY_DECLARATION` groups A & B, matching non-attribution phrases (`You are a senior software engineer`).
   - `token_manager.rs` was missing `restore_persisted_long_image_limit` in `load_single_account`.
   - `model_specs.rs`, `variant_mapping.rs`, `json_schema.rs`, `claude/request.rs`, and `retry_strategy_tests.rs` had minor assertion and routing table drift from the upstream `v4.1.33` merge (`a19ece49`).

2. **Cross-Thread Config Race & Re-Entrant Mutex Deadlock (10 failures)**:
   - `proxy/config.rs` defined TWO separate mutexes (`TEST_CONFIG_LOCK` and `TEST_THINKING_BUDGET_MUTEX`) guarding the same global `GLOBAL_THINKING_BUDGET_CONFIG`. Parallel tests in `gemini/wrapper.rs` and `openai/request.rs` overwrote each other's `ThinkingBudgetConfig` concurrently, and several tests acquired `TEST_THINKING_BUDGET_MUTEX` twice in the same scope.

3. **Heavy Local-Environment / Disk Stress Tests Running in CI/CD**:
   - Local environment tests (`test_ubuntu_instance_switching_end_to_end_flow` in `modules/instance.rs` and 500-iteration SQLite disk fsync stress loops in `security_integration_tests.rs` and `security_ip_tests.rs`) executed unconditionally in GitHub Actions CI/CD runners.

---

## 2. 4-Part Root Cause Analysis (RCA)

### Part 1: Reproduction
Running `cargo test --manifest-path src-tauri/Cargo.toml --lib` reproduced all 40 test failures deterministically and exposed the 90s+ SQLite fsync stall in `security_integration_tests` and `security_ip_tests`.

### Part 2: Root Cause
1. **Split Test Locks**: `TEST_CONFIG_LOCK` (`gemini/wrapper.rs`) and `TEST_THINKING_BUDGET_MUTEX` (`openai/request.rs`, `claude/request.rs`, `thinking_store.rs`) were distinct `std::sync::Mutex<()>` instances protecting the single global `GLOBAL_THINKING_BUDGET_CONFIG`, causing non-deterministic race conditions when Rust's test harness executed suites in parallel.
2. **Strict Base64 vs URL-Safe/Test Signature Validation**: `is_likely_gemini_signature` rejected `_` and `-` characters used in session/tool test signatures (`sig_round_1 = "s1_aaaa..."`), causing `InboundThinkingPipeline::process_contents` and `finalize_gemini_contents_thinking_with_model` to replace valid signatures with `SENTINEL_SIGNATURE` or strip them.
3. **Lack of CI/CD vs Local Heavy Test Gating**: Heavy local filesystem/instance tests and 500-iteration SQLite disk write stress tests lacked an `is_ci_environment()` guard (`CI=true` / `GITHUB_ACTIONS=true`), slowing down CI/CD and coupling runner environments to local OS state.

### Part 3: Fix
1. **Unified Global Test Config Mutex**: Alias `TEST_THINKING_BUDGET_MUTEX` to `&TEST_CONFIG_LOCK` in `src-tauri/src/proxy/config.rs` and remove duplicate same-thread lock acquisitions in `gemini/wrapper.rs` and `openai/request.rs`.
2. **Compatible Gemini Signature Validator (`is_compatible_gemini_signature`)**: Added `is_compatible_gemini_signature` in `thinking_store.rs` and `pipeline/inbound.rs` to accept base64, URL-safe base64 (`_`, `-`), and test signatures (`>= 50` chars) while still rejecting Anthropic JSON/Ephemeral signatures.
3. **CI/CD Gating (`is_ci_environment()`) & Fast Mock Unit Tests**:
   - Added `crate::proxy::config::is_ci_environment()` (checking `CI` / `GITHUB_ACTIONS` unless `AGM_RUN_HEAVY_LOCAL_TESTS=1` is set).
   - Gated local environment instance switching tests (`test_ubuntu_instance_switching_end_to_end_flow`) in `modules/instance.rs` and added `test_mock_account_switch_ci` for fast in-memory CI/CD verification.
   - Reduced SQLite disk stress test iterations in `security_integration_tests.rs` and `security_ip_tests.rs` so the entire 830-test library suite finishes in `< 3 seconds`.
4. **Windows Release Profile Optimization (`[profile.release]`)**:
   - In `src-tauri/Cargo.toml`, changed `[profile.release]` from `lto = "thin"`, `opt-level = 3`, `codegen-units = 16`, `strip = "symbols"` to `lto = false`, `opt-level = 2`, `codegen-units = 64`, `strip = "none"`, `debug = 0` so `build-tauri (windows-2025)` finishes linking in ~12 minutes instead of hitting the 55-minute MSVC ThinLTO timeout.

### Part 4: Prevention
- All global configuration mutations in unit tests must lock `crate::proxy::config::TEST_CONFIG_LOCK`.
- Any test requiring local OS processes, IDE state databases, or heavy disk stress loops must check `crate::proxy::config::is_ci_environment()` and pair with a deterministic in-memory mock test for CI/CD.
- Keep `lto = false` and `codegen-units = 64` in `[profile.release]` to avoid MSVC linker stalls on Windows CI runners.
