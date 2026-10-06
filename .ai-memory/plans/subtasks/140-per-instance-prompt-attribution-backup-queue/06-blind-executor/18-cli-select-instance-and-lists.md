# Step 18: CLI Instance Selection and `-i` on List Commands

Goal: one way to pick the instance in every `agm` prompt command, and `-i <id|#seq|name>` on every list command (`wpr`, `prompts ls`, `running-prompts ls`, `running-projects`, `tree`). Without `-i` a list command shows all instances, grouped by instance, and every row carries `instance_id` and `instance_name`. Lists join prompts to projects by `instance_id` plus normalized path, never by substring. Subtask `../03-cli-and-ipc-parity.md` Step 1 and Step 2.

Read `00-start-here.md` first. This file never overrides it.

## 1. Depends on

- Step 17 (`list_running_projects(&InstanceScope)`, `instance_name_in`, `instance_short_label_in`).
- Step 01 (`InstanceScope`, `canonical_instance_id_in`, `resolve_instance_input_in`, `default_instance_id_in`).
- Step 08 (`normalize_path_for_compare` is `pub(crate)`).

## 2. Files you may edit

- `src-tauri/src/modules/instance.rs`
- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/bin/agm.rs`

## 3. Find it

Important GitMap rule: a search on the folder `src-tauri/src` or `src-tauri/src/bin` does NOT search `agm.rs`. Always search `agm.rs` by its full path, exactly as below.

| Change | File | `fn` signature or place | Unique search literal | Line hint |
|---|---|---|---|---|
| A | `instance.rs` | insert above the `///` line of `pub fn resolve_instance_id(specifier: &str)` | `Resolve an instance query string` | moves after steps 01 and 17 |
| B | `repo_db.rs` | `pub(crate) fn normalize_path_for_compare(p: &str) -> String {` | `fn normalize_path_for_compare(p: &str)` | `:2473` |
| C | `repo_db.rs` | `pub fn format_tree_view_cli(max_words: usize, only_running: bool) -> String {` | `Format the Project -> Conversation -> 200-Word Prompt Tree View` | `:5135` |
| D | `agm.rs` | insert above `fn cmd_which_prompts_running(args: &[String]) {` | `fn cmd_which_prompts_running(` | `:1597` |
| E | `agm.rs` | `fn cmd_which_prompts_running(args: &[String]) {` (whole function) | `AGM Which Prompts Running:` | `:1597` to `:1771` |
| F | `agm.rs` | `fn cmd_prompts(args: &[String]) {` (four sub-blocks) | `AGM Prompts Management & SQLite Caching:` | `:1773` |
| G | `agm.rs` | `fn cmd_running_prompts(args: &[String]) {` (five sub-blocks) | `AGM Running Prompts Management:` | `:4240` |
| H | `agm.rs` | `fn cmd_running_projects(args: &[String]) {` (whole function) | `AGM Running Projects Management:` | `:4587` to `:4740` |
| I | `agm.rs` | `fn cmd_tree(args: &[String]) {` (two sub-blocks) | `AGM Project → Conversation → 200-Word Prompt Tree View:` | `:7019` |
| T1 | `instance.rs` | `mod tests` after `    use super::*;` | `fn switch_spares_another_instances_pid()` | `:6612` |
| T2 | `repo_db.rs` | `mod tests` after `    use super::*;` | `fn test_uri_decoding()` | `:5879` |

```text
gitmap aum search "Resolve an instance query string" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "fn normalize_path_for_compare" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn format_tree_view_cli" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fn cmd_which_prompts_running|fn cmd_prompts\(|fn cmd_running_prompts\(|fn cmd_running_projects\(|fn cmd_tree\(" src-tauri/src/bin/agm.rs --ext .rs
```

Before adding any new name, search for it. If one already exists, STOP:

```text
gitmap aum search "InstanceSelection|InstancePick|instance_flag_value|strip_instance_args|select_instance_core" src-tauri/src --ext .rs
gitmap aum search "workspace_uris_contain_repo|format_tree_view_cli_for" src-tauri/src --ext .rs
gitmap aum search "fn load_registry_or_exit|fn select_instance|fn selection_scope|fn is_selected_instance|fn instance_label|fn instance_group_header|fn conversation_for_project|fn is_live_prompt_status" src-tauri/src/bin/agm.rs --ext .rs
```

Confirm these exist (each must return a hit):

```text
gitmap aum search "pub fn resolve_instance_input_in" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "pub fn default_instance_id_in" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "pub fn instance_short_label_in" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "list_running_projects\(&instance::InstanceScope::All\)" src-tauri/src/bin/agm.rs --ext .rs
```

## 4. Current code

### Change A (`instance.rs`, insertion point only)

```rust
/// Resolve an instance query string (seq_num like "1", ID like "inst-xyz", name like "Instance 1", or "default"/"active")
/// to a valid concrete instance ID.
pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {
```

Step 17 inserted `instance_name_in`, `instance_seq_in` and `instance_short_label_in` directly above it. Insert this step's block between `instance_short_label_in` and the `///` line.

### Change B (`repo_db.rs:2472`)

```rust
/// Normalize filesystem path for reliable cross-platform comparison
pub(crate) fn normalize_path_for_compare(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}
```

### Change C (`repo_db.rs:5135`, first four lines of the function; leave the rest unchanged)

```rust
/// Format the Project -> Conversation -> 200-Word Prompt Tree View for AGM CLI output
pub fn format_tree_view_cli(max_words: usize, only_running: bool) -> String {
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let tree = get_project_conversation_tree(word_cap, only_running);
    let mut out = String::new();
```

### Change D (`agm.rs`, insertion point only)

```rust
fn cmd_which_prompts_running(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Which Prompts Running:");
```

### Change E (`agm.rs:1597` to `:1771`, whole function)

The whole function is replaced. Verify these sub-blocks; whitespace differences are fine. Expected drift: step 17 changed line `:1629` to `repo_db::list_running_projects(&instance::InstanceScope::All)`.

