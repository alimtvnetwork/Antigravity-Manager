---
plan: code-dryness-and-guard-inversion
subtask: "01"
title: Constants, Enums & Status Centralization
domain: backend-rust-and-frontend-types
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/code-dryness-and-guard-inversion/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/task_history_db/types.rs
  - src-tauri/src/modules/repo_db/models.rs
  - src-tauri/src/modules/notification_hub/channels.rs
  - src-tauri/src/modules/notification_hub/switch_context.rs
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/modules/scheduler.rs
  - src/types/audit.ts
status: pending
---

# Subtask 01: Constants, Enums & Status Centralization

## 1. Context & Motivation

Across Antigravity Manager (`AGM`), multiple core subsystems currently rely on loose string literals for lifecycle statuses, notification dispatch channels, instance execution modes, and scheduler loop delays:
1. **Task History (`task_history_db`)**: Statuses are tracked using bare string literals (`"queued"`, `"running"`, `"ok"`, `"fail"`). Unhandled drops default to `"fail"` via an ad-hoc string, risking silent typos.
2. **Active Prompts (`repo_db`)**: Prompt execution states rely on scattered strings (`"running"`, `"backed_up"`, `"dispatched"`, `"queued"`, `"pending"`, `"completed"`). Multi-state SQL queries use inline tuple literals (`IN ('running', 'in_flight', 'dispatched')`), which are vulnerable to logic drift.
3. **Notification Hub (`notification_hub`)**: Outbound notification channels are passed as unvalidated strings (`"email"`, `"telegram"`, `"supabase"`), and instance modes are created using inline ternary operators (`if is_default { "default" } else { "isolated" }`).
4. **Default Instance Normalization**: Checking whether an instance is the default workspace (`id == "default" || id == "__default__" || id.is_empty()`) is redundantly repeated in over 25 query locations.
5. **Scheduler Durations**: Background workers in `scheduler.rs` contain at least 8 magic numbers (e.g., `Duration::from_secs(60)`, `Duration::from_secs(300)`, `Duration::from_secs(5)`, `Duration::from_secs(30)`), masking timing guarantees and risking violation of the 30-second minimum polling interval.
6. **Frontend Audit Types**: `TaskRecord.status` in `src/types/audit.ts` is defined as a loose `string`, creating a type gap with backend persistence.

This subtask provides the concrete, step-by-step implementation plan to eliminate these magic strings and numbers, replacing them with strongly-typed enums, centralized normalizers, and explicit duration constants.

---

## 2. Target Files & Symbols Breakdown

| Target File | Target Symbols | Modifications |
| :--- | :--- | :--- |
| `src-tauri/src/modules/task_history_db/types.rs` | `AuditTaskStatus`, `AuditTask` | Define `AuditTaskStatus` enum (`Queued`, `Running`, `Succeeded`, `Failed`) with `.as_str()`, `.is_terminal()`, `.is_in_flight()`, `Display`, and `FromStr`. Update `AuditTask` methods (`start`, `succeed`, `fail`, `finish`, `drop`) to use `AuditTaskStatus`. |
| `src-tauri/src/modules/repo_db/models.rs` | `ActivePromptStatus`, `ACTIVE_PROMPT_*` | Define `ActivePromptStatus` enum (`Running`, `BackedUp`, `Dispatched`, `Queued`, `Pending`, `Completed`) with `.as_str()`, `.is_active()`, `.is_queued_or_pending()`, `.is_terminal()`, `Display`, and `FromStr`. Add slice constants for query sets. |
| `src-tauri/src/modules/notification_hub/channels.rs` | `NotificationChannelType`, `record_channel` | Define `NotificationChannelType` enum (`Email`, `Telegram`, `Supabase`) with `.as_str()` and `.display_name()`. Refactor `record_channel` and `deliver_switch_channels` to use typed enum. |
| `src-tauri/src/modules/notification_hub/switch_context.rs` | `InstanceModeType`, `resolve_switch_context` | Define `InstanceModeType` enum (`Default`, `Isolated`) with `.as_str()`, `.from_bool()`, and `.is_default()`. Update `resolve_switch_context` return tuple. |
| `src-tauri/src/modules/instance.rs` | `DEFAULT_INSTANCE_ID`, `LEGACY_DEFAULT_INSTANCE_ID`, `is_default_instance`, `normalize_instance_id` | Centralize default instance ID string constants and pure helper functions. Re-export in `src-tauri/src/modules/repo_db.rs`. |
| `src-tauri/src/modules/scheduler.rs` | `WEEKLY_WARMUP_*`, `PROMPT_QUEUE_*`, `PID_QUOTA_*` | Define named top-level constants for all 8+ magic durations and intervals in the weekly reset, prompt queue, and instance PID schedulers. |
| `src/types/audit.ts` | `AuditTaskStatus`, `TaskRecord`, `InstanceSwitchRecord` | Export union type `AuditTaskStatus = 'queued' \| 'running' \| 'ok' \| 'fail'`. Refine `TaskRecord.status` and `InstanceSwitchRecord.status` to use typed status union. |

