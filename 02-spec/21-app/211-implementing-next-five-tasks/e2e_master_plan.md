# End-to-End Master Test Plan: Sequence 211 (Steps 266–270)

## 1. Scope & Objectives
This document catalogs the comprehensive end-to-end and integration test suite for the 5 governance and architectural invariant modules implemented in Sequence 211 (`Imp-680` through `Imp-684`).
In accordance with the **Zero-Build & Zero-Test Invariant**, live test execution is deferred until an explicit testing prompt is provided.
Any AI agent executing deferred end-to-end verification must execute the scenarios detailed below.

All modules adhere strictly to the invariants codified in `AGENTS.md`:
- **Process Spawning & Job Breakaway Invariant**: Fallback ladder to `cmd.exe /c start` must properly escape arguments to prevent injection.
- **FIFO Queue Invariant**:Drained SQLite batches must verify strictly increasing `(created_at, id)` ordering across all workers.
- **Pipeline Streaming Boundary Invariant**: Limit streaming chunks to bounded memory frames (1MB) with compliant delimiters.
- **Interactive Truncation Callouts**: Format raw markers into interactive dark-glass omission banners with human-readable size labels.
- **PR Revertability Invariant**: Multi-commit changesets must maintain self-contained, individually revertable boundaries.

---

## 2. Step-by-Step Task Mapping
| Step | Improvement ID | Module | Target File | Verification Test Suite |
| :--- | :--- | :--- | :--- | :--- |
| Step 266 | `Imp-680` | `modules::instance` | `src-tauri/src/modules/instance/windows_shell_dispatch_argument_escaper.rs` | `src-tauri/src/modules/instance/tests_windows_shell_dispatch_argument_escaper.rs` |
| Step 267 | `Imp-681` | `modules::db_orm` | `src-tauri/src/modules/db_orm/fifo_queue_drain_monotonicity_verifier.rs` | `src-tauri/src/modules/db_orm/tests_fifo_queue_drain_monotonicity_verifier.rs` |
| Step 268 | `Imp-682` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/streaming_chunk_boundary_guard.rs` | `src-tauri/src/proxy/handlers/common/tests_streaming_chunk_boundary_guard.rs` |
| Step 269 | `Imp-683` | `modules::instance` | `src-tauri/src/modules/instance/truncation_omission_banner_formatter.rs` | `src-tauri/src/modules/instance/tests_truncation_omission_banner_formatter.rs` |
| Step 270 | `Imp-684` | `modules::release_gate` | `src-tauri/src/modules/release_gate/individually_revertable_commit_guard.rs` | `src-tauri/src/modules/release_gate/tests_individually_revertable_commit_guard.rs` |

---

## 3. Comprehensive Test Scenarios

### Test Case `E2E-211-01`: Windows Shell Dispatch Argument Escaper
- **Target Module**: `src-tauri/src/modules/instance/windows_shell_dispatch_argument_escaper.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_windows_shell_dispatch_argument_escaper.rs`
- **Improvement ID**: `Imp-680`
- **Execution Steps**:
  1. Pass plain alphanumeric arguments; assert unquoted or properly passed.
  2. Pass arguments with spaces; assert wrapped in double quotes.
  3. Pass arguments with shell metacharacters (`&`, `|`, `^`, `%`, `>`); assert properly escaped.
  4. Build full command line string for `cmd.exe /c start "" "target.exe"`; assert syntax safety.

### Test Case `E2E-211-02`: FIFO Queue Drain Monotonicity Verifier
- **Target Module**: `src-tauri/src/modules/db_orm/fifo_queue_drain_monotonicity_verifier.rs`
- **Unit Suite**: `src-tauri/src/modules/db_orm/tests_fifo_queue_drain_monotonicity_verifier.rs`
- **Improvement ID**: `Imp-681`
- **Execution Steps**:
  1. Feed series of queue records with strictly increasing `(created_at, id)`; assert `is_monotonic: true`.
  2. Feed series with timestamp regression or inverted IDs; assert failure with `SequenceInversionDetected`.
  3. Feed series with duplicate IDs; assert failure with `DuplicateIdDetected`.

### Test Case `E2E-211-03`: Streaming Chunk Boundary Guard
- **Target Module**: `src-tauri/src/proxy/handlers/common/streaming_chunk_boundary_guard.rs`
- **Unit Suite**: `src-tauri/src/proxy/handlers/common/tests_streaming_chunk_boundary_guard.rs`
- **Improvement ID**: `Imp-682`
- **Execution Steps**:
  1. Feed standard SSE frame chunks (< 64KB) with `\n\n` delimiter; assert valid.
  2. Feed oversized chunk exceeding 1MB capacity; assert rejection with `OversizedChunkDetected`.
  3. Feed chunk missing valid SSE frame delimiter; assert failure with `MalformedFrameDelimiter`.

### Test Case `E2E-211-04`: Truncation Omission Banner Formatter
- **Target Module**: `src-tauri/src/modules/instance/truncation_omission_banner_formatter.rs`
- **Unit Suite**: `src-tauri/src/modules/instance/tests_truncation_omission_banner_formatter.rs`
- **Improvement ID**: `Imp-683`
- **Execution Steps**:
  1. Parse `<truncated 4096 bytes>`; assert formatted label `4.0 KB` and banner generated.
  2. Parse `<truncated 25 lines>`; assert formatted label `25 lines` and banner generated.
  3. Verify dark-glass class styling and interactive toggle callback metadata.

### Test Case `E2E-211-05`: Individually Revertable Commit Guard
- **Target Module**: `src-tauri/src/modules/release_gate/individually_revertable_commit_guard.rs`
- **Unit Suite**: `src-tauri/src/modules/release_gate/tests_individually_revertable_commit_guard.rs`
- **Improvement ID**: `Imp-684`
- **Execution Steps**:
  1. Audit clean, self-contained commit series; assert compliant.
  2. Audit commit with empty or single-word message ("fix"); assert flagged with `VagueCommitMessage`.
  3. Audit commit series where an intermediate commit introduces broken syntax or cross-commit dependency coupling; assert flagged.

---

## 4. Exact Targeted Deferred Test Commands
When an explicit testing prompt is issued, run the following commands sequentially:

```bash
# Step 266 (Imp-680): Windows Shell Dispatch Argument Escaper
cargo test --package antigravity-manager --lib modules::instance::tests_windows_shell_dispatch_argument_escaper

# Step 267 (Imp-681): FIFO Queue Drain Monotonicity Verifier
cargo test --package antigravity-manager --lib modules::db_orm::tests_fifo_queue_drain_monotonicity_verifier

# Step 268 (Imp-682): Streaming Chunk Boundary Guard
cargo test --package antigravity-manager --lib proxy::handlers::common::tests_streaming_chunk_boundary_guard

# Step 269 (Imp-683): Truncation Omission Banner Formatter
cargo test --package antigravity-manager --lib modules::instance::tests_truncation_omission_banner_formatter

# Step 270 (Imp-684): Individually Revertable Commit Guard
cargo test --package antigravity-manager --lib modules::release_gate::tests_individually_revertable_commit_guard
```
