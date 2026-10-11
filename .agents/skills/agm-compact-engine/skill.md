---
name: agm-compact-engine
description: Specialized skill for managing the Antigravity-Manager workspace compaction engine, portable .db bundles, 5-stage workdir resolution ladder, path relativity re-anchoring, and agm compact CLI commands.
---

# AGM Workspace Compaction Engine & Portable Bundle Architecture

Governs workspace snapshotting, portable database packaging (`.db`), work directory resolution, path relativity re-anchoring, and migration workflows in Antigravity-Manager (`src-tauri/src/modules/compact_engine/` and `src-tauri/src/bin/agm/compact_cmds.rs`).

---

## 1. Architectural Overview

The **Compact Engine** allows users to package pinned projects, active conversations, running prompts, and instance configurations into self-contained SQLite bundles. These bundles can be transferred between machines, shared across developer environments, or used for catastrophic recovery.

```
+--------------------------------------------------------------------------------------------------+
|                                    WorkDirResolver Priority Ladder                               |
|  P1: CLI --work-dir <path> -> P2: AGM_WORK_DIR -> P3: gitmap.db -> P4: Git Ancestor -> P5: D:\work|
+-------------------------------------------------+------------------------------------------------+
                                                  |
                                                  v
+--------------------------------------------------------------------------------------------------+
|                                   Export Pipeline (export.rs)                                    |
|   1. Discover Pinned Projects from repo_prompts.db & gitmap.db                                   |
|   2. Compute Relative Paths: to_relative_work_path(abs_path, base_work_dir)                      |
|   3. Snapshot Running Prompts & Active Conversations                                             |
|   4. Bundle into Portable SQLite Database (CompactExportData -> schema v2.0)                    |
+-------------------------------------------------+------------------------------------------------+
                                                  |
                                                  v
+--------------------------------------------------------------------------------------------------+
|                                   Import Pipeline (import.rs)                                    |
|   1. Inspect Metadata: verify schema_version == "2.0" & source_work_dir                          |
|   2. Re-Anchor Paths: from_relative_work_path(rel_path, target_work_dir)                         |
|   3. Merge Pinned Projects & Restore State into Local repo_prompts.db                            |
|   4. Atomic Conflict Resolution: Skip, Overwrite, or Re-Map duplicate project keys (P001)        |
+--------------------------------------------------------------------------------------------------+
```

---

## 2. Core Modules & Key Files

| File Path | Role & Responsibilities |
|---|---|
| `src-tauri/src/modules/compact_engine/mod.rs` | Public facade re-exporting `storage`, `work_dir`, `models`, and roundtrip verification tests. |
| `src-tauri/src/modules/compact_engine/workdir.rs` | `WorkDirResolver` engine implementing the 5-stage priority ladder and cross-platform path normalization. |
| `src-tauri/src/modules/compact_engine/types.rs` | Strongly typed domain structs: `CompactExportData`, `CompactMetadata`, `CompactPinnedProject`, `CompactConversation`. |
| `src-tauri/src/modules/compact_engine/db.rs` | Portable SQLite database driver creating tables (`compact_meta`, `compact_projects`, `compact_prompts`). |
| `src-tauri/src/modules/compact_engine/export.rs` | Workspace export runner packaging projects and prompt states with relative path conversion. |
| `src-tauri/src/modules/compact_engine/import.rs` | Workspace import runner restoring projects, re-anchoring relative paths, and resolving collisions. |
| `src-tauri/src/bin/agm/compact_cmds.rs` | Standalone CLI handlers for `agm compact export`, `agm compact import`, `agm compact inspect`. |

---

## 3. Work Directory 5-Stage Priority Ladder

When resolving the base directory for path relativity and project discovery, `WorkDirResolver::resolve_work_dir` strictly evaluates:

1. **P1 (CLI Argument)**: `--work-dir <path>` or `-w <path>` explicitly passed to the CLI command.
2. **P2 (Environment Variable)**: `AGM_WORK_DIR` defined in current process environment.
3. **P3 (GitMap Master Store)**: Active work directory recorded in `gitmap.db` under `work_directories`.
4. **P4 (Git Ancestor Parent)**: Dynamic upward filesystem walk from `std::env::current_dir()` locating the top-most `.git` folder parent.
5. **P5 (Platform Default)**:
   - Windows: `D:\work` (or `C:\work` if drive D: does not exist).
   - Linux/macOS: `$HOME/work`.

All paths are normalized via `clean_path_str` (forward slashes `/`, trimmed whitespace, trailing slashes removed).

---

## 4. Portability Invariant & Relative Path Conversion

Portable bundles must never hardcode absolute host filesystem paths (e.g. `D:\work\Antigravity-Manager` or `/home/user/work/Antigravity-Manager`).

- **Export Rule**: All paths stored in `CompactPinnedProject.relative_path` and `CompactConversation.relative_path` MUST be relative to the resolved `source_work_dir`:
  ```rust
  let rel_path = to_relative_work_path(&abs_path, &base_work_dir)?;
  ```
- **Import Rule**: On importing a portable bundle onto a new machine with a different workspace root, paths MUST be re-anchored:
  ```rust
  let reanchored = from_relative_work_path(&rel_path, &target_work_dir);
  ```

---

## 5. CLI Interface Reference

```bash
# Export active workspace into a compact database bundle
agm compact export --file ./backup-2026-10-12.db --work-dir D:\work

# Export with JSON envelope output for automated scripts
agm compact export --file ./backup.db --json

# Inspect an existing compact bundle without applying changes
agm compact inspect --file ./backup-2026-10-12.db

# Import bundle with automatic path re-anchoring to current workspace
agm compact import --file ./backup-2026-10-12.db --work-dir D:\new-work

# Import with conflict overwrite policy
agm compact import --file ./backup-2026-10-12.db --overwrite
```