---

## 3. Step-by-Step Implementation Instructions

### 3.1 Step 1: Define `AuditTaskStatus` in `src-tauri/src/modules/task_history_db/types.rs`

1. Open `src-tauri/src/modules/task_history_db/types.rs`.
2. Add imports for `std::fmt` and `std::str::FromStr`.
3. Add the `AuditTaskStatus` enum above `pub struct SplitInfo`:

```rust
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditTaskStatus {
    Queued,
    Running,
    #[serde(rename = "ok")]
    Succeeded,
    #[serde(rename = "fail")]
    Failed,
}

impl AuditTaskStatus {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "ok",
            Self::Failed => "fail",
        }
    }

    #[inline]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }

    #[inline]
    pub const fn is_in_flight(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

impl fmt::Display for AuditTaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AuditTaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "ok" | "succeeded" | "success" => Ok(Self::Succeeded),
            "fail" | "failed" | "error" => Ok(Self::Failed),
            other => Err(format!("Unknown AuditTaskStatus: '{other}'")),
        }
    }
}
```

4. Refactor `AuditTask` implementation to use `AuditTaskStatus`:
   - In `AuditTask::start`:
     ```rust
     pub fn start(action: AuditAction, subject: &str, instance_id: Option<&str>) -> Self {
         match enqueue(action, subject, AuditTaskStatus::Queued.as_str(), instance_id) {
             Ok(id) => Self { id, open: true },
             Err(err) => {
                 crate::modules::logger::log_warn(&format!("[History] enqueue failed: {}", err));
                 Self {
                     id: String::new(),
                     open: false,
                 }
             }
         }
     }
     ```
   - In `AuditTask::succeed`:
     ```rust
     pub fn succeed(&mut self, detail: &str) {
         self.finish(AuditTaskStatus::Succeeded, detail, None);
     }
     ```
   - In `AuditTask::succeed_with_payload`:
     ```rust
     pub fn succeed_with_payload(&mut self, detail: &str, payload_json: &str) {
         self.finish(AuditTaskStatus::Succeeded, detail, Some(payload_json));
     }
     ```
   - In `AuditTask::fail`:
     ```rust
     pub fn fail(&mut self, detail: &str) {
         self.finish(AuditTaskStatus::Failed, detail, None);
     }
     ```
   - In `AuditTask::finish`:
     ```rust
     pub(crate) fn finish(&mut self, status: AuditTaskStatus, detail: &str, payload_json: Option<&str>) {
         if !self.open {
             return;
         }
         if let Err(err) = complete(&self.id, status.as_str(), detail, payload_json) {
             crate::modules::logger::log_warn(&format!("[History] complete failed: {}", err));
         }
         self.open = false;
     }
     ```
   - In `Drop for AuditTask`:
     ```rust
     impl Drop for AuditTask {
         fn drop(&mut self) {
             if self.open {
                 // Justification: best-effort call; failure logged without changing control flow
                 crate::error::record_ignored(
                     complete(&self.id, AuditTaskStatus::Failed.as_str(), "stopped before the task finished", None),
                     "complete",
                 );
             }
         }
     }
     ```

