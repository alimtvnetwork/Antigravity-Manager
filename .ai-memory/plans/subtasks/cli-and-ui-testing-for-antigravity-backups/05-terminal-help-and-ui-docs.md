# Subtask 05: Terminal Help Segments & UI Documentation

## 1. Objective

Implement comprehensive, self-documenting terminal help segments in `src-tauri/src/bin/agm/help_text.rs`, specialized command help handlers in `running_prompts_cmds.rs` and `running_backup_cmds.rs`, and interactive CLI cheatsheet references in the React UI (`src/pages/Telegram.tsx`, `src/pages/instances/InstancesHeader.tsx`, and `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`).

---

## 2. Prerequisite State & File Invariants

- Core running prompt inspection and backup commands implemented in `src-tauri/src/bin/agm/running_prompts_cmds.rs` and `running_backup_cmds.rs`.
- Specification documented in `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/03-verification-and-help-spec.md`.
- **Strictly Relative Paths Invariant**: All documentation references, code comments, and output templates must strictly use repository-relative paths (e.g., `02-spec/...`, `src/...`, `src-tauri/...`). Absolute paths are strictly forbidden.
- **No Silent Errors Invariant**: All CLI error responses must route through standard exit codes (`1` on failure, `2` on caller safety block) with clear stderr messages.

---

## 3. Detailed Implementation Steps

### Step 5.1: Extend Terminal Help Text in `src-tauri/src/bin/agm/help_text.rs`

1. **Update `print_help()` Human-Readable Banner**:
   In the `PARALLEL PROMPT BACKUP & WORKSPACE RESTORATION` section:
   - Add `agm running [ls] [--words <N>] [--limit <Y>] [--full] [--json] [-i <instance>]`:
     ```text
     running, running-prompts [ls] [--words <N>] [--limit <Y>] [--full] [--json] [-i <id|#seq>]
         List instances (#1, #2) and running prompts with configurable word preview and image counts
     ```
   - Add positional `agm backup running [instance]` and `agm restore running [instance]`:
     ```text
     backup running [instance] [-f <file.db>] [--json]
         Atomic parallel snapshot of active prompts & multimodal images with step-by-step progress
     restore running [instance] [--keep] [-f <file.db>] [--json]
         Restore prompts from split SQLite DB back into workspace queues in strict FIFO stack order
     ```
   - Update `REAL-WORLD EXAMPLES` section with running prompt and backup examples:
     ```text
     agm running --words 200            # Inspect running prompts with 200-word preview
     agm running --json                 # Machine-readable instance and prompt enumeration
     agm backup running #1              # Snapshot running prompts for instance #1 with image tracing
     agm restore running #1             # Re-inject backed up prompts for instance #1
     gitmap agy running --json          # GitMap parity invocation
     gitmap agy backup rp               # GitMap shorthand prompt backup with images
     gitmap ag restore rp #1            # GitMap shorthand prompt restore for instance #1
     ```

2. **Update `print_help_json()` Structured JSON Metadata**:
   Under the `"Parallel Prompt Backup & Workspace Restoration"` command group, add structured entries:
   ```json
   {
     "name": "running",
     "aliases": ["running-prompts", "active"],
     "flags": ["ls", "--words <N>", "--limit <Y>", "--full", "-i <id|#seq>", "--json"],
     "description": "List sandbox instances and running prompts with configurable word preview and image counts"
   },
   {
     "name": "backup running",
     "aliases": ["running backup", "brp"],
     "flags": ["[instance]", "-f <path.db>", "--json"],
     "description": "Atomic snapshot of active prompts with real-time step progress and image tracing"
   },
   {
     "name": "restore running",
     "aliases": ["running restore", "rrp"],
     "flags": ["[instance]", "--keep", "-f <path.db>", "--json"],
     "description": "Restore backed up prompts into active workspace queues in strict FIFO stack order"
   }
   ```

3. **Update `print_commands_table()` Table**:
   Add rows for running prompts and backup/restore:
   ```text
   List Running Prompts         cmd: <node> | agm running             agm running --json
   Backup Running Prompts       cmd: <node> | agm backup running      agm backup running [inst]
   Restore Running Prompts      cmd: <node> | agm restore running     agm restore running [inst]
   ```

---

### Step 5.2: Dedicated Subcommand Help in CLI Handlers

