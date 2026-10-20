# Subtask 01: Running Prompts Inspection & Instance Hierarchy CLI

**Parent Plan**: `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md`  
**Target Files**:  
- `src-tauri/src/bin/agm/main.rs`  
- `src-tauri/src/bin/agm/running_prompts_cmds.rs`  

---

## 1. Scope & Objectives
1. Fix command routing in `src-tauri/src/bin/agm/main.rs` around line 311 so that `agm running`, `agm running --json`, `agm active`, and `agm running-prompts` cleanly route to `crate::running_prompts_cmds::cmd_running_prompts(&cmd_args)`.
2. Upgrade `src-tauri/src/bin/agm/running_prompts_cmds.rs` to support multi-instance discovery (`#1`, `#2`), assigning Dual Project Sequences (`[AGM:P001 | GM:#1]`) and Conversation Sequences (`[AGM:C001 | GM:<cid>]`).
3. Integrate multimodal image extraction in running prompts, reporting exact image counts (Base64 data URIs + file path attachments).
4. Implement Two-Tier JSON Envelope v2.0 schema output when `--json` is supplied.

---

## 2. Invariants & Guardrails
- **Strict Relative Git Paths Invariant**: All file references in plans, code, and comments must be strictly repository-relative (e.g., `src-tauri/...`). Never emit absolute paths (`D:\...`, `C:\...`).
- **No Silent Errors Invariant**: Database reads (`repo_db`, `backup_prompts_db`) and instance registry loads must propagate errors or use `crate::error::record_ignored` with explicit justification.
- **Caller IDE Self-Preservation**: When running inspection or switching operations, verify caller process PID/instance ID to prevent disturbing the active developer session.

---

## 3. Detailed Implementation Steps

### Step 1: Update CLI Main Domain Routing (`src-tauri/src/bin/agm/main.rs`)
- Locate domain dispatch match around line 311:
  ```rust
  // Replace current active/running dispatch:
  "active" | "running" | "running-prompts" | "wpr" => {
      crate::running_prompts_cmds::cmd_running_prompts(&cmd_args);
  }
  ```
- Ensure flags like `agm running --json`, `agm running -i default`, `agm running --words 150` pass directly into `cmd_running_prompts` without being diverted to `prompt_cmds::dispatch_prompt_domain("tree", &[])` or `misc_cmds::cmd_agy`.

### Step 2: Argument Parsing in `src-tauri/src/bin/agm/running_prompts_cmds.rs`
- In `cmd_running_prompts(args: &[String])`:
  - Retain subcommand routing for `help`, `backup`, `restore`, `export`, `import`.
  - Parse CLI flags:
    - `--json`: Output Two-Tier JSON Envelope v2.0.
    - `--full`: Display complete prompt text without word truncation.
    - `--limit`, `-l <Y>`: Maximum prompts displayed per instance (default: 8).
    - `--words`, `--wordcount`, `--wc`, `-w <N>`: Explicit preview word count override.
    - `--instance`, `-i <spec>`: Restrict inspection to a specific instance ID or sequence number.
  - Read default word limit from `AppConfig` via `crate::modules::config::get_config()`:
    - Use `cfg.prompt_preview_max_words` (clamped to `100..=200`).
    - Fallback to `100` if configuration cannot be loaded.
    - Explicit `--words <N>` overrides configuration.

### Step 3: Instance Discovery & Project Tree Matching
- Load registered instances via `crate::modules::instance::load_registry()`.
- Assign 1-based instance sequence numbers:
  - Instance `#1` = Default instance (`is_default == true` or `id == "default"`).
  - Instance `#2`, `#3` ... = Secondary registered instances.
- If `--instance <spec>` is specified:
  - Resolve instance via `crate::modules::instance::resolve_instance_id(spec)`.
  - Filter list to include only the targeted instance.
- For each instance:
  - Query active/running prompts from `repo_prompts.db` via `repo_db::list_running_prompts_for_instance(&inst.id)` or `repo_db::list_all_prompts()`.
  - Query project conversation tree via `repo_db::get_project_conversation_tree_cached(Some(&inst.id), 200, false, false)` to resolve:
    - Project sequence code (`seq_code`, e.g. `AGM:P001`).
    - GitMap project sequence (`gitmap_seq_code`, e.g. `GM:#1`).
    - Conversation sequence code (`AGM:C001`).
    - GitMap conversation sequence (`GM:<cid>`).

### Step 4: Multimodal Image Count Extraction & Word Truncation
- For each running prompt item:
  - Clean prompt text using `repo_db::extract_clean_user_prompt(&p.prompt_content)`.
  - Truncate prompt words:
    - If `--full`: use un-truncated prompt.
    - Else: invoke `crate::modules::repo_db::text_utils::truncate_words_bounded(&cleaned_text, effective_max_words)`.
  - Extract image metrics using `repo_db::extract_image_uris_or_paths(&p.prompt_content)`:
    - `data_uris`: Base64 data URIs count (`image_uris_count`).
    - `found_paths`: Local image files count (`image_paths_count`).
    - `total_image_count = data_uris.len() + found_paths.len()`.
    - `has_images = total_image_count > 0 || p.image_payload.is_some()`.
    - Collect image file paths into `image_paths: Vec<String>`.

### Step 5: Format Two-Tier JSON Envelope v2.0 (`--json`)
- When `is_json` is true, assemble the envelope:
  - `attributes`:
    - `type`: `"agm/running-prompts"`
    - `version`: `"2.0"`
    - `source`: `"agm-cli"`
    - `workDirectory`: `{ "path": "${workDir}", "isApplied": true, "isEnforced": false }`
    - `command`: `"agm running --json"`
  - `variables`:
    - `workDir`: `"workspace"`
    - `repoDir`: `"${workDir}/antigravity-manager"`
  - `data`:
    - `total_instances`: count of scanned instances.
    - `total_running_prompts`: total active prompts found.
    - `word_limit`: effective word limit applied.
    - `instances`: array of instance objects containing `instance_seq`, `instance_id`, `instance_name`, `is_active`, and `prompts` array.
- Serialize with `serde_json::to_string_pretty`.

### Step 6: Format Human Tabular Terminal Output
- When `is_json` is false:
  - Output header: `\n=== AGM Running Prompts Across Instances (Word Preview Limit: {}) ===\n`.
  - For each instance:
    - Render instance banner: `[Instance #{}: {} ({}) - {}]` (e.g. `[Instance #1: Default (default) - ACTIVE]`).
    - If no prompts: print `No active or queued prompts running.`
    - If prompts exist: print table with columns:
      `{:<5} {:<24} {:<24} {:<10} {:<12} {:<8} PROMPT PREVIEW`
      `SEQ`, `PROJECT SEQ`, `CONV SEQ`, `STATUS`, `IMAGES`, `WORDS`, `PROMPT PREVIEW`.
    - Column formatting:
      - `IMAGES`: render `None` or `N imgs` (e.g. `2 imgs`).
      - `WORDS`: total word count of the full prompt.
      - `PROMPT PREVIEW`: word-bounded snippet with ellipsis.

---

## 4. Verification & Validation Steps
1. Execute `cargo check --bin agm` to ensure clean compilation.
2. Run `agm running` without arguments and verify human table output with instance banners and Dual Sequences (`[AGM:P001 | GM:#1]`).
3. Run `agm running --json` and verify valid Two-Tier JSON Envelope v2.0 schema matching `attributes.type == "agm/running-prompts"`.
4. Run `agm running --words 150` and verify prompt snippets expand up to 150 words.
5. Verify test prompt with attached image shows `1 imgs` or `2 imgs` instead of generic `Yes`.