```rust
    let is_json = args.iter().any(|a| a == "--json");

    // Refresh live projects across registered instances
    if let Ok(reg) = instance::load_registry() {
        for inst in &reg.instances {
            let _ = repo_db::detect_running_projects(&inst.id);
        }
    }

    let projects = repo_db::list_running_projects().unwrap_or_default();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let conversations = agy_cleaner::scan_conversations(100);
```

```rust
            .filter(|p| {
                (p.project_id == proj.id || p.repo_path.eq_ignore_ascii_case(&proj.repo_path))
                    && (p.status == "running"
                        || p.status == "backed_up"
                        || p.status == "dispatched")
            })
```

```rust
        let repo_norm = proj.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || (!proj.repo_name.is_empty()
                    && uris_norm.contains(&proj.repo_name.to_lowercase()))
        });
```

The function ends with:

```rust
        println!(
            "#{:<4} {:<22} {:<24} {:<16} {:<26} {}",
            s, proj, id, cid, cname, qcount
        );
    }
    println!();
}

fn cmd_prompts(args: &[String]) {
```

Delete from the line `fn cmd_which_prompts_running(args: &[String]) {` through the closing `}` directly above `fn cmd_prompts(args: &[String]) {`.

### Change F (`agm.rs`, `cmd_prompts`, four sub-blocks; leave the rest unchanged)

F1 (help usage line, `:1778`):

```rust
            println!("  agm prompts [ls] [N] [--words <W>] [--running] [--json]");
```

F2 (`:1877`):

```rust
    let is_ls = args
        .first()
        .map(|a| a.eq_ignore_ascii_case("ls") || a.eq_ignore_ascii_case("list"))
        .unwrap_or(false);
    let is_json = args.iter().any(|a| a == "--json");
    let running_only = args.iter().any(|a| a == "--running") || is_ls;
```

F3 (`:1908`):

```rust
    let mut all_prompts = repo_db::list_all_prompts().unwrap_or_default();
```

F4 (`:1942` to `:2001`, from `let mut stack_items = Vec::new();` through the end of the table block):

```rust
    let mut stack_items = Vec::new();
    for (idx, p) in all_prompts.iter().enumerate() {
        let (snippet, word_count) = truncate_words(&p.prompt_content, max_words);
        stack_items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": p.status,
            "model": p.model.clone().unwrap_or_else(|| "default".to_string()),
            "word_count": word_count,
            "words_limit": max_words,
            "prompt": snippet,
            "has_image": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&stack_items).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!(
        "\n[Table Mode: Displaying {} prompt(s) in ASC stack order (truncated to {} words; pass --json for raw JSON)]",
        stack_items.len(),
        max_words
    );

    if stack_items.is_empty() {
        println!("No active prompt tasks tracked in repo_prompts.db.");
    } else {
        println!(
            "{:<5} {:<10} {:<22} {:<12} {:<10} PROMPT SNIPPET (ASC STACK)",
            "SEQ", "ID", "PROJECT", "STATUS", "WORDS"
        );
        println!("{}", "-".repeat(110));
        for item in &stack_items {
            let seq = item["seq"].as_u64().unwrap_or(0);
            let short_id: String = item["id"].as_str().unwrap_or("-").chars().take(8).collect();
            let proj: String = item["project_id"]
                .as_str()
                .unwrap_or("-")
                .chars()
                .take(20)
                .collect();
            let status = item["status"].as_str().unwrap_or("-");
            let wc = item["word_count"].as_u64().unwrap_or(0);
            let prompt_txt = item["prompt"].as_str().unwrap_or("");
            println!(
                "#{:<4} {:<10} {:<22} {:<12} {:<10} {}",
                seq, short_id, proj, status, wc, prompt_txt
            );
        }
        println!();
    }
```

### Change G (`agm.rs`, `cmd_running_prompts`, five sub-blocks; leave the rest unchanged)

G1 (`:4245`):

```rust
            println!("  agm running-prompts [ls] [--limit <Y>] [--words <N>] [--full] [--json]");
```

G2 (`:4269`):

```rust
            println!("    --limit, -l <Y>     Limit number of prompts displayed (default: 8)");
```

G3 (`:4303`):

```rust
    // Default to running-prompts ls
    let is_json = args.iter().any(|a| a == "--json");
```

G4 (`:4351` to `:4360`):

```rust
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let mut running_prompts: Vec<_> = all_prompts
        .into_iter()
        .filter(|p| {
            p.status == "running"
                || p.status == "queued"
                || p.status == "dispatched"
                || p.status == "backed_up"
        })
        .collect();
```

G5 (`:4376` to `:4387`, then the table header and row print at `:4413` to `:4433`):

```rust
        items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": p.status,
            "word_count": word_count,
            "prompt": snippet,
            "has_images": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
```

```rust
    println!(
        "{:<5} {:<10} {:<22} {:<12} {:<10} PROMPT SNIPPET (ASC STACK)",
        "SEQ", "ID", "PROJECT", "STATUS", "WORDS"
    );
    println!("{}", "-".repeat(110));
    for it in &items {
        let seq = it["seq"].as_u64().unwrap_or(0);
        let sid: String = it["id"].as_str().unwrap_or("-").chars().take(8).collect();
        let proj: String = it["project_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(20)
            .collect();
        let status = it["status"].as_str().unwrap_or("-");
        let wc = it["word_count"].as_u64().unwrap_or(0);
        let prompt_txt = it["prompt"].as_str().unwrap_or("");
        println!(
            "#{:<4} {:<10} {:<22} {:<12} {:<10} {}",
            seq, sid, proj, status, wc, prompt_txt
        );
    }
    println!();
}
```

The table header literal `"{:<5} {:<10} {:<22} {:<12} {:<10} PROMPT SNIPPET (ASC STACK)"` appears twice in `agm.rs` (in `cmd_prompts` and in `cmd_running_prompts`). Edit the one inside `cmd_running_prompts` here; the one in `cmd_prompts` is replaced by F4.

### Change H (`agm.rs:4587` to `:4740`, whole function)

