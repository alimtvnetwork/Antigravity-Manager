# Architecture Specification: Boolean & Complex Condition Remediation (`repo_db`)

**Slug:** `boolean-and-complex-conditions-remediation`  
**Status:** `ACTIVE`  
**Version:** `1.0.0`  
**Subsystem:** `src-tauri/src/modules/repo_db`  
**Touchpoints:** `tree_collect.rs`, `tree_node_conv.rs`, `tree_phases.rs`  
**Governing Guidelines:** `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/`

---

## 1. Executive Summary & Problem Classification

During the execution of complex prompt tree aggregation and conversation lifecycle sweeps in `repo_db`, multiple modules accumulated complex boolean expressions that violate project coding guidelines:
1. **Multi-Clause Compound Conjunctions / Disjunctions (P4a Violation)**: Expressions containing 3 or more chained operands joined with `&&` or `||` (e.g. `!cascade_status.is_idle_or_terminal() && is_owning_inst_alive && cascade_status.is_running() && not_fully_idle > 0 && is_recent` in `tree_phases.rs:173-177`).
2. **Operator Mixing without Discrete Scoping (P4b Violation)**: Mixing `&&` and `||` in single un-decomposed expressions, creating precedence ambiguity (e.g. `matches_inst` in `tree_node_conv.rs:81-86`, `is_target_default` in `tree_phases.rs:66-84`).
3. **Mixed Polarity Conditionals (P4c & P6 Violation)**: Combining affirmative predicates with raw negations in the same expression (e.g. `is_conv_running && !assigned_paths.is_empty() && !assigned_paths.iter().any(...)` in `tree_collect.rs:269-276` and `only_running && !is_run` in `tree_node_conv.rs:109, 401`).
4. **Call-Site Raw Negations on Queries and Inverted Logic (P3 Violation)**: Calling raw negations like `!pids.is_empty()` and inverted checks on strings (`!title.trim().is_empty()`) instead of positive predicate helpers.
5. **Duplicated Instance Identity & Liveness Logic**: Redundant inline implementations of default instance checks (`inst_id == "default" || inst_id == "__default__"`) and OS process liveness evaluations scattered across `tree_collect.rs` and `tree_phases.rs`.

This architecture specification formalizes 6 affirmative predicate helper functions, decomposes all identified violation sites into clean step-by-step boolean evaluations (maximum two operands, zero mixed polarity), eliminates code duplication, and defines comprehensive test verification gates.

---

## 2. Affirmative Predicate Helper Architecture

All affirmative predicate helpers are declared as `pub(crate)` in `src-tauri/src/modules/repo_db/tree_node_conv.rs` (or `tree_collect.rs`) and shared across `tree_collect.rs`, `tree_node_conv.rs`, and `tree_phases.rs`.

### 2.1 `is_default_instance_match`
- **Signature:**
  ```rust
  pub(crate) fn is_default_instance_match(inst_id: &str) -> bool
  ```
- **Rationale:**
  Replaces repetitive ad-hoc comparisons against `"default"`, `"__default__"`, empty strings, and clone aliases (`"default-copy*"`). Consolidates the canonical definition of default instance identity across tree collection and phase filtering.
- **Decomposed Implementation:**
  ```rust
  pub(crate) fn is_default_instance_match(inst_id: &str) -> bool {
      let is_exact_default = inst_id == "default" || inst_id == "__default__";
      let is_alias_default = inst_id.is_empty() || inst_id.starts_with("default-copy");
      is_exact_default || is_alias_default
  }
  ```

### 2.2 `is_instance_compatible`
- **Signature:**
  ```rust
  pub(crate) fn is_instance_compatible(
      inst_id: &str,
      norm_proj_inst: &str,
      is_proj_default: bool,
  ) -> bool
  ```
- **Rationale:**
  Replaces the 5-branch mixed `&&`/`||` condition in `tree_node_conv.rs:81-86`. Evaluates whether a candidate conversation's owning instance matches the project's target instance, honoring default alias equivalence.
