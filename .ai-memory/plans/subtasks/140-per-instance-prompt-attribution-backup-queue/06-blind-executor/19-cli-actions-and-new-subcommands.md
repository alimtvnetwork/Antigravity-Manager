# Step 19: CLI Actions Name Their Instance, Plus `prompt enqueue`, `prompts queue ls`, `prompts trace`

Goal: every `agm` action that writes or sends a prompt (`prompt`, `brp`, `rrp`, `rrc`, `qs`) acts on exactly one instance chosen by the shared rule from step 18, refuses with exit code 2 when the choice is ambiguous, and calls the shared library functions from step 17. Three new subcommands: `agm prompt enqueue`, `agm prompts queue ls`, `agm prompts trace`. Subtask `../03-cli-and-ipc-parity.md` Step 1 (actions), Step 3 (CLI part), Step 4 (CLI part).

Read `00-start-here.md` first. This file never overrides it.

## 1. Depends on

- Step 18 (`select_instance`, `load_registry_or_exit`, `instance_group_header`, `instance::instance_flag_value`, `instance::strip_instance_args`, `InstanceSelection`).
- Step 17 (`enqueue_prompt_for_instance`, `send_prompt_now_for_instance`, `list_queued_prompts_for_instance`, `trace_prompts_for_instance`, `InstancePromptRow`, `backup_prompts_for_instance`).
- Step 12 (`backup_prompts_db::restore_backed_up_prompts_for_instance`, `RestoreReport`).
- Step 13 (`resend_running_commands_for_instance(instance_id: &str, limit: usize)`).
- Step 09 (`repo_owner_instances`).

## 2. Files you may edit

- `src-tauri/src/bin/agm.rs`

Nothing else. Scripts: the only script that calls one of these actions is `scripts/test-instance-e2e.ps1:271` (`& $agm backup-running-prompts --instance $instA`); it already passes an instance, so no script change is needed.

## 3. Find it

GitMap rule: always search `agm.rs` by its full path. A search on `src-tauri/src` or `src-tauri/src/bin` skips it.

| Change | `fn` | Unique search literal | Line hint |
|---|---|---|---|
| A | `fn cmd_prompts(args: &[String]) {` routing | `cmd_prompts_show(&args[1..]);` followed by `first_lower == "backup" \|\| first_lower == "brp"` | `:1839` |
| B | `fn cmd_prompts` help | `println!("  agm prompts status [--json]");` | `:1785` |
| C | insert above `fn cmd_prompt_dispatch(args: &[String]) {` | `fn cmd_prompt_dispatch(` | `:2913` |
| D1 | `fn cmd_prompt_dispatch` routing | `if first_lower == "ls" \|\| first_lower == "list" \|\| first_lower == "--running" {` | `:2955` |
| D2 | `fn cmd_prompt_dispatch` help | `AGM Prompt Dispatch (By AGM Seq ID, Instance, Project, or Node):` | `:2922` |
| D3 | `fn cmd_prompt_dispatch` parse loop | `explicit_seq_or_target = Some(args[i + 1].clone());` | `:3016` |
| D4 | `fn cmd_prompt_dispatch` tail | `// Pull latest changes before dispatching prompt locally` | `:3102` to `:3203` |
| E | `fn cmd_resend_running_commands(args: &[String]) {` | `AGM Resend Running Commands:` | `:3326` |
| F | `fn cmd_queue_scheduler(args: &[String]) {` | `AGM Prompt Queue Scheduler (10-Minute Loop & Audit Trail):` | `:3843` |
| G | `fn cmd_backup_running_prompts(args: &[String]) {` | `AGM Backup Running Prompts (Split SQLite):` | `:3928` |
| H | `fn cmd_restore_running_prompts(args: &[String]) {` | `AGM Restore Running Prompts (Split SQLite):` | `:4119` |

```text
gitmap aum search "cmd_prompts_show\(&args\[1..\]\);" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "agm prompts status \[--json\]" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "fn cmd_prompt_dispatch|fn cmd_rerun\(" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "Pull latest changes before dispatching prompt locally" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "fn cmd_resend_running_commands|fn cmd_queue_scheduler|fn cmd_backup_running_prompts|fn cmd_restore_running_prompts" src-tauri/src/bin/agm.rs --ext .rs
```

`cmd_prompts_show(&args[1..]);` appears twice (in `cmd_prompts` and in `cmd_prompt_dispatch`). Change A edits the one inside `cmd_prompts` only.

Before adding a new name, search for it. If one exists, STOP:

```text
gitmap aum search "fn require_instance|fn action_instance|fn resolve_instance_or_exit|fn prompt_row_json|fn cmd_prompts_queue|fn cmd_prompts_trace|fn cmd_prompt_enqueue" src-tauri/src/bin/agm.rs --ext .rs
```

Confirm these exist (each must return a hit):

```text
gitmap aum search "fn select_instance\(|fn load_registry_or_exit|fn instance_group_header" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "pub fn send_prompt_now_for_instance|pub fn enqueue_prompt_for_instance|pub fn trace_prompts_for_instance|pub fn list_queued_prompts_for_instance" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn backup_prompts_for_instance|pub fn restore_backed_up_prompts_for_instance" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "pub fn resend_running_commands_for_instance" src-tauri/src/modules/repo_db.rs --ext .rs
```

