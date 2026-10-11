# Subtask Implementation Plan: AGM Update Downgrade & Revert Support

> **Subtask ID:** `04-agm-update-downgrade-revert-fix`  
> **Parent Slug:** `auto-switch-and-rotate-fix`  
> **Target Files:** `src-tauri/src/bin/agm/update_cmds.rs`, `src-tauri/src/modules/delegate_updater/`, `install.ps1`  
> **Status:** `[DONE]`  
> **Author:** @aukgit  

---

## 1. Context & Objectives

Users trying to downgrade or revert to previous stable versions like `v4.162.0` via `agm update to v4.162` or `agm update 4.162` found that the CLI ignored the target version, queried GitHub `releases/latest`, reported `[UP TO DATE]`, and refused to download or install the requested version.

### Root Cause:
1. `src-tauri/src/bin/agm/update_cmds.rs`: Positional arguments following `update` were discarded. The updater always hit `releases/latest`.
2. `update_cmds.rs:217`: `is_up_to_date` triggered when latest matched current, ignoring user intent to downgrade.
3. `install.ps1:1455`: Early exit if `TargetVersion -le curr` prevented downgrades unless `-isPinned` or `-Force` was respected.

---

## 2. Implementation Summary

1. In `src-tauri/src/bin/agm/update_cmds.rs`:
   - Added `extract_target_version` to parse `to <ver>`, `<ver>`, `--version <ver>`, `revert <ver>`, and `downgrade <ver>`.
   - Normalizes versions to SemVer tags (e.g. `v4.162.0`).
   - Queries `releases/tags/v{target}` when an explicit target is specified.
   - Forwards `--version` and allows downgrade execution without early exit.
2. In `src-tauri/src/modules/delegate_updater/`:
   - Ingests `--version` and forwards `-Version "<target>"` to the PowerShell installer.
3. In `install.ps1`:
   - Added `-and -not $isPinned` to early exit conditions, allowing explicit version downgrades.
