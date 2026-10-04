# Subtask 01: Fix Instance Prompt Matching & Database Hygiene

- **Task Identifier**: `123-prompt-running-instance-detection-and-audit-logging`
- **Subtask Slug**: `01-fix-instance-prompt-matching-and-db-hygiene`
- **Worker Assignment**: Worker 01
- **Master Plan**: [123-prompt-running-instance-detection-and-audit-logging.md](../../123-prompt-running-instance-detection-and-audit-logging.md)
- **Architecture Spec**: [01-architecture-spec.md](../../../02-spec/21-app/123-prompt-running-instance-detection-and-audit-logging/01-architecture-spec.md)
- **Component Spec**: [02-component-spec.md](../../../02-spec/21-app/123-prompt-running-instance-detection-and-audit-logging/02-component-spec.md)
- **Root Cause Analysis**: [123-per-instance-prompt-running-detection-root-cause.md](../../../02-spec/22-app-issues/123-per-instance-prompt-running-detection-root-cause.md)
- **Target Source Modules**:
  - `src-tauri/src/modules/repo_db.rs`
  - `src/pages/Instances.tsx`

---

## 1. Objectives & Executive Scope

Worker 01 is responsible for implementing the core backend and frontend fixes for the 5 root causes identified in `01-architecture-spec.md`:

1. **Eliminate Cross-Instance Prompt Bleed in Dispatchers**:
   Remove `|| instance_repo_paths.contains(...)` from `dispatch_running_prompts` and `resend_running_commands_for_instance`. Enforce strict tenant identity matching.
2. **Universal Suffix Resolution & Composite Keying in `detect_running_projects`**:
   Resolve raw `instance_id` strings (such as `"8159"`) via `crate::modules::instance::resolve_instance_id`, construct composite primary keys `{base_project_id}__{canonical_instance_id}`, and prune non-composite legacy records.
3. **Disambiguate Queued vs Running in `get_live_project_execution_info`**:
   Ensure `status = 'queued'` records do not evaluate `is_running = true` (`entry.0 = true`). Only prompts with `status = 'running'` and freshness age `<= 300s` may set `entry.0 = true`.
4. **Epistemic Supremacy of Concrete Conversations in `compute_project_conversation_tree`**:
   When `conv_nodes` is non-empty, evaluate `proj_is_running` exclusively from `conv_nodes.iter().any(|c| c.is_running)`. Eliminate the `has_active_prompt` fallback that resurrected stale database rows.
5. **Strict Instance Scoping & Process Gating in `Instances.tsx`**:
   Eliminate permissive `!node.instance_id` adoption by default in `hasActiveTask` and `instanceProjects`. Implement `isNodeOwnedByInstance` with sequence number and suffix matching. Gate running status on `Boolean(inst.is_running)` and rely on backend `proj.is_running`.
6. **Implement Cache Invalidation**:
   Add and call `invalidate_prompt_tree_cache(instance_id: Option<&str>)` upon prompt state or instance state transitions.

---

## 2. Detailed Backend Implementation: `src-tauri/src/modules/repo_db.rs`

### 2.1 Refactor `dispatch_running_prompts`

#### Location
- `src-tauri/src/modules/repo_db.rs` (lines ~1352–1410)

#### Required Changes
1. Normalize target instance ID:
   ```rust
   let target_inst = crate::modules::instance::resolve_instance_id(instance_id)
       .unwrap_or_else(|_| {
           if instance_id == "__default__" || instance_id.is_empty() {
               "default".to_string()
           } else {
               instance_id.to_string()
           }
       });
   let is_default_target = target_inst == "default" || target_inst == "__default__";
   ```
2. Remove all path-based matching:
   Delete:
   ```rust
   // DELETE THIS:
   || instance_repo_paths.contains(&normalize_path_for_compare(&p.repo_path))
   ```
