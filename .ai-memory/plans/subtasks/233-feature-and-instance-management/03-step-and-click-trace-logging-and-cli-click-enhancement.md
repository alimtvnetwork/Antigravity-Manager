# Subtask 03: UI Step & Click Trace Logging & CLI Click Expansion

## Target Files
- `src/utils/uiLogger.ts`
- `src-tauri/src/commands/cmd_ui_trace.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/bin/agm/ui_cmds.rs`
- `src/hooks/useUiRemoteControl.ts`

## Objectives
1. Create `src/utils/uiLogger.ts`:
   - Export `logUiTrace(event: { step: string; action: string; target: string; metadata?: Record<string, unknown> })`.
   - Intercept action buttons and step transitions across `InstanceCardActions.tsx`, `SwitchAccountDialog.tsx`, and `InstanceRowActions.tsx`.
2. Add Tauri command `record_ui_trace`:
   - In `src-tauri/src/commands/cmd_ui_trace.rs`:
     Logs `tracing::info!(target: "ui_trace", "[UI_STEP] [{}] {} on '{}' (metadata: {:?})", step, action, target, metadata)`.
     Appends audit entry into `crate::modules::audit::append_entry`.
   - Register in `src-tauri/src/commands/mod.rs`.
3. Enhance `src-tauri/src/bin/agm/ui_cmds.rs`:
   - Expand `resolve_element_target` to permit dynamic instance targets (`instance-switch-*`, `instance-launch-*`, `instance-stop-*`, `instance-restart-*`, `instance-more-*`).
   - Add alias `agm instances click <instance_id> <action>` routing to `agm ui click`.
4. Enhance `src/hooks/useUiRemoteControl.ts`:
   - Query selector matches `getElementById(id)` || `[data-ui-id="${id}"]` || `[data-ui-name="${id}"]`.
   - Emits `logUiTrace({ step: 'remote_control', action: 'cli_click', target: id })` upon triggering element click.