Verify these sub-blocks. Expected drift: step 17 changed `:4627` to `repo_db::list_running_projects(&instance::InstanceScope::All)`.

```rust
fn cmd_running_projects(args: &[String]) {
    if let Some(first) = args.first() {
        if first.eq_ignore_ascii_case("help") || first == "--help" || first == "-h" {
            println!("AGM Running Projects Management:");
            println!("  agm running-projects [ls] [--json] [-f <path.json>] [--ssh]");
```

```rust
    let projects = repo_db::list_running_projects().unwrap_or_default();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let conversations = agy_cleaner::scan_conversations(100);
```

```rust
            .filter(|p| {
                (p.project_id == proj.id || p.repo_path.eq_ignore_ascii_case(&proj.repo_path))
                    && (p.status == "running"
                        || p.status == "queued"
                        || p.status == "dispatched"
                        || p.status == "backed_up")
            })
```

The function ends with:

```rust
        println!(
            "#{:<4} {:<24} {:<24} {:<16} {:<12} {}",
            seq, proj, id, cid, st, qc
        );
    }
    println!();
}

fn cmd_finish_prompts_until_green(args: &[String]) {
```

Delete from `fn cmd_running_projects(args: &[String]) {` through the closing `}` directly above `fn cmd_finish_prompts_until_green(args: &[String]) {`.

### Change I (`agm.rs`, `cmd_tree`, two sub-blocks)

I1 (`:7025` and `:7037`):

```rust
        println!("  agm tree [all] [--words <N>] [--json]");
```

```rust
        println!("  --json, -j          Output full tree structure as JSON");
```

I2 (`:7047` to `:7073`, to the end of the function):

```rust
    let only_running = !args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("all") || a == "--all" || a == "-a");
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let mut max_words = 200usize;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--words" || args[i] == "-w") && i + 1 < args.len() {
            if let Ok(w) = args[i + 1].parse::<usize>() {
                max_words = w.max(1);
            }
            i += 2;
            continue;
        }
        i += 1;
    }

    if is_json {
        let tree = repo_db::get_project_conversation_tree(max_words, only_running);
        println!(
            "{}",
            serde_json::to_string_pretty(&tree).unwrap_or_else(|_| "[]".to_string())
        );
    } else {
        println!("{}", repo_db::format_tree_view_cli(max_words, only_running));
    }
}
```

### Change T1 / T2 (insertion points only)

```rust
#[cfg(test)]
mod tests {
    use super::*;
```

## 5. New code

### Change A (`instance.rs`): insert directly above the `///` line of `resolve_instance_id`

```rust
/// What a CLI command acts on: one canonical instance, or every instance (list commands only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceSelection {
    One { id: String, name: String },
    All,
}

/// A selection plus the stderr note that explains an implicit choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstancePick {
    pub selection: InstanceSelection,
    pub note: Option<String>,
}

fn is_instance_flag(arg: &str) -> bool {
    arg == "-i" || arg == "--instance" || arg == "-instance"
}

fn instance_flag_inline_value(arg: &str) -> Option<&str> {
    arg.strip_prefix("--instance=")
        .or_else(|| arg.strip_prefix("-instance="))
        .or_else(|| arg.strip_prefix("-i="))
}

/// Value of `-i`, `--instance` or `-instance` (also the `=` forms). `Some("")` when the flag has no value.
pub fn instance_flag_value(args: &[String]) -> Option<String> {
    for (idx, arg) in args.iter().enumerate() {
        if let Some(value) = instance_flag_inline_value(arg) {
            return Some(value.trim().to_string());
        }
        if is_instance_flag(arg) {
            return Some(
                args.get(idx + 1)
                    .filter(|next| !next.starts_with('-'))
                    .map(|next| next.trim().to_string())
                    .unwrap_or_default(),
            );
        }
    }
    None
}

/// `args` without the instance flag and its value, so positional parsing never sees them.
pub fn strip_instance_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    let mut should_skip_value = false;
    for arg in args {
        if should_skip_value {
            should_skip_value = false;
            if !arg.starts_with('-') {
                continue;
            }
        }
        if instance_flag_inline_value(arg).is_some() {
            continue;
        }
        if is_instance_flag(arg) {
            should_skip_value = true;
            continue;
        }
        out.push(arg.clone());
    }
    out
}

/// Pure selection rule shared by every `agm` prompt command.
/// Explicit `-i` always wins. List commands without `-i` act on all instances. Actions without
/// `-i` use the only running instance, the default instance when none runs, and refuse when two
/// or more run.
pub fn select_instance_core(
    args: &[String],
    is_action: bool,
    registry: &InstanceRegistry,
    running_ids: &[String],
) -> Result<InstancePick, String> {
    let one = |id: String| InstanceSelection::One {
        name: instance_name_in(registry, &id),
        id,
    };

    if let Some(raw) = instance_flag_value(args) {
        if raw.is_empty() {
            return Err("instance required: -i needs a value (<id|#seq|name>)".to_string());
        }
        let id = canonical_instance_id_in(registry, &raw).or_else(|_| {
            resolve_instance_input_in(registry, &registry.active_instance_id, &raw)
        })?;
        return Ok(InstancePick {
            selection: one(id),
            note: None,
        });
    }

    if !is_action {
        return Ok(InstancePick {
            selection: InstanceSelection::All,
            note: None,
        });
    }

    match running_ids {
        [] => {
            let id = default_instance_id_in(registry);
            let note = format!(
                "[instance] using {} (no instance running; default)",
                instance_short_label_in(registry, &id)
            );
            Ok(InstancePick {
                selection: one(id),
                note: Some(note),
            })
        }
        [only] => {
            let id = canonical_instance_id_in(registry, only)?;
            let note = format!(
                "[instance] using {} (only running instance)",
                instance_short_label_in(registry, &id)
            );
            Ok(InstancePick {
                selection: one(id),
                note: Some(note),
            })
        }
        many => Err(format!(
            "instance required: {} instances running ({}); pass -i <id|#seq|name>",
            many.len(),
            many.iter()
                .map(|id| instance_short_label_in(registry, id))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}
```