Open `resend_running_commands_for_instance` and confirm its first parameter is `instance_id: &str` (step 13). If it is still `Option<&str>`, STOP.

## 4. Current code

### Change A (`cmd_prompts` routing, `:1839` to `:1843`)

```rust
        if first_lower == "show" {
            cmd_prompts_show(&args[1..]);
            return;
        }
        if first_lower == "backup" || first_lower == "brp" {
```

### Change B (`cmd_prompts` help, two places)

```rust
            println!("  agm prompts status [--json]");
```

```rust
            println!("  show                Display detailed prompt record and full word content");
```

### Change C (insertion point only)

```rust
fn cmd_prompt_dispatch(args: &[String]) {
    if args.is_empty() {
        cmd_prompts(args);
        return;
    }
```

### Change D1 (`cmd_prompt_dispatch` routing, `:2955`)

```rust
        if first_lower == "ls" || first_lower == "list" || first_lower == "--running" {
            cmd_prompts(args);
            return;
        }
```

### Change D2 (`cmd_prompt_dispatch` help, `:2922` to `:2931`)

```rust
            println!("AGM Prompt Dispatch (By AGM Seq ID, Instance, Project, or Node):");
            println!("  agm prompt [C001|P001|project] <text> [--instance <id>] [--node <alias>] [--prefix <cat>] [--suffix <cat>]");
            println!("\nDescription:");
            println!("  Dispatches a prompt to a specific conversation sequence (C001), project sequence (P001),");
            println!("  instance (--instance <id>), or remote SSH node (--node <alias>), with automatic git pull");
            println!("  and optional canonical prompt template framing from 01-prompts/.");
            println!("\nAliases: agm prompt");
            println!("\nOptions:");
            println!("    --instance, -i <id> Target a specific sandbox instance (default: active/default)");
            println!("    --node, -n <alias>  Dispatch prompt to a remote cluster machine via GitMap SSH");
```

### Change D3 (`cmd_prompt_dispatch` parse loop, two places)

```rust
    let mut explicit_seq_or_target: Option<String> = None;
    let mut text_parts: Vec<String> = Vec::new();
```

```rust
        } else if arg == "--seq" || arg == "--conv" || arg == "-c" {
            if i + 1 < args.len() {
                explicit_seq_or_target = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else {
            text_parts.push(arg.clone());
        }
```

### Change D4 (`cmd_prompt_dispatch` tail, `:3102` to `:3203`)

Expected drift, all fine because every line of this block is deleted:

- Step 09 (K2) deleted the `let task_file = PathBuf::from(&cwd_str).join(".antigravity_resume_task.json");` block and added `let _ = repo_db::write_resume_handoff(&active_p, "dispatched");` above the spawn line.
- Step 03 added a `source_dir: ...,` line to the `repo_db::ActivePrompt {` initializer.

Verify the first lines and the last lines, then delete from the line `    // Pull latest changes before dispatching prompt locally` through the closing `}` of the function, directly above `fn cmd_rerun(args: &[String]) {`.

First lines:

```rust
    // Pull latest changes before dispatching prompt locally
    if Path::new(".git").exists() {
        println!("[*] Synchronizing repository via git pull before prompt dispatch...");
        let _ = Command::new("git").args(["pull"]).status();
    }

    let resolved_explicit_inst = explicit_instance
        .as_deref()
        .map(|s| instance::resolve_instance_id(s).unwrap_or_else(|_| s.to_string()));
```

Middle (must be there; this is the old row writer):

```rust
        let _ = conn.execute(
            "INSERT INTO active_prompts \
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, 'gemini-3.8-flash-high', ?6, 'dispatched', ?7, ?7)",
            rusqlite::params![&prompt_id, &slug, &inst_id, &cwd_str, &final_prompt, &session_id, now],
        );
```

Last lines:

```rust
    } else {
        println!(
            "[SUCCESS] Dispatched prompt ({} chars) to workspace '{}' [Instance: {}].",
            final_prompt.len(),
            slug,
            inst_id
        );
    }
}

fn cmd_rerun(args: &[String]) {
```

### Change E (`cmd_resend_running_commands`)

E1 (help, `:3332`):

```rust
        println!("  agm resend-running-commands [N] [--json] [-f <file.json>]");
```

E2 (`:3350` to `:3382`):

Expected drift: none from earlier steps. If line `:3376` already reads `resend_running_commands_for_instance(...)`, that is fine; the block is replaced.

```rust
    let is_json = args.iter().any(|a| a == "--json");
    let mut limit_n = 20usize;
    let mut file_out: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-f" || arg == "--file" {
            if i + 1 < args.len() {
                file_out = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                limit_n = n.max(1);
            }
        }
        i += 1;
    }

    // Step 1: Backup current in-flight prompts from workspaceStorage into SQLite before resend
    let active_inst = instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let _ = repo_db::backup_running_prompts(&active_inst);

    // Step 2: Resend running commands from SQLite DB and write .antigravity_resume_task.json
    let resent_prompts = match repo_db::resend_all_running_commands(limit_n) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERROR] Failed to resend running commands: {}", e);
            std::process::exit(1);
        }
    };
```

