---
plan: 153-verify-running-prompts-instance-section
subtask: "04"
title: Native Terminal CLI Parity and Headless Verification
domain: cli-rust
status: verified
evidence:
  - "agm running-prompts ls verified to query active running prompts across active workspaces"
  - "agm prompt running verified to enforce dead-process gating parity (0 running when PID is dead)"
  - "agm prompts tree verified to return dual sequence identifiers [AGM:P001 | GM:#1]"
  - "CLI commands verified to emit structured JSON envelopes via --json flag"
verified_invariants:
  - "Invariant I13 (Headless CLI Parity): CLI outputs match GUI instance section data identically"
  - "Invariant I14 (CLI Dead-Process Gating): CLI suppresses running prompt output if instance process is dead"
  - "Invariant I15 (Structured JSON Envelope): All running prompt CLI queries support machine-readable JSON envelopes"
  - "Invariant I16 (Dual Sequence Formatting): Project and conversation nodes render unified AGM and GitMap sequence codes"
citations:
  architecture_spec: 02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md
  root_cause_analysis: 02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md
target_files:
  - src-tauri/src/bin/agm/prompt_cmds.rs
  - src-tauri/src/bin/agm/running_prompts_cmds.rs
  - src-tauri/src/bin/agm/running_backup_cmds.rs
  - src-tauri/src/bin/agm/main.rs
  - src-tauri/src/modules/cli.rs
---

# Subtask 04: Native Terminal CLI Parity and Headless Verification

## 1. Context & Objectives

In accordance with Antigravity-Manager architecture guidelines, every capability exposed in the desktop user interface must maintain strict parity in headless environments and the native terminal CLI binary (`agm`). This subtask verifies that command-line queries for running prompts, active conversations, and project trees adhere to the exact same liveness gating, per-instance isolation, and formatting rules as the GUI instance section.

Key verification objectives:
1. Verify `agm running-prompts ls` and `agm prompts running` command routing and execution.
2. Verify dead-process gating parity: if an instance is not running on the OS, the CLI reports 0 active prompts.
3. Verify JSON envelope output (`--json`) via `CliEnvelope::ok`.
4. Verify project conversation tree rendering in the terminal (`agm prompts tree`) with dual sequence tags (`[AGM:P001 | GM:#1]`).
5. Verify snapshot backup and restore capabilities (`agm running-prompts backup / restore`).

---

## 2. Verified Architectural Invariants

### Invariant I13: Headless CLI & GUI Parity
- **Rule**: CLI queries route to the exact same backend engine functions (`repo_db::list_running_prompts_for_instance`, `repo_db::get_project_conversation_tree_for_instance`, `repo_db::is_prompt_running_for_project`).
- **Implementation**:
  - In `src-tauri/src/bin/agm/prompt_cmds.rs` (lines 235-255): `cmd_prompts_running` checks if the target instance process is alive.
  - In `src-tauri/src/bin/agm/running_prompts_cmds.rs` (lines 125-155): Queries `repo_db::list_all_prompts()` and filters active running states (`running`, `queued`, `dispatched`, `backed_up`).

### Invariant I14: CLI Dead-Process Gating Parity
- **Rule**: If the target instance process is not actively running on the operating system, the CLI must return 0 running prompts, preventing ghost statuses in headless shell scripts.
- **Implementation**:
  - In `src-tauri/src/bin/agm/prompt_cmds.rs` (lines 230-241):
    ```rust
    let is_instance_proc_alive = {
        let (is_alive, _, _) = instance::is_instance_process_running_smart(&resolved_inst);
        is_alive || instance::find_instance_pid(&resolved_inst, None)
            .map(instance::is_pid_alive_targeted)
            .unwrap_or(false)
    };

    let running: Vec<repo_db::ActivePrompt> = if !is_instance_proc_alive {
        Vec::new()
    } else {
        ...
    };
    ```

### Invariant I15: Machine-Readable JSON Envelopes
- **Rule**: Every running prompt CLI command supports `--json`, outputting standard `CliEnvelope` format (`status: "ok"`, `command`, `instance_id`, `data`).
- **Implementation**:
  - In `src-tauri/src/bin/agm/prompt_cmds.rs` (lines 257-259):
    ```rust
    if ctx.json_output {
        CliEnvelope::ok("prompt running", Some(target_instance), running).print_and_exit();
    }
    ```
  - Allows headless daemons, Telegram bots, and CI/CD pipelines to consume running prompt telemetry with zero scraping overhead.

### Invariant I16: Dual Sequence Formatting
- **Rule**: Project and conversation tree displays in the terminal render unified sequence codes containing both internal AGM sequences and GitMap identifiers.
- **Implementation**:
  - In `src-tauri/src/modules/repo_db/tree_node_conv.rs` (lines 102-106): Populates `seq_code: format!("C{:03}", c_seq)` and `gitmap_seq_code: format!("GM:{}", short_id)`.

---

## 3. Technical Evidence & Command Verification

1. **Listing Running Prompts Across Workspaces**:
   - Command: `agm running-prompts ls --limit 5`
   - Output lists active prompts across workspaces with sequence codes, target workspace paths, and truncated snippets.
2. **Instance-Scoped Prompt Status**:
   - Command: `agm prompt running --instance default`
   - Checks host process liveness. When alive, returns running prompts for the default profile; when dead, outputs `Instance 'default': 0 active/running prompts.`
3. **Structured JSON Telemetry**:
   - Command: `agm prompt running --instance default --json`
   - Outputs:
     ```json
     {
       "status": "ok",
       "command": "prompt running",
       "instance_id": "default",
       "data": []
     }
     ```
4. **Snapshot Backup and Restore**:
   - Command: `agm running-prompts backup` snapshots all running prompts into `backup_prompts.db`.
   - Command: `agm running-prompts restore` re-injects saved prompts post-switch, ensuring continuity during automated account rotation.

---

## 4. Target Files & Relative References

- `src-tauri/src/bin/agm/prompt_cmds.rs` — Prompt command dispatcher and `cmd_prompts_running` handler.
- `src-tauri/src/bin/agm/running_prompts_cmds.rs` — Running prompts listing, limit, word count, export/import CLI handlers.
- `src-tauri/src/bin/agm/running_backup_cmds.rs` — Prompt snapshot backup and restoration CLI handlers.
- `src-tauri/src/bin/agm/main.rs` — Root CLI entry point and argument router.
- `src-tauri/src/modules/cli.rs` — CLI context parser and JSON envelope builder (`CliEnvelope`).
- `02-spec/21-app/153-verify-running-prompts-instance-section/01-architecture-spec.md` — Specification reference.
- `02-spec/21-app/153-verify-running-prompts-instance-section/03-root-cause-analysis.md` — Root cause analysis and matrix.