### Change B (`repo_db.rs`): replace with

The `agm` binary is a separate crate, so `pub(crate)` items are invisible to it.

```rust
/// Normalize filesystem path for reliable cross-platform comparison
pub fn normalize_path_for_compare(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}
```

### Change C (`repo_db.rs`): replace the first four lines shown in section 4 with

Leave every line after `let mut out = String::new();` unchanged.

```rust
/// True when one of a summary row's workspace URIs is exactly this repo (normalized path).
pub fn workspace_uris_contain_repo(workspace_uris: &str, repo_path: &str) -> bool {
    let target = normalize_path_for_compare(repo_path);
    if target.is_empty() {
        return false;
    }
    let trimmed = workspace_uris.trim();
    let uris: Vec<String> = serde_json::from_str::<Vec<String>>(trimmed)
        .unwrap_or_else(|_| vec![trimmed.to_string()]);
    uris.iter()
        .filter(|uri| !uri.trim().is_empty())
        .any(|uri| normalize_path_for_compare(&decode_uri_to_path(uri)) == target)
}

/// Format the Project -> Conversation -> 200-Word Prompt Tree View for AGM CLI output
pub fn format_tree_view_cli(max_words: usize, only_running: bool) -> String {
    format_tree_view_cli_for(None, max_words, only_running)
}

/// Tree view of one instance (`Some(id)`) or of every instance (`None`).
pub fn format_tree_view_cli_for(
    instance_id: Option<&str>,
    max_words: usize,
    only_running: bool,
) -> String {
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let tree = get_project_conversation_tree_cached(instance_id, word_cap, only_running, false);
    let mut out = String::new();
```

### Change D (`agm.rs`): insert directly above `fn cmd_which_prompts_running(args: &[String]) {`

```rust
fn load_registry_or_exit() -> instance::InstanceRegistry {
    match instance::load_registry() {
        Ok(registry) => registry,
        Err(e) => {
            eprintln!("[ERROR] Failed to load instance registry: {}", e);
            std::process::exit(1);
        }
    }
}

/// Applies `instance::select_instance_core`; prints the note on stderr; exits 2 on refusal.
fn select_instance(args: &[String], is_action: bool) -> instance::InstanceSelection {
    let registry = load_registry_or_exit();
    let has_flag = instance::instance_flag_value(args).is_some();
    let running_ids: Vec<String> = if is_action && !has_flag {
        registry
            .instances
            .iter()
            .filter(|inst| instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid))
            .map(|inst| inst.id.clone())
            .collect()
    } else {
        Vec::new()
    };
    match instance::select_instance_core(args, is_action, &registry, &running_ids) {
        Ok(pick) => {
            if let Some(note) = pick.note {
                eprintln!("{}", note);
            }
            pick.selection
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    }
}

fn selection_scope(selection: &instance::InstanceSelection) -> instance::InstanceScope {
    match selection {
        instance::InstanceSelection::One { id, .. } => instance::InstanceScope::One(id.clone()),
        instance::InstanceSelection::All => instance::InstanceScope::All,
    }
}

fn is_selected_instance(selection: &instance::InstanceSelection, instance_id: &str) -> bool {
    match selection {
        instance::InstanceSelection::One { id, .. } => id.eq_ignore_ascii_case(instance_id),
        instance::InstanceSelection::All => true,
    }
}

fn instance_label(registry: &instance::InstanceRegistry, instance_id: &str) -> (String, String) {
    (
        instance_id.to_string(),
        instance::instance_name_in(registry, instance_id),
    )
}

fn instance_group_header(registry: &instance::InstanceRegistry, instance_id: &str) -> String {
    format!(
        "== {} ({}) ==",
        instance::instance_short_label_in(registry, instance_id),
        instance_id
    )
}

fn is_live_prompt_status(status: &str) -> bool {
    status == "running" || status == "backed_up" || status == "dispatched"
}

fn conversation_for_project<'a>(
    conversations: &'a [agy_cleaner::ConversationItem],
    proj: &repo_db::RunningProject,
) -> Option<&'a agy_cleaner::ConversationItem> {
    conversations
        .iter()
        .find(|c| repo_db::workspace_uris_contain_repo(&c.workspace_uris, &proj.repo_path))
}
```

If the compiler says `agy_cleaner::scan_conversations` returns something other than `Vec<agy_cleaner::ConversationItem>`, STOP.

### Change E (`agm.rs`): the new `cmd_which_prompts_running`

