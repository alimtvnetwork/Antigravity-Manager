# Specification 193: Implementing Next Five Tasks (Process Execution, Parity & Deep Inspection)

## 1. Overview & Context
This specification defines the implementation requirements for the 5 continuous compliance and reliability tasks in Sequence 193.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

## 2. Five Planned Tasks Breakdown

### Task 1: Profile Duplication Parity & Workspace Continuity
- **Module**: `src-tauri/src/modules/release_gate/profile_duplication_parity_guard.rs`
- **Specification**:
  - Enforce that all profile/instance duplication and cloning routes (from both GUI and CLI) invoke identical underlying logic (`copy_instance_with_options`).
  - Guarantee that `workspaceStorage` directory is preserved across copies to maintain developer project continuity and conversation state.
  - Test assertions: verify options structure, path validation, and workspace directory preservation flag.

### Task 2: Settings Deep-Merge Synchronization
- **Module**: `src-tauri/src/modules/release_gate/settings_deep_merge_synchronizer.rs`
- **Specification**:
  - Ensure deep-merge synchronization between parent and cloned instances without overwriting instance-specific identity tokens (port, instance id, name).
  - Recursively merge nested JSON configurations (e.g., telemetry flags, editor preferences, model configs).
  - Test assertions: verify recursive key merging, array retention, and collision preservation.

### Task 3: Windows Job Object Breakaway Fallback Ladder
- **Module**: `src-tauri/src/modules/release_gate/job_breakaway_fallback_ladder.rs`
- **Specification**:
  - Prohibit unconditional specification of `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`).
  - Implement a 2-stage fallback ladder:
    1. Attempt spawn with `CREATE_BREAKAWAY_FROM_JOB`.
    2. If `os error 5: Access is denied` occurs (restricted CI or container Job Object), fall back to standard spawn without breakaway flags.
  - Test assertions: error code 5 detection, fallback decision, and logging.

### Task 4: Windows Batch File Direct Execution Guard
- **Module**: `src-tauri/src/modules/release_gate/batch_file_spawn_guard.rs`
- **Specification**:
  - Prevent passing `.cmd` or `.bat` batch files directly as executable path in `std::process::Command::new(...)`.
  - Enforce 3-stage launcher fallback: target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`).
  - Test assertions: batch extension detection (`.bat`, `.cmd`, case-insensitive), shell dispatch argument wrapping.

### Task 5: Deep Transcript Inspection & Telemetry Bypass
- **Module**: `src-tauri/src/modules/release_gate/transcript_deep_inspection_scanner.rs`
- **Specification**:
  - When inspecting conversation transcripts, prioritize un-truncated `transcript_full.jsonl` over `transcript.jsonl`.
  - Reverse-scan across at least 10 lines, bypassing trailing `TOKEN_USAGE`, `HEARTBEAT`, and `TELEMETRY` records.
  - Extract genuine tool executions (`tool_calls`) and assistant thoughts (`thinking`).
  - Test assertions: reverse-scan window depth >= 10, skipping non-content records, extracting target tool calls.

## 3. Invariants & Rules
- Zero builds and zero tests during implementation phase.
- Strictly repository-relative Git paths.
- Search exclusively via GitMap.
- All 5 tasks marked `IN_PROGRESS` and committed as taken before assigning execution.