3. Filter strictly on canonical instance identity:
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
           if is_default_target {
               prompt_inst == "default" || prompt_inst == "__default__" || prompt_inst.is_empty()
           } else {
               prompt_inst == target_inst
           }
       })
       .collect();
   ```
4. Emit structured audit logging using `crate::modules::logger::log_instance_prompt_audit` for dispatched candidates.
5. Invalidate `prompt_tree_cache`:
   ```rust
   invalidate_prompt_tree_cache(Some(&target_inst));
   ```

---

### 2.2 Refactor `resend_running_commands_for_instance`

#### Location
- `src-tauri/src/modules/repo_db.rs` (lines ~2960–3015)

#### Required Changes
1. Resolve instance specifier:
   ```rust
   let target_inst_opt = instance_id.map(|id| {
       crate::modules::instance::resolve_instance_id(id).unwrap_or_else(|_| id.to_string())
   });
   ```
2. Remove path-based matching:
   Delete `|| instance_repo_paths.contains(&normalize_path_for_compare(&p.repo_path))`.
3. Filter prompts strictly:
   ```rust
   let prompts: Vec<ActivePrompt> = all_prompts
       .into_iter()
       .filter(|p| match target_inst_opt.as_deref() {
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
       })
       .collect();
   ```
4. Invalidate `prompt_tree_cache`:
   ```rust
   invalidate_prompt_tree_cache(target_inst_opt.as_deref());
   ```

---

### 2.3 Refactor `detect_running_projects`

#### Location
- `src-tauri/src/modules/repo_db.rs` (lines ~466–540)

#### Required Changes
1. Resolve shorthand instance specifier at the function entrypoint:
   ```rust
   pub fn detect_running_projects(instance_id: &str) -> Result<Vec<RunningProject>, String> {
       let resolved_id = crate::modules::instance::resolve_instance_id(instance_id)
           .unwrap_or_else(|_| {
               if instance_id == "__default__" || instance_id.is_empty() {
                   "default".to_string()
               } else {
                   instance_id.to_string()
               }
           });
       let target_id = resolved_id.as_str();
       let registry = crate::modules::instance::load_registry()?;
       let instance = registry
           .instances
           .iter()
           .find(|i| i.id == target_id || (target_id == "default" && i.is_default))
           .ok_or_else(|| format!("Instance '{}' (resolved: '{}') not found", instance_id, target_id))?;
   ```
2. Build composite primary keys:
   ```rust
   let base_project_id = format!(
       "{}-{}",
       repo_name.to_lowercase(),
       entry.file_name().to_string_lossy()
   );
   let composite_id = format!("{}__{}", base_project_id, target_id);
   ```
3. In `projects.push`:
   Ensure `id: composite_id` and `instance_id: target_id.to_string()`.
4. Prune non-composite legacy records before persisting:
   ```rust
   if let Ok(conn) = connect_db() {
       let _ = conn.execute("DELETE FROM running_projects WHERE id NOT LIKE '%__%'", []);
       // Continue with upsert loop...
   }
   ```
5. Invalidate `prompt_tree_cache(Some(target_id))`.

---

### 2.4 Refactor `get_live_project_execution_info`

#### Location
- `src-tauri/src/modules/repo_db.rs` (lines ~2360–2395)

#### Required Changes
1. In the query, retrieve `updated_at`:
   ```rust
   let mut stmt = conn.prepare(
       "SELECT instance_id, repo_path, prompt_content, status, updated_at 
        FROM active_prompts 
        WHERE status = 'running' OR status = 'queued'",
   )?;
   ```
2. Distinguish `running` from `queued`:
   ```rust
   for item in rows.flatten() {
       let (p_inst, p_path, p_content, status, updated_at): (String, String, String, String, i64) = item;
       let norm_ap_inst = if p_inst == "__default__" || p_inst.is_empty() {
           "default".to_string()
       } else {
           crate::modules::instance::resolve_instance_id(&p_inst)
               .unwrap_or_else(|_| p_inst.clone())
       };
       let norm_inst = norm_ap_inst.to_lowercase();
       let clean_p = normalize_path_for_compare(&p_path);
       let entry = live_map
           .entry((norm_inst, clean_p))
           .or_insert((false, None, now));

       // ONLY set is_running (entry.0) true if status is running and fresh
       if status == "running" && (now - updated_at <= 300) {
           entry.0 = true;
       }

       // For both running and queued, retain the prompt preview content if empty
       if entry.1.is_none() {
           entry.1 = Some(p_content.chars().take(120).collect());
       }
   }
   ```

---

### 2.5 Refactor `compute_project_conversation_tree`

#### Location
- `src-tauri/src/modules/repo_db.rs` (lines ~4030–4055)

#### Required Changes
1. Enforce concrete conversation supremacy:
   ```rust
   let has_active_conv = conv_nodes.iter().any(|c| c.is_running);

   let (proj_is_running, rationale) = if !is_inst_alive {
       (
           false,
           "INSTANCE_PROCESS_DEAD: instance PID not found or process terminated -> forced idle".to_string(),
       )
   } else if !conv_nodes.is_empty() {
       // Concrete conversation summaries exist on disk: this is verified empirical truth
       if has_active_conv {
           (
               true,
               "ACTIVE_IN_FLIGHT_TASKS: active non-idle conversation turn detected -> marked running".to_string(),
           )
       } else {
           (
               false,
               "IDLE_EXPLICIT_STATUS: verified conversation summaries on disk are all idle -> marked idle".to_string(),
           )
       }
   } else {
       // Fallback ONLY when workspace is empty on disk (new project before first turn summary is written)
       let active_prompt = is_prompt_running_for_project(&proj.repo_path, &proj.instance_id)
           || (!project_key.is_empty()
               && is_prompt_running_for_project(&project_key, &proj.instance_id));
       if active_prompt {
           (
               true,
               "ACTIVE_IN_FLIGHT_TASKS: active prompt detected in database for empty workspace -> marked running".to_string(),
           )
       } else {
           (
               false,
               "IDLE_NO_ACTIVE_TASKS: process alive but no in-flight tasks or active conversations -> marked idle".to_string(),
           )
       }
   };
   ```
2. Log the verdict via `log_instance_prompt_audit`.

---

### 2.6 Implement `invalidate_prompt_tree_cache`

#### Location
- `src-tauri/src/modules/repo_db.rs`

#### Implementation
```rust
/// Invalidate cached project conversation trees in SQLite `prompt_tree_cache`
pub fn invalidate_prompt_tree_cache(instance_id: Option<&str>) {
    if let Ok(conn) = connect_db() {
        match instance_id {
            Some(id) if id != "all" => {
                let norm = crate::modules::instance::resolve_instance_id(id)
                    .unwrap_or_else(|_| id.to_string());
                let _ = conn.execute(
                    "DELETE FROM prompt_tree_cache WHERE instance_id = ?1 OR instance_id = 'all'",
                    rusqlite::params![&norm],
                );
            }
            _ => {
                let _ = conn.execute("DELETE FROM prompt_tree_cache", []);
            }
        }
    }
}
```

---

## 3. Detailed Frontend Implementation: `src/pages/Instances.tsx`

### 3.1 Define `isNodeOwnedByInstance` Helper

#### Location
- Top or helper section of `src/pages/Instances.tsx`

#### Implementation
```typescript
export const isNodeOwnedByInstance = (
    node: AgmProjectTreeNode,
    instConfig: { id: string; is_default?: boolean; seq_num?: number }
): boolean => {
    if (instConfig.is_default) {
        if (node.instance_id === 'default' || node.instance_id === '__default__' || node.instance_id === instConfig.id) {
            return true;
        }
    }
    if (node.instance_id === instConfig.id) {
        return true;
    }
    if (instConfig.seq_num !== undefined && node.instance_seq_num === instConfig.seq_num) {
        return true;
    }
    // Suffix match for cloned instances (e.g. node.instance_id is "8159" and instConfig.id is "default-copy-8159")
    if (node.instance_id && instConfig.id.endsWith(node.instance_id) && node.instance_id.length >= 4) {
        return true;
    }
    return false;
};
```

### 3.2 Update `hasActiveTask` Calculation

#### Location
- `src/pages/Instances.tsx` (~line 1015)

#### Remediated Logic
```typescript
const hasActiveTask = Boolean(inst.is_running) && runningTreeNodes.some((node) => {
    const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
    const isNodeRunning = Boolean(node.is_running);
    return isInstanceMatch && isNodeRunning;
});
```

### 3.3 Update `instanceProjects` Scoped Filtering

#### Location
- `src/pages/Instances.tsx` (~line 1360)

#### Remediated Logic
```typescript
const instanceProjects = projectTreeNodes.filter((node) => isNodeOwnedByInstance(node, inst.config));
```

### 3.4 Update `isProjRunning` Resolution

#### Location
- `src/pages/Instances.tsx` (~lines 1373–1381, 1406–1410)

#### Remediated Logic
```typescript
const isProjRunning = Boolean(inst.is_running) && Boolean(proj.is_running);
```

---

## 4. Acceptance Criteria & Quality Gates

Worker 01 must verify that all the following criteria are strictly satisfied:

- [ ] **No Path-Based Prompt Bleed**: Backed-up or queued prompts belonging to `default` are never dispatched to `8159`, even when `8159` contains matching repo paths.
- [ ] **Shorthand `"8159"` Resolves Properly**: Invoking `detect_running_projects("8159")` does not return `Instance '8159' not found`; it resolves to `"default-copy-8159"`.
- [ ] **Composite Key Isolation**: Rows in `running_projects` use composite key `{base_project_id}__{canonical_instance_id}`.
- [ ] **Queued Prompts Idle**: Prompts with `status = 'queued'` do not set `is_running = true` in `get_live_project_execution_info`.
- [ ] **Conversation Disk Supremacy**: When conversation summaries exist on disk and all turns are idle, `compute_project_conversation_tree` marks the project as idle without falling back to SQLite queries.
- [ ] **Target Ground Truth Satisfied**:
  - Sequence 1 (`default`): `Antigravity-Manager` = **RUNNING**, `SpecBuilder` = **IDLE**, `coding-guidelines` = **IDLE**.
  - Sequence 2 (`8159` / `default-copy-8159`): `coding-guidelines` = **RUNNING**, `Antigravity-Manager` = **IDLE**, `SpecBuilder` = **IDLE**.
- [ ] **Frontend Accuracy**:
  - Default card pulses only for `Antigravity-Manager`.
  - 8159 card pulses only for `coding-guidelines`.
  - No untagged projects adopted by default card.

---

## 5. Verification Commands for Worker 01

Execute the following commands from workspace root `d:\work\Antigravity-Manager`:

```powershell
# 1. Rust Format Gate
cd src-tauri ; cargo fmt -- --check ; cd ..

# 2. Rust Clippy Gate (Must be 100% clean with zero warnings/errors)
cd src-tauri ; cargo clippy --all-targets --all-features ; cd ..

# 3. Targeted Integration Test Execution
cd src-tauri ; cargo test --test per_instance_prompt_liveness_test ; cd ..

# 4. Frontend Build Gate (Must compile with zero TypeScript errors)
npm run build
```

> [!CAUTION]
> **TOTAL BAN ON GIT COMMANDS**:
> Worker 01 is strictly forbidden from running any git commands (`git add`, `git commit`, `git status`, `git push`, etc.).
> Search codebase exclusively via GitMap commands (`gitmap aum search`, `gitmap find`, `gitmap cat`).