---

### 3.2 Step 2: Define `ActivePromptStatus` in `src-tauri/src/modules/repo_db/models.rs`

1. Open `src-tauri/src/modules/repo_db/models.rs`.
2. Add imports for `std::fmt` and `std::str::FromStr`.
3. Add the `ActivePromptStatus` enum and constants:

```rust
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivePromptStatus {
    Running,
    BackedUp,
    Dispatched,
    Queued,
    Pending,
    Completed,
}

impl ActivePromptStatus {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::BackedUp => "backed_up",
            Self::Dispatched => "dispatched",
            Self::Queued => "queued",
            Self::Pending => "pending",
            Self::Completed => "completed",
        }
    }

    #[inline]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Dispatched)
    }

    #[inline]
    pub const fn is_queued_or_pending(&self) -> bool {
        matches!(self, Self::Queued | Self::Pending)
    }

    #[inline]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

impl fmt::Display for ActivePromptStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ActivePromptStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "running" => Ok(Self::Running),
            "backed_up" | "backedup" => Ok(Self::BackedUp),
            "dispatched" | "in_flight" => Ok(Self::Dispatched),
            "queued" => Ok(Self::Queued),
            "pending" => Ok(Self::Pending),
            "completed" | "done" => Ok(Self::Completed),
            other => Err(format!("Unknown ActivePromptStatus: '{other}'")),
        }
    }
}

/// Common status subsets for SQL queries across repo_db
pub const ACTIVE_PROMPT_IN_FLIGHT_STATUSES: &[&str] = &["running", "in_flight", "dispatched"];
pub const ACTIVE_PROMPT_QUEUED_STATUSES: &[&str] = &["queued", "pending"];
```

4. Export `ActivePromptStatus`, `ACTIVE_PROMPT_IN_FLIGHT_STATUSES`, and `ACTIVE_PROMPT_QUEUED_STATUSES` in `src-tauri/src/modules/repo_db.rs`.

---

### 3.3 Step 3: Define `NotificationChannelType` in `src-tauri/src/modules/notification_hub/channels.rs`

1. Open `src-tauri/src/modules/notification_hub/channels.rs`.
2. Add imports for `std::fmt`.
3. Define the enum:

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationChannelType {
    Email,
    Telegram,
    Supabase,
}

impl NotificationChannelType {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Telegram => "telegram",
            Self::Supabase => "supabase",
        }
    }

    #[inline]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Email => "Email Alert",
            Self::Telegram => "Telegram Bot",
            Self::Supabase => "Supabase Fleet Sync",
        }
    }
}

impl fmt::Display for NotificationChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
```

4. Update `record_channel`:
```rust
pub(crate) fn record_channel(channel: NotificationChannelType, result: Result<String, String>) -> String {
    let name = channel.as_str();
    match result {
        Ok(msg) => {
            let line = format!("[Notify] {}: OK {}", name, msg);
            logger::log_info(&line);
            println!("  [OK] {}", line);
            line
        }
        Err(err) => {
            let trace = std::backtrace::Backtrace::force_capture();
            let line = format!(
                "[Notify] {}: FAIL {}\n{}",
                name,
                redact_secrets(&err),
                trace
            );
            logger::log_error(&line);
            eprintln!("{}", line);
            format!("[Notify] {}: FAIL {}", name, redact_secrets(&err))
        }
    }
}
```

5. Update `deliver_switch_channels`:
```rust
async fn deliver_switch_channels(details: &SwitchNotificationDetails) -> String {
    let email = record_channel(NotificationChannelType::Email, dispatch_email_switch_alert(details));
    dispatch_self_json_in_use_broadcast(details);
    let telegram = record_channel(NotificationChannelType::Telegram, dispatch_telegram_switch_alert(details).await);
    let supabase = record_channel(
        NotificationChannelType::Supabase,
        crate::modules::supabase_sync::push_and_read_instance_email(&details.instance_id)
            .await
            .map_err(|err| redact_secrets(&err.to_string())),
    );
    format!("{}\n{}\n{}", email, telegram, supabase)
}
```

---

### 3.4 Step 4: Define `InstanceModeType` in `src-tauri/src/modules/notification_hub/switch_context.rs`

1. Open `src-tauri/src/modules/notification_hub/switch_context.rs`.
2. Add imports for `std::fmt`.
3. Define the enum:

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceModeType {
    Default,
    Isolated,
}

impl InstanceModeType {
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Isolated => "isolated",
        }
    }

    #[inline]
    pub const fn from_bool(is_default: bool) -> Self {
        if is_default {
            Self::Default
        } else {
            Self::Isolated
        }
    }

    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl fmt::Display for InstanceModeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
```

