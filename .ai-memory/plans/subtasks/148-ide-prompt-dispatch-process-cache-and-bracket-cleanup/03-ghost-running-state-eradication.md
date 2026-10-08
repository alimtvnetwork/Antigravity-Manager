# Subtask 03: Ghost Running State Eradication & Synthetic Prompt Elimination

- **Subtask ID**: `148-03`
- **Parent Task**: `148-ide-prompt-dispatch-process-cache-and-bracket-cleanup`
- **Target Files**:
  - `src-tauri/src/modules/repo_db.rs`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: READY_FOR_EXECUTION

---

## 1. Objective

Purge synthetic prompt generation from `auto_resume_recent_prompts` and enforce empirical worker/transcript verification in `compute_project_conversation_tree` within `src-tauri/src/modules/repo_db.rs`. Ensure that idle workspaces never manufacture fake prompt rows, never report ghost `1 RUNNING` statuses, and only flag projects as active when verified live processes or transcripts exist.

---

## 2. Context & Root Cause Analysis

1. **Synthetic Prompt Manufacturing**:
   - In `auto_resume_recent_prompts` (~L3860-3870), when an active project had no existing prompt record in `active_prompts` and no file in `.antigravity_resume_task.json` (`maybe_prompt.is_none()`), the code synthesized a fake prompt:
     ```rust
     None => {
         let new_id = Uuid::new_v4().to_string();
         let content = format!(
             "Resume active project workspace for '{}' [{}] after IDE crash recovery",
             project.repo_name, project.repo_path
         );
         (new_id, content, Some("gemini-pro".to_string()), None)
     }
     ```
   - This synthetic prompt was immediately inserted into `active_prompts` with status `'dispatched'`, creating an artificial in-flight prompt record.
2. **Ghost Running State in Conversation Tree**:
   - `compute_project_conversation_tree` evaluated `active_prompts` and conversation summaries. Because synthetic prompts had status `'dispatched'`, the query marked the project as `is_running = true` and `running_count = 1`, causing idle projects to permanently display glowing green running indicators.
3. **Absence of Empirical Process Verification**:
   - `compute_project_conversation_tree` previously relied on database status flags without cross-verifying whether the assigned worker PID was actually alive in the OS process table.

---

## 3. Detailed Implementation Steps

### Step 1: Eliminate Synthetic Prompt Manufacturing in `auto_resume_recent_prompts`
- Locate `auto_resume_recent_prompts` in `src-tauri/src/modules/repo_db.rs` (~L3860-3870).
- Replace the `match maybe_prompt` fallback. If `maybe_prompt.is_none()`, do **not** synthesize a fake crash recovery prompt.
- Instead, log an informational message and skip the project:
  ```rust
  let (prompt_id, prompt_text, prompt_model, image_payload) = match maybe_prompt {
      Some(tuple) => tuple,
      None => {
          crate::modules::logger::log_info(&format!(
              "[RepoDB] auto_resume_recent_prompts: No prompt found for active project '{}' ({}), skipping auto-resume",
              project.repo_name, project.repo_path
          ));
          continue;
      }
  };
  ```
- This ensures `active_prompts` is never polluted with fictitious crash recovery tasks.

### Step 2: Enforce Empirical Process Verification in `compute_project_conversation_tree`
- Locate `compute_project_conversation_tree` (~L5388-5395 and ~L5505-5535).
- For `active_prompts` entries with status `'running'`, `'in_flight'`, or `'dispatched'`:
  - Cross-check against active workers in `get_active_agy_workers()`.
  - Validate that the associated worker PID is alive via `sysinfo` or `is_pid_alive_os`.
  - If the prompt is stale (> 300s without update) and has no live worker PID, transition its status to `'completed'` or mark `is_running = false`.
- For conversation nodes loaded from `conversation_summaries.db`:
  - Verify that `not_fully_idle > 0` is accompanied by fresh transcript activity (`inspection.is_recent_active`) and a living instance process.
  - If the instance process is dead, force `is_running = false` regardless of summary flags.

### Step 3: Audit Project-Level Running Determination
- In `compute_project_conversation_tree` (~L5514-5532):
  - If `has_conv_nodes` is true:
    - Project is marked running **only** if `grouped_convs.iter().any(|c| c.is_running)` evaluates to true.
    - If all conversation nodes are idle, project is strictly `is_running = false`, with rationale `"IDLE_ALL_TURNS_COMPLETED"`.
  - Calculate `running_count`:
    ```rust
    let running_count = grouped_convs.iter().filter(|c| c.is_running).count();
    ```
  - Ensure `running_count` evaluates to `0` for idle projects.

---

## 4. Invariants & Constraints

- **Strict Relative Git Paths**: All paths cited in code and documentation must be relative (e.g. `src-tauri/src/modules/repo_db.rs`).
- **Positive Booleans Only**: Exclusively use positive boolean identifiers (`is_running`, `is_alive`, `has_active_worker`, `has_valid_ws`).
- **No Data Loss**: Never drop legitimate active prompts. Only prune synthetic prompt generation and stale unbacked flags.
- **Thread Safety**: Safely acquire locks on `get_active_agy_workers()` and `get_dispatched_prompts_cache()`.

---

## 5. Verification & Pre-flight Checklist

```bash
# Verify Rust compilation and linting
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::repo_db
```

---

## 6. Done When

- [ ] Synthetic prompt creation in `auto_resume_recent_prompts` is completely removed.
- [ ] No fake "Resume active project workspace for..." prompts are inserted into `active_prompts`.
- [ ] `compute_project_conversation_tree` requires empirical worker/transcript verification before setting `is_running = true`.
- [ ] Idle projects reliably report `0 RUNNING` and `is_running = false`.
- [ ] All unit and module tests pass with zero clippy warnings.