```rust
fn cmd_which_prompts_running(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Which Prompts Running:");
        println!("  agm which-prompts-running [-i <id|#seq|name>] [--json]");
        println!("\nDescription:");
        println!(
            "  Inspects all registered workspaces and Antigravity conversation queues to detect"
        );
        println!("  actively executing, queued, and in-flight prompts with associated friendly project names.");
        println!("  Without -i every instance is listed, grouped by instance.");
        println!("\nAliases: agm which-prompts-running, agm wpr");
        println!("\nOptions:");
        println!("    -i, --instance <id|#seq|name>  Only this instance (default: all instances)");
        println!("    --json, -j          Output running prompts and project metadata as JSON");
        println!("\nExamples:");
        println!(
            "  agm which-prompts-running           # Display formatted table of running prompts"
        );
        println!("  agm wpr --json                      # Output active prompt queues as JSON");
        println!("  agm wpr -i 2                        # Only instance #2");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
    let selection = select_instance(args, false);
    let registry = load_registry_or_exit();

    for inst in &registry.instances {
        if is_selected_instance(&selection, &inst.id) {
            let _ = repo_db::detect_running_projects(&inst.id);
        }
    }

    let projects =
        repo_db::list_running_projects(&selection_scope(&selection)).unwrap_or_default();
    let all_prompts: Vec<repo_db::ActivePrompt> = repo_db::list_all_prompts()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| is_selected_instance(&selection, &p.instance_id))
        .collect();
    let conversations = agy_cleaner::scan_conversations(100);

    let mut rows: Vec<serde_json::Value> = Vec::new();

    for proj in &projects {
        let proj_norm = repo_db::normalize_path_for_compare(&proj.repo_path);
        let proj_prompts: Vec<&repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                p.instance_id == proj.instance_id
                    && repo_db::normalize_path_for_compare(&p.repo_path) == proj_norm
                    && is_live_prompt_status(&p.status)
            })
            .collect();

        if !proj.is_running && proj_prompts.is_empty() {
            continue;
        }

        let matched_conv = conversation_for_project(&conversations, proj);
        let conv_id = proj_prompts
            .first()
            .and_then(|p| p.session_id.clone())
            .or_else(|| matched_conv.map(|c| c.conversation_id.clone()))
            .unwrap_or_else(|| "-".to_string());
        let conv_name = matched_conv
            .and_then(|c| {
                if c.title.trim().is_empty() {
                    None
                } else {
                    Some(c.title.clone())
                }
            })
            .unwrap_or_else(|| proj.repo_name.clone());
        let queue_count = if proj_prompts.is_empty() && proj.is_running {
            1usize
        } else {
            proj_prompts.len()
        };
        let (instance_id, instance_name) = instance_label(&registry, &proj.instance_id);

        rows.push(serde_json::json!({
            "seq": 0,
            "project": proj.repo_name,
            "id": proj.id,
            "instance_id": instance_id,
            "instance_name": instance_name,
            "repo_path": proj.repo_path,
            "conv_id": conv_id,
            "conv_name": conv_name,
            "prompts_count": queue_count,
            "is_running": proj.is_running,
        }));
    }

    for p in &all_prompts {
        if !is_live_prompt_status(&p.status) {
            continue;
        }
        let p_norm = repo_db::normalize_path_for_compare(&p.repo_path);
        let is_included = rows.iter().any(|r| {
            r["instance_id"].as_str() == Some(p.instance_id.as_str())
                && r["repo_path"]
                    .as_str()
                    .map(repo_db::normalize_path_for_compare)
                    .as_deref()
                    == Some(p_norm.as_str())
        });
        if !is_included {
            let (instance_id, instance_name) = instance_label(&registry, &p.instance_id);
            rows.push(serde_json::json!({
                "seq": 0,
                "project": p.project_id,
                "id": p.project_id,
                "instance_id": instance_id,
                "instance_name": instance_name,
                "repo_path": p.repo_path,
                "conv_id": p.session_id.clone().unwrap_or_else(|| "-".to_string()),
                "conv_name": p.project_id,
                "prompts_count": 1,
                "is_running": true,
            }));
        }
    }

    rows.sort_by_key(|r| r["instance_id"].as_str().unwrap_or("").to_string());
    for (idx, row) in rows.iter_mut().enumerate() {
        row["seq"] = serde_json::json!(idx + 1);
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    if rows.is_empty() {
        println!("No projects currently have running or queued prompts.");
        return;
    }

    println!(
        "\nProjects with Running / Queued Prompts ({} active):",
        rows.len()
    );

    let mut current_instance: Option<String> = None;
    for item in &rows {
        let inst_id = item["instance_id"].as_str().unwrap_or("-").to_string();
        if current_instance.as_deref() != Some(inst_id.as_str()) {
            println!("\n{}", instance_group_header(&registry, &inst_id));
            println!(
                "{:<5} {:<22} {:<24} {:<16} {:<26} PROMPTS (QUEUE)",
                "SEQ", "PROJECT", "ID", "CONV ID", "CONV NAME"
            );
            println!("{}", "-".repeat(110));
            current_instance = Some(inst_id);
        }
        let s = item["seq"].as_u64().unwrap_or(0);
        let proj = item["project"].as_str().unwrap_or("-");
        let id: String = item["id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(22)
            .collect();
        let cid: String = item["conv_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(14)
            .collect();
        let cname: String = item["conv_name"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(24)
            .collect();
        let qcount = item["prompts_count"].as_u64().unwrap_or(0);
        println!(
            "#{:<4} {:<22} {:<24} {:<16} {:<26} {}",
            s, proj, id, cid, cname, qcount
        );
    }
    println!();
}
```

### Change F (`agm.rs`, `cmd_prompts`)

F1 becomes:

```rust
            println!("  agm prompts [ls] [N] [-i <id|#seq|name>] [--words <W>] [--running] [--json]");
```

F2 becomes (the `-i` value must be stripped first, otherwise `-i 2` is read as the limit `2`):

```rust
    let selection = select_instance(args, false);
    let registry = load_registry_or_exit();
    let stripped_args = instance::strip_instance_args(args);
    let args = stripped_args.as_slice();
    let is_ls = args
        .first()
        .map(|a| a.eq_ignore_ascii_case("ls") || a.eq_ignore_ascii_case("list"))
        .unwrap_or(false);
    let is_json = args.iter().any(|a| a == "--json");
    let running_only = args.iter().any(|a| a == "--running") || is_ls;
```

F3 becomes:

```rust
    let mut all_prompts: Vec<repo_db::ActivePrompt> = repo_db::list_all_prompts()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| is_selected_instance(&selection, &p.instance_id))
        .collect();
```

F4 becomes:

```rust
    let mut stack_items = Vec::new();
    for (idx, p) in all_prompts.iter().enumerate() {
        let (snippet, word_count) = truncate_words(&p.prompt_content, max_words);
        stack_items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "instance_name": instance::instance_name_in(&registry, &p.instance_id),
            "repo_path": p.repo_path,
            "status": p.status,
            "model": p.model.clone().unwrap_or_else(|| "default".to_string()),
            "word_count": word_count,
            "words_limit": max_words,
            "prompt": snippet,
            "has_image": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&stack_items).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!(
        "\n[Table Mode: Displaying {} prompt(s) in ASC stack order (truncated to {} words; pass --json for raw JSON)]",
        stack_items.len(),
        max_words
    );

    if stack_items.is_empty() {
        println!("No active prompt tasks tracked in repo_prompts.db.");
    } else {
        println!(
            "{:<5} {:<10} {:<18} {:<22} {:<12} {:<10} PROMPT SNIPPET (ASC STACK)",
            "SEQ", "ID", "INSTANCE", "PROJECT", "STATUS", "WORDS"
        );
        println!("{}", "-".repeat(126));
        for item in &stack_items {
            let seq = item["seq"].as_u64().unwrap_or(0);
            let short_id: String = item["id"].as_str().unwrap_or("-").chars().take(8).collect();
            let inst_label: String = instance::instance_short_label_in(
                &registry,
                item["instance_id"].as_str().unwrap_or(""),
            )
            .chars()
            .take(16)
            .collect();
            let proj: String = item["project_id"]
                .as_str()
                .unwrap_or("-")
                .chars()
                .take(20)
                .collect();
            let status = item["status"].as_str().unwrap_or("-");
            let wc = item["word_count"].as_u64().unwrap_or(0);
            let prompt_txt = item["prompt"].as_str().unwrap_or("");
            println!(
                "#{:<4} {:<10} {:<18} {:<22} {:<12} {:<10} {}",
                seq, short_id, inst_label, proj, status, wc, prompt_txt
            );
        }
        println!();
    }
```

### Change G (`agm.rs`, `cmd_running_prompts`)

G1 becomes:

```rust
            println!("  agm running-prompts [ls] [-i <id|#seq|name>] [--limit <Y>] [--words <N>] [--full] [--json]");
```

G2 becomes:

```rust
            println!("    --limit, -l <Y>     Limit number of prompts displayed (default: 8)");
            println!("    -i, --instance <id|#seq|name>  Only this instance (default: all instances)");
```

G3 becomes:

```rust
    // Default to running-prompts ls
    let selection = select_instance(args, false);
    let registry = load_registry_or_exit();
    let stripped_args = instance::strip_instance_args(args);
    let args = stripped_args.as_slice();
    let is_json = args.iter().any(|a| a == "--json");
```

G4 becomes:

```rust
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let mut running_prompts: Vec<_> = all_prompts
        .into_iter()
        .filter(|p| {
            (p.status == "running"
                || p.status == "queued"
                || p.status == "dispatched"
                || p.status == "backed_up")
                && is_selected_instance(&selection, &p.instance_id)
        })
        .collect();
```

G5: the `items.push` block becomes:

```rust
        items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "instance_name": instance::instance_name_in(&registry, &p.instance_id),
            "repo_path": p.repo_path,
            "status": p.status,
            "word_count": word_count,
            "prompt": snippet,
            "has_images": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
```

and the table block (from the header `println!` through the end of the function) becomes:

```rust
    println!(
        "{:<5} {:<10} {:<18} {:<22} {:<12} {:<10} PROMPT SNIPPET (ASC STACK)",
        "SEQ", "ID", "INSTANCE", "PROJECT", "STATUS", "WORDS"
    );
    println!("{}", "-".repeat(126));
    for it in &items {
        let seq = it["seq"].as_u64().unwrap_or(0);
        let sid: String = it["id"].as_str().unwrap_or("-").chars().take(8).collect();
        let inst_label: String =
            instance::instance_short_label_in(&registry, it["instance_id"].as_str().unwrap_or(""))
                .chars()
                .take(16)
                .collect();
        let proj: String = it["project_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(20)
            .collect();
        let status = it["status"].as_str().unwrap_or("-");
        let wc = it["word_count"].as_u64().unwrap_or(0);
        let prompt_txt = it["prompt"].as_str().unwrap_or("");
        println!(
            "#{:<4} {:<10} {:<18} {:<22} {:<12} {:<10} {}",
            seq, sid, inst_label, proj, status, wc, prompt_txt
        );
    }
    println!();
}
```

### Change H (`agm.rs`): the new `cmd_running_projects`