1. **`src-tauri/src/bin/agm/running_prompts_cmds.rs` (`cmd_running_prompts`)**:
   Ensure invocation with `--help`, `-h`, or `help` outputs the complete specification:
   - Command synopsis, parameter options (`--words`, `--limit`, `--full`, `--json`, `-i`).
   - Detailed explanation of dual sequence formatting (`[AGM:P001 | GM:#1]`, `[AGM:C001 | GM:<cid>]`).
   - Examples of filtering by instance `#1` and adjusting preview word limits.

2. **`src-tauri/src/bin/agm/running_backup_cmds.rs` (`cmd_backup_running_prompts`)**:
   Ensure `agm backup running --help` and `agm backup --help` display:
   - Positional instance parameter explanation (e.g., `#1`, `default`, `Worker-2`).
   - Explanation of real-time 4-step progress logging (`[STEP 1/4]` through `[STEP 4/4]`).
   - Multimodal image tracing details (`extract_image_uris_or_paths`, Base64 tokens, local screenshot IDs).

3. **`src-tauri/src/bin/agm/running_backup_cmds.rs` (`cmd_restore_running_prompts`)**:
   Ensure `agm restore running --help` and `agm restore --help` display:
   - Explanation of FIFO order guarantee (`ORDER BY created_at ASC, id ASC`).
   - Explanation of `--keep` flag for preserving backup batch state.
   - Positional instance filter behavior (restoring single instance vs all instances).

---

### Step 5.3: Main Dispatch Routing Verification in `src-tauri/src/bin/agm/main.rs`

1. Audit `main.rs` lines 311–317:
   ```rust
   "active" | "running" => {
       if cmd_args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
           crate::running_prompts_cmds::cmd_running_prompts(&cmd_args);
       } else if cmd_args.iter().any(|a| a == "--json") || !cmd_args.is_empty() {
           crate::running_prompts_cmds::cmd_running_prompts(&cmd_args);
       } else {
           crate::running_prompts_cmds::cmd_running_prompts(&[]);
       }
   }
   ```
2. Verify `backup` and `restore` match arms:
   - If `cmd_args.first()` is `"running"`, forward `&cmd_args[1..]` into `cmd_backup_running_prompts` and `cmd_restore_running_prompts` while stripping the modifier.
   - Ensure `--help` or `-h` anywhere in the arguments triggers the dedicated help screen.

---

### Step 5.4: UI Documentation Links & Fast CLI Cheatsheet Integration

1. **`src/pages/Telegram.tsx` Cheatsheet**:
   Add fast CLI copy buttons for running prompts, backup, and restore:
   - `agm running --json`
   - `agm running --words 200`
   - `agm backup running #1`
   - `agm restore running #1`
   - `gitmap agy running --json`
   - `gitmap agy backup rp`
   - `gitmap ag restore rp`

2. **`src/pages/instances/InstancesHeader.tsx` Documentation**:
   - Add documentation icon button linking to `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/03-verification-and-help-spec.md`.
   - Tooltip explaining running prompts inspection and backup lifecycle.

3. **`src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`**:
   - Add a footer cheatsheet bar displaying:
     `agm running --json` | `agm backup running` | `agm restore running`
   - Click-to-copy handlers with toast notification feedback.

---

## 4. Verification & Testing Protocol

Execute the following commands to confirm help output and formatting:

```bash
# 1. Verify general help includes running, backup running, and restore running
cargo run --bin agm -- help

# 2. Verify machine-readable JSON help includes new commands
cargo run --bin agm -- help --json

# 3. Verify dedicated running prompt help
cargo run --bin agm -- running --help

# 4. Verify dedicated backup running help
cargo run --bin agm -- backup running --help

# 5. Verify dedicated restore running help
cargo run --bin agm -- restore running --help

# 6. Verify GitMap help parity (if gitmap binary available)
gitmap agy running --help
```

---

## 5. Acceptance Criteria Checklist

- [ ] `agm running --help` displays synopsis, options (`--words`, `--limit`, `--full`, `--json`, `-i`), description, and examples.
- [ ] `agm backup running --help` displays positional instance argument, 4-step progress logging description, and image tracing notes.
- [ ] `agm restore running --help` displays FIFO ordering guarantee, `--keep` retention description, and positional instance filtering.
- [ ] `agm help --json` includes structured definitions with aliases, flags, and descriptions.
- [ ] React UI contains interactive click-to-copy cheatsheet items and documentation links.
- [ ] Code compiles cleanly with zero warnings (`cargo clippy --all-targets --all-features`).
- [ ] Frontend builds with zero TypeScript errors (`npm run build`).
- [ ] Strictly relative repository paths used across all files.
