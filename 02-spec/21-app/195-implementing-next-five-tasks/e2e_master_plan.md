# End-to-End Master Test Plan: Sequence 195 (Release Gate & Architectural Invariants)

## 1. Scope & Objectives
This document catalogs the comprehensive end-to-end and integration test suite for the 5 compliance modules implemented in Sequence 195.
In accordance with the **Zero-Build & Zero-Test Invariant**, live execution is deferred until an explicit testing prompt is provided.
Any AI agent executing deferred end-to-end verification must execute the scenarios detailed below.

---

## 2. Test Cases Specification

### Test Case `E2E-195-01`: Risky Path Replacement & Explicit Fallback Verification
- **Target Module**: `src-tauri/src/modules/release_gate/risky_path_fallback_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_risky_path_fallback.rs`
- **Objective**: Ensure that any speculative optimization or replaced path guarantees explicit fallback to the verified path and specifically handles 0-match conditions (prohibiting silent no-ops).
- **Execution Steps**:
  1. Initialize `RiskyPathReplacement` with `FallbackStrategy::VerifiedPathRestart`, verifying that validation returns `Ok(report)` with `is_valid: true`.
  2. Initialize `RiskyPathReplacement` with `FallbackStrategy::CustomFallbackHandler`, verifying acceptance of registered fallback handlers.
  3. Attempt to register a replacement with `FallbackStrategy::SilentNoOp`; verify immediate rejection with `ReplacementViolation::SilentNoOpDisallowed`.
  4. Attempt to register a replacement with `FallbackStrategy::Unspecified`; verify rejection with `ReplacementViolation::UnspecifiedFallback`.
  5. Set `handles_zero_matches = false`; verify validation marks `is_valid: false` and flags missing zero-match fallback.
  6. Pass speculative source code containing `if matches.is_empty() { return Ok(()); }` to `lint_risky_path_code`; verify detection of `EarlyReturnWithoutFallback`.
  7. Pass clean fallback code with `full_restart_verified(content)`; verify 0 linter violations.
- **Expected Outcome**: All 14 unit tests pass, enforcing that no speculative path can fail silently without falling back to baseline verified execution.

---

### Test Case `E2E-195-02`: Side Effect Ordering & Point-of-No-Return Verification
- **Target Module**: `src-tauri/src/modules/release_gate/side_effect_ordering_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_side_effect_ordering.rs`
- **Objective**: Ensure operations order side effects to fail before the point of no return: write credentials before killing a process, validate before deleting.
- **Execution Steps**:
  1. Construct a valid sequence: `PreValidation` -> `CredentialPersistence` -> `StateBackup` -> `PointOfNoReturnBoundary` -> `ProcessKill` -> `EntityDeletion` -> `FilesystemPurge`.
  2. Invoke `validate_operation_sequence`; verify `is_order_compliant: true` and point-of-no-return index detected at boundary.
  3. Invert order: place `ProcessKill` before `CredentialPersistence`; verify immediate failure with `OrderingViolation::DestructivePrecedesPreparation` referencing the exact AGENTS.md invariant ("Write credentials before killing a process").
  4. Invert order: place `EntityDeletion` before `PreValidation`; verify failure with `OrderingViolation::DestructivePrecedesPreparation` ("Validate before deleting").
  5. Invert order: place `FilesystemPurge` before `StateBackup`; verify failure ("Backup state before purging filesystem").
  6. Place a `CredentialPersistence` operation *after* `PointOfNoReturnBoundary`; verify rejection with `OrderingViolation::PreparationAfterPointOfNoReturn`.
  7. Place two boundary markers in the sequence; verify rejection with `OrderingViolation::MultiplePointsOfNoReturn`.
- **Expected Outcome**: All 13 unit tests pass, guaranteeing that irreversible side effects can never precede validation and persistence.

---