4. Refactor lines 77-83 in `resolve_switch_context`:
```rust
    let instance_mode = InstanceModeType::from_bool(is_default);

    (old_email, inst_id, inst_name, instance_mode.as_str().to_string())
```

---

### 3.5 Step 5: Centralize Default Instance ID in `src-tauri/src/modules/instance.rs`

1. Open `src-tauri/src/modules/instance.rs`.
2. Above the `pub use` statements, declare the constants and helpers:

```rust
/// Canonical identifier for the default shared instance.
pub const DEFAULT_INSTANCE_ID: &str = "default";

/// Legacy identifier for the default instance used in SQLite records.
pub const LEGACY_DEFAULT_INSTANCE_ID: &str = "__default__";

/// Determine whether an instance ID designates the default instance.
#[inline]
pub fn is_default_instance(id: &str) -> bool {
    let trimmed = id.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case(DEFAULT_INSTANCE_ID)
        || trimmed == LEGACY_DEFAULT_INSTANCE_ID
}

/// Normalize an instance ID to "default" if it represents the default instance.
#[inline]
pub fn normalize_instance_id<'a>(id: &'a str) -> &'a str {
    if is_default_instance(id) {
        DEFAULT_INSTANCE_ID
    } else {
        id.trim()
    }
}
```

3. In `src-tauri/src/modules/repo_db.rs`, re-export them:
```rust
pub use crate::modules::instance::{
    is_default_instance, normalize_instance_id, DEFAULT_INSTANCE_ID, LEGACY_DEFAULT_INSTANCE_ID,
};
```

---

### 3.6 Step 6: Centralize Named Durations in `src-tauri/src/modules/scheduler.rs`

1. Open `src-tauri/src/modules/scheduler.rs`.
2. In the top section (around line 9), declare the named timing constants:

```rust
// --- Scheduler Timing & Duration Constants ---
pub const WEEKLY_WARMUP_STARTUP_DELAY: Duration = Duration::from_secs(60);
pub const WEEKLY_WARMUP_SCAN_INTERVAL: Duration = Duration::from_secs(300);
pub const MODEL_WARMUP_THROTTLE_DELAY: Duration = Duration::from_secs(2);
pub const WARMUP_UI_REFRESH_DELAY: Duration = Duration::from_secs(2);
pub const PROMPT_QUEUE_STARTUP_DELAY: Duration = Duration::from_secs(5);
pub const PROMPT_QUEUE_ACTIVE_POLL_INTERVAL: Duration = Duration::from_secs(5);
pub const PROMPT_QUEUE_IDLE_BACKOFF_INTERVAL: Duration = Duration::from_secs(30);
pub const PID_QUOTA_SYNC_STARTUP_DELAY: Duration = Duration::from_secs(35);
pub const PID_QUOTA_SYNC_INTERVAL: Duration = Duration::from_secs(600);
pub const WARMUP_HISTORY_RETENTION_SECS: i64 = 30 * 86_400;
```