```rust
fn cmd_running_projects(args: &[String]) {
    if let Some(first) = args.first() {
        if first.eq_ignore_ascii_case("help") || first == "--help" || first == "-h" {
            println!("AGM Running Projects Management:");
            println!(
                "  agm running-projects [ls] [-i <id|#seq|name>] [--json] [-f <path.json>] [--ssh]"
            );
            println!("\nDescription:");
            println!("  Lists active workspace projects having running or queued prompts.");
            println!("  Without -i every instance is listed, grouped by instance.");
            println!("\nAliases: agm running-projects, agm projects");
            println!("\nOptions:");
            println!("    -i, --instance <id|#seq|name>  Only this instance (default: all instances)");
            println!("    --json              Output pure JSON array");
            println!("    -f, --file <path>   Write output to specified file path (default: agm-running-projects.json)");
            println!("    --ssh               Include multi-node cluster fleet projects");
            println!("\nExamples:");
            println!("  agm running-projects                # Display table of active projects");
            println!(
                "  agm running-projects --json         # Output active projects in JSON format"
            );
            println!("  agm running-projects -f proj.json   # Export project inventory to file");
            println!("  agm running-projects -i 2           # Only instance #2");
            return;
        }
    }

    let selection = select_instance(args, false);
    let registry = load_registry_or_exit();
    let stripped_args = instance::strip_instance_args(args);
    let args = stripped_args.as_slice();
    let is_json = args.iter().any(|a| a == "--json");
    let is_ssh = args.iter().any(|a| a == "--ssh");
    let mut file_dest: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "-f" || args[i] == "--file" || args[i] == "-file" {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                file_dest = Some(args[i + 1].clone());
                i += 2;
                continue;
            } else {
                file_dest = Some("agm-running-projects.json".to_string());
            }
        }
        i += 1;
    }

    let projects =
        repo_db::list_running_projects(&selection_scope(&selection)).unwrap_or_default();
    let all_prompts: Vec<repo_db::ActivePrompt> = repo_db::list_all_prompts()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| is_selected_instance(&selection, &p.instance_id))
        .collect();
    let conversations = agy_cleaner::scan_conversations(100);

    let mut rows: Vec<serde_json::Value> = Vec::new();

    for proj in &projects {
        let proj_norm = repo_db::normalize_path_for_compare(&proj.repo_path);
        let proj_prompts: Vec<&repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                p.instance_id == proj.instance_id
                    && repo_db::normalize_path_for_compare(&p.repo_path) == proj_norm
                    && (is_live_prompt_status(&p.status) || p.status == "queued")
            })
            .collect();

        if !proj.is_running && proj_prompts.is_empty() {
            continue;
        }

        let matched_conv = conversation_for_project(&conversations, proj);
        let conv_id = proj_prompts
            .first()
            .and_then(|p| p.session_id.clone())
            .or_else(|| matched_conv.map(|c| c.conversation_id.clone()))
            .unwrap_or_else(|| "-".to_string());
        let conv_name = matched_conv
            .and_then(|c| {
                if c.title.trim().is_empty() {
                    None
                } else {
                    Some(c.title.clone())
                }
            })
            .unwrap_or_else(|| proj.repo_name.clone());
        let (instance_id, instance_name) = instance_label(&registry, &proj.instance_id);

        rows.push(serde_json::json!({
            "seq": 0,
            "project": proj.repo_name,
            "id": proj.id,
            "instance_id": instance_id,
            "instance_name": instance_name,
            "repo_path": proj.repo_path,
            "conv_id": conv_id,
            "conv_name": conv_name,
            "prompts_count": proj_prompts.len().max(if proj.is_running { 1 } else { 0 }),
            "status": if proj.is_running { "running" } else { "idle" },
            "is_ssh": false,
            "node": "localhost",
        }));
    }

    rows.sort_by_key(|r| r["instance_id"].as_str().unwrap_or("").to_string());
    for (idx, row) in rows.iter_mut().enumerate() {
        row["seq"] = serde_json::json!(idx + 1);
    }

    if is_ssh {
        let m_name = email_watcher::detect_machine_name();
        let m_ip = email_watcher::detect_local_ip();
        println!(
            "[SSH Cluster Mode] Queried local node '{}' ({})",
            m_name, m_ip
        );
    }

    if let Some(dest) = file_dest {
        let json_content = serde_json::to_string_pretty(&rows).unwrap_or_default();
        let _ = fs::write(&dest, json_content);
        println!("[SUCCESS] Saved running projects JSON to '{}'", dest);
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!("\n=== Running Projects ({} active) ===", rows.len());
    if rows.is_empty() {
        println!("No projects currently running with active prompts.");
        return;
    }

    let mut current_instance: Option<String> = None;
    for r in &rows {
        let inst_id = r["instance_id"].as_str().unwrap_or("-").to_string();
        if current_instance.as_deref() != Some(inst_id.as_str()) {
            println!("\n{}", instance_group_header(&registry, &inst_id));
            println!(
                "{:<5} {:<24} {:<24} {:<16} {:<12} PROMPTS (QUEUE)",
                "SEQ", "PROJECT", "ID", "CONV ID", "STATUS"
            );
            println!("{}", "-".repeat(95));
            current_instance = Some(inst_id);
        }
        let seq = r["seq"].as_u64().unwrap_or(0);
        let proj = r["project"].as_str().unwrap_or("-");
        let id: String = r["id"].as_str().unwrap_or("-").chars().take(22).collect();
        let cid: String = r["conv_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(14)
            .collect();
        let st = r["status"].as_str().unwrap_or("-");
        let qc = r["prompts_count"].as_u64().unwrap_or(0);
        println!(
            "#{:<4} {:<24} {:<24} {:<16} {:<12} {}",
            seq, proj, id, cid, st, qc
        );
    }
    println!();
}
```

### Change I (`agm.rs`, `cmd_tree`)

I1: replace the two help lines with:

```rust
        println!("  agm tree [all] [-i <id|#seq|name>] [--words <N>] [--json]");
```

```rust
        println!("  --json, -j          Output full tree structure as JSON");
        println!("  -i, --instance <id> Only this instance (default: all instances)");
```

I2 becomes:

```rust
    let selection = select_instance(args, false);
    let instance_filter = match &selection {
        instance::InstanceSelection::One { id, .. } => Some(id.clone()),
        instance::InstanceSelection::All => None,
    };
    let stripped_args = instance::strip_instance_args(args);
    let args = stripped_args.as_slice();
    let only_running = !args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("all") || a == "--all" || a == "-a");
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let mut max_words = 200usize;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--words" || args[i] == "-w") && i + 1 < args.len() {
            if let Ok(w) = args[i + 1].parse::<usize>() {
                max_words = w.max(1);
            }
            i += 2;
            continue;
        }
        i += 1;
    }

    if is_json {
        let tree = repo_db::get_project_conversation_tree_cached(
            instance_filter.as_deref(),
            max_words,
            only_running,
            false,
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&tree).unwrap_or_else(|_| "[]".to_string())
        );
    } else {
        println!(
            "{}",
            repo_db::format_tree_view_cli_for(instance_filter.as_deref(), max_words, only_running)
        );
    }
}
```

Compiler-forced adaptations you may make (and nothing else): if clippy reports `useless_conversion` or `needless_borrow` on a line of this step, apply exactly the suggestion it prints.

## 6. Tests

`agm.rs` cannot hold tests: `src-tauri/Cargo.toml` declares the `agm` binary with `test = false`, and `cargo test --lib` never compiles it. The selection rule is therefore a pure library function (`select_instance_core`) and is tested in `instance.rs`.

### T1: paste directly after `    use super::*;` in `mod tests` of `src-tauri/src/modules/instance.rs`