### Test Case `E2E-195-03`: Bounded Matcher Scope & Asynchronous Timeout Verification
- **Target Module**: `src-tauri/src/modules/release_gate/bounded_matcher_timeout_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_bounded_matcher_timeout.rs`
- **Objective**: Validate that string matchers stay inside intended boundaries (never across line breaks, tags, or clauses) and that all asynchronous waits are strictly bounded with explicit timeouts.
- **Execution Steps**:
  1. Test `SingleLineOnly`: verify single line match passes; verify multi-line match crossing `\n` fails with `BoundaryViolation::SpansLineBreaksForbidden`.
  2. Test `ClauseBounded`: verify match crossing statement semicolon (`;`) or sentence terminal period (`. `) fails with `BoundaryViolation::CrossesClausesForbidden`.
  3. Verify SemVer decimal points (e.g. `4.8.2`) are correctly recognized as non-clause delimiters and pass validation.
  4. Test `TagEnclosed`: verify match crossing HTML/XML tags (`<div>`, `<span>`) fails with `BoundaryViolation::CrossesHtmlTagsForbidden`.
  5. Test `TimeoutConfig::bounded(3000, 10000, "op")`; verify `validate_wait_timeout` passes with effective timeout 3000ms.
  6. Test `TimeoutConfig::unbounded(10000, "op")` (`wait_ms: None`) and `wait_ms: Some(0)`; verify rejection with `TimeoutViolation::UnboundedWaitForbidden`.
  7. Test `TimeoutConfig::bounded(15000, 10000, "op")`; verify rejection with `TimeoutViolation::ExceedsMaxAllowed`.
  8. Run `lint_wait_calls_in_source` across snippets; verify detection of naked `std::thread::sleep`, naked `tokio::time::sleep`, and unbounded `loop { sleep(...) }` while allowing bounded timeouts and deadlines.
- **Expected Outcome**: All 18 unit tests pass, verifying complete protection against matcher bleeding and infinite hangs.

---

### Test Case `E2E-195-04`: Remote Branch Base Precedence Verification
- **Target Module**: `src-tauri/src/modules/release_gate/remote_branch_base_verifier.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_remote_branch_base.rs`
- **Objective**: Verify that git branch bases strictly precede from remote tracking branches (`origin/main`, `origin/beta`, `upstream/main`, `upstream/beta`) and prohibit local bases to eliminate untracked ancestor commits.
- **Execution Steps**:
  1. Verify `BranchBaseVerification` with channel `StableMain` and base `origin/main` (or `upstream/main`); assert `is_base_valid: true`.
  2. Verify channel `PreviewBeta` with base `origin/beta` (or `upstream/beta`); assert `is_base_valid: true`.
  3. Attempt to branch `StableMain` from local `main`; assert rejection with `BranchBaseViolation::LocalBranchBaseForbidden`.
  4. Attempt to branch `PreviewBeta` from local `beta` or `feature/xyz`; assert rejection with `BranchBaseViolation::LocalBranchBaseForbidden`.
  5. Attempt to branch `PreviewBeta` from `origin/main`; assert rejection with `BranchBaseViolation::ChannelBaseMismatch` ("preview releases must base from origin/beta").
  6. Attempt to branch `StableMain` from `origin/beta`; assert rejection with `BranchBaseViolation::ChannelBaseMismatch` ("stable releases must base from origin/main").
  7. Verify `normalize_git_ref` properly strips `refs/remotes/` and `refs/heads/`.
- **Expected Outcome**: All 14 unit tests pass, preventing untracked ancestor commits and cross-channel misplacement.

---

### Test Case `E2E-195-05`: Local Commit Convergence & Noise Elimination Verification
- **Target Module**: `src-tauri/src/modules/release_gate/commit_convergence_auditor.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_commit_convergence.rs`
- **Objective**: Audit git branch commit history to ensure intermediate trial-and-error noise is consolidated into clean final-state commits before PR creation or merge.
- **Execution Steps**:
  1. Audit a history of clean, well-rationalized commits; verify `is_converged: true`, `noise_commit_count: 0`, and recommendation indicating ready for PR.
  2. Inject commits with WIP keywords: `"wip: initial draft"`, `"wip2"`, `"checkpoint before refactor"`; verify classification as `NoiseClassification::WipCheckpoint` and `is_converged: false`.
  3. Inject commits with trial-and-error phrases: `"try fix"`, `"temp"`, `"undo test"`, `"fixed typo"`, `"revert previous"`, `"test again"`, `"fix fix"`, `"oops"`; verify classification as `NoiseClassification::TrialAndErrorNoise(...)`.
  4. Inject scattered generic micro-fixes touching identical files (`"fix api"`, `"tweak handler"`, `"adjust response"`); verify classification as `NoiseClassification::ScatteredFixes(3)`.
  5. Audit empty commit list; verify `is_converged: true`.
  6. Verify JSON serde serialization and deserialization roundtrip.
- **Expected Outcome**: All 14 unit tests pass, ensuring branch commit hygiene prior to opening PRs.

---

## 3. Test Execution Command (Deferred)
When the user explicitly issues the testing prompt:
```bash
cargo test --package antigravity-manager --lib modules::release_gate::tests_risky_path_fallback
cargo test --package antigravity-manager --lib modules::release_gate::tests_side_effect_ordering
cargo test --package antigravity-manager --lib modules::release_gate::tests_bounded_matcher_timeout
cargo test --package antigravity-manager --lib modules::release_gate::tests_remote_branch_base
cargo test --package antigravity-manager --lib modules::release_gate::tests_commit_convergence
```
