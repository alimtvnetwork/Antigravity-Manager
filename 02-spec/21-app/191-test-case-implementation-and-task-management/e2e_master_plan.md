# Deferred Master End-to-End Test Plan — Sequence 191

## Overview
This document catalogs the comprehensive End-to-End (E2E) test suites enqueued during **Sequence 191**.
In accordance with user instructions, zero test runs and zero build commands were executed during this implementation increment. These E2E scenarios are structured for autonomous execution during the dedicated final verification phase.

---

## Test Cases Inventory

### `E2E-191-01`: Docker Latest Tag Synchronization on Release
- **Target Module**: `src-tauri/src/modules/release_gate/docker_sync_verifier.rs`
- **Preconditions**: Release ceremony triggered on `main` branch.
- **Actions**:
  1. Dispatch release pipeline for tag `v4.251.0` with `makeLatest: true` and `prerelease: false`.
  2. Verify `evaluate_docker_tag_sync` returns `is_valid: true` and `latest_tag_allowed: true`.
  3. Attempt preview dispatch on `main` with `v4.251.0-beta.1`; verify immediate rejection.
- **Expected Outcome**: Production channels and `latest` tags update strictly on `main` releases without contamination.

### `E2E-191-02`: Preview Channel Isolation on Beta Branch
- **Target Module**: `src-tauri/src/modules/release_gate/docker_sync_verifier.rs`
- **Preconditions**: Release ceremony triggered on `beta` branch.
- **Actions**:
  1. Dispatch preview pipeline for tag `v4.252.0-beta.1` with `makeLatest: false` and `prerelease: true`.
  2. Verify `evaluate_docker_tag_sync` returns `is_valid: true` and `latest_tag_allowed: false`.
  3. Attempt dispatch on `beta` with `makeLatest: true`; verify immediate rejection.
- **Expected Outcome**: Beta releases are isolated as pre-releases and never touch production update channels.

### `E2E-191-03`: Pre-flight Gate Classification for Polyglot Workspaces
- **Target Module**: `src-tauri/src/modules/release_gate/preflight_gate_runner.rs`
- **Preconditions**: Code changes staged in working directory.
- **Actions**:
  1. Stage changes touching `src-tauri/src/lib.rs`; verify requirements mandate `cargo fmt` and `cargo clippy`.
  2. Stage changes touching `src/App.tsx`; verify requirements mandate `npm run build`.
  3. Stage changes touching only `.ai-memory/` and `02-spec/`; verify requirements allow compiler bypass.
- **Expected Outcome**: Developer pre-flight gates execute precisely according to touched module classifications.

### `E2E-191-04`: SemVer Monotonic Increment Gate
- **Target Module**: `src-tauri/src/modules/release_gate/semver_monotonic_validator.rs`
- **Preconditions**: Existing released tag `v4.250.0`.
- **Actions**:
  1. Evaluate bump to `v4.250.1`; verify `BumpType::Patch`.
  2. Evaluate bump to `v4.251.0`; verify `BumpType::Minor`.
  3. Evaluate bump to `v4.249.9`; verify failure with `BumpType::Downgrade`.
  4. Evaluate identical version `v4.250.0`; verify failure with `BumpType::Identical`.
- **Expected Outcome**: Monotonic SemVer progression is mathematically guaranteed across all release automation.

### `E2E-191-05`: Changelog Attribution and Absolute Path Linter Gate
- **Target Module**: `src-tauri/src/modules/release_gate/changelog_structure_validator.rs`
- **Preconditions**: Formatted release notes generated for version bump.
- **Actions**:
  1. Inspect release notes containing `(Thanks to @aukgit)`; verify attribution pass.
  2. Inject test unauthorized handle `@randomdev`; verify rejection.
  3. Inject test absolute path `D:\work\...`; verify rejection.
- **Expected Outcome**: Strict attribution to `@aukgit` and relative path hygiene are enforced before git tag creation.

### `E2E-191-06`: Local Rollback Reference & Content Drift Verification
- **Target Module**: `src-tauri/src/modules/release_gate/rollback_ref_guard.rs`
- **Preconditions**: Pre-flight state before history rewriting or rebase.
- **Actions**:
  1. Generate backup ref name via `format_backup_ref_name`; verify format `backup/<slug>-<timestamp>`.
  2. Simulate `git diff --stat <backup> HEAD` with changes; verify `has_drift: true`.
  3. Simulate empty diff; verify `has_drift: false`.
- **Expected Outcome**: Rollback safety checkpoints are verified zero-drift prior to destructive git operations.

### `E2E-191-07`: SQLite Split-DB Schema Integrity Boot Verification
- **Target Module**: `src-tauri/src/modules/db_orm/schema_integrity_gate.rs`
- **Preconditions**: Fresh connection to split database (`active_prompts.db`, `repo_prompts.db`).
- **Actions**:
  1. Invoke `SchemaIntegrityGate::verify_tables` with required table set.
  2. Simulate missing table; verify boot failure with explicit missing table list.
  3. Verify automatic exclusion of SQLite internal system tables.
- **Expected Outcome**: Split SQLite databases validate table presence before servicing queries.

### `E2E-191-08`: Dynamic Physical RAM SQLite Page Cache Tuning
- **Target Module**: `src-tauri/src/modules/db_orm/page_cache_manager.rs`
- **Preconditions**: Application boot on system with known physical memory.
- **Actions**:
  1. Simulate 4 GB host; verify `MemoryProfile::LowMemory` (-8192 KiB).
  2. Simulate 16 GB host; verify `MemoryProfile::Standard` (-32768 KiB).
  3. Simulate 64 GB host; verify `MemoryProfile::HighThroughput` (-131072 KiB).
  4. Apply cache size and verify `PRAGMA cache_size` reflects requested value.
- **Expected Outcome**: Memory cache automatically scales to host capacity without manual tuning.

### `E2E-191-09`: Protocol Canonical Role Normalization Across Multi-Model Adapters
- **Target Module**: `src-tauri/src/proxy/handlers/common/role_normalization_validator.rs`
- **Preconditions**: Incoming request payload from Claude, Gemini, or OpenAI clients.
- **Actions**:
  1. Parse input roles `human`, `bot`, `developer`, `model`.
  2. Verify translation to Gemini IR: `user` and `model`.
  3. Verify translation to OpenAI IR: `user`, `assistant`, `system`, `developer`.
  4. Verify turn sequence validation rejects payloads missing user/system origins.
- **Expected Outcome**: Complete role compatibility across all 4 adapter protocols.

### `E2E-191-10`: Client HTTP Request Content-Encoding Negotiation
- **Target Module**: `src-tauri/src/proxy/handlers/common/request_content_encoding_negotiator.rs`
- **Preconditions**: Incoming HTTP request with compressed payload.
- **Actions**:
  1. Send request with `Content-Encoding: gzip`; verify identification as `SupportedEncoding::Gzip`.
  2. Send request with `Content-Encoding: deflate`; verify identification as `SupportedEncoding::Deflate`.
  3. Send request with unsupported encoding; verify rejection before pipeline processing.
- **Expected Outcome**: Transparent decompression pipeline negotiates client encodings safely.