- **Decomposed Implementation:**
  ```rust
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

### 2.3 `is_vacant_ghost_conversation`
- **Signature:**
  ```rust
  pub(crate) fn is_vacant_ghost_conversation(
      title: &str,
      prompt: &str,
      word_count: usize,
      has_zero_turns: bool,
  ) -> bool
  ```
- **Rationale:**
  Unifies ghost conversation identification across `tree_collect.rs:246-255` and `tree_node_conv.rs:101-106, 311-316`. Replaces mixed `&&`/`||` compound logic with early positive exits.
- **Decomposed Implementation:**
  ```rust
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

### 2.4 `should_retain_unrunning_conversation`
- **Signature:**
  ```rust
  pub(crate) fn should_retain_unrunning_conversation(
      only_running: bool,
      is_running: bool,
  ) -> bool
  ```
- **Rationale:**
  Replaces mixed-polarity `if only_running && !is_running { continue; }` guards at call sites with an affirmative filter retention check.
- **Decomposed Implementation:**
  ```rust
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

### 2.5 `should_retain_prompt_candidate`
- **Signature:**
  ```rust
  pub(crate) fn should_retain_prompt_candidate(
      is_non_prompt: bool,
      is_conv_running: bool,
      has_meaningful_title: bool,
      has_prompt_content: bool,
  ) -> bool
  ```
- **Rationale:**
  Decomposes the 4-clause compound negative check in `tree_collect.rs:210-216` (`inspection.is_non_prompt && !is_conv_running && !has_meaningful_title && !has_prompt_content`) into a readable affirmative retention rule.
- **Decomposed Implementation:**
  ```rust
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

### 2.6 `is_owning_instance_alive`
- **Signature:**
  ```rust
  pub(crate) fn is_owning_instance_alive(
      owning_inst_id: &str,
      registry: &crate::models::instance::InstanceRegistry,
  ) -> bool
  ```
- **Rationale:**
  Eliminates identical 20-line copy-pasted blocks between `tree_collect.rs:103-120` and `tree_phases.rs:117-134`. Enforces positive PID check logic and isolates system-level process queries.
