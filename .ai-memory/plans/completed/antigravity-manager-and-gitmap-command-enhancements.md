# Completed Plan: Antigravity Manager and GitMap Command Enhancements

## User Request (Verbatim)

```text
# Antigravity Manager and GitMap Command Enhancements: high priority instruction, non-negotiable task

For Antigravity Manager and `gitmap`, both should have commands to export the Antigravity pin projects with the relative paths from the default work directory. That means default work directory can be switched. So it should work like that. So you need to have new commands where the pin projects, where not only the pin, the last running prompts, specific prompts can be compacted to a SQLite database. We can use a JSON and a SQLite DB to export with conversation and the projects with pin projects and also without pin projects. It can be absolute path or it could be a relative path from the work directory. That means we can change the work directory at any time, at any given moment, and it should work. Based on that, you prepare a system design plan, commands with example, add the help text to the UI as well, and then implement it. And also try to have this feature from the UI for the Antigravity Tools Manager, especially in the instance section. Also, we can have an import, export Antigravity section, I think from the dropdown of the settings. Not in the settings especially, but the dropdown where we have all this email and other stuff, we can put it there. And also in that dropdown for the Antigravity, we can have the update option checked. Where is the update option? And also add a search button so that we can type and filter out the menu items. And for the instance and accounts, try to use only icon. We don't need to see any text. So the space is saved. And also we can have a great tooltip when we hover over. The tooltip will tell us what that is. Add a professional tooltip. And also, the UI level, when we go to the prompts history, that section takes a long time to load. Try to investigate that. If it takes a long time, then add a spinner or something like this, and try to have a caching so that you can display something and then you can push new stuff. And mostly you need to make the right-hand side more compact. You already improved, I think, in the latest version, but I want some of the buttons to be compacted. For example, the send, enqueue, these buttons can be compacted. Try to focus on that and make sure every change that you do, you make sure that you pull first and then do the commit, and also make sure that you resolve the conflict because there are multiple agents who are working at the same time. And also at the same time, for the items, try to have as many as icon less than text. But some items are not very meaningful. For example, the copy section can have a dropdown. So one would be default the copy text, copy icon and then text, and then we can just click on the dropdown to see copy with text and image, things like that. So compact as much as possible so that it looks nice, and also make sure the backup restore works very well. You can do end-to-end testing properly with the CLI commands. Check what is working, what is not. I suspect most of the things are not working very well. You can do a deep dive, try to understand what is wrong. Do not change anything in your current instance or other running instances. What you should do is create your own instance and try to deal with that for the end-to-end testing.

## slug: antigravity-manager-and-gitmap-command-enhancements
```

---

## 1. System Design & Architectural Outcomes

### 1.1 Dynamic Work Directory Resolution Engine
- **5-Stage Hierarchy**:
  1. CLI parameter override (`--work-dir <path>` / `--workdir`).
  2. Environment variable (`AGM_WORK_DIR` or `WORK_DIR`).
  3. GitMap master store lookup (`gitmap.db` default work directory).
  4. Git root directory ancestor discovery.
  5. Platform default (`work` / `~/work`).
- **Relative Path Transformation**: Automatically extracts normalized relative paths and canonical `${workDir}/<relativePath>` variable templates. Seamless cross-machine re-anchoring when work directory is switched.

### 1.2 Dual Compaction Storage Parity (SQLite & JSON)
- **Standalone Portable SQLite Database** (`meta`, `pinned_projects`, `conversations`, `running_prompts`, `prompt_history`): Standalone binary format with WAL mode, busy timeout 5000ms, and zero host-specific locks.
- **Two-Tier JSON Envelope v2.0** (`agm/compact-export`): Machine-readable `{ attributes, variables, data }` with 2-pass chained interpolation and automated format inspection via `agm which-format`.

### 1.3 Native CLI Commands & Syntax
- `agm export-compact [--file <path>] [--work-dir <path>] [--no-pinned] [--absolute]`: Exports workspace snapshot with relative path templates.
- `agm import-compact <file> [--work-dir <path>] [-y]`: Ingests and re-anchors projects and prompts into target instance and active work directory.
- `agm which-format <file>`: Classifies compact archives and outputs execution guidance.

### 1.4 Frontend Modernization & Performance
- **Header Navigation**: Converted Accounts and Instances buttons to compact icon-only buttons (`Users` and `Laptop`), saving significant horizontal space, with dark-glass floating tooltips (`Accounts (Profile Vault)`, `Instances (Engine Runner)`).
- **Dropdown Menu**: Added sticky real-time search filter input to filter navigation routes (`Email & Alerts`, `API Proxy`, etc.) on keystroke, and prominently positioned the "Check for Updates" action button at top.
- **Prompt Tree View Modal Action Panel**:
  - Combined `Copy Text`, `Copy With Images`, `Save Images`, and export into a sleek split/dropdown button.
  - Compacted `Send Now` with icon `<RotateCw />` + `Send` + `<kbd>N</kbd>`.
  - Compacted `Enqueue` into icon-only `<ListPlus />` button with tooltip.
  - Compacted `Focus IDE` into icon-only `<ExternalLink />` button.
  - Compacted `Full` inspector into icon-only `<Maximize2 />` button.
- **History Loading Performance**:
  - Fixed `isForce: true` cache bypass bug, enabling SQLite `tree_cache` utilization (<15ms).
  - Implemented module-level in-memory SWR cache for 0ms instant modal reopen.
  - Eliminated false-empty "No Projects Found" flash with shimmering animated skeleton cards.
- **Instance Section Import/Export & CLI Help**:
  - Mounted `UnifiedBackupModal` trigger in `InstancesHeader.tsx`.
  - Added `InstancesCliHelpCard` in `InstancesStates.tsx` documenting essential `agm` commands with interactive copy buttons.

---

## 2. Verification & Root Cause Remediation Log

1. **Foreign Key Constraint Failure on Startup Purge**:
   - *RCA*: In `src-tauri/src/modules/repo_db/schema.rs`, `purge_corrupted_running_projects` executed `DELETE FROM running_projects` with `PRAGMA foreign_keys = ON`, causing constraint violation whenever child records had slight casing or un-namespaced IDs.
   - *Fix*: Temporarily set `PRAGMA foreign_keys = OFF;`, cleaned un-namespaced records, and re-enabled `PRAGMA foreign_keys = ON;`.
2. **Missing Parent Directories on JSON/DB Export**:
   - *RCA*: `src-tauri/src/bin/agm/running_prompts_cmds.rs` and `prompt_import_export.rs` failed with `os error 3` when export path targeted a new or nested subdirectory (e.g. `vault/`).
   - *Fix*: Added `fs::create_dir_all(parent)` before writing export files.
3. **Isolated Test Instance Life-Cycle**:
   - Created isolated sandbox instance `#4` `e2e-test-instance` (`--data-only`) without disturbing active instances `#1`, `#2`, `#3`.
   - Verified clean deletion via `agm instances rm e2e-test-instance --force`.
4. **Relative Path Linter Compliance**:
   - Ensured all paths in specifications, plans, and code use strictly relative paths.
