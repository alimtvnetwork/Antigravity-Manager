# Component Specification: Update CLI Command and Pipeline Fixes

**Task Slug**: `update-cli-command-and-pipeline-fixes`  
**Target Release**: `v4.186.0`  
**Maintainer Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)  
**Status**: APPROVED

---

## 1. Subsystem 1: Universal Windows Updater Self-Delegation

### Target Files:
- `src-tauri/src/bin/agm/update_cmds.rs`
- `src-tauri/src/modules/delegate_updater/process.rs`
- `src-tauri/src/modules/delegate_updater/run.rs`
- `install.ps1`

### 1.1 Universal Self-Delegation in `update_cmds.rs`
- **Location**: `src-tauri/src/bin/agm/update_cmds.rs` (lines 380–416)
- **Logic**:
  On Windows, when `agm update` is invoked from any location outside `%TEMP%\agm-updater`, regardless of whether `ui_running` is detected as true or false, the CLI process must **never** execute `install.ps1` or run the installer directly in-process.
  Instead:
  ```rust
  #[cfg(target_os = "windows")]
  {
      let current_exe = std::env::current_exe().unwrap_or_default();
      let exe_str = current_exe.to_string_lossy().to_lowercase();
      let is_isolated_temp = exe_str.contains("agm-updater");
      if !is_isolated_temp && !is_json {
          println!("[*] Windows executable lock guard: Self-delegating to isolated updater in %TEMP%...");
          let mut delegate_args = vec![
              "--relaunch".to_string(),
              "--version".to_string(),
              tag_name.clone(),
          ];
          if is_force || has_explicit_target {
              delegate_args.push("--force".to_string());
          }
          antigravity_tools_lib::modules::delegate_updater::run(&delegate_args);
          std::process::exit(0);
      }
  }
  ```
  Calling `std::process::exit(0)` terminates the calling `agm.exe` process immediately so Windows releases all file locks on `%LOCALAPPDATA%\agm-cli\agm.exe` and `InstallDir\agm.exe`.

### 1.2 Process Taxonomy Expansion in `process.rs`
- **Location**: `src-tauri/src/modules/delegate_updater/process.rs:57`
- **Logic**:
  Expand `process_names` to include canonical production executable names:
  ```rust
  let process_names = [
      "agm-alim.exe",
      "antigravity-tools.exe",
      "antigravity-manager.exe",
      "agm.exe",
  ];
  ```

### 1.3 Installer Lock Candidate Expansion in `install.ps1`
- **Location**: `install.ps1` lines 275–281 & 1577–1583
- **Logic**:
  Add `"antigravity-manager"` and `"agm"` to `$processNames`.
  In `$lockCandidates`:
  ```powershell
  $lockCandidates = @(
      (Join-Path $InstallDir $BinaryName),
      (Join-Path $InstallDir "agm-alim.exe"),
      (Join-Path $InstallDir "AGM by Alim.exe"),
      (Join-Path $InstallDir "Anti-Gravity Tools by Alim.exe"),
      (Join-Path $InstallDir "antigravity-tools.exe"),
      (Join-Path $InstallDir "antigravity-manager.exe"),
      (Join-Path $InstallDir "agm.exe"),
      (Join-Path $cliDir "agm.exe"),
      (Join-Path $cliDir "agm-update-cli.exe")
  )
  ```

---

## 2. Subsystem 2: Prompt Pipeline Dispatch & Queue (QE) Options

### Target Files:
- `src-tauri/src/bin/agm/prompt_cmds.rs`
- `src-tauri/src/bin/agm/instance_cmds.rs`
- `src-tauri/src/modules/repo_db/agy.rs`
- `src-tauri/src/modules/repo_db/queue.rs`

### 2.1 CLI Flags & Resolution in `prompt_cmds.rs`
- Support flags in `cmd_prompts_send` and `cmd_prompts_queue`:
  - `--conversation <id>` / `-c <id>`: Target an explicit conversation ID.
  - `--existing`: Pick the most recent conversation associated with the repository.
  - `--new`: Explicitly create a new conversation (`conv-<uuid>`).
  - `--repo <path>`: Default to `std::env::current_dir()` instead of `"scratch/test-repo"`.
  - Alias `qe`: Map `qe` directly to `cmd_prompts_queue`.
