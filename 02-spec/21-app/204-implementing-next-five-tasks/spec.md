# Sequence 204 Specification: Implementing Next Five Tasks (Steps 231–235)

## Overview
This specification details the design, constraints, and verification requirements for Batch 204 (Steps 231–235 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-645`)**: `src-tauri/src/proxy/handlers/common/pipeline_request_header_sanitizer.rs` & `tests_pipeline_request_header_sanitizer.rs`
   - Enforce AGENTS.md request header sanitization within the pipeline: uniformly handles converted Gemini payloads, sanitizing risky headers, stripping unauthorized metadata, filtering Hop-by-Hop headers, and preserving authenticated Gemini pipeline headers.
2. **Task 2 (`Imp-646`)**: `src-tauri/src/modules/release_gate/strictly_relative_git_paths_guard.rs` & `tests_strictly_relative_git_paths_guard.rs`
   - Enforce AGENTS.md strictly relative Git paths invariant: audits commit messages, changelogs, release notes, and markdown documentation, strictly blocking absolute OS filesystem paths (`C:\`, `D:\`, `/home/`, `/Users/`) and `file:///` URIs.
3. **Task 3 (`Imp-647`)**: `src-tauri/src/modules/instance/transient_sandbox_lifecycle_verifier.rs` & `tests_transient_sandbox_lifecycle_verifier.rs`
   - Enforce AGENTS.md caller IDE self-preservation & test sandboxing invariant: verifies isolated test sandbox lifecycles (`agm instances test-create` and `test-clean`), validating that transient test instances leave zero orphan processes or filesystem remnants.
4. **Task 4 (`Imp-648`)**: `src-tauri/src/modules/release_gate/rollback_reference_drift_verifier.rs` & `tests_rollback_reference_drift_verifier.rs`
   - Enforce AGENTS.md branch & history hygiene invariant: audits history rewrites, ensuring `backup/*` local rollback references exist, zero content drift is confirmed via diff stats, and open in-flight PRs are inspected before force operations.
5. **Task 5 (`Imp-649`)**: `src-tauri/src/modules/instance/multi_stage_launcher_dispatcher.rs` & `tests_multi_stage_launcher_dispatcher.rs`
   - Enforce AGENTS.md process spawning & job breakaway invariant: executes multi-stage launcher fallback (target binary -> detected base binary -> Windows Shell dispatch `cmd.exe /c start`) while preventing direct `.cmd`/`.bat` execution and handling Windows Job Object access denial.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt restore queries must enforce `ORDER BY created_at ASC, id ASC`.