These reuse `scope_registry` from step 01 (`default` #1 Default, `work-1234` #2 Work, `a-8159` #3 Alpha, `b-8159` #4 Beta). If `scope_registry` does not exist, STOP.

```rust
    fn sel_args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn select_instance_refuses_action_when_two_running() {
        let registry = scope_registry("default");
        let running = sel_args(&["a-8159", "b-8159"]);
        let err = select_instance_core(&sel_args(&["--json"]), true, &registry, &running)
            .unwrap_err();
        assert!(
            err.starts_with("instance required: 2 instances running"),
            "{}",
            err
        );
        assert!(err.contains("#3 Alpha"), "{}", err);
        assert!(err.contains("#4 Beta"), "{}", err);
        assert!(err.contains("pass -i <id|#seq|name>"), "{}", err);
    }

    #[test]
    fn select_instance_uses_only_running_instance() {
        let registry = scope_registry("default");
        let pick =
            select_instance_core(&sel_args(&[]), true, &registry, &sel_args(&["a-8159"])).unwrap();
        assert_eq!(
            pick.selection,
            InstanceSelection::One {
                id: "a-8159".to_string(),
                name: "Alpha".to_string()
            }
        );
        assert_eq!(
            pick.note.as_deref(),
            Some("[instance] using #3 Alpha (only running instance)")
        );

        let idle = select_instance_core(&sel_args(&[]), true, &registry, &[]).unwrap();
        assert_eq!(
            idle.selection,
            InstanceSelection::One {
                id: "default".to_string(),
                name: "Default".to_string()
            }
        );
        assert!(idle.note.unwrap().contains("no instance running; default"));
    }

    #[test]
    fn select_instance_list_without_flag_is_all() {
        let registry = scope_registry("default");
        let running = sel_args(&["a-8159", "b-8159"]);
        let pick =
            select_instance_core(&sel_args(&["--json"]), false, &registry, &running).unwrap();
        assert_eq!(pick.selection, InstanceSelection::All);
        assert_eq!(pick.note, None);
    }

    #[test]
    fn select_instance_explicit_id_wins_over_running_count() {
        let registry = scope_registry("default");
        let running = sel_args(&["a-8159", "b-8159"]);
        for args in [
            sel_args(&["-i", "b-8159"]),
            sel_args(&["--instance", "#4"]),
            sel_args(&["-i=Beta"]),
            sel_args(&["--instance=4"]),
        ] {
            let pick = select_instance_core(&args, true, &registry, &running).unwrap();
            assert_eq!(
                pick.selection,
                InstanceSelection::One {
                    id: "b-8159".to_string(),
                    name: "Beta".to_string()
                },
                "{:?}",
                args
            );
            assert_eq!(pick.note, None);
        }
        let missing =
            select_instance_core(&sel_args(&["-i"]), true, &registry, &running).unwrap_err();
        assert!(missing.starts_with("instance required"), "{}", missing);
        let unknown = select_instance_core(&sel_args(&["-i", "nope"]), false, &registry, &running)
            .unwrap_err();
        assert!(unknown.contains("unknown instance"), "{}", unknown);
    }

    #[test]
    fn strip_instance_args_removes_flag_and_value() {
        let stripped = strip_instance_args(&sel_args(&[
            "ls",
            "-i",
            "#2",
            "5",
            "--instance=work-1234",
            "--json",
        ]));
        assert_eq!(stripped, sel_args(&["ls", "5", "--json"]));
        assert_eq!(instance_flag_value(&sel_args(&["ls", "5"])), None);
        assert_eq!(
            instance_flag_value(&sel_args(&["ls", "-i"])),
            Some(String::new())
        );
    }
```

### T2: paste directly after `    use super::*;` in `mod tests` of `src-tauri/src/modules/repo_db.rs`

The expected path is built with the same decoder, so the test is correct on Windows and on Unix.

```rust
    #[test]
    fn workspace_uris_match_exact_repo_only() {
        let repo = decode_uri_to_path("file:///d:/work/app");
        assert!(workspace_uris_contain_repo(
            "[\"file:///d:/work/app\"]",
            &repo
        ));
        assert!(workspace_uris_contain_repo("file:///d:/work/app", &repo));
        assert!(!workspace_uris_contain_repo(
            "[\"file:///d:/work/app-v2\"]",
            &repo
        ));
        assert!(!workspace_uris_contain_repo(
            "[\"file:///d:/work/app/sub\"]",
            &repo
        ));
        assert!(!workspace_uris_contain_repo("[]", &repo));
        assert!(!workspace_uris_contain_repo("[\"file:///d:/work/app\"]", ""));
    }
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 select_instance_; cargo test --lib -- --test-threads=1 strip_instance_args_removes_flag_and_value; cargo test --lib -- --test-threads=1 workspace_uris_match_exact_repo_only; cd ..
```

`select_instance_` must report 4 tests. `clippy --all-targets` compiles the `agm` binary; that is the only compile check for Changes D to I.

## 8. Commit

```text
Feature: prompts - agm -i on list commands and shared selection
```

## 9. Done when

- [ ] `InstanceSelection`, `InstancePick`, `instance_flag_value`, `strip_instance_args`, `select_instance_core` exist once in `instance.rs`.
- [ ] `normalize_path_for_compare` is `pub`; `workspace_uris_contain_repo` and `format_tree_view_cli_for` exist in `repo_db.rs`.
- [ ] This is the only step that widens `normalize_path_for_compare` to `pub`. Step 08 made it `pub(crate)`, and no later step changes it. Its first uses from `agm.rs` (a separate bin crate, which cannot see `pub(crate)` items) are in Changes E and H of this same step, so the visibility change lands no later than its first use.
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. No caller edit is needed outside this step. `format_tree_view_cli(usize, bool) -> String` keeps its signature (callers `agm.rs:7071` and `:7087`), `normalize_path_for_compare` only widens, and `src-tauri/tests` has 0 hits for either name.
- [ ] `agm.rs` has `select_instance`, `instance_label`, `instance_group_header` and the other helpers from Change D, once each.
- [ ] `gitmap aum search "eq_ignore_ascii_case(&proj.repo_path)" src-tauri/src/bin/agm.rs --ext .rs` returns no hit.
- [ ] `gitmap aum search "uris_norm.contains" src-tauri/src/bin/agm.rs --ext .rs` returns no hit.
- [ ] Every JSON row of `wpr`, `prompts ls`, `running-prompts ls` and `running-projects` has `instance_id` and `instance_name`.
- [ ] Six new tests pass; the gate exits 0.
- [ ] Only the three files from section 2 are staged.
