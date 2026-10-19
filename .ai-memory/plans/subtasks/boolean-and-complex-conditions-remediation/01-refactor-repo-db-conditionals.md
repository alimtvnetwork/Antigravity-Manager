# Subtask 01: Refactor Repo DB Conditionals & Extract Affirmative Predicate Helpers

**Parent Task:** `boolean-and-complex-conditions-remediation`  
**Subtask Code:** `Task-01`  
**Status:** `READY`  
**Assigned Worker:** `Worker 01`  
**Owned Files:**
- `src-tauri/src/modules/repo_db/tree_collect.rs`
- `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- `src-tauri/src/modules/repo_db/tree_phases.rs`

**Governing Spec:**
- `02-spec/21-app/boolean-and-complex-conditions-remediation/01-architecture-spec.md`
- `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/`

---

## 1. Objectives & Scope

Refactor all multi-clause compound expressions, mixed-polarity conditions, and ad-hoc string comparisons across `src-tauri/src/modules/repo_db/` (`tree_collect.rs`, `tree_node_conv.rs`, and `tree_phases.rs`). Extract 6 canonical affirmative predicate helpers into `tree_node_conv.rs` (or `tree_collect.rs`) and share them across tree aggregation modules to ensure zero code duplication, full branch immutability, and strict two-operand conditional bounds.

---

## 2. Affirmative Helper Functions Specification

The following 6 helpers must be implemented and exported (`pub(crate)`):

### Helper 1: `is_default_instance_match`
```rust
/// Returns true if the instance ID represents the default instance or its copies/aliases.
pub(crate) fn is_default_instance_match(inst_id: &str) -> bool {
    let is_exact_default = inst_id == "default" || inst_id == "__default__";
    let is_alias_default = inst_id.is_empty() || inst_id.starts_with("default-copy");
    is_exact_default || is_alias_default
}
```

### Helper 2: `is_instance_compatible`
```rust
/// Returns true if inst_id matches the project's instance directly or via default alias compatibility.
pub(crate) fn is_instance_compatible(
    inst_id: &str,
    norm_proj_inst: &str,
    is_proj_default: bool,
) -> bool {
    if inst_id == norm_proj_inst {
        return true;
    }
    let is_inst_default = is_default_instance_match(inst_id);
    is_proj_default && is_inst_default
}
```

### Helper 3: `is_vacant_ghost_conversation`
```rust
/// Affirmative detection of empty, placeholder, or untitled conversations.
pub(crate) fn is_vacant_ghost_conversation(
    title: &str,
    prompt: &str,
    word_count: usize,
    has_zero_turns: bool,
) -> bool {
    if !has_zero_turns {
        return false;
    }
    let is_prompt_empty = prompt.trim().is_empty();
    let has_zero_words = word_count == 0;
    let is_prompt_vacant = is_prompt_empty || has_zero_words;
    if !is_prompt_vacant {
        return false;
    }
    let title_clean = title.trim();
    if title_clean.is_empty() {
        return true;
    }
    let title_lower = title_clean.to_lowercase();
    let is_untitled = title_lower.starts_with("untitled");
    let is_new_conv = title_lower == "new conversation";
    is_untitled || is_new_conv
}
```

### Helper 4: `should_retain_unrunning_conversation`
```rust
/// Affirmative retention check when filtering conversations by only_running.
pub(crate) fn should_retain_unrunning_conversation(
    only_running: bool,
    is_running: bool,
) -> bool {
    if !only_running {
        return true;
    }
    is_running
}
```

### Helper 5: `should_retain_prompt_candidate`
```rust
/// Evaluates whether an unrunning conversation candidate has sufficient title or prompt content.
pub(crate) fn should_retain_prompt_candidate(
    is_non_prompt: bool,
    is_conv_running: bool,
    has_meaningful_title: bool,
    has_prompt_content: bool,
) -> bool {
    if !is_non_prompt {
        return true;
    }
    if is_conv_running {
        return true;
    }
    has_meaningful_title || has_prompt_content
}
```

### Helper 6: `is_owning_instance_alive`
```rust
/// Affirmative evaluation of owning instance OS liveness (shared between tree_collect and tree_phases).
pub(crate) fn is_owning_instance_alive(
    owning_inst_id: &str,
    registry: &crate::models::instance::InstanceRegistry,
) -> bool {
    if is_default_instance_match(owning_inst_id) {
        let is_process_running = crate::modules::process::is_antigravity_running(None);
        if is_process_running {
            return true;
        }
        let def_dir = crate::modules::instance::get_default_antigravity_data_dir();
        let pids = crate::modules::instance::find_pids_for_data_dir(&def_dir.to_string_lossy(), true);
        let has_pids = !pids.is_empty();
        return has_pids;
    }
    if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.id == owning_inst_id || i.name == owning_inst_id)
    {
        let is_inst_running =
            crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        if is_inst_running {
            return true;
        }
        let is_global_running = crate::modules::process::is_antigravity_running(None);
        return inst.is_default && is_global_running;
    }
    crate::modules::process::is_antigravity_running(None)
}
```

---

## 3. Detailed File Modification Tasks

### Phase 1: Implement Predicates in `tree_node_conv.rs` & Re-export
1. Insert the 6 helper functions at the top of `src-tauri/src/modules/repo_db/tree_node_conv.rs`.
2. Make them `pub(crate)` and import them into `tree_collect.rs` and `tree_phases.rs` via `use super::tree_node_conv::*;`.

### Phase 2: Refactor `src-tauri/src/modules/repo_db/tree_collect.rs`
1. **Lines 61-66 & 153**: Replace `ap.instance_id == "default" || ap.instance_id == "__default__"` and `owning_inst_id == "__default__" || owning_inst_id.is_empty()` with `is_default_instance_match(...)`.
2. **Lines 103-120**: Replace inline liveness logic with `is_owning_instance_alive(owning_inst_id, &registry)`.
3. **Lines 172-187**: Decompose `is_conv_running` into discrete steps:
   - `is_terminal = is_explicit_idle || inspection.is_terminal_done;`
   - `is_live_active = inspection.is_recent_active && !inspection.is_terminal_done;`
   - `has_live_supremacy = is_owning_inst_alive && is_live_active;`
   - `has_activity_marker = not_fully_idle > 0 && is_recent;`
   - `is_active_fallback = is_owning_inst_alive && has_activity_marker;`
4. **Lines 198-204**: Decompose `has_meaningful_title` into:
   - `let is_placeholder = title_lower.starts_with("untitled") || title_lower == "new conversation";`
   - `let has_meaningful_title = !title_clean.is_empty() && !is_placeholder;`
5. **Lines 210-216**: Replace 4-clause `if inspection.is_non_prompt && !is_conv_running ...` with:
   - `let should_retain = should_retain_prompt_candidate(inspection.is_non_prompt, is_conv_running, has_meaningful_title, has_prompt_content);`
   - `if !should_retain { continue; }`
6. **Lines 246-255**: Replace ghost conversation condition with `is_vacant_ghost_conversation(&title, &effective_prompt, eff_wc, prompt_turns.is_empty())`.
7. **Lines 269-276**: Decompose path prefix check:
   - `let has_assigned_paths = !assigned_paths.is_empty();`
   - `let has_valid_prefix = assigned_paths.iter().any(|p| p.len() >= 6);`
   - `let has_invalid_paths = has_assigned_paths && !has_valid_prefix;`
   - `let is_conv_running = if is_conv_running && has_invalid_paths { false } else { is_conv_running };`

### Phase 3: Refactor `src-tauri/src/modules/repo_db/tree_node_conv.rs`
1. **Lines 19 & 32**: Replace equality checks with `is_default_instance_match(instance_id)`.
2. **Lines 72-75 & 81-86**: Replace `is_proj_default` and `matches_inst` with `is_default_instance_match` and `is_instance_compatible(inst_id, norm_proj_inst, is_proj_default)`.
3. **Lines 101-106 & 311-316**: Replace untitled checks with `is_vacant_ghost_conversation(&item.title, &item.prompt, word_count, true)`.
4. **Lines 109-111 & 401-403**: Replace `only_running && !effective_is_run` and `only_running && !is_run` with `should_retain_unrunning_conversation(...)`.
5. **Lines 216-224**: Decompose `target_idx` closure into early returns for `is_id_match` and `has_matching_prompt`.
6. **Lines 228-234 & 238-243**: Separate active prompt stages into `is_pending_stage` and `is_active_stage`.
7. **Lines 260-262 & 300**: Decompose `can_enqueue = is_queued && !target_conv.is_running`.
8. **Lines 325-337**: Decompose worker matching into `is_inst_match` and `is_path_match` using `is_default_instance_match`.
9. **Lines 375-377**: Decompose `is_turn_busy = is_cascade_running && has_busy_flag;` and `is_active_turn = is_turn_busy && !has_idle_marker;`.
10. **Lines 395-400**: Decompose `is_run` in active prompt synthesis into `is_status_fresh && is_executing`.

### Phase 4: Refactor `src-tauri/src/modules/repo_db/tree_phases.rs`
1. **Lines 66-84**: Decompose `is_target_default` using `is_default_instance_match`. Decompose the `filter` closure to avoid nested `||` inside `&&`.
2. **Line 89**: Invert string empty check to affirmative `let is_defined = !p.instance_id.trim().is_empty();`.
3. **Lines 117-134**: Replace duplicate liveness check with `is_owning_instance_alive(owning_inst_id, &registry)`.
4. **Lines 173-177**: Decompose `is_running` in `discover_fallback_projects` into:
   - `let is_status_runnable = is_active_status && !is_terminal;`
   - `let is_node_active = is_owning_inst_alive && has_activity;`
   - `let is_running = is_status_runnable && is_node_active;`
5. **Lines 251-267**: Decompose target clean filtering into `is_scoped_target` and affirmative default comparisons.
6. **Lines 321-323**: Use `is_default_instance_match(target_id)` in telemetry probe.

---

## 4. Verification Evidence Contract

The following test suites and preflight gates must be executed to certify zero regressions:

1. **Targeted Unit / Integration Tests for `repo_db`:**
   ```bash
   cargo test --lib modules::repo_db
   cargo test --test per_instance_prompt_liveness_test
   ```
   - **Expectation:** Exit code 0, all tests pass.
2. **Code Formatting Gate:**
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
   - **Expectation:** Exit code 0, zero formatting deviations.
3. **Rust Clippy Gate:**
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
   - **Expectation:** Exit code 0, zero warnings or errors.

---

## 5. Non-Negotiable Operational Invariants

- **TOTAL BAN ON GIT COMMANDS:** The worker MUST NEVER run `git add`, `git commit`, `git push`, `git status`, `git diff`, or any other git commands.
- **Strict Relative Git Paths:** All file links and references MUST use repository-relative paths (e.g. `src-tauri/src/modules/repo_db/tree_collect.rs`). Never include absolute paths or URI schemes.
- **Zero Behavioral Drift:** The output structures of `collect_tree_conversation_items`, `build_transcript_conv_nodes`, `build_active_prompt_conv_nodes`, and `query_tree_projects` must produce identical JSON payloads and tree structures.