3. Replace magic numbers:
   - Line 104: `tokio::time::sleep(WEEKLY_WARMUP_STARTUP_DELAY).await;`
   - Line 107: `let mut interval = time::interval(WEEKLY_WARMUP_SCAN_INTERVAL);`
   - Line 288: `tokio::time::sleep(MODEL_WARMUP_THROTTLE_DELAY).await;`
   - Line 292: `tokio::time::sleep(WARMUP_UI_REFRESH_DELAY).await;`
   - Line 309: `let cutoff = now_ts - WARMUP_HISTORY_RETENTION_SECS;`
   - Line 416: `tokio::time::sleep(PROMPT_QUEUE_STARTUP_DELAY).await;`
   - Line 453: `tokio::time::sleep(PROMPT_QUEUE_ACTIVE_POLL_INTERVAL).await;`
   - Line 456: `tokio::time::sleep(PROMPT_QUEUE_IDLE_BACKOFF_INTERVAL).await;`
   - Line 471: `tokio::time::sleep(PID_QUOTA_SYNC_STARTUP_DELAY).await;`
   - Line 473: `let mut interval = time::interval(PID_QUOTA_SYNC_INTERVAL);`

---

### 3.7 Step 7: Update Frontend TypeScript Types in `src/types/audit.ts`

1. Open `src/types/audit.ts`.
2. Add the `AuditTaskStatus` type definition and type guards:

```typescript
/**
 * Canonical task execution status emitted by the task history database.
 */
export type AuditTaskStatus = 'queued' | 'running' | 'ok' | 'fail';

/** Check if a task status represents a terminal state. */
export function isTaskTerminal(status: AuditTaskStatus | string): boolean {
    return status === 'ok' || status === 'fail';
}

/** Check if a task status represents an in-flight state. */
export function isTaskInFlight(status: AuditTaskStatus | string): boolean {
    return status === 'queued' || status === 'running';
}
```

3. Update `TaskRecord`:
```typescript
export interface TaskRecord {
    id: string;
    action_code: number;
    action: string;
    action_label: string;
    status: AuditTaskStatus | string;
    subject: string;
    detail: string;
    instance_id: string;
    split_path: string;
    from_email?: string;
    to_email?: string;
    created_at: number;
    finished_at?: number | null;
}
```

4. Update `InstanceSwitchRecord`:
```typescript
export interface InstanceSwitchRecord {
    id: string;
    from_email: string;
    to_email: string;
    created_at: number;
    finished_at?: number | null;
    status: AuditTaskStatus | string;
    switch_reason: string;
    how: string;
    steps?: SwitchAuditSteps;
    raw_payload?: string;
}
```

---

## 4. Edge Cases & Defensive Invariants

1. **Wire Compatibility**: `AuditTaskStatus::Succeeded.as_str()` outputs `"ok"` (NOT `"succeeded"`), matching SQLite legacy data and existing frontend badge renderers.
2. **FromStr Case-Insensitivity**: Both `AuditTaskStatus` and `ActivePromptStatus` `FromStr` parsers accept uppercase, lowercase, and mixed cases (e.g., `"OK"`, `"Ok"`, `"running"`, `"RUNNING"`, `"in_flight"`).
3. **Empty Instance ID Normalization**: `is_default_instance("")` and `is_default_instance("   ")` evaluate to `true`, correctly treating unset instance IDs as `"default"`.
4. **No Silent Error Swallowing**: Any invalid string passed to `FromStr` returns a descriptive `Err(String)`.

---

## 5. Verification & Pre-flight Checklist

- [ ] `cd src-tauri && cargo fmt -- --check`
- [ ] `cd src-tauri && cargo clippy --all-targets --all-features`
- [ ] Run targeted Rust unit tests: `cd src-tauri && cargo test task_history_db`
- [ ] Run targeted Rust unit tests: `cd src-tauri && cargo test scheduler`
- [ ] Verify frontend build: `npm run build`
- [ ] Ensure zero git staging modifications outside target files.