E3 (`:3391` to `:3406`):

```rust
        items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": "running",
            "prompt": snippet,
            "word_count": word_count,
            "has_images": has_image,
            "image_paths": img_paths,
            "image_payload": final_img,
            "resent_via": ".antigravity_resume_task.json",
            "resend_status": "success",
            "updated_at": p.updated_at,
        }));
```

### Change F (`cmd_queue_scheduler`)

F1 (help, `:3847` to `:3853`):

```rust
            println!("  agm queue-scheduler [--once] [instance_id]   Check enqueued prompts, verify project idleness, and auto-dispatch FIFO");
            println!("\nOptions:");
            println!("  --once, -o, once         Run a single bookkeeping cycle and exit");
            println!(
                "  [instance_id]            Filter to a specific instance (default: all instances)"
            );
```

F2 (`:3866` to `:3869`):

```rust
    let target_instance = args
        .iter()
        .find(|a| !a.starts_with('-') && *a != "once")
        .map(|s| s.as_str());
```

### Change G (`cmd_backup_running_prompts`)

G1 (help, `:3933`):

```rust
            println!("  agm backup [-file/-f <path.db>] [--json]   Create split SQLite snapshot of active/queued prompts");
```

G2 (`:4034` to `:4116`, from `let is_json` after the `clean` block through the end of the function):

```rust
    let is_json = args.iter().any(|a| a == "--json");
    let mut custom_file: Option<&str> = None;
    let mut target_instance =
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "-f" || args[i] == "--file" || args[i] == "-file") && i + 1 < args.len() {
            custom_file = Some(&args[i + 1]);
            i += 2;
            continue;
        } else if (args[i] == "-i" || args[i] == "--instance" || args[i] == "-instance")
            && i + 1 < args.len()
        {
            let spec = &args[i + 1];
            target_instance = instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.clone());
            i += 2;
            continue;
        } else if args[i].starts_with("--instance=") || args[i].starts_with("-i=") {
            if let Some(spec) = args[i].split('=').nth(1) {
                target_instance =
                    instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.to_string());
            }
        }
        i += 1;
    }

    let _ = repo_db::backup_running_prompts(&target_instance);
    match backup_prompts_db::backup_active_running_prompts(Some(&target_instance), custom_file) {
        Ok((batch, records)) => {
            if is_json {
                let payload = serde_json::json!({
                    "batch_id": batch.id,
                    "file_path": batch.file_path,
                    "prompts_count": records.len(),
                    "created_at": batch.created_at,
                    "records": records,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            println!("\n================================================================================");
            println!("  AGM BACKUP RUNNING PROMPTS (Split SQLite)");
            println!(
                "================================================================================"
            );
            println!(
                "[Successfully secured {} prompt(s) in split SQLite DB: {}]\n",
                records.len(),
                batch.file_path
            );
            if records.is_empty() {
                println!("No active or running prompts found to back up.");
                return;
            }
            println!(
                "{:<5} {:<10} {:<24} {:<10} {:<16} PROMPT SNIPPET",
                "SEQ", "ID", "PROJECT", "STATUS", "IMAGES"
            );
            println!("{}", "-".repeat(110));
            for (idx, r) in records.iter().enumerate() {
                let short_id: String = r.prompt_id.chars().take(8).collect();
                let (snippet, _) = truncate_words(&r.prompt_text, 15);
                let img_label = if r.has_images { "Yes" } else { "None" };
                println!(
                    "#{:<4} {:<10} {:<24} {:<10} {:<16} {}",
                    idx + 1,
                    short_id,
                    r.project_name,
                    r.status,
                    img_label,
                    snippet
                );
            }
            println!("\n[SUCCESS] Backup batch '{}' recorded.", batch.id);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to backup running prompts: {}", e);
            std::process::exit(1);
        }
    }
}
```

### Change H (`cmd_restore_running_prompts`)

H1 (help, `:4123` and `:4131`):

```rust
            println!("  agm restore [--keep/-k] [--json] [-file/-f <path>]");
```

```rust
            println!("    -f, --file      Target custom SQLite database file");
```

H2 (`:4142` to `:4237`, from `let is_json` through the end of the function). Expected drift: the line `let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);` may read `(&target_instance, 20)` or may be gone (steps 12 and 13). That is fine; the block is replaced. Verify the first and last lines:

```rust
    let is_json = args.iter().any(|a| a == "--json");
    let keep_backup = args.iter().any(|a| a == "--keep" || a == "-k");
    let mut custom_file: Option<&str> = None;
    let mut target_instance =
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
```

```rust
    match backup_prompts_db::restore_running_prompts(
        Some(&target_instance),
        keep_backup,
        custom_file,
    ) {
```

```rust
        Err(e) => {
            eprintln!("[ERROR] Failed to restore running prompts: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_running_prompts(args: &[String]) {
```

