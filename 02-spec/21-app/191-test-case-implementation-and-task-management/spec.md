# Specification 191: Release Gate Compliance and Engine Test Harnesses

## 1. Context & Objectives
This specification codifies the test suites, release gate compliance engines, and memory management harnesses implemented during Sequence 191.
All implementations enforce strict zero-warning compilation, zero runtime regressions, and absolute decoupling between proxy pipelines and adapter protocols.

## 2. Architectural Components

### 2.1 Release Gate & Compliance Architecture (`src-tauri/src/modules/release_gate/`)
1. **`docker_sync_verifier.rs` & `tests_docker_tag_sync.rs`**:
   - Production Docker/GitHub `latest` tag sync discipline exclusively on `main` branch.
   - Pre-release isolation exclusively on `beta` (`makeLatest: false`, `prerelease: true`).
2. **`preflight_gate_runner.rs` & `tests_preflight_gate.rs`**:
   - Classifies repository-relative changed paths into required preflight gates (`cargo fmt`, `cargo clippy`, `npm run build`).
3. **`semver_monotonic_validator.rs` & `tests_semver_monotonic.rs`**:
   - Mathematical verification of monotonic SemVer progression (patch, minor, major, beta increments).
   - Rejects version downgrades and tag format anomalies.
4. **`changelog_structure_validator.rs` & `tests_changelog_structure.rs`**:
   - Release note structure verification, strictly `@aukgit` attribution, and zero forbidden absolute paths (`D:\`, `C:\`, `/home/`).
5. **`rollback_ref_guard.rs` & `tests_rollback_ref_guard.rs`**:
   - Retention of local rollback refs (`backup/*`) and diff-stat zero drift confirmation prior to history rewrites.

### 2.2 Split-SQLite ORM & Protocol Test Suites
1. **`schema_integrity_gate.rs` & `tests_schema_integrity_gate.rs`**:
   - Validates required table existence and case-insensitive table matching across split databases during boot.
2. **`page_cache_manager.rs` & `tests_page_cache_manager.rs`**:
   - Dynamic host physical memory profile selection and negative KiB `PRAGMA cache_size` application.
3. **`role_normalization_validator.rs` & `tests_role_normalization_validator.rs`**:
   - Normalization and translation of canonical roles across Gemini, OpenAI, and Claude protocol representations.
4. **`request_content_encoding_negotiator.rs` & `tests_request_content_encoding_negotiator.rs`**:
   - Client HTTP request compression detection and support verification.

## 3. Verification & Compliance
- **Zero-Build & Zero-Test Invariant**: Fully respected during implementation.
- **Repository-Relative Paths**: All file paths strictly formatted as repo-relative.
- **E2E Enqueueing**: 10 comprehensive end-to-end test scenarios documented in `e2e_master_plan.md`.
