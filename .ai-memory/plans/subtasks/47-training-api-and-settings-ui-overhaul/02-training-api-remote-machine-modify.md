# Subtask 02: Training REST API & Full IDE Machine Modification

## Objective
Provide comprehensive Machine Training REST API endpoints and reinforce remote node modifications:
1. Expose `/api/v1/training`, `/api/v1/status`, `/api/v1/training/telemetry`, `/api/v1/training/learn`, `/api/v1/training/machines`, `/api/v1/training/modify`, and `/api/v1/machines`.
2. Connect `"switch_account"` action directly to `instance::switch_account_to_instance(&target.id, Some(&inst_id))`, ensuring OAuth tokens, `state.vscdb`, keyring, and IDE process restarts are executed.
3. Connect `"trigger_rotation"` action to `auto_switcher::trigger_manual_rotation_for_instance(req.instance_id.as_deref())`.
4. Ensure all training endpoints are strictly guarded by `training_api_enabled` toggle in AppConfig.

## Status
- [x] Implemented endpoint aliases in `src-tauri/src/proxy/server.rs`.
- [x] Connected full IDE credential injection in `src-tauri/src/modules/training_api.rs`.
- [x] Verified GUI toggle switch exists in General settings.