## 5. New code

### Change A: insert between the `show` block and the `backup` block of `cmd_prompts`

```rust
        if first_lower == "show" {
            cmd_prompts_show(&args[1..]);
            return;
        }
        if first_lower == "queue" {
            cmd_prompts_queue(&args[1..]);
            return;
        }
        if first_lower == "trace" {
            cmd_prompts_trace(&args[1..]);
            return;
        }
        if first_lower == "backup" || first_lower == "brp" {
```

### Change B: help lines

```rust
            println!("  agm prompts status [--json]");
            println!("  agm prompts queue ls -i <id|#seq|name> [--json]");
            println!("  agm prompts trace -i <id|#seq|name> [--conversation <cid>] [--json]");
```

```rust
            println!("  show                Display detailed prompt record and full word content");
            println!("  queue ls            Queued, dispatching and failed prompts of one instance (-i required)");
            println!("  trace               Identity chain (instance, conversation, repo, row, status) per prompt (-i required)");
```

### Change C: insert directly above `fn cmd_prompt_dispatch(args: &[String]) {`

```rust
fn resolve_instance_or_exit(raw: &str) -> String {
    match instance::resolve_instance_input(raw) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            std::process::exit(2);
        }
    }
}

/// One instance for an action: explicit `-i`, else the shared refusal rule (exit 2 when ambiguous).
fn action_instance(args: &[String]) -> (String, String) {
    match select_instance(args, true) {
        instance::InstanceSelection::One { id, name } => (id, name),
        instance::InstanceSelection::All => {
            eprintln!("instance required: pass -i <id|#seq|name>");
            std::process::exit(2);
        }
    }
}

/// Like `action_instance`, but `-i` is mandatory (new subcommands never guess).
fn require_instance(args: &[String], usage: &str) -> (String, String) {
    if instance::instance_flag_value(args).is_none() {
        eprintln!("instance required: pass -i <id|#seq|name>");
        eprintln!("Usage: {}", usage);
        std::process::exit(2);
    }
    action_instance(args)
}

fn prompt_row_json(row: &repo_db::InstancePromptRow) -> serde_json::Value {
    serde_json::json!({
        "row_id": row.prompt.id,
        "instance_id": row.prompt.instance_id,
        "instance_name": row.instance_name,
        "repo_path": row.prompt.repo_path,
        "conversation_id": row.prompt.session_id,
        "project_id": row.prompt.project_id,
        "status": row.prompt.status,
        "status_reason": row.status_reason,
        "attempts": row.attempts,
        "source_dir": row.prompt.source_dir,
    })
}

fn cmd_prompt_enqueue(args: &[String]) {
    let usage = "agm prompt enqueue -i <id|#seq|name> [--repo <path>] [--conversation <cid>] <text> [--json]";
    if args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
        println!("AGM Prompt Enqueue (one instance):");
        println!("  {}", usage);
        println!("\nDescription:");
        println!("  Writes one 'queued' prompt row for exactly this instance. The queue scheduler");
        println!("  (agm qs) sends it when the project is idle. Unknown instances exit with code 2.");
        println!("\nOptions:");
        println!("    -i, --instance <id|#seq|name>  Target instance (required)");
        println!("    --repo, -r <path>              Repository folder (default: current folder)");
        println!("    --conversation, --conv, -c <cid>  Conversation id to continue");
        println!("    --json                         Print the new row as JSON");
        return;
    }

    let (inst_id, inst_name) = require_instance(args, usage);
    let stripped = instance::strip_instance_args(args);
    let mut is_json = false;
    let mut repo_path: Option<String> = None;
    let mut conversation_id: Option<String> = None;
    let mut text_parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < stripped.len() {
        let arg = &stripped[i];
        if arg == "--json" {
            is_json = true;
        } else if (arg == "--repo" || arg == "-r") && i + 1 < stripped.len() {
            repo_path = Some(stripped[i + 1].clone());
            i += 2;
            continue;
        } else if (arg == "--conversation" || arg == "--conv" || arg == "-c")
            && i + 1 < stripped.len()
        {
            conversation_id = Some(stripped[i + 1].clone());
            i += 2;
            continue;
        } else {
            text_parts.push(arg.clone());
        }
        i += 1;
    }

    let repo = repo_path.unwrap_or_else(|| {
        env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
    let text = text_parts.join(" ");

    match repo_db::enqueue_prompt_for_instance(
        &inst_id,
        &repo,
        &text,
        conversation_id.as_deref(),
        "cli",
    ) {
        Ok(row) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&prompt_row_json(&row))
                        .unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            println!(
                "[SUCCESS] Enqueued prompt {} for instance {} ({}) in '{}' (conversation: {}, status: {}).",
                row.prompt.id,
                inst_name,
                row.prompt.instance_id,
                row.prompt.repo_path,
                row.prompt.session_id.as_deref().unwrap_or("-"),
                row.prompt.status
            );
        }
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_prompts_queue(args: &[String]) {
    let usage = "agm prompts queue ls -i <id|#seq|name> [--json]";
    let is_help = args.iter().any(|a| a == "--help" || a == "-h" || a == "help");
    let is_ls = args
        .first()
        .map(|a| a.eq_ignore_ascii_case("ls") || a.eq_ignore_ascii_case("list"))
        .unwrap_or(false);
    if is_help || !is_ls {
        println!("AGM Prompt Queue (one instance):");
        println!("  {}", usage);
        println!("\nDescription:");
        println!("  Lists queued, dispatching and failed prompts of one instance, oldest first.");
        println!("\nOptions:");
        println!("    -i, --instance <id|#seq|name>  Instance to list (required)");
        println!("    --json                         Output rows as JSON");
        return;
    }

    let (inst_id, _) = require_instance(args, usage);
    let is_json = args.iter().any(|a| a == "--json");
    let rows = match repo_db::list_queued_prompts_for_instance(&inst_id) {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            std::process::exit(1);
        }
    };

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    let registry = load_registry_or_exit();
    println!("\n{}", instance_group_header(&registry, &inst_id));
    if rows.is_empty() {
        println!("No queued, dispatching or failed prompts for this instance.");
        return;
    }
    println!(
        "{:<5} {:<46} {:<12} {:<24} {:<6} PROMPT",
        "SEQ", "ROW ID", "STATUS", "REASON", "TRIES"
    );
    println!("{}", "-".repeat(120));
    for (idx, row) in rows.iter().enumerate() {
        let (snippet, _) = truncate_words(&row.prompt.prompt_content, 12);
        println!(
            "#{:<4} {:<46} {:<12} {:<24} {:<6} {}",
            idx + 1,
            row.prompt.id,
            row.prompt.status,
            row.status_reason.as_deref().unwrap_or("-"),
            row.attempts,
            snippet
        );
    }
    println!();
}

fn cmd_prompts_trace(args: &[String]) {
    let usage = "agm prompts trace -i <id|#seq|name> [--conversation <cid>] [--json]";
    if args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
        println!("AGM Prompts Trace (identity chain per prompt):");
        println!("  {}", usage);
        println!("\nDescription:");
        println!("  For every prompt row of one instance prints: source_dir, conversation_id, repo_path,");
        println!("  row_id, status, status_reason, status_history, backup_row_id, handoff{{path, legacy}},");
        println!("  dispatch_result, instance_id, instance_name.");
        println!("\nOptions:");
        println!("    -i, --instance <id|#seq|name>     Instance to trace (required)");
        println!("    --conversation, --conv, -c <cid>  Only this conversation");
        println!("    --json                            Output as JSON");
        return;
    }

    let (inst_id, _) = require_instance(args, usage);
    let is_json = args.iter().any(|a| a == "--json");
    let stripped = instance::strip_instance_args(args);
    let mut conversation_id: Option<String> = None;
    let mut i = 0;
    while i < stripped.len() {
        let arg = &stripped[i];
        if (arg == "--conversation" || arg == "--conv" || arg == "-c") && i + 1 < stripped.len() {
            conversation_id = Some(stripped[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    let traces = match repo_db::trace_prompts_for_instance(&inst_id, conversation_id.as_deref()) {
        Ok(traces) => traces,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            std::process::exit(1);
        }
    };

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&traces).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    let registry = load_registry_or_exit();
    println!("\n{}", instance_group_header(&registry, &inst_id));
    if traces.is_empty() {
        println!("No prompt rows found for this instance.");
        return;
    }
    for t in &traces {
        println!("row_id:          {}", t.row_id);
        println!(
            "  status:        {} ({})",
            t.status,
            t.status_reason.as_deref().unwrap_or("-")
        );
        println!(
            "  conversation:  {}",
            t.conversation_id.as_deref().unwrap_or("-")
        );
        println!("  repo_path:     {}", t.repo_path);
        println!("  source_dir:    {}", t.source_dir.as_deref().unwrap_or("-"));
        println!(
            "  backup_row_id: {}",
            t.backup_row_id.as_deref().unwrap_or("-")
        );
        println!("  handoff:       {} ({})", t.handoff.path, t.handoff.legacy);
        println!(
            "  dispatch:      {}",
            t.dispatch_result.as_deref().unwrap_or("-")
        );
        println!("  attempts:      {}", t.attempts);
    }
    println!();
}
```

