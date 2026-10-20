# Subtask 04: GitMap Command Alignment & Instance Routing Parity

## 1. Overview & Objectives

Achieve complete functional parity between the Antigravity Manager (`agm`) CLI and the GitMap (`gitmap agy` / `gitmap ag`) command ecosystem:
- `gitmap agy running [--json]`
- `gitmap agy ff [instance] [--json]`
- `gitmap agy backup rp [instance]` / `gitmap agy backup running-prompts`
- `gitmap ag restore rp [instance]` / `gitmap ag restore running-prompts [instance]`

Key Objectives:
1. Ensure `agm agy` in `src-tauri/src/bin/agm/misc_cmds.rs` accurately delegates or handles `running`, `ff`, `backup`, and `restore` subcommands.
2. Ensure command arguments (such as `rp`, `running-prompts`, sequence tokens `#1`, and instance IDs) pass seamlessly between GitMap and AGM without syntax errors.
3. Align exit codes (0 for success, 1 for runtime failure, 2 for caller self-preservation intercept).
4. Preserve identical two-tier JSON envelope outputs when `--json` is specified across both tools.

---

## 2. Files Involved

- `src-tauri/src/bin/agm/misc_cmds.rs`: `cmd_agy` routing and fallback ladder.
- `src-tauri/src/bin/agm/instance_fastforward.rs`: `cmd_fast_forward` execution handler.
- `src-tauri/src/bin/agm/running_backup_cmds.rs`: Multimodal backup and restore handlers.
- `src-tauri/src/bin/agm/running_prompts_cmds.rs`: Running prompts and instance enumerator.
- `src-tauri/src/bin/agm/main.rs`: Top-level CLI routing aliases.

---

## 3. Detailed Implementation Plan

### Step 1: Align `cmd_agy` Subcommand Dispatcher
In `src-tauri/src/bin/agm/misc_cmds.rs`, expand `cmd_agy` to recognize all GitMap shorthand aliases:

```rust
pub(crate) fn cmd_agy(args: &[String]) {
    let sub = args
        .first()
        .map(|s| s.trim_start_matches('/').to_lowercase())
        .unwrap_or_else(|| "help".to_string());
    
    match sub.as_str() {
        "running" | "active" | "running-prompts" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::running_prompts_cmds::cmd_running_prompts(rest);
        }
        "ff" | "fast-forward" | "smart-switch" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::instance_fastforward::cmd_fast_forward(rest);
        }
        "backup" | "backup-running-prompts" | "brp" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::running_backup_cmds::cmd_backup_running_prompts(rest);
        }
        "restore" | "restore-running-prompts" | "rrp" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::running_backup_cmds::cmd_restore_running_prompts(rest);
        }
        // ... existing arms for tree, fpug, sug, rerun, queues, etc.
    }
}
```

### Step 2: Handle GitMap-Specific Argument Shorthands
In `src-tauri/src/bin/agm/running_backup_cmds.rs`:
- Ensure that shorthand arguments like `rp` (`gitmap agy backup rp #1`) and `running-prompts` are stripped as noise tokens before resolving positional instance arguments.
- Enable `gitmap ag restore rp worker-1` to pass `["rp", "worker-1"]` to `cmd_restore_running_prompts`, correctly stripping `rp` and resolving `worker-1`.

### Step 3: Fast-Forward Delegation Alignment
In `src-tauri/src/bin/agm/instance_fastforward.rs`:
- Verify that positional instance arguments passed from `gitmap agy ff [instance]` resolve through `instance::resolve_instance_id`.
- Ensure JSON output conforms to the standardized telemetry schema (`tool_version`, `previous_account`, `selected_account`, `prompts_running`, `prompts_resent`, `has_images`).
- Enforce caller session preservation (`guard_caller_instance`).

### Step 4: Bidirectional Command Fallback
In `misc_cmds.rs`:
- If an `agm agy <subcommand>` is invoked that AGM does not natively handle, fall back to executing `gitmap agy <subcommand>` via `std::process::Command`.
- Capture stdout and stderr cleanly, returning GitMap's exit code without swallowing errors.

---

## 4. Verification & Testing

1. **GitMap Subcommand Routing**:
   - Run `agm agy running --json` and verify instance enumeration and prompt preview.
   - Run `agm agy ff --help` and verify help output for fast-forwarding.
   - Run `agm agy backup rp --help` and verify backup running prompts help output.
   - Run `agm agy restore rp --help` and verify restore running prompts help output.
2. **GitMap Direct Delegation**:
   - Verify `gitmap agy running` executes without error if `gitmap` is on PATH.
   - Verify `gitmap ag restore rp #1` delegates cleanly to AGM restore.
3. **Exit Code & JSON Structure Gate**:
   - Ensure `--json` output returns valid JSON with `records`, `prompts_count`, or `success` keys.
   - Ensure exit code 2 is returned when targeting the caller instance without `--force`.