- **Decomposed Implementation:**
  ```rust
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

## 3. Comprehensive Violation Site Catalog & Decomposed Forms

### 3.1 `src-tauri/src/modules/repo_db/tree_collect.rs`

#### Site TC-01: Default Instance Resolution (Lines 61-66 & Line 153)
- **Original Code:**
  ```rust
  let inst = if ap.instance_id == "default" || ap.instance_id == "__default__" {
      "default".to_string()
  } else { ... };
  ...
  let norm_owning_inst = if owning_inst_id == "__default__" || owning_inst_id.is_empty() {
      "default".to_string()
  } else { ... };
  ```
- **Violation:** P4a chained string comparisons and duplicated default alias checks.
- **Decomposed Form:**
  ```rust
  let inst = if is_default_instance_match(&ap.instance_id) {
      "default".to_string()
  } else {
      crate::modules::instance::resolve_instance_id(&ap.instance_id)
          .unwrap_or_else(|_| ap.instance_id.clone())
  };
  ...
  let norm_owning_inst = if is_default_instance_match(owning_inst_id) {
      "default".to_string()
  } else {
      crate::modules::instance::resolve_instance_id(owning_inst_id)
          .unwrap_or_else(|_| owning_inst_id.to_string())
  };
  ```

#### Site TC-02: Owning Instance Liveness Detection (Lines 103-120)
- **Original Code:**
  ```rust
  let is_owning_inst_alive = if owning_inst_id == "default" || owning_inst_id == "__default__" { ... };
  ```
- **Violation:** P3/P4b raw negation `!find_pids(...).is_empty()`, mixed `||` and closures, logic duplicated in `tree_phases.rs`.
- **Decomposed Form:**
  ```rust
  let registry = crate::modules::instance::registry::load_registry().unwrap_or_default();
  let is_owning_inst_alive = is_owning_instance_alive(owning_inst_id, &registry);
  ```

#### Site TC-03: Compound Running State Evaluation (Lines 172-187)
- **Original Code:**
  ```rust
  let is_conv_running = if is_owning_inst_alive
      && inspection.is_recent_active
      && !inspection.is_terminal_done
  {
      true
  } else if is_explicit_idle || inspection.is_terminal_done {
      false
  } else if is_owning_inst_alive && not_fully_idle > 0 && is_recent {
      true
  } else {
      false
  };
  ```
- **Violation:** P4a 3-clause conjunction with mixed polarity (`!inspection.is_terminal_done`) and 3-clause conjunction at line 183.
- **Decomposed Form:**
  ```rust
  let is_terminal = is_explicit_idle || inspection.is_terminal_done;
  let is_live_active = inspection.is_recent_active && !inspection.is_terminal_done;
  let has_live_supremacy = is_owning_inst_alive && is_live_active;

  let has_activity_marker = not_fully_idle > 0 && is_recent;
  let is_active_fallback = is_owning_inst_alive && has_activity_marker;

  let is_conv_running = if has_live_supremacy {
      true
  } else if is_terminal {
      false
  } else {
      is_active_fallback
  };
  ```

#### Site TC-04: Meaningful Title Extraction (Lines 198-204)
- **Original Code:**
  ```rust
  let has_meaningful_title = !title.trim().is_empty()
      && !title.to_lowercase().starts_with("untitled")
      && title.to_lowercase() != "new conversation";

  if effective_prompt.trim().is_empty() && has_meaningful_title {
      effective_prompt = title.clone();
  }
  ```
- **Violation:** P4a/P4c 3 chained negative conditions followed by mixed polarity expression.
- **Decomposed Form:**
  ```rust
  let title_clean = title.trim();
  let title_lower = title_clean.to_lowercase();
  let is_placeholder_title = title_lower.starts_with("untitled") || title_lower == "new conversation";
  let has_meaningful_title = !title_clean.is_empty() && !is_placeholder_title;

  let is_effective_prompt_empty = effective_prompt.trim().is_empty();
  if is_effective_prompt_empty && has_meaningful_title {
      effective_prompt = title.clone();
  }
  ```

#### Site TC-05: Non-Prompt Discard Guard (Lines 210-216)
- **Original Code:**
  ```rust
  if inspection.is_non_prompt
      && !is_conv_running
      && !has_meaningful_title
      && !has_prompt_content
  {
      continue;
  }
  ```
- **Violation:** P4a 4-clause conjunction with 3 stacked negations.
- **Decomposed Form:**
  ```rust
  let should_retain = should_retain_prompt_candidate(
      inspection.is_non_prompt,
      is_conv_running,
      has_meaningful_title,
      has_prompt_content,
  );
  if !should_retain {
      continue;
  }
  ```

#### Site TC-06: Ghost Conversation Skip (Lines 246-255)
- **Original Code:**
  ```rust
  let is_untitled_candidate = title.trim().is_empty()
      || title.to_lowercase().starts_with("untitled")
      || title.to_lowercase() == "new conversation";
  let (_, eff_wc) = extract_prompt_words_preview(&effective_prompt, 5);
  if is_untitled_candidate
      && (effective_prompt.trim().is_empty() || eff_wc == 0)
      && prompt_turns.is_empty()
  {
      continue;
  }
  ```
- **Violation:** P4a/P4b mixed `&&` and `||` in a compound 3-clause expression.
- **Decomposed Form:**
  ```rust
  let (_, eff_wc) = extract_prompt_words_preview(&effective_prompt, 5);
  let has_zero_turns = prompt_turns.is_empty();
  let is_ghost = is_vacant_ghost_conversation(
      &title,
      &effective_prompt,
      eff_wc,
      has_zero_turns,
  );
  if is_ghost {
      continue;
  }
  ```

#### Site TC-07: Path Prefix Length Guard (Lines 269-276)
- **Original Code:**
  ```rust
  let is_conv_running = if is_conv_running
      && !assigned_paths.is_empty()
      && !assigned_paths.iter().any(|p| p.len() >= 6)
  {
      false
  } else {
      is_conv_running
  };
  ```
- **Violation:** P4a 3-clause conjunction and P4c raw negations on collection queries.
- **Decomposed Form:**
  ```rust
  let has_assigned_paths = !assigned_paths.is_empty();
  let has_valid_prefix = assigned_paths.iter().any(|p| p.len() >= 6);
  let has_invalid_paths = has_assigned_paths && !has_valid_prefix;

  let is_conv_running = if is_conv_running && has_invalid_paths {
      false
  } else {
      is_conv_running
  };
  ```

---

### 3.2 `src-tauri/src/modules/repo_db/tree_node_conv.rs`

#### Site NC-01: Default Instance Meta Resolution (Lines 19 & 32)
- **Original Code:**
  ```rust
  if instance_id == "default" || instance_id == "__default__" { ... }
  ...
  if inst.is_default || inst.id == "default" || inst.id == "__default__" { ... }
  ```
- **Violation:** P4a 3 chained `||` operands at line 32.
- **Decomposed Form:**
  ```rust
  if is_default_instance_match(instance_id) { ... }
  ...
  let is_inst_default = inst.is_default || is_default_instance_match(&inst.id);
  if is_inst_default { ... }
  ```

#### Site NC-02: Instance Compatibility Matching (Lines 72-75 & 81-86)
- **Original Code:**
  ```rust
  let is_proj_default = norm_proj_inst == "default"
      || norm_proj_inst == "__default__"
      || norm_proj_inst.starts_with("default-copy");

  let matches_inst = inst_id == norm_proj_inst
      || (is_proj_default
          && (inst_id == "default"
              || inst_id == "__default__"
              || inst_id.starts_with("default-copy")));
  ```
- **Violation:** P4a/P4b mixed `&&` and `||` with 5 total conditional branches.
- **Decomposed Form:**
  ```rust
  let is_proj_default = is_default_instance_match(norm_proj_inst);
  let matches_inst = is_instance_compatible(inst_id, norm_proj_inst, is_proj_default);
  ```

#### Site NC-03: Untitled / Empty Conversation Skip (Lines 101-106 & 311-316)
- **Original Code:**
  ```rust
  let is_untitled =
      item.title.trim().is_empty() || item.title.to_lowercase().starts_with("untitled");
  let is_empty_prompt = item.prompt.trim().is_empty() || word_count == 0;
  if is_untitled && is_empty_prompt {
      continue;
  }
  ```
- **Violation:** Duplicated ghost conversation check.
- **Decomposed Form:**
  ```rust
  let is_ghost = is_vacant_ghost_conversation(&item.title, &item.prompt, word_count, true);
  if is_ghost {
      continue;
  }
  ```

#### Site NC-04: Running Filter Retention Guards (Lines 109-111 & 401-403)
- **Original Code:**
  ```rust
  let effective_is_run = item.is_running;
  if only_running && !effective_is_run {
      continue;
  }
  ...
  if only_running && !is_run {
      continue;
  }
  ```
- **Violation:** P4c mixed polarity (`only_running && !is_run`).
- **Decomposed Form:**
  ```rust
  let should_retain = should_retain_unrunning_conversation(only_running, item.is_running);
  if !should_retain {
      continue;
  }
  ...
  let should_retain_ap = should_retain_unrunning_conversation(only_running, is_run);
  if !should_retain_ap {
      continue;
  }
  ```

#### Site NC-05: Conversation Target Match Search (Lines 216-224)
- **Original Code:**
  ```rust
  let target_idx = conv_nodes.iter().position(|c| {
      c.conversation_id == cid
          || c.conversation_id.eq_ignore_ascii_case(&cid)
          || c.prompts.iter().any(|p| {
              p.prompt_text.trim() == ap.prompt_content.trim()
                  || p.preview_200w.trim() == ap.prompt_content.trim()
          })
          || c.full_prompt_text.as_deref().unwrap_or("").trim()
              == ap.prompt_content.trim()
  });
  ```
- **Violation:** P4a 4-clause `||` chain with nested closures and multi-operator conditions.
- **Decomposed Form:**
  ```rust
  let target_idx = conv_nodes.iter().position(|c| {
      let is_id_match = c.conversation_id == cid || c.conversation_id.eq_ignore_ascii_case(&cid);
      if is_id_match {
          return true;
      }
      let content_trimmed = ap.prompt_content.trim();
      let has_matching_prompt = c.prompts.iter().any(|p| {
          let is_text_match = p.prompt_text.trim() == content_trimmed;
          let is_preview_match = p.preview_200w.trim() == content_trimmed;
          is_text_match || is_preview_match
      });
      if has_matching_prompt {
          return true;
      }
      c.full_prompt_text.as_deref().unwrap_or("").trim() == content_trimmed
  });
  ```

#### Site NC-06: Active Prompt Status & Running Checks (Lines 228-234 & 238-243)
- **Original Code:**
  ```rust
  if ap_status_lower.contains("queue")
      || ap_status_lower.contains("running")
      || ap_status_lower.contains("pending")
      || ap_status_lower.contains("dispatched")
      || ap_status_lower.contains("in_flight")
  ...
  let is_ap_run = is_inst_alive
      && is_fresh
      && (ap_status_lower.contains("running")
          || ap_status_lower.contains("dispatched")
          || ap_status_lower.contains("in_flight"));
  ```
- **Violation:** P4a 5-clause chained `||` at line 228 and mixed `&&`/`||` 4-condition conjunction at line 238.
- **Decomposed Form:**
  ```rust
  let is_pending_stage = ap_status_lower.contains("queue") || ap_status_lower.contains("pending");
  let is_active_stage = ap_status_lower.contains("running")
      || ap_status_lower.contains("dispatched")
      || ap_status_lower.contains("in_flight");
  let is_relevant_status = is_pending_stage || is_active_stage;

  if is_relevant_status {
      let is_queued = is_pending_stage;
      let is_fresh = (now - ap.updated_at) <= 600;
      let is_alive_and_fresh = is_inst_alive && is_fresh;
      let is_ap_run = is_alive_and_fresh && is_active_stage;
      ...
  }
  ```

#### Site NC-07: Queued Status State Updates (Lines 260-262 & 300)
- **Original Code:**
  ```rust
  if !target_conv.is_running {
      target_conv.is_queued = true;
      target_conv.status = "QUEUED".to_string();
  }
  ...
  else if is_queued && !target_conv.is_running {
  ```
- **Violation:** P4c mixed polarity `is_queued && !target_conv.is_running`.
- **Decomposed Form:**
  ```rust
  let can_enqueue = is_queued && !target_conv.is_running;
  if can_enqueue {
      target_conv.is_queued = true;
      target_conv.status = "QUEUED".to_string();
  }
  ```

#### Site NC-08: Active AGY Worker Detection (Lines 325-337)
- **Original Code:**
  ```rust
  let is_inst_match =
      if proj.instance_id == "default" || proj.instance_id == "__default__" {
          w_inst == "default" || w_inst == "__default__"
      } else {
          w_inst.eq_ignore_ascii_case(&proj.instance_id)
      };
  let is_path_match = !clean_target.is_empty()
      && normalize_path_for_compare(w_path) == clean_target;
  if is_inst_match && is_path_match && wpid > 0 {
      crate::modules::instance::is_pid_alive_os(wpid)
  }
  ```
- **Violation:** P4c in `is_path_match` (`!clean_target.is_empty() && ...`), P4a in `is_inst_match && is_path_match && wpid > 0`.
- **Decomposed Form:**
  ```rust
  let is_inst_match = if is_default_instance_match(&proj.instance_id) {
      is_default_instance_match(w_inst)
  } else {
      w_inst.eq_ignore_ascii_case(&proj.instance_id)
  };
  let has_clean_target = !clean_target.is_empty();
  let is_path_equal = normalize_path_for_compare(w_path) == clean_target;
  let is_path_match = has_clean_target && is_path_equal;

  let is_target_match = is_inst_match && is_path_match;
  let has_valid_pid = wpid > 0;
  if is_target_match && has_valid_pid {
      crate::modules::instance::is_pid_alive_os(wpid)
  } else {
      false
  }
  ```

#### Site NC-09: Non-Idle Turn Verification (Lines 375-377)
- **Original Code:**
  ```rust
  let is_cascade_running = cascade_status.is_running();
  let is_active_turn =
      is_cascade_running && not_idle > 0 && !has_idle_marker;
  ```
- **Violation:** P4a/P4c 3 operands with mixed polarity (`!has_idle_marker`).
- **Decomposed Form:**
  ```rust
  let is_cascade_running = cascade_status.is_running();
  let has_busy_flag = not_idle > 0;
  let is_turn_busy = is_cascade_running && has_busy_flag;
  let is_active_turn = is_turn_busy && !has_idle_marker;
  ```

#### Site NC-10: Active Prompt In-Flight Running State (Lines 395-400)
- **Original Code:**
  ```rust
  let is_run = is_inst_alive
      && (ap.status == "running"
          || ap.status == "in_flight"
          || ap.status == "dispatched")
      && (now - ap.updated_at <= max_allowed_ttl)
      && has_confirmed_running;
  ```
- **Violation:** P4a/P4b 4 compound operands with nested 3-clause `||`.
- **Decomposed Form:**
  ```rust
  let is_running_status = ap.status == "running"
      || ap.status == "in_flight"
      || ap.status == "dispatched";
  let is_within_ttl = (now - ap.updated_at) <= max_allowed_ttl;
  let is_status_fresh = is_running_status && is_within_ttl;
  let is_executing = is_inst_alive && has_confirmed_running;
  let is_run = is_status_fresh && is_executing;
  ```

---

### 3.3 `src-tauri/src/modules/repo_db/tree_phases.rs`

#### Site TP-01: Query Tree Projects Target Filter (Lines 66-84)
- **Original Code:**
  ```rust
  let is_target_default = target_id == "default"
      || target_id == "__default__"
      || norm_target == "default"
      || norm_target == "default-copy-5597";
  all_projects
      .into_iter()
      .filter(|p| {
          if p.instance_id == target_id || p.instance_id == norm_target {
              return true;
          }
          if is_target_default
              && (p.instance_id == "default"
                  || p.instance_id == "__default__"
                  || p.instance_id == "default-copy-5597")
          {
              return true;
          }
          false
      })
  ```
- **Violation:** P4a 4 chained `||` operands in `is_target_default`. P4b mixed `&&` and nested 3-clause `||` in filter closure.
- **Decomposed Form:**
  ```rust
  let is_target_default = is_default_instance_match(target_id)
      || is_default_instance_match(&norm_target);
  all_projects
      .into_iter()
      .filter(|p| {
          let is_exact_match = p.instance_id == target_id || p.instance_id == norm_target;
          if is_exact_match {
              return true;
          }
          let is_p_default = is_default_instance_match(&p.instance_id);
          is_target_default && is_p_default
      })
      .collect()
  ```

#### Site TP-02: Project Identifier Emptiness Guard (Line 89)
- **Original Code:**
  ```rust
  .filter(|p| !p.instance_id.trim().is_empty())
  ```
- **Violation:** P3 raw negation on string emptiness test.
- **Decomposed Form:**
  ```rust
  .filter(|p| {
      let is_defined = !p.instance_id.trim().is_empty();
      is_defined
  })
  ```

#### Site TP-03: Fallback Discovery Liveness Check (Lines 117-134)
- **Original Code:**
  ```rust
  let is_owning_inst_alive = if owning_inst_id == "default" || owning_inst_id == "__default__" { ... };
  ```
- **Violation:** Exact duplicate of `tree_collect.rs:103-120`.
- **Decomposed Form:**
  ```rust
  let registry = crate::modules::instance::registry::load_registry().unwrap_or_default();
  let is_owning_inst_alive = is_owning_instance_alive(owning_inst_id, &registry);
  ```

#### Site TP-04: Fallback Conversation Running State (Lines 173-177)
- **Original Code:**
  ```rust
  let is_running = !cascade_status.is_idle_or_terminal()
      && is_owning_inst_alive
      && cascade_status.is_running()
      && not_fully_idle > 0
      && is_recent;
  ```
- **Violation:** P4a 5-clause conjunction with P4c raw negation `!cascade_status.is_idle_or_terminal()`.
- **Decomposed Form:**
  ```rust
  let is_active_status = cascade_status.is_running();
  let is_terminal = cascade_status.is_idle_or_terminal();
  let is_status_runnable = is_active_status && !is_terminal;

  let has_activity = not_fully_idle > 0 && is_recent;
  let is_node_active = is_owning_inst_alive && has_activity;

  let is_running = is_status_runnable && is_node_active;
  ```

#### Site TP-05: Target Filter Discrimination (Lines 251-267)
- **Original Code:**
  ```rust
  let target_clean = target_id.trim();
  if !target_clean.is_empty() && !target_clean.eq_ignore_ascii_case("all") {
      let norm_target =
          if target_clean == "default" || target_clean == "__default__" {
              "default".to_string()
          } else {
              crate::modules::instance::resolve_instance_id(target_clean)
                  .unwrap_or_else(|_| target_clean.to_string())
          };
      if norm_target == "default" {
          if norm_inst != "default" && norm_inst != "__default__" {
              continue;
          }
      } else if norm_inst != norm_target {
          continue;
      }
  }
  ```
- **Violation:** P4c stacked negative checks (`!is_empty() && !eq_ignore_ascii_case("all")`, `norm_inst != "default" && norm_inst != "__default__"`).
- **Decomposed Form:**
  ```rust
  let target_clean = target_id.trim();
  let has_target = !target_clean.is_empty();
  let is_universal_target = target_clean.eq_ignore_ascii_case("all");
  let is_scoped_target = has_target && !is_universal_target;

  if is_scoped_target {
      let norm_target = if is_default_instance_match(target_clean) {
          "default".to_string()
      } else {
          crate::modules::instance::resolve_instance_id(target_clean)
              .unwrap_or_else(|_| target_clean.to_string())
      };
      let is_target_default = is_default_instance_match(&norm_target);
      if is_target_default {
          let is_norm_default = is_default_instance_match(&norm_inst);
          if !is_norm_default {
              continue;
          }
      } else if norm_inst != norm_target {
          continue;
      }
  }
  ```

#### Site TP-06: Telemetry Probe Default Check (Lines 321-323)
- **Original Code:**
  ```rust
  let (inst_name, data_dir, pids, is_alive) = if target_id == "default"
      || target_id == "__default__"
  ```
- **Violation:** Ad-hoc default string comparison.
- **Decomposed Form:**
  ```rust
  let (inst_name, data_dir, pids, is_alive) = if is_default_instance_match(target_id)
  ```

---

## 4. Blast Radius & System Invariant Analysis

1. **Prompt Tree Assembly & Serialization Parity**:
   - `AgmConversationNode` sequence codes (`C001`, `GM:12345678`), status flags (`RUNNING`, `QUEUED`, `IDLE`), and step summaries must preserve 100% equivalence.
   - Ghost conversation pruning logic remains strictly identical in evaluation: only empty, zero-word untitled conversations with zero turns are pruned.
2. **Process Liveness & Multi-Instance Isolation**:
   - Centralizing `is_owning_instance_alive` prevents drift between tree conversation aggregation and fallback project discovery sweeps.
   - Respects conscious PID checks, data directory resolution, and registry fallbacks.
3. **No File Locking or SQLite Concurrency Regressions**:
   - All connection pragmas (`busy_timeout`) and URI open flags (`SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI`) remain untouched.
   - Best-effort SQLite queries maintain `crate::error::record_ignored` error routing with justification comments.

---

## 5. Verification & Acceptance Criteria

### AC-CG-REPO-001: Affirmative Predicate Helper Completeness
- **Given** The 6 required affirmative helpers:
  - `is_default_instance_match`
  - `is_instance_compatible`
  - `is_vacant_ghost_conversation`
  - `should_retain_unrunning_conversation`
  - `should_retain_prompt_candidate`
  - `is_owning_instance_alive`
- **When** Audited across `src-tauri/src/modules/repo_db/`,
- **Then** Every helper must have explicit return type `bool`, affirmative naming (`is*`, `should*`), and zero call-site duplication.

### AC-CG-REPO-002: Zero Multi-Clause & Mixed-Polarity Violations
- **Given** Refactored files `tree_collect.rs`, `tree_node_conv.rs`, and `tree_phases.rs`,
- **When** Inspected for condition density,
- **Then** Every `if` condition evaluates at most two operands (`&&` or `||`, never mixed) without raw mixed polarities (`is_a && !is_b`).

### AC-CG-REPO-003: Clean Compilation & Targeted Test Suite Execution
- **Given** The Rust codebase under `src-tauri/`,
- **When** Targeted test suites and linters are executed:
  ```bash
  cargo test --lib modules::repo_db
  cargo test --test per_instance_prompt_liveness_test
  cd src-tauri && cargo fmt -- --check
  cd src-tauri && cargo clippy --all-targets --all-features
  ```
- **Then** All commands exit with code 0.