### Change D1: insert directly above the `ls` routing block of `cmd_prompt_dispatch`

```rust
        if first_lower == "enqueue" {
            cmd_prompt_enqueue(&args[1..]);
            return;
        }
        if first_lower == "ls" || first_lower == "list" || first_lower == "--running" {
            cmd_prompts(args);
            return;
        }
```

### Change D2: the help block becomes

```rust
            println!("AGM Prompt Dispatch (By AGM Seq ID, Instance, Project, or Node):");
            println!("  agm prompt [C001|P001|project] <text> [-i <id|#seq|name>] [--node <alias>] [--prefix <cat>] [--suffix <cat>] [--json]");
            println!("  agm prompt enqueue -i <id|#seq|name> [--repo <path>] [--conversation <cid>] <text> [--json]");
            println!("\nDescription:");
            println!("  Dispatches a prompt to a specific conversation sequence (C001), project sequence (P001),");
            println!("  instance (-i <id|#seq|name>), or remote SSH node (--node <alias>), with automatic git pull");
            println!("  and optional canonical prompt template framing from 01-prompts/.");
            println!("  Without -i: the sequence's instance, else the only instance that owns this folder,");
            println!("  else the only running instance; two or more running instances exit with code 2.");
            println!("\nAliases: agm prompt");
            println!("\nOptions:");
            println!("    -i, --instance <id|#seq|name>  Target instance");
            println!("    --node, -n <alias>  Remote SSH node via GitMap cluster (-n is not the instance; use -i)");
            println!("    --json              Print the prompt row as JSON");
```

