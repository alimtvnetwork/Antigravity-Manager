# Specification 192: Process Safety, Release Governance, and Stream Normalization

## 1. Context & Objectives
This specification codifies the release governance engines, caller IDE protection guards, and streaming payload normalization harnesses implemented during Sequence 192.
All implementations enforce strict zero-warning compilation, zero runtime regressions, and absolute decoupling between proxy pipelines and adapter protocols.

## 2. Architectural Components

### 2.1 Release Governance & Process Safety (`src-tauri/src/modules/release_gate/`)
1. **`caller_ide_preservation_guard.rs` & `tests_caller_ide_guard.rs`**:
   - Detects caller active IDE instances and enforces fail-fast protection (exit code 2) unless `--force` is specified.
2. **`test_sandbox_lifecycle_verifier.rs` & `tests_sandbox_lifecycle.rs`**:
   - Manages standardized transient test sandboxes (`agm-test-sandbox-*`) and enforces zero-remnant file and orphan process cleanup.
3. **`thinking_cache_invalidation_guard.rs` & `tests_thinking_cache_guard.rs`**:
   - Controls version-targeted one-time thinking cache invalidation prompts during schema refactor releases.
4. **`coauthor_attribution_verifier.rs` & `tests_coauthor_attribution.rs`**:
   - Formats and validates `Co-authored-by:` commit trailers to maintain open-source contributor attribution.
5. **`pr_template_compliance_linter.rs` & `tests_pr_template_linter.rs`**:
   - Lints pull request descriptions against the mandatory 4 sections (problem classification, behavior alterations, unverified paths, rollback strategy).

### 2.2 Streaming Protocol Normalization & Decompression (`src-tauri/src/proxy/handlers/common/`)
1. **`decompressor.rs` & `tests_decompressor.rs`**:
   - Transparent upstream payload decompression across gzip, deflate, and auto-detected magic byte headers.
2. **`boundary_normalizer.rs` & `tests_boundary_normalizer.rs`**:
   - SSE stream chunk boundary normalizer reconstructing complete event messages from network byte fragments.
3. **`chunk_boundary_validator.rs` & `tests_chunk_boundary_validator.rs`**:
   - Multi-byte UTF-8 boundary validator preventing character corruption across chunk splits.
4. **`json_payload_sanitizer.rs` & `tests_json_payload_sanitizer.rs`**:
   - Deep JSON sanitizer eliminating unescaped control characters and trailing commas before deserialization.

## 3. Verification & Compliance
- **Zero-Build & Zero-Test Invariant**: Fully respected during implementation.
- **Repository-Relative Paths**: All file paths strictly formatted as repo-relative.
- **E2E Enqueueing**: 10 comprehensive end-to-end test scenarios documented in `e2e_master_plan.md`.
