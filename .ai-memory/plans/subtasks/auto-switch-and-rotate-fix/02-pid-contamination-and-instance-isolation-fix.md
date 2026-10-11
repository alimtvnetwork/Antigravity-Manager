# Subtask Implementation Plan: PID Contamination & Instance Isolation Fix

> **Subtask ID:** `02-pid-contamination-and-instance-isolation-fix`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Target Files:** `src-tauri/src/modules/instance/pid_scan.rs`  
> **Status:** `[DONE]`  
> **Author:** @aukgit  

---

## 1. Context & Objectives

Comparing current HEAD against `v4.162.0` revealed that Fast-Forward (instance restart) and profile switching failed because stopped instances were identified as still running.

### Root Cause:
In `src-tauri/src/modules/instance/pid_scan.rs:302`:
```rust
if matched_set.contains(&p) || instance_root_pids.contains(&p) {
```
The check examined `instance_root_pids`, which represents all global root PIDs across every instance on the host. When walking child process trees, child processes of active instances were attached to the target stopped instance. Consequently, stopped instances returned positive PID counts, preventing Fast-Forward restart logic and credential injection.

---

## 2. Implementation Summary

1. In `src-tauri/src/modules/instance/pid_scan.rs`:
   - Stripped `instance_root_pids.contains(&p)`.
   - Constrained child process matching strictly to `matched_set.contains(&p)`.
   - Recorded newly matched child PIDs directly into `matched_set` to guarantee full transitive closure for the isolated instance only.
