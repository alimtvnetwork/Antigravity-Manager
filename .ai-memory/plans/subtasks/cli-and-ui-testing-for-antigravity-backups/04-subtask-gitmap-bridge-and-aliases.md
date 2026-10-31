# Subtask 04: GitMap Bridge Parity and Command Aliases (`gitmap agy`)

> **Subtask File:** `.ai-memory/plans/subtasks/cli-and-ui-testing-for-antigravity-backups/04-subtask-gitmap-bridge-and-aliases.md`  
> **Parent Plan:** `.ai-memory/plans/pending/cli-and-ui-testing-for-antigravity-backups.md`  
> **Spec Reference:** [02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/02-component-spec.md](../../../../02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/02-component-spec.md)  
> **State:** `[QUEUED]`  
> **Assignee:** Backend CLI Engineer  

---

## 1. Objective & Scope

Establish bidirectional command execution parity between the GitMap terminal companion (`gitmap agy`) and Antigravity-Manager (`agm`):
1. Bridge `gitmap agy running [--json]` to native running prompt and instance inspection.
2. Bridge `gitmap agy ff` (fast-forward) to AGM smart account rotation without infinite process spawning loops.
3. Bridge `gitmap agy backup running-prompts` (and shorthand `backup rp`) to the 4-step multimodal backup engine.
4. Bridge `gitmap ag restore running-prompts` (and shorthand `restore rp [$instance]`) to the 4-step multimodal restoration engine.
5. Synchronize terminal help text and command reference tables across AGM and GitMap.

---

## 2. Target Files & Symbols

| File | Type of Change | Key Symbols / Locations |
| :--- | :--- | :--- |
| `src-tauri/src/bin/agm/misc_cmds.rs` | Function Modification | `pub(crate) fn cmd_agy(args: &[String])` |
| `src-tauri/src/bin/agm/help_text.rs` | Documentation Update | `pub(crate) fn print_commands_table()` |

---

## 3. Implementation Steps

### Step 3.1: Enhance `cmd_agy` Dispatcher in `src-tauri/src/bin/agm/misc_cmds.rs`
1. Open `src-tauri/src/bin/agm/misc_cmds.rs`.
2. Inspect `pub(crate) fn cmd_agy(args: &[String])`.
3. Normalize the incoming subcommand:
   ```rust
   let sub = args
       .first()
       .map(|s| s.trim_start_matches('/').trim_start_matches('-').to_lowercase())
       .unwrap_or_else(|| "help".to_string());
   ```

### Step 3.2: Implement `running` Inspection Bridge
Add or update the `"active" | "running"` arm:
```rust
"active" | "running" => {
    let rest = if args.len() > 1 { &args[1..] } else { &[] };
    crate::running_prompts_cmds::cmd_running_inspection(rest);
}
```
*Requirement:* When `agm agy running --json` or `gitmap agy running --json` is invoked, it outputs pure valid JSON containing active instances and prompt counts without printing extraneous banners.

### Step 3.3: Implement `ff` / Fast-Forward Bridge
Add the `"ff" | "fast-forward" | "fastforward" | "smart-switch"` arm:
```rust
"ff" | "fast-forward" | "fastforward" | "smart-switch" => {
    let rest = if args.len() > 1 { &args[1..] } else { &[] };
    crate::instance_fastforward::cmd_fast_forward(rest);
}
```
*Safety Invariant:* This arm directly invokes `cmd_fast_forward` in-process. It MUST NOT shell out to `gitmap agy ff`, which would create a recursive loop.

### Step 3.4: Implement `backup` Running Prompts (`rp`) Bridge
Add and normalize the backup arm to support multi-token GitMap idioms:
```rust
"backup" | "backup-running-prompts" | "brp" => {
    let rest = if args.len() > 1 { &args[1..] } else { &[] };
    let normalized_args: Vec<String> = if let Some(first) = rest.first() {
        let f_low = first.to_lowercase();
        if f_low == "running-prompts" || f_low == "running" || f_low == "rp" {
            rest[1..].to_vec()
        } else {
            rest.to_vec()
        }
    } else {
        rest.to_vec()
    };
    crate::running_backup_cmds::cmd_backup_running_prompts(&normalized_args);
}
```
*Supported Invocations:*
- `agm agy backup`
- `agm agy backup running-prompts`
- `agm agy backup rp`
- `gitmap agy backup rp`

### Step 3.5: Implement `restore` Running Prompts (`rp`) Bridge
Add and normalize the restore arm:
```rust
"restore" | "restore-running-prompts" | "rrp" => {
    let rest = if args.len() > 1 { &args[1..] } else { &[] };
    let normalized_args: Vec<String> = if let Some(first) = rest.first() {
        let f_low = first.to_lowercase();
        if f_low == "running-prompts" || f_low == "running" || f_low == "rp" {
            rest[1..].to_vec()
        } else {
            rest.to_vec()
        }
    } else {
        rest.to_vec()
    };
    crate::running_backup_cmds::cmd_restore_running_prompts(&normalized_args);
}
```
*Supported Invocations:*
- `agm agy restore`
- `agm agy restore running-prompts`
- `agm agy restore rp [instance]`
- `gitmap agy restore rp [instance]`

### Step 3.6: Update Help Documentation in `misc_cmds.rs`
Update the `println!` help text inside `cmd_agy`:
```rust
println!("  agm agy running [--json]        Inspect active instances & running prompts");
println!("  agm agy ff [instance]           Fast-forward rotate instance to freshest account");
println!("  agm agy backup rp [instance]    4-step multimodal running prompt backup");
println!("  agm agy restore rp [instance]   4-step multimodal running prompt restore");
```

---

## 4. Verification & Quality Gates

Run the following commands to verify implementation:

1. **Compilation Check:**
   ```bash
   cd src-tauri && cargo clippy --bin agm
   ```

2. **GitMap Running Inspection Parity:**
   ```bash
   agm agy running
   agm agy running --json
   ```
   *Expected:* Output mirrors `agm running` without errors.

3. **GitMap Fast-Forward Help:**
   ```bash
   agm agy ff --help
   ```
   *Expected:* Displays Fast-Forward help banner instantly without recursive process execution.

4. **GitMap Running Prompts Backup & Restore Aliases:**
   ```bash
   agm agy backup rp default-copy-5597
   agm agy restore rp default-copy-5597
   ```
   *Expected:* Successfully triggers 4-step progress logging identically to native `agm backup running`.

---
Maintained by @aukgit