Leave the `--prefix`, `--suffix` and example lines after it unchanged.

### Change D3: the two places become

```rust
    let mut explicit_seq_or_target: Option<String> = None;
    let mut text_parts: Vec<String> = Vec::new();
    let mut is_json = false;
```

```rust
        } else if arg == "--seq" || arg == "--conv" || arg == "-c" {
            if i + 1 < args.len() {
                explicit_seq_or_target = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if arg == "--json" {
            is_json = true;
        } else {
            text_parts.push(arg.clone());
        }
```

### Change D4: the new tail of `cmd_prompt_dispatch` (replaces everything from the `// Pull latest changes` comment to the end of the function)

The instance is resolved before `git pull`, so a refusal (exit 2) never touches the working tree.

```rust
    if instance::instance_flag_value(args).is_some_and(|v| v.is_empty()) {
        eprintln!("instance required: -i needs a value (<id|#seq|name>)");
        std::process::exit(2);
    }

    let cwd_str = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let explicit_inst = explicit_instance.as_deref().map(resolve_instance_or_exit);

    let (repo_path, inst_id, conversation_id, seq_label) = if let Some(seq) = resolved_seq {
        let seq_inst = match instance::canonical_instance_id(&seq.instance_id) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "[ERROR] Sequence {} has no valid instance: {}",
                    seq.seq_code, e
                );
                std::process::exit(2);
            }
        };
        if let Some(explicit) = explicit_inst.as_deref() {
            if explicit != seq_inst {
                eprintln!(
                    "[ERROR] Sequence {} belongs to instance {}; remove -i or pass -i {}",
                    seq.seq_code, seq_inst, seq_inst
                );
                std::process::exit(2);
            }
        }
        (
            seq.repo_path,
            seq_inst,
            seq.conversation_id,
            Some(seq.seq_code),
        )
    } else {
        let inst = match explicit_inst {
            Some(id) => id,
            None => {
                let owners = repo_db::repo_owner_instances(&cwd_str);
                match owners.as_slice() {
                    [only] => only.clone(),
                    _ => match select_instance(&[], true) {
                        instance::InstanceSelection::One { id, .. } => id,
                        instance::InstanceSelection::All => {
                            eprintln!("instance required: pass -i <id|#seq|name>");
                            std::process::exit(2);
                        }
                    },
                }
            }
        };
        (cwd_str.clone(), inst, None, None)
    };

    // Pull latest changes before dispatching prompt locally
    if Path::new(".git").exists() {
        println!("[*] Synchronizing repository via git pull before prompt dispatch...");
        let _ = Command::new("git").args(["pull"]).status();
    }

    let row = match repo_db::send_prompt_now_for_instance(
        &inst_id,
        &repo_path,
        &final_prompt,
        conversation_id.as_deref(),
        "cli",
    ) {
        Ok(row) => row,
        Err(e) => {
            eprintln!("[ERROR] {}", e);
            std::process::exit(1);
        }
    };
    let is_failed = row.prompt.status == "failed";

    if is_json {
        let mut payload = prompt_row_json(&row);
        payload["agm_seq_id"] = serde_json::json!(seq_label);
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        );
    } else if is_failed {
        eprintln!(
            "[ERROR] Prompt {} for instance {} ({}) failed: {}",
            row.prompt.id,
            row.instance_name,
            row.prompt.instance_id,
            row.status_reason.as_deref().unwrap_or("unknown")
        );
    } else if let Some(seq_code) = seq_label.as_deref() {
        println!(
            "[SUCCESS] Dispatched prompt ({} chars) to AGM Seq [{}] -> '{}' (conv: {}, instance: {} {}, status: {}).",
            final_prompt.len(),
            seq_code,
            row.prompt.repo_path,
            row.prompt.session_id.as_deref().unwrap_or("-"),
            row.instance_name,
            row.prompt.instance_id,
            row.prompt.status
        );
    } else {
        println!(
            "[SUCCESS] Dispatched prompt ({} chars) to '{}' [Instance: {} {}] (status: {}).",
            final_prompt.len(),
            row.prompt.repo_path,
            row.instance_name,
            row.prompt.instance_id,
            row.prompt.status
        );
    }

    if is_failed {
        std::process::exit(1);
    }
}
```

