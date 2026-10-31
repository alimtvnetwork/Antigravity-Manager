# Component Specification: Feature and Instance Management, Idle Notification & UI Telemetry

## 1. Frontend Component Contracts
### 1.1 `InstanceCard.tsx` & `InstanceCardActions.tsx`
- **Props**:
  - `api`: `InstancePageApi`
  - `inst`: `InstanceStatus`
  - `index`: `number`
  - `seqNumber`: `number`
  - `theme`: `InstanceTheme`
- **DOM Identifiers**:
  - `id="instance-card-${inst.config.id}"`
  - `id="instance-launch-${inst.config.id}"` / `data-ui-name="instance-launch-btn"`
  - `id="instance-stop-${inst.config.id}"` / `data-ui-name="instance-stop-btn"`
  - `id="instance-restart-${inst.config.id}"` / `data-ui-name="instance-restart-btn"`
  - `id="instance-switch-${inst.config.id}"` / `data-ui-name="instance-switch-btn"`
  - `id="instance-fastforward-${inst.config.id}"` / `data-ui-name="instance-fastforward-btn"`
  - `id="instance-more-${inst.config.id}"` / `data-ui-name="instance-more-btn"`
- **Styling Specs**:
  - Container: `rounded-[5px] border border-slate-200/80 dark:border-[#40285e]/80 dark:bg-[#171025]/90 hover:dark:bg-[#1f1533] hover:dark:border-[#7c3aed]/60 transition-all duration-200`
  - Switch Account button:
    ```tsx
    className={cn(
      "flex-1 flex items-center justify-center py-1 transition-all cursor-pointer disabled:opacity-50",
      "text-amber-600 dark:text-amber-400 bg-amber-500/10 dark:bg-amber-500/15 border-x border-amber-500/25",
      "hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:!text-black hover:[&_svg]:!text-black hover:font-bold hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.25)]",
      "active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]"
    )}
    ```

### 1.2 Telemetry Utility `src/utils/uiLogger.ts`
- **Interface**:
  ```ts
  export interface UiTraceEvent {
    step: string;
    action: string;
    target: string;
    metadata?: Record<string, unknown>;
  }
  export function logUiTrace(event: UiTraceEvent): void;
  ```
- **Behavior**:
  - Invokes Tauri command `record_ui_trace` asynchronously.
  - Logs locally to console in debug mode with emoji `⚡ [UI_TRACE]`.
  - Captures errors gracefully via `useErrorStore.getState().trackWarning`.

## 2. Backend Module Contracts
### 2.1 Tauri Command `record_ui_trace` (`src-tauri/src/commands/cmd_ui_trace.rs`)
- **Signature**:
  ```rust
  #[tauri::command]
  pub async fn record_ui_trace(
      step: String,
      action: String,
      target: String,
      metadata: Option<serde_json::Value>,
  ) -> Result<(), String>
  ```
- **Logging**:
  - Calls `tracing::info!(target: "ui_trace", "[UI_STEP] [{}] {} -> {} | meta: {:?}", step, action, target, metadata)`.
  - Appends to `crate::modules::audit::append_entry` with `actor="user"`, `method="ui_trace"`, `trigger="interaction"`.

### 2.2 Notification Hub Idle Dispatcher (`src-tauri/src/modules/notification_hub/idle_alerts.rs`)
- **Function**:
  ```rust
  pub async fn dispatch_idle_mode_alert(
      instance_id: &str,
      instance_name: &str,
      idle_seconds: u64,
  ) -> Result<String, String>
  ```
- **Channels**:
  - Dispatches Email via `email_sender::render_idle_projects_email` and `email_sender::dispatch_email_with_failover`.
  - Dispatches Telegram via `telegram_inbound::send_telegram_message_with_markup` with rich HTML card, quick prompt guidance, and instant rerun buttons.
  - Rate-limited with 600s cooldown per `instance_id`.

### 2.3 CLI Command Expansion (`src-tauri/src/bin/agm/ui_cmds.rs`)
- Enhances element resolver `resolve_element_target` to match dynamic IDs:
  - If `element_id.starts_with("instance-")`, permits pattern matching against running/configured instances in `instance::list_instances()`.
  - Routes action directly to `forward_to_local_rest` on `/ui/click`.