- In `cmd_prompts_send`:
  Actually invoke `repo_db::send_prompt_now_for_instance` or trigger `spawn_prompt_via_agy` with fallback to `.antigravity_resume_task.json` + process focus.
- In `cmd_prompts_queue`:
  Retain `session_id` in SQLite `active_prompts` when specified.

### 2.2 CLI `agm instance prompt` Routing
- In `src-tauri/src/bin/agm/instance_cmds.rs`:
  Route `"prompt"` subcommand in `dispatch_instance_domain` to `crate::prompt_dispatch::cmd_prompt_dispatch(args)`.

### 2.3 Tier 2 Graceful Degradation in `agy.rs`
- In `src-tauri/src/modules/repo_db/agy.rs`:
  When `spawn_prompt_via_agy` returns `false` due to missing `agy` CLI binary, do not fail hard.
  Instead:
  1. Write `.antigravity_resume_task.json` to workspace root.
  2. Copy prompt to system clipboard.
  3. Ensure instance is running or focus window via `crate::modules::instance::ensure_instance_running_for_dispatch`.
  4. Return success `Ok(active_prompt)` with informative logging.

### 2.4 Queue Drain Protection in `queue.rs`
- In `src-tauri/src/modules/repo_db/queue.rs`:
  In `check_and_dispatch_enqueued_prompts`:
  Only transition `status = 'dispatched'` if `spawn_prompt_via_agy` returned true OR fallback task file was verified on disk. If neither succeeded, retain `status = 'queued'`.

---

## 3. Subsystem 3: Prompt Backup & Cross-Instance Restore on Switch & Rotation

### Target Files:
- `src-tauri/src/modules/repo_db/backup.rs`
- `src-tauri/src/modules/auto_switcher/rotation.rs`
- `src-tauri/src/modules/repo_db/auto_resume.rs`

### 3.1 Status Expansion in `backup.rs`
- Transition all active in-flight prompt statuses to `'backed_up'`:
  ```sql
  UPDATE active_prompts 
  SET status = 'backed_up', updated_at = ?1 
  WHERE status IN ('running', 'in_flight', 'dispatched') 
    AND instance_id = ?2
  ```

### 3.2 Cross-Instance Re-Attribution in `rotation.rs`
- When `target.instance_id != current_instance_id`:
  Before shutting down or starting the new instance, re-attribute backed-up prompts from `current_instance_id` to `target.instance_id`:
  ```rust
  if let Ok(conn) = crate::modules::repo_db::connect_db() {
      let _ = conn.execute(
          "UPDATE active_prompts SET instance_id = ?1 WHERE instance_id = ?2 AND status = 'backed_up'",
          rusqlite::params![&target.instance_id, &current_instance_id],
      );
  }
  ```

### 3.3 Inclusion of Backed-Up Prompts in `auto_resume.rs`
- In `auto_resume_recent_prompts`:
  Query `WHERE status IN ('queued', 'pending', 'backed_up')`.

---

## 4. Subsystem 4: UI Conversation Target Choice

### Target Files:
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`

### 4.1 UI Conversation Mode Selector in `DetailHeader.tsx`
- Mounted inside Capsule 2 (Execution & Workflow) using borderless inline select styling:
  ```tsx
  <select
      value={conversationTargetMode}
      onChange={(e) => setConversationTargetMode(e.target.value as 'current' | 'new')}
      className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[95px] truncate pr-1 py-0"
      title="Target destination conversation for Send and Queue"
  >
      <option value="current" className="bg-white dark:bg-[#0c2438]">Existing Conv</option>
      <option value="new" className="bg-white dark:bg-[#0c2438]">New Conv</option>
  </select>
  ```

### 4.2 Handling Target Mode in `usePromptTreePromptActions.ts`
- When `conversationTargetMode === 'current'`, pass `conv?.conversation_id`.
- When `conversationTargetMode === 'new'`, pass `undefined` to create a fresh conversation session.
