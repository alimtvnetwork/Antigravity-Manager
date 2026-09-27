# Subtask 02: Auto-Switcher Pre-Switch Prompt Backup and Post-Switch Re-injection

## Description
In `auto_switcher.rs`, invoke `repo_db::backup_running_prompts(inst_id)` before initiating rotation or profile switch in `execute_profile_rotation_with_context`. Once lease acquisition and rotation succeed, trigger `repo_db::resend_all_running_commands(20)` and `repo_db::dispatch_running_prompts(inst_id)`.

## Actions
- Edit `src-tauri/src/modules/auto_switcher.rs`:
  - Call `repo_db::backup_running_prompts(inst_id)` prior to switching.
  - Call `repo_db::resend_all_running_commands(20)` and `repo_db::dispatch_running_prompts(inst_id)` upon successful rotation.
