# Subtask 02 — Instance Cloning: Deep Copy Settings, Themes, and Projects

## Status: Pending

## Scope
1. Update `src-tauri/src/modules/instance.rs`:
   - In `copy_instance`, call `detect_running_projects` on the source instance before copying repo rows.
   - Deep copy `User/settings.json`, `User/keybindings.json`, `User/snippets/`, `User/globalStorage/`, and `User/workspaceStorage/`.
   - On Windows, copy `source_home/AppData/Roaming/Antigravity/User/` to `target_home/AppData/Roaming/Antigravity/User/` and to `target_data_dir/User/`.
   - If copying from default instance on Windows, read `%APPDATA%\Antigravity\User\` as source.
   - Ensure `extensions_dir` from source is retained/inherited.
2. Update `src-tauri/src/modules/repo_db.rs`:
   - In `clone_instance_repo_rows` / `clone_repo_rows_on`:
     - Match projects mapped to both `default` and `__default__` when `source_id == "default"`.
     - Insert cloned rows into `running_projects` and `active_prompts` with target instance ID.
