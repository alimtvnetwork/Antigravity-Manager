# Master E2E Test Plan: Batch 204 (Steps 231–235)

## Overview
This master verification plan outlines the deferred end-to-end testing criteria for all components introduced in Batch 204.
All tests are implemented as unit tests with mocked environments, preserving the Zero-Build / Zero-Test Invariant during normal coding runs.

## Suite Matrix
| Step | Implementation ID | Module Target | Test Suite File |
| :--- | :--- | :--- | :--- |
| Step 231 | `Imp-645` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/tests_pipeline_request_header_sanitizer.rs` |
| Step 232 | `Imp-646` | `modules::release_gate` | `src-tauri/src/modules/release_gate/tests_strictly_relative_git_paths_guard.rs` |
| Step 233 | `Imp-647` | `modules::instance` | `src-tauri/src/modules/instance/tests_transient_sandbox_lifecycle_verifier.rs` |
| Step 234 | `Imp-648` | `modules::release_gate` | `src-tauri/src/modules/release_gate/tests_rollback_reference_drift_verifier.rs` |
| Step 235 | `Imp-649` | `modules::instance` | `src-tauri/src/modules/instance/tests_multi_stage_launcher_dispatcher.rs` |

## Acceptance Criteria
1. `pipeline_request_header_sanitizer`:
   - Strips hop-by-hop headers (`connection`, `keep-alive`, `transfer-encoding`, `upgrade`).
   - Strips dangerous authorization leaks and prompt injection metadata (`x-goog-user-project`, `x-antigravity-internal`, `cf-connecting-ip`).
   - Retains genuine Gemini protocol headers (`x-goog-api-key`, `content-type`, `authorization`).
2. `strictly_relative_git_paths_guard`:
   - Scans text strings, commit messages, and changelogs.
   - Detects and flags Windows drive paths (e.g. drive letter followed by colon and slash).
   - Detects and flags POSIX home or system paths (e.g. root home, Users, or system directories).
   - Detects and flags file URI schemes.
   - Passes clean repository-relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
3. `transient_sandbox_lifecycle_verifier`:
   - Verifies sandbox creation generates unique directory under `target/sandboxes/`.
   - Protects caller IDE instance from termination unless `--force` is provided.
   - Verifies complete cleanup leaving zero orphan processes and zero directory remnants.
4. `rollback_reference_drift_verifier`:
   - Rejects history rewrites if no `backup/*` reference is recorded.
   - Rejects history rewrites if diff stat reports content drift between backup and HEAD.
   - Detects open in-flight PRs targeting the shared branch before force push.
5. `multi_stage_launcher_dispatcher`:
   - Disallows direct execution of `.cmd` or `.bat` batch files via `std::process::Command`.
   - Executes multi-stage ladder: Stage 1 (Direct verified binary) -> Stage 2 (Base binary) -> Stage 3 (Windows Shell `cmd.exe /c start`).
   - Handles Windows Job Object `os error 5` by falling back to standard spawn without job breakaway.