After pasting, confirm the function no longer contains `derive_current_repo_slug`, `get_active_instance_id`, `resolve_instance_id`, `INSERT INTO active_prompts` or `spawn_prompt_via_agy`. `derive_current_repo_slug` is still used by other functions; do not delete it.

### Change E (`cmd_resend_running_commands`)

E1 becomes:

```rust
        println!("  agm resend-running-commands [N] [-i <id|#seq|name>] [--json] [-f <file.json>]");
        println!("  Without -i: the only running instance; two or more running instances exit with code 2.");
```

E2 becomes:

```rust
    let (inst_id, inst_name) = action_instance(args);
    let stripped_args = instance::strip_instance_args(args);
    let args = stripped_args.as_slice();
    let is_json = args.iter().any(|a| a == "--json");
    let mut limit_n = 20usize;
    let mut file_out: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-f" || arg == "--file" {
            if i + 1 < args.len() {
                file_out = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                limit_n = n.max(1);
            }
        }
        i += 1;
    }

    if let Err(e) = backup_prompts_db::backup_prompts_for_instance(&inst_id, None) {
        eprintln!(
            "[WARN] Backup before resend failed for instance {} ({}): {}",
            inst_name, inst_id, e
        );
    }

    let resent_prompts = match repo_db::resend_running_commands_for_instance(&inst_id, limit_n) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERROR] Failed to resend running commands: {}", e);
            std::process::exit(1);
        }
    };
```

E3 becomes:

```rust
        items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project": p.project_id,
            "instance_id": p.instance_id,
            "instance_name": inst_name,
            "repo_path": p.repo_path,
            "status": p.status,
            "prompt": snippet,
            "word_count": word_count,
            "has_images": has_image,
            "image_paths": img_paths,
            "image_payload": final_img,
            "resent_via": "active_prompts",
            "resend_status": "success",
            "updated_at": p.updated_at,
        }));
```

### Change F (`cmd_queue_scheduler`)

F1 becomes:

```rust
            println!("  agm queue-scheduler [--once] [-i <id|#seq|name> | instance_id]   Check enqueued prompts, verify project idleness, and auto-dispatch FIFO");
            println!("\nOptions:");
            println!("  --once, -o, once         Run a single bookkeeping cycle and exit");
            println!(
                "  -i, --instance <id|#seq|name> | [instance_id]  Filter to one instance (default: all instances)"
            );
```

F2 becomes:

```rust
    let raw_target = instance::instance_flag_value(args).or_else(|| {
        instance::strip_instance_args(args)
            .into_iter()
            .find(|a| !a.starts_with('-') && a != "once")
    });
    let target_id = raw_target.as_deref().map(resolve_instance_or_exit);
    let target_instance = target_id.as_deref();
```

The scheduler has no refusal rule: without an instance it serves all instances, as before.

### Change G (`cmd_backup_running_prompts`)

G1 becomes:

```rust
            println!("  agm backup [-i <id|#seq|name>] [-file/-f <path.db>] [--json]   Create split SQLite snapshot of one instance");
            println!("  Without -i: the only running instance; two or more running instances exit with code 2.");
```

G2 becomes:

```rust
    let (inst_id, _) = action_instance(args);
    let stripped_args = instance::strip_instance_args(args);
    let args = stripped_args.as_slice();
    let is_json = args.iter().any(|a| a == "--json");
    let mut custom_file: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "-f" || args[i] == "--file" || args[i] == "-file") && i + 1 < args.len() {
            custom_file = Some(&args[i + 1]);
            i += 2;
            continue;
        }
        i += 1;
    }

    match backup_prompts_db::backup_prompts_for_instance(&inst_id, custom_file) {
        Ok(report) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            println!("\n================================================================================");
            println!("  AGM BACKUP RUNNING PROMPTS (Split SQLite)");
            println!(
                "================================================================================"
            );
            println!(
                "[Instance {}: secured {} prompt(s) in split SQLite DB: {}]\n",
                report.instance_id,
                report.records.len(),
                report.file_path
            );
            if report.records.is_empty() {
                println!("No active or running prompts found to back up.");
                return;
            }
            println!(
                "{:<5} {:<10} {:<24} {:<10} {:<16} PROMPT SNIPPET",
                "SEQ", "ID", "PROJECT", "STATUS", "IMAGES"
            );
            println!("{}", "-".repeat(110));
            for (idx, r) in report.records.iter().enumerate() {
                let short_id: String = r.prompt_id.chars().take(8).collect();
                let (snippet, _) = truncate_words(&r.prompt_text, 15);
                let img_label = if r.has_images { "Yes" } else { "None" };
                println!(
                    "#{:<4} {:<10} {:<24} {:<10} {:<16} {}",
                    idx + 1,
                    short_id,
                    r.project_name,
                    r.status,
                    img_label,
                    snippet
                );
            }
            println!("\n[SUCCESS] Backup batch '{}' recorded.", report.batch_id);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to backup running prompts: {}", e);
            std::process::exit(1);
        }
    }
}
```

### Change H (`cmd_restore_running_prompts`)

H1 becomes:

