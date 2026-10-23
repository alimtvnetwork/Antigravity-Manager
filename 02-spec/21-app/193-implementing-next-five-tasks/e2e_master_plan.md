# Deferred Master End-to-End Test Plan — Sequence 193

## Overview
This document catalogs the comprehensive End-to-End (E2E) test suites enqueued during **Sequence 193**.
In accordance with user instructions, zero test runs and zero build commands were executed during this implementation increment. These E2E scenarios are structured for autonomous execution during the dedicated final verification phase.

---

## Test Cases Inventory

### `E2E-193-01`: Profile Duplication Parity Across GUI and CLI
- **Target Module**: `src-tauri/src/modules/release_gate/profile_duplication_parity_guard.rs`
- **Preconditions**: Base instance `inst-primary` exists with configured `workspaceStorage`.
- **Actions**:
  1. Trigger instance clone via GUI modal.
  2. Trigger instance clone via CLI `agm instances copy inst-primary inst-clone-cli`.
  3. Verify both operations pass through `copy_instance_with_options`.
  4. Verify target directory contains identical `workspaceStorage` directory preserving conversations and project state.
- **Expected Outcome**: Headless and GUI clone operations produce byte-identical workspace continuity.

### `E2E-193-02`: Settings Deep-Merge with Identity Token Preservation
- **Target Module**: `src-tauri/src/modules/release_gate/settings_deep_merge_synchronizer.rs`
- **Preconditions**: Cloned instance `inst-clone-1` (port 9225) synchronizing from primary (port 9222).
- **Actions**:
  1. In primary instance, update nested config `editor.fontSize: 18`, add custom plugin, and change theme.
  2. Trigger settings sync to cloned instance.
  3. Verify cloned instance receives `fontSize: 18`, new plugin, and theme.
  4. Verify cloned instance retains port `9225` and its unique `id` and `profile_dir`.
- **Expected Outcome**: Settings synchronize deep-merge without identity collisions.

### `E2E-193-03`: Windows Job Object Breakaway Fallback Ladder Execution
- **Target Module**: `src-tauri/src/modules/release_gate/job_breakaway_fallback_ladder.rs`
- **Preconditions**: Application running inside a restricted Windows Job Object (e.g. GitHub Actions runner or sandbox).
- **Actions**:
  1. Attempt child process spawn with `CREATE_BREAKAWAY_FROM_JOB`.
  2. Intercept Win32 error 5 (`Access is denied`).
  3. Step down to standard spawn without breakaway flags.
  4. Verify process spawns successfully without throwing unhandled fatal error.
- **Expected Outcome**: Fallback ladder guarantees process launch in restricted environments.

### `E2E-193-04`: Direct Batch File Execution Interception and Shell Dispatch
- **Target Module**: `src-tauri/src/modules/release_gate/batch_file_spawn_guard.rs`
- **Preconditions**: Launching external helper script `tools/setup.bat`.
- **Actions**:
  1. Submit executable target `tools/setup.bat` with arguments `["--clean"]`.
  2. Verify `validate_executable_path` intercepts direct execution.
  3. Verify `resolve_windows_execution_command` formats command as `cmd.exe /c start "" tools/setup.bat --clean`.
  4. Verify successful launch via Windows shell.
- **Expected Outcome**: Raw batch files are never passed directly to `Command::new()`.

### `E2E-193-05`: Deep Transcript Reverse-Scan and Telemetry Bypass
- **Target Module**: `src-tauri/src/modules/release_gate/transcript_deep_inspection_scanner.rs`
- **Preconditions**: Long-running conversation transcript with 20 trailing telemetry, heartbeat, and token-usage records.
- **Actions**:
  1. Select log file; verify precedence chooses `transcript_full.jsonl` when available.
  2. Reverse-scan across minimum 10-line window.
  3. Verify telemetry and heartbeat lines are counted in `bypassed_telemetry_count` and bypassed.
  4. Extract genuine `tool_calls` and `thinking` blocks from earlier assistant turn.
- **Expected Outcome**: Genuine assistant turns and tool calls are accurately parsed without telemetry noise.
