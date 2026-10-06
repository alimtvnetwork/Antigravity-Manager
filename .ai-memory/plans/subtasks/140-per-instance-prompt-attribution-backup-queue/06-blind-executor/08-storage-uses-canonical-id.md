# Step 08: Storage Code Uses the Canonical Instance Id

Goal: `src-tauri/src/modules/repo_db.rs` stops calling `resolve_instance_id` (which turns `""` and `"active"` into whatever instance is active, and guesses by suffix). Parameters go through `canonical_instance_id`; values read from stored rows go through a new helper `canonical_row_instance_in`, which keeps an empty value empty (unassigned, DR-7) and never merges an unknown id into another instance. `normalize_path_for_compare` becomes `pub(crate)`.

## 1. Depends on

- Step 01 (`canonical_instance_id`, `canonical_instance_id_in` in `src-tauri/src/modules/instance.rs`).
- Step 04 (the tagged dirs passed to sites 5, 12 and 14 come from `gemini_dirs_tagged(&InstanceScope)`).

Steps run in numeric order, so 01 to 07 are already committed.

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/modules/telegram_inbound.rs` (caller C3 only)
- `src-tauri/src/modules/backup_prompts_db.rs` (caller C4 only)
- `src-tauri/src/bin/agm.rs` (caller C5 only)

Nothing else. The three extra files hold callers of `save_or_requeue_prompt` that may pass a display name, a seq, or `""`; site 9 makes that function return `Err` for them, so each caller is converted in this same step (section 5, C1 to C5).

## 3. Find it

List every call first:

```text
gitmap aum search "resolve_instance_id" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn canonical_instance_id_in" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "pub fn canonical_instance_id(" src-tauri/src/modules/instance.rs --ext .rs
```

The first command returned 19 hits on 2026-10-06 (lines below). Steps 05 and 06 add code above some of them, so lines may have moved down; anchor on the enclosing `fn` and the literal. The other two commands must return one hit each with signatures `canonical_instance_id_in(registry: &InstanceRegistry, raw: &str) -> Result<String, String>` and `canonical_instance_id(raw: &str) -> Result<String, String>`. If not, STOP.

| Site | Enclosing `fn` | Old line | What happens |
|---|---|---|---|
| H | `fn normalize_path_for_compare` | `:2473` | becomes `pub(crate)`; two helpers added after it |
| 1 | `pub fn detect_running_projects(instance_id: &str)` | `:514` | `canonical_instance_id(instance_id)?` |
| 2 | `pub fn dispatch_running_prompts(instance_id: &str)` | `:1496` | `canonical_instance_id(instance_id)?`, registry loaded once |
| 3 | same function, `.filter(` closure | `:1537` | `canonical_row_instance_in`, empty never matches |
| 4 | `pub fn is_prompt_running_for_project(project_id: &str, instance_id: &str)` | `:1706` | `canonical_row_instance` |
| 5 | `pub fn get_live_project_execution_info()` | `:2534` | tagged owner through `canonical_row_instance_in` |
| 6 | same function, active_prompts section | `:2618` | `canonical_row_instance_in` |
| 7 | same function, running_projects merge | `:2650` | `canonical_row_instance_in` |
| 8 | `pub fn get_project_execution_status(` | `:2700` | `canonical_row_instance` |
| 9 | `pub fn save_or_requeue_prompt(prompt: &ActivePrompt)` | `:2785` | empty is an error; `canonical_instance_id(..)?` |
| 10 | `pub fn resend_running_commands_for_instance(` | `:3274`, `:3283`, `:3288` | `canonical_row_instance_in`, one match arm |
| 11 | `pub fn invalidate_prompt_tree_cache(instance_id: Option<&str>)` | `:4028` | `canonical_row_instance` |
| 12 | `fn compute_project_conversation_tree(` fallback block | `:4152` | `canonical_row_instance_in(&registry, ..)` |
| 13 | same function, active prompt map | `:4235` | `canonical_row_instance_in(&registry, ..)` |
| 14 | same function, summaries loop | `:4316` | `canonical_row_instance_in(&registry, ..)` |
| 15 | same function, project loop | `:4442` | `canonical_row_instance_in(&registry, ..)` |
| (skip) | `pub fn check_and_dispatch_enqueued_prompts(` | `:2088`, `:2090` | NOT in this step. Step 11 replaces the whole function. Leave these two lines as they are. |

### Callers whose input changes meaning in this step

Sites 1, 2 and 9 now return `Err` for an id the registry does not know (and site 9 also for `""`) instead of guessing. The signatures do not change, so everything still compiles, but `save_or_requeue_prompt` callers that may pass user text, a seq, or `""` must convert the value first. Search all of them (GitMap does not descend into `src-tauri/src/bin` or `src-tauri/tests` from `src-tauri/src`):

```text
gitmap aum search "save_or_requeue_prompt" src-tauri/src --ext .rs
gitmap aum search "save_or_requeue_prompt" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "save_or_requeue_prompt|detect_running_projects|dispatch_running_prompts|normalize_path_for_compare" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "save_or_requeue_prompt|detect_running_projects|dispatch_running_prompts|normalize_path_for_compare" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "save_or_requeue_prompt|detect_running_projects|dispatch_running_prompts|normalize_path_for_compare" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
gitmap aum search "detect_running_projects(|dispatch_running_prompts(" src-tauri/src --ext .rs
gitmap aum search "detect_running_projects(|dispatch_running_prompts(" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "start_prompt_goal_heartbeat(" src-tauri/src --ext .rs
gitmap aum search "start_prompt_goal_heartbeat(" src-tauri/src/bin/agm.rs --ext .rs
```

Expected on 2026-10-06: the three test files have 0 hits. `save_or_requeue_prompt` callers:

| Caller | Value passed as `instance_id` | Action |
|---|---|---|
| C1 `repo_db.rs:5112` in `pub fn prompt_target_by_sequence_scoped(` | user `--instance` text, or the sequence row's `instance_id` (may be `""` or `"__default__"`) | convert with `resolve_instance_input` / `canonical_instance_id`; `Err` returns before any write |
| C2 `repo_db.rs:5445` in `pub fn start_prompt_goal_heartbeat(` | its `instance_id: &str` parameter (CLI text from `agm.rs:2692`, registry id from `repo_db.rs:5735`) | canonicalize at the top of the function, before any file is written |
| C3 `telegram_inbound.rs:2443` in `pub async fn execute_prompt_injection(` | `resolved_instance`: `--instance`/`ins:` text (already through `resolve_instance_id`, raw text on failure), the sequence row's `instance_id`, or `"default"` | convert with `resolve_instance_input`; on `Err` log and reply, never save |
| C4 `backup_prompts_db.rs:596` in `pub fn restore_running_prompts_for_instance(` | `rec.instance_id` or `target_inst` (may be `""` for legacy rows) | keep the value (DR-7: never guess an owner); log the `Err` instead of discarding it |
| C5 `agm.rs:2692` `fn cmd_prompt_start_goal(args: &[String])` | raw `--instance` CLI text, passed to `start_prompt_goal_heartbeat` | convert with `resolve_instance_input`; on `Err` print and exit 1 |
| `agm.rs:4563` `fn cmd_running_prompts_import` | literal `"default"` | no change (`canonical_instance_id("default")` always resolves) |
| `agm.rs:14365`, `:14366`, `:14367` (e2e command) | `new_inst.id` (registry id) | no change |

`detect_running_projects` and `dispatch_running_prompts` callers (`instance.rs`, `telegram_inbound.rs`, `integration.rs`, `auto_switcher.rs`, `repo_db.rs`, and `agm.rs:1625`, `:4170`, `:10928`, `:14613`, `:14789`) pass registry ids, `"default"` or `"__default__"`, and all discard or `unwrap_or` the result; no change. `normalize_path_for_compare` only widens visibility; no caller changes.

## 4. Current code

Each block below is copied verbatim from the file. Replace exactly that block, nothing around it.

### H. `normalize_path_for_compare` (`:2472` to `:2475`)

```rust
/// Normalize filesystem path for reliable cross-platform comparison
fn normalize_path_for_compare(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}
```

### 1. `detect_running_projects` (`:513` to `:521`)

```rust
    let resolved_id =
        crate::modules::instance::resolve_instance_id(instance_id).unwrap_or_else(|_| {
            if instance_id == "__default__" || instance_id.is_empty() {
                "default".to_string()
            } else {
                instance_id.to_string()
            }
        });
    let target_id = resolved_id.as_str();
```

### 2. `dispatch_running_prompts` target (`:1495` to `:1503`)

```rust
    let target_inst =
        crate::modules::instance::resolve_instance_id(instance_id).unwrap_or_else(|_| {
            if instance_id == "__default__" || instance_id.is_empty() {
                "default".to_string()
            } else {
                instance_id.to_string()
            }
        });
    let is_default_target = target_inst == "default" || target_inst == "__default__";
```

### 3. `dispatch_running_prompts` filter (`:1534` to `:1549`)

```rust
    let prompts: Vec<ActivePrompt> = all_backed_up
        .into_iter()
        .filter(|p| {
            let prompt_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                .unwrap_or_else(|_| {
                    if p.instance_id == "__default__" || p.instance_id.is_empty() {
                        "default".to_string()
                    } else {
                        p.instance_id.clone()
                    }
                });
            let is_match = if is_default_target {
                prompt_inst == "default" || prompt_inst == "__default__" || prompt_inst.is_empty()
            } else {
                prompt_inst == target_inst
            };
```

### 4. `is_prompt_running_for_project` (`:1705` to `:1713`)

```rust
    let resolved_inst =
        crate::modules::instance::resolve_instance_id(instance_id).unwrap_or_else(|_| {
            if instance_id == "__default__" || instance_id.is_empty() {
                "default".to_string()
            } else {
                instance_id.to_string()
            }
        });
    let norm_inst = resolved_inst.as_str();
```

### 5a. `get_live_project_execution_info` start (`:2480` to `:2482`)

```rust
pub fn get_live_project_execution_info() -> Vec<ProjectExecutionInfo> {
    let mut results: Vec<ProjectExecutionInfo> = Vec::new();
    let now = Utc::now().timestamp();
```

### 5b. Owner normalization in the summaries loop (`:2531` to `:2537`)

```rust
                    let norm_owning_inst = if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
                        "default".to_string()
                    } else {
                        crate::modules::instance::resolve_instance_id(owning_inst_id)
                            .unwrap_or_else(|_| owning_inst_id.to_string())
                    };
                    let norm_inst = norm_owning_inst.to_lowercase();
```

### 6. active_prompts section (`:2613` to `:2621`)

```rust
                    let norm_ap_inst = if p_inst == "default" || p_inst == "__default__" {
                        "default".to_string()
                    } else if p_inst.trim().is_empty() {
                        "unassigned".to_string()
                    } else {
                        crate::modules::instance::resolve_instance_id(&p_inst)
                            .unwrap_or_else(|_| p_inst.clone())
                    };
                    let norm_inst = norm_ap_inst.to_lowercase();
```

### 7. running_projects merge (`:2645` to `:2653`)

```rust
        let norm_proj_inst = if p.instance_id == "default" || p.instance_id == "__default__" {
            "default".to_string()
        } else if p.instance_id.trim().is_empty() {
            "unassigned".to_string()
        } else {
            crate::modules::instance::resolve_instance_id(&p.instance_id)
                .unwrap_or_else(|_| p.instance_id.clone())
        };
        let norm_inst = norm_proj_inst.to_lowercase();
```

### 8. `get_project_execution_status` (`:2700` to `:2702`)

```rust
    let norm_inst = crate::modules::instance::resolve_instance_id(instance_id)
        .unwrap_or_else(|_| instance_id.to_string())
        .to_lowercase();
```

### 9. `save_or_requeue_prompt` (`:2785` to `:2792`)

```rust
    let canonical_inst = crate::modules::instance::resolve_instance_id(&prompt.instance_id)
        .unwrap_or_else(|_| {
            if prompt.instance_id == "__default__" || prompt.instance_id.trim().is_empty() {
                "default".to_string()
            } else {
                prompt.instance_id.clone()
            }
        });
```

### 10. `resend_running_commands_for_instance` (`:3273` to `:3292`)

```rust
    let target_inst_opt = instance_id.map(|id| {
        crate::modules::instance::resolve_instance_id(id).unwrap_or_else(|_| id.to_string())
    });

    let prompts: Vec<ActivePrompt> = all_prompts
        .into_iter()
        .filter(|p| {
            let is_match = match target_inst_opt.as_deref() {
                None | Some("all") => true,
                Some("default") | Some("__default__") => {
                    let p_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                        .unwrap_or_else(|_| p.instance_id.clone());
                    p_inst == "default" || p_inst == "__default__" || p_inst.is_empty()
                }
                Some(inst) => {
                    let p_inst = crate::modules::instance::resolve_instance_id(&p.instance_id)
                        .unwrap_or_else(|_| p.instance_id.clone());
                    p_inst == inst
                }
            };
```

### 11. `invalidate_prompt_tree_cache` (`:4028` to `:4029`)

```rust
            let norm = crate::modules::instance::resolve_instance_id(id)
                .unwrap_or_else(|_| id.to_string());
```

### 12. `compute_project_conversation_tree` fallback (`:4148` to `:4154`)

```rust
                            let norm_inst =
                                if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
                                    "default".to_string()
                                } else {
                                    crate::modules::instance::resolve_instance_id(owning_inst_id)
                                        .unwrap_or_else(|_| owning_inst_id.clone())
                                };
```

### 13. `compute_project_conversation_tree` active prompt map (`:4232` to `:4237`)

```rust
            let inst = if ap.instance_id == "default" || ap.instance_id == "__default__" {
                "default".to_string()
            } else {
                crate::modules::instance::resolve_instance_id(&ap.instance_id)
                    .unwrap_or_else(|_| ap.instance_id.clone())
            };
```

### 14. `compute_project_conversation_tree` summaries loop (`:4313` to `:4317`)

```rust
                            let norm_owning_inst = if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
                                "default".to_string()
                            } else {
                                crate::modules::instance::resolve_instance_id(owning_inst_id).unwrap_or_else(|_| owning_inst_id.to_string())
                            };
```

### 15. `compute_project_conversation_tree` project loop (`:4439` to `:4444`)

```rust
        let norm_proj_inst = if proj.instance_id == "default" || proj.instance_id == "__default__" {
            "default".to_string()
        } else {
            crate::modules::instance::resolve_instance_id(&proj.instance_id)
                .unwrap_or_else(|_| proj.instance_id.clone())
        };
```

`compute_project_conversation_tree` already has `let registry = crate::modules::instance::load_registry().unwrap_or_default();` near its top (`:4048`). Sites 12 to 15 use that variable. Confirm it is still there and not shadowed before `:4442`.

### C1. `prompt_target_by_sequence_scoped` instance override (`repo_db.rs:5068` to `:5091`)

Anchor: `gitmap aum search "Resolve optional instance override" src-tauri/src/modules/repo_db.rs --ext .rs` (1 hit). `prompt_target_by_sequence_scoped` returns `Result<String, String>`. `resolved.instance_id` is a `String`.

```rust
    // Resolve optional instance override (#1, #2, instance name, or UUID)
    let effective_instance_id = if let Some(inst_raw) = instance_override
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if inst_raw.eq_ignore_ascii_case("default") || inst_raw == "#1" || inst_raw == "1" {
            "default".to_string()
        } else if let Ok(reg) = crate::modules::instance::load_registry() {
            let clean_seq = inst_raw.trim_start_matches('#').parse::<u32>().ok();
            if let Some(found) = reg.instances.iter().find(|i| {
                i.id.eq_ignore_ascii_case(inst_raw)
                    || i.name.eq_ignore_ascii_case(inst_raw)
                    || (clean_seq.is_some() && i.seq_num == clean_seq)
            }) {
                found.id.clone()
            } else {
                inst_raw.to_string()
            }
        } else {
            inst_raw.to_string()
        }
    } else {
        resolved.instance_id.clone()
    };
```

### C2. `start_prompt_goal_heartbeat` start (`repo_db.rs:5357` to `:5366`)

Anchor: `gitmap aum search "pub fn start_prompt_goal_heartbeat" src-tauri/src/modules/repo_db.rs --ext .rs` (1 hit).

```rust
pub fn start_prompt_goal_heartbeat(
    instance_id: &str,
    data_dir: &str,
    repo_path: &str,
    heartbeat_file: &str,
    prompt: &str,
    interval_secs: u64,
) -> Result<PromptGoalHeartbeatConfig, String> {
    let ws_dir = PathBuf::from(repo_path);
    if !ws_dir.exists() {
```

### C3. `telegram_inbound.rs` (`:2427`, inside `pub async fn execute_prompt_injection(`)

Anchor: `gitmap aum search "let prompt_id = format!(" src-tauri/src/modules/telegram_inbound.rs --ext .rs` (1 hit). It is the first line after the `let (final_proj_id, repo_path, resolved_instance, resolved_conv_id, seq_badge) = ...;` block. The function returns `String`; `clean_for_telegram_html(&str, usize)` is defined in the same file.

```rust
        let prompt_id = format!("p-{}", &uuid::Uuid::new_v4().to_string()[..8]);
```

### C4. `backup_prompts_db.rs:596` (restore loop of `restore_running_prompts_for_instance`; step 03 added the `source_dir` line)

Anchor: `gitmap aum search "save_or_requeue_prompt" src-tauri/src/modules/backup_prompts_db.rs --ext .rs` (1 hit).

```rust
            image_payload: rec.images_payload.clone(),
            source_dir: None,
        };
        let _ = repo_db::save_or_requeue_prompt(&active_p);
    }
```

### C5. `agm.rs` `fn cmd_prompt_start_goal(args: &[String])` (`:2684` to `:2692`)

Anchor: `gitmap aum search "match repo_db::start_prompt_goal_heartbeat" src-tauri/src/bin/agm.rs --ext .rs` (1 hit). The same `if data_dir.is_empty() {` block also exists at `:2630` in the goal-worker command; edit only the one directly above `match repo_db::start_prompt_goal_heartbeat(`. `instance_id` is a `String` here, defaulting to `"default"`.

```rust
    if data_dir.is_empty() {
        if let Ok(registry) = instance::load_registry() {
            if let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) {
                data_dir = inst.data_dir.clone();
            }
        }
    }

    match repo_db::start_prompt_goal_heartbeat(
```

## 5. New code

### H. Replace with

```rust
/// Normalize filesystem path for reliable cross-platform comparison
pub(crate) fn normalize_path_for_compare(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// Canonical instance id for a value read from a stored row or a tagged gemini dir.
/// Empty stays empty (unassigned). An id the registry does not know is kept as stored,
/// so it is never merged into another instance.
pub(crate) fn canonical_row_instance_in(
    registry: &crate::modules::instance::InstanceRegistry,
    raw: &str,
) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }
    crate::modules::instance::canonical_instance_id_in(registry, raw)
        .unwrap_or_else(|_| raw.to_string())
}

pub(crate) fn canonical_row_instance(raw: &str) -> String {
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    canonical_row_instance_in(&registry, raw)
}
```

### 1. Replace with

```rust
    let resolved_id = crate::modules::instance::canonical_instance_id(instance_id)?;
    let target_id = resolved_id.as_str();
```

### 2. Replace with

```rust
    let target_inst = crate::modules::instance::canonical_instance_id(instance_id)?;
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
```

`is_default_target` is gone. Its only use was the filter in site 3, which you replace next. Check: `gitmap aum search "is_default_target" src-tauri/src/modules/repo_db.rs --ext .rs` must return 0 hits after site 3.

### 3. Replace with

```rust
    let prompts: Vec<ActivePrompt> = all_backed_up
        .into_iter()
        .filter(|p| {
            let prompt_inst = canonical_row_instance_in(&registry, &p.instance_id);
            let is_match = !prompt_inst.is_empty() && prompt_inst == target_inst;
```

The rest of the closure (`crate::modules::logger::log_instance_prompt_audit(` ... `is_match` ... `})` ... `.collect();`) stays as it is.

### 4. Replace with

```rust
    let resolved_inst = canonical_row_instance(instance_id);
    let norm_inst = resolved_inst.as_str();
```

### 5a. Replace with

```rust
pub fn get_live_project_execution_info() -> Vec<ProjectExecutionInfo> {
    let mut results: Vec<ProjectExecutionInfo> = Vec::new();
    let now = Utc::now().timestamp();
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
```

### 5b. Replace with

```rust
                    let norm_inst =
                        canonical_row_instance_in(&registry, owning_inst_id).to_lowercase();
```

### 6. Replace with

```rust
                    let norm_ap_inst = if p_inst.trim().is_empty() {
                        "unassigned".to_string()
                    } else {
                        canonical_row_instance_in(&registry, &p_inst)
                    };
                    let norm_inst = norm_ap_inst.to_lowercase();
```

### 7. Replace with

```rust
        let norm_proj_inst = if p.instance_id.trim().is_empty() {
            "unassigned".to_string()
        } else {
            canonical_row_instance_in(&registry, &p.instance_id)
        };
        let norm_inst = norm_proj_inst.to_lowercase();
```

### 8. Replace with

```rust
    let norm_inst = canonical_row_instance(instance_id).to_lowercase();
```

### 9. Replace with

```rust
    if prompt.instance_id.trim().is_empty() {
        return Err(format!("Prompt '{}' has no instance id", prompt.id));
    }
    let canonical_inst = crate::modules::instance::canonical_instance_id(&prompt.instance_id)?;
```

### 10. Replace with

```rust
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    let target_inst_opt = instance_id.map(|id| canonical_row_instance_in(&registry, id));

    let prompts: Vec<ActivePrompt> = all_prompts
        .into_iter()
        .filter(|p| {
            let is_match = match target_inst_opt.as_deref() {
                None | Some("all") => true,
                Some(inst) => {
                    let p_inst = canonical_row_instance_in(&registry, &p.instance_id);
                    !p_inst.is_empty() && p_inst == inst
                }
            };
```

The rest of the closure (`let target_display = ...` and the audit call) stays.

### 11. Replace with

```rust
            let norm = canonical_row_instance(id);
```

### 12. Replace with

```rust
                            let norm_inst = canonical_row_instance_in(&registry, owning_inst_id);
```

### 13. Replace with

```rust
            let inst = canonical_row_instance_in(&registry, &ap.instance_id);
```

### 14. Replace with

```rust
                            let norm_owning_inst =
                                canonical_row_instance_in(&registry, owning_inst_id);
```

### 15. Replace with

```rust
        let norm_proj_inst = canonical_row_instance_in(&registry, &proj.instance_id);
```

Compiler notes you may hit:

- If `owning_inst_id` is a `&&String` in site 5b, 12 or 14 and the compiler complains, write `owning_inst_id.as_str()` on that line only.
- Do not add a `use crate::modules::instance::InstanceRegistry;` import; the helper uses the full path on purpose (another step may already import it).

### C1. Replace the block from 4 C1 with

```rust
    // Resolve optional instance override (#1, #2, instance name, or UUID)
    let effective_instance_id = match instance_override
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        Some(inst_raw) => crate::modules::instance::resolve_instance_input(inst_raw)?,
        None if resolved.instance_id.trim().is_empty() => {
            return Err(format!(
                "Target '{}' has no owning instance; pass --instance <id|#seq|name>.",
                target_token.trim()
            ));
        }
        None => crate::modules::instance::canonical_instance_id(&resolved.instance_id)?,
    };
```

The `ActivePrompt` literal below keeps `instance_id: effective_instance_id.clone(),`.

### C2. Replace the block from 4 C2 with

```rust
pub fn start_prompt_goal_heartbeat(
    instance_id: &str,
    data_dir: &str,
    repo_path: &str,
    heartbeat_file: &str,
    prompt: &str,
    interval_secs: u64,
) -> Result<PromptGoalHeartbeatConfig, String> {
    let canonical_id = crate::modules::instance::canonical_instance_id(instance_id)?;
    let instance_id = canonical_id.as_str();
    let ws_dir = PathBuf::from(repo_path);
    if !ws_dir.exists() {
```

The shadowed `instance_id` keeps the type `&str`, so every later use in the function compiles unchanged. An unknown id now returns `Err` before any folder or file is created.

### C3. Insert directly above the line from 4 C3

```rust
        let resolved_instance =
            match crate::modules::instance::resolve_instance_input(&resolved_instance) {
                Ok(id) => id,
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "[Telegram] Prompt not saved; instance '{}' rejected: {}",
                        resolved_instance, e
                    ));
                    return format!(
                        "⚠️ <b>Unknown Instance:</b> <code>{}</code>",
                        clean_for_telegram_html(&e, 200)
                    );
                }
            };
```

Every later use of `resolved_instance` in the function (the `ActivePrompt` literal and the reply text) now gets the canonical id.

### C4. Replace the block from 4 C4 with

```rust
            image_payload: rec.images_payload.clone(),
            source_dir: None,
        };
        if let Err(e) = repo_db::save_or_requeue_prompt(&active_p) {
            crate::modules::logger::log_warn(&format!(
                "[BackupDB] Restore prompt {} skipped: {}",
                rec.prompt_id, e
            ));
        }
    }
```

### C5. Replace the block from 4 C5 with

```rust
    let instance_id = match instance::resolve_instance_input(&instance_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[ERROR] Prompt goal not started: {}", e);
            std::process::exit(1);
        }
    };

    if data_dir.is_empty() {
        if let Ok(registry) = instance::load_registry() {
            if let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) {
                data_dir = inst.data_dir.clone();
            }
        }
    }

    match repo_db::start_prompt_goal_heartbeat(
```

## 6. Tests

Paste at the end of `mod tests` in `src-tauri/src/modules/repo_db.rs` (before its final `}`). `prompt_tests_registry` is reused by steps 09 and 10; define it only once.

```rust
    fn prompt_tests_registry(ids: &[&str]) -> crate::modules::instance::InstanceRegistry {
        let instances = ids
            .iter()
            .enumerate()
            .map(|(idx, id)| crate::modules::instance::InstanceConfig {
                id: id.to_string(),
                name: format!("Instance {}", id),
                data_dir: format!("agm-test-data/{}", id),
                executable_path: None,
                extensions_dir: None,
                bound_account_id: None,
                bound_email: None,
                created_at: 0,
                last_used: 0,
                is_default: idx == 0,
                pid: None,
                seq_num: Some(idx as u32 + 1),
            })
            .collect();
        crate::modules::instance::InstanceRegistry {
            active_instance_id: ids.first().map(|id| id.to_string()).unwrap_or_default(),
            instances,
        }
    }

    #[test]
    fn canonical_row_instance_keeps_empty_unassigned() {
        let registry = prompt_tests_registry(&["default", "inst-a"]);
        assert_eq!(canonical_row_instance_in(&registry, ""), "");
        assert_eq!(canonical_row_instance_in(&registry, "   "), "");
    }

    #[test]
    fn canonical_row_instance_maps_default_aliases() {
        let registry = prompt_tests_registry(&["default", "inst-a"]);
        assert_eq!(canonical_row_instance_in(&registry, "default"), "default");
        assert_eq!(canonical_row_instance_in(&registry, "DEFAULT"), "default");
        assert_eq!(canonical_row_instance_in(&registry, "__default__"), "default");
    }

    #[test]
    fn canonical_row_instance_uses_registry_casing() {
        let registry = prompt_tests_registry(&["default", "inst-a"]);
        assert_eq!(canonical_row_instance_in(&registry, "INST-A"), "inst-a");
        assert_eq!(canonical_row_instance_in(&registry, "inst-a"), "inst-a");
    }

    #[test]
    fn canonical_row_instance_keeps_unknown_values() {
        let registry = prompt_tests_registry(&["default", "inst-a"]);
        assert_eq!(canonical_row_instance_in(&registry, "ghost"), "ghost");
        assert_eq!(canonical_row_instance_in(&registry, "all"), "all");
    }

    #[test]
    fn instance_id_and_ide_flavor_are_not_confused() {
        let registry = prompt_tests_registry(&["default", "inst-a"]);
        let default_id = canonical_row_instance_in(&registry, "default");
        for flavor in ["ide", "agy"] {
            let value = canonical_row_instance_in(&registry, flavor);
            assert_eq!(value, flavor);
            assert_ne!(value, default_id);
        }
    }

    #[test]
    fn normalize_path_for_compare_ignores_separators_case_and_trailing_slash() {
        assert_eq!(normalize_path_for_compare("D:\\Work\\App\\"), "d:/work/app");
        assert_eq!(normalize_path_for_compare("d:/work/app"), "d:/work/app");
        assert_ne!(
            normalize_path_for_compare("d:/work/app-v2"),
            normalize_path_for_compare("d:/work/app")
        );
    }
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 canonical_row_instance instance_id_and_ide_flavor normalize_path_for_compare; cd ..
```

Then run:

```text
gitmap aum search "resolve_instance_id" src-tauri/src/modules/repo_db.rs --ext .rs
```

It must return exactly 2 hits, both inside `check_and_dispatch_enqueued_prompts` (`let norm_target = ...` and `let norm_inst = ...`). Any other hit means a site was missed.

## 8. Commit

Stage exactly: `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/bin/agm.rs`.

```text
Fix: prompts - storage uses canonical instance ids
```

## 9. Done when

- [ ] All 17 listed `resolve_instance_id` calls outside `check_and_dispatch_enqueued_prompts` are gone.
- [ ] `normalize_path_for_compare` is `pub(crate)`; its body is unchanged.
- [ ] `canonical_row_instance_in` and `canonical_row_instance` exist right after it.
- [ ] `save_or_requeue_prompt` returns `Err` for an empty or unknown instance id instead of writing `"default"`.
- [ ] `dispatch_running_prompts` and `resend_running_commands_for_instance` never match a row whose `instance_id` is empty.
- [ ] No `get_active_instance_id` call exists in `repo_db.rs` (`gitmap aum search "get_active_instance_id" src-tauri/src/modules/repo_db.rs --ext .rs` returns 0 hits).
- [ ] Callers of `save_or_requeue_prompt`, `detect_running_projects`, `dispatch_running_prompts` and `start_prompt_goal_heartbeat` were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file.
- [ ] C1 to C5 are applied: every `save_or_requeue_prompt` caller either passes a registry id, converts user text with `resolve_instance_input` / `canonical_instance_id` first, or logs the `Err`; none can write a row with `instance_id = ''`.
- [ ] Gate exits 0. Only the four files in section 8 are staged.