```rust
            println!("  agm restore [-i <id|#seq|name>] [--keep/-k] [--json] [-file/-f <path>]");
            println!("  Without -i: the only running instance; two or more running instances exit with code 2.");
```

```rust
            println!("    -f, --file      Target custom SQLite database file");
            println!("    -i, --instance  Instance to restore (id, #seq or name)");
```

H2 becomes:

```rust
    let (inst_id, _) = action_instance(args);
    let stripped_args = instance::strip_instance_args(args);
    let args = stripped_args.as_slice();
    let is_json = args.iter().any(|a| a == "--json");
    let keep_backup = args.iter().any(|a| a == "--keep" || a == "-k");
    let mut custom_file: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "-f" || args[i] == "--file" || args[i] == "-file") && i + 1 < args.len() {
            custom_file = Some(&args[i + 1]);
            i += 2;
            continue;
        }
        i += 1;
    }

    match backup_prompts_db::restore_backed_up_prompts_for_instance(
        &inst_id,
        keep_backup,
        custom_file,
    ) {
        Ok(report) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            println!("\n================================================================================");
            println!("  AGM RESTORE RUNNING PROMPTS");
            println!(
                "================================================================================"
            );
            println!("  Instance:              {}", report.instance_id);
            println!("  Restored from backup:  {}", report.restored_from_backup);
            println!("  Dispatched:            {}", report.dispatched);
            println!("  Failed:                {}", report.failed);
            println!("  Skipped:               {}", report.skipped);
            if let Some(reason) = report.skipped_reason.as_deref() {
                println!("  Skipped reason:        {}", reason);
            }
            println!();
            if keep_backup {
                println!("[INFO] Backup records preserved as unrestored (--keep specified).");
            } else {
                println!("[INFO] Marked records as restored. Will be automatically cleaned up after 1 day.");
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to restore running prompts: {}", e);
            std::process::exit(1);
        }
    }
}
```

If the compiler says `RestoreReport` has no field `skipped_reason`, delete the three lines of the `if let Some(reason)` block only (step 12 defines it, so this should not happen).

## 6. Tests

No new unit tests. `src-tauri/Cargo.toml` declares the `agm` binary with `test = false`, and `agm.rs` has no test module, so `cargo test --lib` cannot reach this code. The behavior is covered as follows:

- The refusal rule (`select_instance_refuses_action_when_two_running`, `select_instance_uses_only_running_instance`, `select_instance_list_without_flag_is_all`) is tested in step 18 on the pure core this step calls.
- Row shape, unknown instance, and repo-missing behavior are tested in step 17 on the shared cores this step calls.
- The binary wiring is verified live in step 22 (cases E2E-08, E2E-16, E2E-17, E2E-18).

The gate re-runs the step 18 tests as a regression check and compiles the binary through `clippy --all-targets`.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 select_instance_; cd ..
```

Smoke check (no side effects; prints help only):

```text
cd src-tauri; cargo build --bin agm; cd ..
.\src-tauri\target\debug\agm.exe prompts trace --help
.\src-tauri\target\debug\agm.exe prompt enqueue --help
.\src-tauri\target\debug\agm.exe prompts queue ls --help
```

The first must print a line containing `agm prompts trace` on stdout (step 22's build detection depends on it). Do not run any action command here.

## 8. Commit

```text
Feature: prompts - agm prompt enqueue, queue ls, trace, -i refusal
```

## 9. Done when

- [ ] `agm prompt enqueue`, `agm prompts queue ls` and `agm prompts trace` are routed and print help.
- [ ] `cmd_prompt_dispatch` calls `repo_db::send_prompt_now_for_instance` and contains no `INSERT INTO active_prompts`, `spawn_prompt_via_agy`, `get_active_instance_id` or `resolve_instance_id`.
- [ ] `gitmap aum search "get_active_instance_id" src-tauri/src/bin/agm.rs --ext .rs` shows no hit inside `cmd_resend_running_commands`, `cmd_backup_running_prompts` or `cmd_restore_running_prompts`.
- [ ] `brp`, `rrp` and `rrc` call `action_instance(args)`; `qs` resolves `-i` or the positional instance with `resolve_instance_or_exit`.
- [ ] The `prompt` help says `-n` is the remote node, not the instance.
- [ ] The gate exits 0 and `agm prompts trace --help` prints `agm prompts trace`.
- [ ] The gate references no bin tests: `src-tauri/Cargo.toml` sets `test = false` for both bins (`agm-alim` and `agm`), so the gate uses only `cargo test --lib` filters plus the `cargo build --bin agm` / `--help` smoke check.
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. This step edits only `agm.rs` and changes no library signature, field, visibility or return type. Every library function it calls (`send_prompt_now_for_instance`, `enqueue_prompt_for_instance`, `list_queued_prompts_for_instance`, `trace_prompts_for_instance`, `backup_prompts_for_instance`, `restore_backed_up_prompts_for_instance`) is `pub` in its defining step. The deleted `ActivePrompt` initializer near `agm.rs:3171` leaves no dangling caller.
- [ ] Only `src-tauri/src/bin/agm.rs` is staged.
