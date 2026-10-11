# Subtask Implementation Plan: Win32 Job Breakaway & E7001 Resilience

> **Subtask ID:** `05-win32-job-breakaway-and-e7001-resilience`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Target Files:** `src-tauri/src/modules/process/launch.rs`, `src-tauri/src/modules/instance/launch.rs`, `src-tauri/src/modules/instance/executable.rs`  
> **Status:** `[DONE]`  
> **Author:** @aukgit  

---

## 1. Context & Objectives

Clicking "Launch Instance" on cloned profiles threw runtime error `E7001: Process error: Failed to spawn instance process: Access is denied. (os error 5)`.

### Root Cause:
1. `src-tauri/src/modules/process/launch.rs`: Passing `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`) on Windows when running within restricted Job Objects causes `CreateProcessW` to fail with `os error 5`.
2. `src-tauri/src/modules/instance/executable.rs`: Generating `launch-<id>.cmd` batch wrappers resulted in `os error 5` because `Command::new` cannot invoke batch files without `cmd.exe /c`.
3. Lack of multi-stage launch fallback when spawning cloned executables fails.

---

## 2. Implementation Summary

1. In `src-tauri/src/modules/process/launch.rs`:
   - Added `spawn_with_job_breakaway_fallback` with multi-tier retry (`BREAKAWAY` -> `PROCESS_GROUP` -> standard flags).
2. In `src-tauri/src/modules/instance/executable.rs`:
   - Eliminated `.cmd` batch scripts; cloned executables are copied to user-writable paths (`~/.antigravity_tools/instances/<id>/bin/Antigravity-<id>.exe`) as true PE binaries.
   - Executable resolvers reject `.cmd` and `.bat` files.
3. In `src-tauri/src/modules/instance/launch.rs`:
   - Implemented multi-stage launch fallback: target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start "" "<bin>"`).
