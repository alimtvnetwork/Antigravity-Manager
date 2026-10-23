# Deferred Master End-to-End Test Plan — Sequence 192

## Overview
This document catalogs the comprehensive End-to-End (E2E) test suites enqueued during **Sequence 192**.
In accordance with user instructions, zero test runs and zero build commands were executed during this implementation increment. These E2E scenarios are structured for autonomous execution during the dedicated final verification phase.

---

## Test Cases Inventory

### `E2E-192-01`: Caller Active IDE Termination Protection
- **Target Module**: `src-tauri/src/modules/release_gate/caller_ide_preservation_guard.rs`
- **Preconditions**: Developer working in an active IDE instance detected via `detect_caller_instance_id()`.
- **Actions**:
  1. Dispatch `agm instances stop <caller_id>` without `--force`.
  2. Verify immediate fail-fast termination with exit code 2.
  3. Dispatch `agm instances stop <caller_id> --force`; verify action permitted.
- **Expected Outcome**: Active developer IDE instances are preserved from accidental termination.

### `E2E-192-02`: Transient Test Sandbox Lifecycle and Zero Remnants
- **Target Module**: `src-tauri/src/modules/release_gate/test_sandbox_lifecycle_verifier.rs`
- **Preconditions**: Automated integration test execution.
- **Actions**:
  1. Generate test sandbox instance via `generate_transient_sandbox_id`.
  2. Create temporary directories, storage, and spawned mock processes.
  3. Run `test-clean` routine and verify `validate_sandbox_cleanup` reports `is_clean: true`.
- **Expected Outcome**: Tests leave zero orphan filesystem remnants or detached background PIDs.

### `E2E-192-03`: Thinking Cache Invalidation Release Notification Control
- **Target Module**: `src-tauri/src/modules/release_gate/thinking_cache_invalidation_guard.rs`
- **Preconditions**: Application upgrade to target schema version.
- **Actions**:
  1. Launch app with `SUGGESTION_DELETE_THINKING_STORE = false`; verify no cache prompt modal.
  2. Launch app after major refactor with flag enabled matching current version; verify one-time modal shown.
  3. Acknowledge modal and verify state persists in `gui_config.json`, suppressing subsequent prompts.
- **Expected Outcome**: One-time cache cleaning prompts trigger strictly on designated refactor releases.

### `E2E-192-04`: Commit Co-Author Attribution Trailer Enforcement
- **Target Module**: `src-tauri/src/modules/release_gate/coauthor_attribution_verifier.rs`
- **Preconditions**: Git commit message authoring on merge or proxy PRs.
- **Actions**:
  1. Format co-author trailer with valid name and email; verify format `Co-authored-by: Name <email>`.
  2. Parse multi-author commit message; verify all valid authors extracted.
  3. Test invalid email formatting; verify rejection before commit creation.
- **Expected Outcome**: Contributor attribution trailers are strictly validated and formatted.

### `E2E-192-05`: Pull Request Template 4-Section Compliance Gate
- **Target Module**: `src-tauri/src/modules/release_gate/pr_template_compliance_linter.rs`
- **Preconditions**: GitHub PR description submission.
- **Actions**:
  1. Submit PR markdown containing Problem Classification, Behavior Alterations, Unverified Paths, and Rollback Strategy; verify `is_compliant: true`.
  2. Omit Rollback Strategy section; verify linter rejection with explicit missing section warning.
- **Expected Outcome**: Every pull request comprehensively documents its blast radius and rollback plan.

### `E2E-192-06`: Upstream Response Payload Decompression
- **Target Module**: `src-tauri/src/proxy/handlers/common/decompressor.rs`
- **Preconditions**: Upstream HTTP response containing gzip or deflate compressed error payloads.
- **Actions**:
  1. Receive gzip response with `Content-Encoding: gzip`; verify decompression to valid JSON text.
  2. Receive deflate response with `Content-Encoding: deflate`; verify decompression.
  3. Receive unadvertised gzip response with magic bytes `0x1F, 0x8B`; verify auto-decompression.
- **Expected Outcome**: Gateway inspection pipeline reads upstream error responses transparently.

### `E2E-192-07`: Network Chunk SSE Event Boundary Normalization
- **Target Module**: `src-tauri/src/proxy/handlers/common/boundary_normalizer.rs`
- **Preconditions**: Incoming streaming LLM tokens split across arbitrary TCP packet boundaries.
- **Actions**:
  1. Stream fragmented chunks split across field names and double-newline delimiters.
  2. Verify `feed_chunk` buffers incomplete fragments and emits only fully formed event frames.
  3. Flush remaining buffer at stream completion.
- **Expected Outcome**: Client response channels receive well-formed SSE frames without parsing errors.

### `E2E-192-08`: Multimodal Streaming Chunk UTF-8 Boundary Splitting
- **Target Module**: `src-tauri/src/proxy/handlers/common/chunk_boundary_validator.rs`
- **Preconditions**: Streaming chunks containing multi-byte UTF-8 characters (e.g. CJK ideographs, emojis).
- **Actions**:
  1. Slice multi-byte character across chunk boundary.
  2. Verify `validate_chunk_boundary` identifies `IncompleteUtf8` with exact `valid_up_to` offset.
  3. Split chunk via `split_at_safe_utf8_boundary`, forwarding valid bytes and buffering tail.
- **Expected Outcome**: Zero UTF-8 decoding replacement characters (``) in streamed client responses.

### `E2E-192-09`: Control Character Sanitization in Large JSON Payloads
- **Target Module**: `src-tauri/src/proxy/handlers/common/json_payload_sanitizer.rs`
- **Preconditions**: Client request payload containing raw ASCII control characters or terminal sequences.
- **Actions**:
  1. Feed raw JSON string with unescaped control bytes (< 0x20).
  2. Verify `sanitize_raw_json` converts unescaped bytes into valid Unicode escape sequences (`\u0007`).
  3. Verify deserialization succeeds cleanly via `serde_json`.
- **Expected Outcome**: Control character injection errors are eradicated before reaching LLM backends.

### `E2E-192-10`: Malformed JSON Trailing Comma Normalization
- **Target Module**: `src-tauri/src/proxy/handlers/common/json_payload_sanitizer.rs`
- **Preconditions**: Client request payload containing trailing commas in object or array definitions.
- **Actions**:
  1. Feed payload `{"models": ["claude-3-opus", "gpt-4o",], "debug": true,}`.
  2. Verify `strip_trailing_commas` strips invalid trailing commas while preserving commas in string values.
  3. Verify JSON parses without syntax errors.
- **Expected Outcome**: Resilient payload handling accommodates minor client formatting imperfections.
