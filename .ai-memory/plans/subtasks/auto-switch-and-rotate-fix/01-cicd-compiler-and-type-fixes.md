# Subtask Implementation Plan: CI/CD Compiler & Type Fixes

> **Subtask ID:** `01-cicd-compiler-and-type-fixes`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Target Files:** `src-tauri/src/modules/process/running.rs`, `src-tauri/src/modules/auto_switcher/selection.rs`, `src-tauri/src/modules/instance/launch.rs`  
> **Status:** `[DONE]`  
> **Author:** @aukgit  

---

## 1. Context & Objectives

Investigating CI/CD errors via `gitmap pe -t` revealed compilation and Clippy failures blocking pipeline checks:
1. `src-tauri/src/modules/process/running.rs:14,21`: `find_pids_for_data_dir` expected `&str` but was passed `&PathBuf`.
2. `src-tauri/src/modules/auto_switcher/selection.rs:538,575`: Unit tests failed compilation because `TokenData` was missing from `mod tests` imports.
3. `src-tauri/src/modules/instance/launch.rs:490`: Rust compiler error `E0425: cannot find value inst_home_opt in this scope` occurred because `inst_home_opt` was declared inside a nested block and out of scope in the fallback launcher.

---

## 2. Implementation Summary

1. In `src-tauri/src/modules/process/running.rs`:
   - Converted `&default_dir` to `&default_dir.to_string_lossy()` on lines 14 and 21.
2. In `src-tauri/src/modules/auto_switcher/selection.rs`:
   - Added `use crate::models::TokenData;` inside `mod tests`.
3. In `src-tauri/src/modules/instance/launch.rs`:
   - Lifted `inst_home_opt` declaration outside the conditional block to provide uniform access in primary and fallback spawn routines.
