# Backend & Fleet Specification: Database Reset and Fleet Mesh SSH Node Linkage

**Document ID**: `02-spec/21-app/button-functionality-and-testing-instructions/03-fleet-ssh-and-reset-spec.md`  
**Task Slug**: `button-functionality-and-testing-instructions`  
**Status**: APPROVED  
**Date**: 2026-10-20  

---

## 1. Prompt Tree SQLite Database Reset & Regeneration

### 1.1 Tauri Command: `reset_prompt_tree_db`
- **Location**: `src-tauri/src/commands/cmd_repo_db.rs`, `src-tauri/src/modules/repo_db/`
- **Behavior**:
  1. Closes active prepared statements on `repo_prompts.db`.
  2. Deletes or truncates records for the target instance (or drops table/file if total reset requested).
  3. Re-scans `transcript_full.jsonl` from disk directory (`~/.gemini/antigravity/brain/<conv_id>/.system_generated/logs/transcript_full.jsonl`).
  4. Parses all turns, tools, thoughts, and media references fresh from disk.
  5. Inserts back into SQLite with normalized `created_at` timestamps in strict FIFO order (`ORDER BY created_at ASC, id ASC`).
  6. Returns summary response `{ success: true, count: N, message: "Regenerated N prompt turns from disk" }`.

---

## 2. Fleet Nodes Mesh SSH Account Alignment & GitMap Linkage

### 2.1 Fleet Node Identification
- **Files**:
  - `src-tauri/src/modules/ssh_manager.rs`
  - `src/components/fleet/FleetNodesTable.tsx`
- **Requirements**:
  - Main machine (`win #ssh_main`, `192.168.1.20`) must properly reflect Git link and connected accounts instead of defaulting to `Unassigned`.
  - Check `~/.gitmap/gitmap-ssh-nodes.json` and sync accounts/git links across mesh nodes.
  - Fix any SSH connection or alias mismatches where `main` vs `w1` are inverted or missing Google accounts.
