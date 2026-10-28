# Component Specification: 100-Improvements Continuous Flywheel

- **Specification Slug:** `235-add-new-option-for-100-improvements`
- **Specification Path:** `02-spec/21-app/235-add-new-option-for-100-improvements/02-component-spec.md`
- **Parent Architecture Spec:** `02-spec/21-app/235-add-new-option-for-100-improvements/01-architecture-spec.md`
- **Sequence ID:** `235`
- **Status:** MANDATED
- **Parity Invariant:** GUI, CLI (`agm improvements …`), Telegram (`/improvements`, `/improve`, `/next10`), and Tauri IPC all call the same Rust module. No adapter-local copies of claim or dispatch logic.

## User Request (Verbatim)

Add a new option for 100 improvements covering: Rust module `src-tauri/src/modules/improvements/` with types `FlywheelState`, `ImprovementKind` (`Ui`, `Code`, `System`, `Ideas`, `All`), `BatchRecord`, functions `plan_flywheel`, `next_batch`, `status_report`, `check_pending` (`--dry-run` must not dispatch prompts); CLI `src-tauri/src/bin/agm/improvements_cmds.rs` registered in `main.rs` and `help_text.rs` (group Email & Telegram or new Continuous Improvement group; help must list `plan` / `next-10` / `status` / `check` / `detail`); Telegram `processing.rs` verbs `improvements|improve|next10`, `config.rs` `setMyCommands`, daemon intercept + HTML card with 4 buttons `im:pl` / `im:n10` / `im:st` / `im:ck`, help in `reports_b.rs`; UI `src/pages/Telegram.tsx` plus optional `src/components/telegram/ImprovementsFlywheel.tsx` (segmented `rounded-full` capsule, transparent borderless selects, `.agm-hover-shade`, invoke via `telegramService` or new `improvementsService`); Tauri commands in `src-tauri/src/commands/` matching invoke names; prompt templates loaded from `01-prompts/30-improvements/0N-*.md` by relative path from repo root; JSON status envelope for `--json`; unit tests for batch claim (write tests only; do not run `cargo test --lib`).

---

## 1. Rust Flywheel Module

### 1.1 Files (new)

| Path | Responsibility | Line budget |
| :--- | :--- | ---: |
| `src-tauri/src/modules/improvements/mod.rs` | Re-exports; `pub use` of types and the four verbs | ≤ 80 |
| `src-tauri/src/modules/improvements/types.rs` | `ImprovementKind`, `FlywheelAction`, `FlywheelParams`, `FlywheelState`, `BatchRecord`, `PendingItem`, `DispatchOutcome` | ≤ 220 |
| `src-tauri/src/modules/improvements/templates.rs` | Repo-root resolver + `load_prompt_template` from `01-prompts/30-improvements/0N-*.md` | ≤ 180 |
| `src-tauri/src/modules/improvements/claim.rs` | FIFO pending-plan scan, `preview_next_batch`, `claim_next_batch` | ≤ 220 |
| `src-tauri/src/modules/improvements/flywheel.rs` | `plan_flywheel`, `next_batch`, `status_report`, `check_pending`, `detail_improvements`; dry-run gate | ≤ 280 |
| `src-tauri/src/modules/improvements/card.rs` | Telegram HTML card + 4-button markup (`im:pl` / `im:n10` / `im:st` / `im:ck`) | ≤ 160 |
| `src-tauri/src/modules/improvements/tests.rs` | Batch-claim, dry-run, callback-byte, kind-path, JSON envelope tests | ≤ 280 |

Register in `src-tauri/src/modules/mod.rs` (alphabetically after `i18n`, before `integration`):

```rust
pub mod improvements;
```

### 1.2 Types (`types.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImprovementKind {
    Ui,
    Code,
    System,
    Ideas,
    All,
}

impl ImprovementKind {
    pub fn parse(raw: &str) -> Result<Self, AppError>; // "ui"|"code"|"system"|"ideas"|"all"
    pub fn as_str(self) -> &'static str;               // lowercase wire value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlywheelAction {
    Plan,
    Next10,
    Status,
    Check,
    Detail,
}

impl FlywheelAction {
    pub fn parse(raw: &str) -> Result<Self, AppError>;
    // aliases: "plan"; "next-10"|"next10"|"next"; "status"|"st"; "check"|"ck"; "detail"|"det"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelParams {
    pub action: FlywheelAction,
    pub kind: ImprovementKind,      // default All
    pub is_dry_run: bool,           // default false
    pub instance_id: Option<String>,
    pub repo_path: Option<String>,
    pub batch_size: usize,          // default 10; only Next10 reads this
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingItem {
    pub id: String,                 // pending plan slug, e.g. "234-telegram-feature-enhancement"
    pub path: String,               // relative: ".ai-memory/plans/pending/<slug>.md"
    pub created_at: i64,            // file created/mtime unix seconds (stable FIFO key)
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    pub batch_id: u64,              // monotonically increasing
    pub kind: ImprovementKind,
    pub claimed_at: i64,            // unix seconds
    pub item_ids: Vec<String>,      // exactly batch_size (or remaining < 10)
    pub is_dispatched: bool,        // false on dry-run preview records (never persisted)
    pub prompt_relpath: String,     // e.g. "01-prompts/30-improvements/07-implement-next-10-improvements.md"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelState {
    pub kind: ImprovementKind,
    pub last_action: Option<FlywheelAction>,
    pub last_batch_id: u64,
    pub claimed_ids: Vec<String>,   // union of every BatchRecord.item_ids (FIFO-stable)
    pub batches: Vec<BatchRecord>,  // append-only history
    pub pending_count: usize,
    pub claimed_count: usize,
    pub last_error: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchOutcome {
    pub action: FlywheelAction,
    pub kind: ImprovementKind,
    pub is_dry_run: bool,
    pub is_dispatched: bool,        // MUST be false when is_dry_run
    pub prompt_relpath: Option<String>,
    pub prompt_bytes: usize,
    pub queued_prompt_id: Option<String>, // Some only when enqueue succeeded
    pub preview_item_ids: Vec<String>,
    pub state: FlywheelState,
}
```

Positive booleans only: `is_dry_run`, `is_dispatched`. No `not_dispatched` / `skip_dispatch` flags.

### 1.3 Persistence

- **Path (relative to repo root):** `.ai-memory/improvements/flywheel_state.json`
- **Envelope:** `crate::modules::json_envelope::JsonEnvelope::new("agm/improvements-flywheel", state)`
- **Load:** `load_flywheel_state() -> Result<FlywheelState, AppError>` — missing file yields `FlywheelState::default()` (empty claimed set, `last_batch_id = 0`). Parse via `json_envelope::extract_payload::<FlywheelState>`; fall back to bare JSON.
- **Save:** `save_flywheel_state(&FlywheelState) -> Result<(), AppError>` — create `.ai-memory/improvements/` if absent. Failures must propagate as `AppError::Io(...)`. No `let _ = std::fs::write`.
- **Import/export help strings** (extend `json_envelope/types.rs` `default_commands_for_type` for `"improvements-flywheel"`):
  - import: `agm improvements status --json`
  - export: `agm improvements status --json`
  - help: `agm improvements help`

### 1.4 Prompt template loader (`templates.rs`)

Resolve repo root as the nearest ancestor of `std::env::current_dir()` that contains both `.git` and `01-prompts/30-improvements/`. If none, return `AppError::Config("improvements: repo root with 01-prompts/30-improvements not found")`.

Load **full markdown by relative path from that root**. Do not use the short fallback snippets in `src-tauri/src/modules/telegram_inbound/reports_cluster_b.rs::resolve_prompt_template_content`.

| `ImprovementKind` / action | Relative path |
| :--- | :--- |
| `Ui` + `Plan` | `01-prompts/30-improvements/01-next-100-ui-improvements.md` |
| `Code` + `Plan` | `01-prompts/30-improvements/02-next-100-code-improvements.md` |
| `System` + `Plan` | `01-prompts/30-improvements/03-next-100-whole-system-improvements.md` |
| `Detail` (any kind) | `01-prompts/30-improvements/04-next-detail-out-100-improvements.md` |
| `Ideas` + `Plan` | `01-prompts/30-improvements/05-next-100-ideas-for-improvements.md` |
| `Check` (any kind) | `01-prompts/30-improvements/06-check-pending-improvements.md` |
| `Next10` (any kind) | `01-prompts/30-improvements/07-implement-next-10-improvements.md` |
| `All` + `Plan` | Concatenate `01`, `02`, `03`, `05` in that order, separated by `\n\n---\n\n` |

```rust
pub fn template_relpath(action: FlywheelAction, kind: ImprovementKind) -> Vec<&'static str>;
pub fn load_prompt_template(action: FlywheelAction, kind: ImprovementKind) -> Result<String, AppError>;
```

`load_prompt_template` joins `repo_root.join(relpath)`, reads UTF-8, trims BOM. Missing file → `AppError::Io(format!("missing prompt template {relpath}"))`.

Older duplicate filenames (`02-find-100-improvements-in-codebase-coding-guidelines.md`, `03-find-100-improvements-in-whole-system-ux.md`) are **not** loaded.

### 1.5 Batch claim (`claim.rs`) — FIFO invariant

Scan `.ai-memory/plans/pending/` for `*.md` (skip `readme.md`). Each file becomes a `PendingItem`:

- `id` = filename stem
- `path` = `.ai-memory/plans/pending/<file>`
- `created_at` = filesystem created time (fallback mtime) as unix seconds
- `title` = first markdown heading or stem

**FIFO sort (mandatory):** `created_at ASC`, then `id ASC`. `ORDER BY updated_at DESC` / mtime-newest-first is forbidden.

```rust
pub fn list_pending_items() -> Result<Vec<PendingItem>, AppError>;

pub fn preview_next_batch(state: &FlywheelState, count: usize) -> Result<Vec<PendingItem>, AppError>;
// filter where !state.claimed_ids.contains(&item.id), take(count). NO mutation.

pub fn claim_next_batch(state: &mut FlywheelState, kind: ImprovementKind, count: usize) -> Result<BatchRecord, AppError>;
```

`claim_next_batch`:

1. `items = preview_next_batch(state, count)?`
2. If `items.is_empty()` → `AppError::Config("improvements: no unclaimed pending plans")`
3. `state.last_batch_id += 1`
4. Append ids to `state.claimed_ids` (preserve insertion order; never sort claimed_ids newest-first)
5. Push `BatchRecord { batch_id: state.last_batch_id, is_dispatched: false, ... }`
6. Recompute `pending_count` / `claimed_count`
7. Return the record — caller persists after setting `is_dispatched`

Claim must be single-threaded in-process (`std::sync::Mutex<FlywheelState>` around load-mutate-save) so two `next_batch` calls cannot overlap ids. Tests cover this without `cargo test --lib`.

### 1.6 Four verbs (`flywheel.rs`)

Shared entry:

```rust
pub fn run_flywheel(params: FlywheelParams) -> Result<DispatchOutcome, AppError>;
```

`run_flywheel` matches `params.action` onto the four public functions (plus `detail_improvements` for CLI `detail`).

```rust
pub fn plan_flywheel(params: &FlywheelParams) -> Result<DispatchOutcome, AppError>;
pub fn next_batch(params: &FlywheelParams) -> Result<DispatchOutcome, AppError>;
pub fn status_report(params: &FlywheelParams) -> Result<DispatchOutcome, AppError>;
pub fn check_pending(params: &FlywheelParams) -> Result<DispatchOutcome, AppError>;
pub fn detail_improvements(params: &FlywheelParams) -> Result<DispatchOutcome, AppError>;
```

**`--dry-run` contract (non-negotiable):**

When `params.is_dry_run == true`:

- Do **not** call `repo_db::enqueue_prompt_for_instance`
- Do **not** call `repo_db::send_prompt_now_for_instance`
- Do **not** call `commands::send_prompt_now` / `commands::enqueue_prompt`
- Do **not** call `telegram_inbound::injection::execute_prompt_injection`
- Do **not** persist a new `BatchRecord` (`claim_next_batch` is skipped; use `preview_next_batch` only)
- Do **not** mutate `flywheel_state.json`
- Return `DispatchOutcome { is_dry_run: true, is_dispatched: false, queued_prompt_id: None, ... }` with composed prompt byte length and preview ids

When `params.is_dry_run == false` and action is `Plan` | `Next10` | `Check` | `Detail`:

1. Load template via `load_prompt_template`
2. For `Next10` only: `claim_next_batch` then prepend a FIFO claim header listing the 10 relative paths
3. Resolve instance with `instance::resolve_instance_id` / active instance (`default` if unset)
4. Dispatch via FIFO enqueue (not immediate send):

```rust
crate::modules::repo_db::enqueue_prompt_for_instance(
    &instance_id,
    repo_path.unwrap_or(""),
    &prompt_body,
    None,
)
.map_err(|e| AppError::Other(format!("improvements enqueue: {e}")))?;
```

5. Set `is_dispatched = true` on the new `BatchRecord` (Next10 only), `save_flywheel_state`

`status_report` never dispatches. It reloads state, refreshes `pending_count` from `list_pending_items`, returns the envelope.

`check_pending` always computes the local dashboard (pending vs claimed vs remaining). Live mode additionally enqueues `06-check-pending-improvements.md`. Dry-run returns the dashboard plus template bytes with `is_dispatched: false`.

### 1.7 JSON status envelope (`--json`)

Reuse `crate::modules::cli::CliEnvelope<DispatchOutcome>`:

```rust
CliEnvelope::ok("improvements status", instance_id, outcome)
```

Printed with `serde_json::to_string_pretty`. Required fields:

```json
{
  "success": true,
  "status": "success",
  "command": "improvements status",
  "instance_id": "default",
  "timestamp": 1730000000,
  "data": {
    "action": "status",
    "kind": "all",
    "is_dry_run": false,
    "is_dispatched": false,
    "prompt_relpath": null,
    "prompt_bytes": 0,
    "queued_prompt_id": null,
    "preview_item_ids": [],
    "state": {
      "kind": "all",
      "last_action": "next-10",
      "last_batch_id": 3,
      "claimed_ids": ["221-...", "222-..."],
      "batches": [],
      "pending_count": 12,
      "claimed_count": 30,
      "last_error": null,
      "updated_at": 1730000000
    }
  },
  "meta": {
    "command": "improvements status",
    "instance_id": "default",
    "timestamp": "2026-10-28T16:00:00+00:00",
    "version": "<CARGO_PKG_VERSION>"
  }
}
```

Error path: `CliEnvelope::err("improvements next-10", instance_id, "IMPROVEMENTS_EMPTY", message)` then `print_and_exit()`.

`command` values: `improvements plan` | `improvements next-10` | `improvements status` | `improvements check` | `improvements detail`.

---

## 2. CLI — `src-tauri/src/bin/agm/improvements_cmds.rs`

### 2.1 Registration

`src-tauri/src/bin/agm/main.rs`:

```rust
mod improvements_cmds; // after install_cmds, before instance_cmds
```

Match arm in `fn main()` (do **not** steal existing `"check"` → doctor):

```rust
"improvements" | "improve" | "flywheel" => {
    crate::improvements_cmds::dispatch_improvements_domain(&cmd_args);
}
"next10" | "next-10" => {
    let mut forward = vec!["next-10".to_string()];
    forward.extend(cmd_args);
    crate::improvements_cmds::dispatch_improvements_domain(&forward);
}
```

### 2.2 Dispatch (`improvements_cmds.rs`)

```rust
pub(crate) fn dispatch_improvements_domain(args: &[String]);
fn print_improvements_help();
fn cmd_improvements_run(action: FlywheelAction, args: &[String]);
```

`dispatch_improvements_domain`:

- `--help` / `-h` / `help` / empty → `print_improvements_help()`
- First positional (or default `status`) maps through `FlywheelAction::parse`
- Flags parsed via `CliContext::parse` plus `--kind <ui|code|system|ideas|all>` (default `all`) and `--dry-run`

Aliases for the first token: `plan`; `next-10` | `next10` | `next`; `status` | `st`; `check` | `ck`; `detail` | `det`.

### 2.3 Help text (must list all five)

`print_improvements_help()` **must** contain the literals `plan`, `next-10`, `status`, `check`, `detail`:

```text
AGM Continuous Improvement Flywheel:
  agm improvements plan    [--kind ui|code|system|ideas|all] [--dry-run] [--json] [-i <instance>]
  agm improvements next-10 [--kind ...] [--dry-run] [--json] [-i <instance>]
  agm improvements status  [--kind ...] [--json]
  agm improvements check   [--kind ...] [--dry-run] [--json]
  agm improvements detail  [--kind ...] [--dry-run] [--json] [-i <instance>]

Aliases: agm improve … ; agm flywheel … ; agm next10  (≡ next-10)

--dry-run   Compose the prompt and preview the next FIFO batch. Never enqueue or send.
--json, -j  CliEnvelope JSON on stdout.

Examples:
  agm improvements plan --kind ui --dry-run
  agm improvements next-10 --json
  agm improvements status --json
  agm improvements check --dry-run --json
  agm improve detail --kind code
  agm next10 --dry-run
```

### 2.4 `help_text.rs` group

Add a **new** group in `print_help_json()` immediately after `"Email & Telegram Telemetry"`:

```json
{
  "group": "Continuous Improvement Flywheel",
  "items": [
    { "name": "improvements plan",    "aliases": ["improve plan"],    "flags": ["--kind <k>", "--dry-run", "--json", "-i <id>"], "description": "Enqueue 01/02/03/05 discovery prompt (100 improvements) for kind" },
    { "name": "improvements next-10", "aliases": ["improve next-10", "next10", "next-10"], "flags": ["--kind <k>", "--dry-run", "--json", "-i <id>"], "description": "FIFO-claim next 10 pending plans and enqueue 07-implement-next-10-improvements.md" },
    { "name": "improvements status",  "aliases": ["improve status"],  "flags": ["--json"], "description": "Print flywheel claimed/pending counters and last batch" },
    { "name": "improvements check",   "aliases": ["improve check"],   "flags": ["--dry-run", "--json"], "description": "Scan pending plans and optionally enqueue 06-check-pending-improvements.md" },
    { "name": "improvements detail",  "aliases": ["improve detail"],  "flags": ["--kind <k>", "--dry-run", "--json", "-i <id>"], "description": "Enqueue 04-next-detail-out-100-improvements.md review-docket prompt" }
  ]
}
```

Human `print_help()` — new section **after** `EMAIL & TELEGRAM MULTI-VM TELEMETRY` (around line 345):

```text
  ────────────────────────────────────────────────────────────────────────────
  CONTINUOUS IMPROVEMENT FLYWHEEL
  ────────────────────────────────────────────────────────────────────────────
    improvements, improve, flywheel [plan|next-10|status|check|detail]
        Drive the 100-improvements loop (01–07 prompts). --dry-run never dispatches.
    next10, next-10 [--dry-run] [--json]
        Alias for improvements next-10 (FIFO claim of 10 pending plans)
```

Cross-link one line inside the Email & Telegram block:

```text
    improvements, improve  (see Continuous Improvement Flywheel)
```

`print_commands_table()` add a row:

```text
  Plan 100 Improvements     cmd: {alias} | agm improvements plan    agm improvements plan --kind all
  Next 10 Improvements      cmd: {alias} | agm next10               agm improvements next-10
```

### 2.5 CLI examples (acceptance)

| Command | Behavior |
| :--- | :--- |
| `agm improvements` | Status (human table) |
| `agm improvements help` | Lists `plan` `next-10` `status` `check` `detail` |
| `agm improvements plan --kind ui --dry-run` | Loads `01-next-100-ui-improvements.md`; prints byte count; **no** enqueue |
| `agm improvements next-10 --dry-run --json` | `preview_next_batch` of 10; `is_dispatched: false`; state file unchanged |
| `agm improvements next-10` | Claims 10 FIFO, enqueues `07-implement-next-10-improvements.md` |
| `agm improvements status --json` | `CliEnvelope` with `FlywheelState` |
| `agm improvements check --dry-run` | Local pending dashboard; no enqueue of `06-…` |
| `agm improve detail --kind code` | Enqueues `04-next-detail-out-100-improvements.md` |
| `agm next10 --dry-run` | Same as `improvements next-10 --dry-run` |

---

## 3. Telegram

### 3.1 Verbs — `src-tauri/src/modules/telegram_inbound/processing.rs`

Inside `process_telegram_command_text`, add a match arm **before** the `_ =>` fallback (do not collide with `"status"`):

```rust
"improvements" | "improve" | "next10" => {
    Some(crate::modules::improvements::handle_telegram_verb(&cmd_base, rest).await)
}
```

`handle_telegram_verb(cmd_base, rest) -> String`:

- `cmd_base == "next10"` → action `Next10` regardless of rest (rest may still carry `--dry-run` / `--kind ui`)
- otherwise parse first token of `rest` via `FlywheelAction::parse`, default `Status`
- parse `--kind` / `--dry-run` from remaining tokens
- `run_flywheel` then `card::format_improvements_card(&outcome)`

Empty `/improvements` returns the HTML card for current status (4 buttons), not a dump of the JSON envelope.

### 3.2 `setMyCommands` — `src-tauri/src/modules/telegram_inbound/config.rs`

Append to the `commands` array in `register_telegram_bot_commands` (keep existing entries; add these three before `help`):

```json
{ "command": "improvements", "description": "100-improvements flywheel: plan / next-10 / status / check / detail" },
{ "command": "improve",      "description": "Alias for /improvements" },
{ "command": "next10",       "description": "FIFO-claim and enqueue the next 10 improvements" }
```

### 3.3 Daemon intercept — `src-tauri/src/modules/telegram_inbound/daemon.rs`

Telegram `callback_data` cap remains 60 bytes (`MAX_CALLBACK_BYTES = 60`). Payloads:

| Button label | `callback_data` | Bytes | Action |
| :--- | :--- | ---: | :--- |
| `📋 Plan` | `im:pl` | 5 | `Plan` (kind = last / `All`) |
| `⏩ Next 10` | `im:n10` | 6 | `Next10` |
| `📊 Status` | `im:st` | 5 | `Status` |
| `🔍 Check` | `im:ck` | 5 | `Check` |

`im:dt` (detail) is **not** on the 4-button card; CLI / `/improvements detail` still expose it.

Intercept in **both** `callback_query` loops (the one-shot poll around the existing `if data.starts_with("i:")` at ~line 662, and the background daemon loop ~line 852), **after** `deliver_send_flow_callback` and **before** `cmd:` normalization:

```rust
if data.starts_with("im:") {
    answer_callback_query(&clean_token, cb_id).await; // already answered above; keep idempotent
    send_chat_action(&clean_token, chat_id, "typing").await;
    let handled = crate::modules::improvements::handle_improvements_callback(
        &clean_token,
        chat_id,
        data,
    )
    .await;
    if handled {
        processed.push((chat_id, data.to_string(), "ok".to_string()));
    }
    continue;
}
```

```rust
pub fn parse_improvements_callback(data: &str) -> Option<FlywheelAction>;
// "im:pl" -> Plan, "im:n10" -> Next10, "im:st" -> Status, "im:ck" -> Check
pub async fn handle_improvements_callback(token: &str, chat_id: i64, data: &str) -> bool;
```

Unknown `im:*` returns `false` (do not fall through to prompt injection). Callbacks never set `is_dry_run`; they execute live. Operators who want a preview use `/improvements next-10 --dry-run`.

### 3.4 HTML card (`improvements/card.rs`)

```rust
pub fn format_improvements_card(outcome: &DispatchOutcome) -> (String, serde_json::Value);
```

HTML (palette via `telegram_inbound::active_card_palette()`):

```text
🔁 <b>Improvements flywheel</b>
• Kind: <code>all</code>
• Action: <code>status</code>
• Pending: <code>12</code> · Claimed: <code>30</code> · Last batch: <code>#3</code>
• Dry-run: <code>no</code> · Dispatched: <code>no</code>
━━━━━━━━━━━━━━━━━━━━━━━━
Next FIFO ids: <code>234-…</code>, <code>235-…</code>
```

Inline keyboard (one row of four; `callback_data` exactly the strings above):

```json
{
  "inline_keyboard": [[
    { "text": "📋 Plan",    "callback_data": "im:pl" },
    { "text": "⏩ Next 10", "callback_data": "im:n10" },
    { "text": "📊 Status",  "callback_data": "im:st" },
    { "text": "🔍 Check",   "callback_data": "im:ck" }
  ]]
}
```

Send via existing `send_telegram_message_with_markup`. Do not embed prompt bodies in `callback_data` or `copy_text`.

### 3.5 Help — `src-tauri/src/modules/telegram_inbound/reports_b.rs`

In `format_help_manual()`, add a section after `🟡 Backup, Cleanup, Updates & Email`:

```text
🔁 <b>Continuous Improvement Flywheel</b>
• <code>/improvements</code> — Status card with Plan / Next 10 / Status / Check buttons
• <code>/improvements plan</code> · <code>/improve plan --kind ui</code>
• <code>/improvements next-10</code> · <code>/next10</code> · <code>/improve next-10 --dry-run</code>
• <code>/improvements status</code> · <code>/improvements check</code> · <code>/improvements detail</code>
```

Update `help_manual_has_all_required_code_blocks` in `src-tauri/src/modules/telegram_inbound/tests.rs` to assert `/improvements`, `/next10`, `/improvements next-10`, `/improvements check`, `/improvements detail`.

---

## 4. Tauri IPC — `src-tauri/src/commands/improvements.rs`

New file. Register:

```rust
// src-tauri/src/commands/mod.rs
pub mod improvements;
pub use improvements::*;
```

Add to `src-tauri/src/lib/run.rs` `generate_handler![]` next to the Telegram block:

```rust
commands::improvements_plan,
commands::improvements_next_batch,
commands::improvements_status,
commands::improvements_check_pending,
commands::improvements_detail,
```

Invoke names (snake_case, matching the TS `invoke('…')` strings):

| Rust / invoke name | Params | Maps to |
| :--- | :--- | :--- |
| `improvements_plan` | `kind: Option<String>, is_dry_run: bool, instance_id: Option<String>` | `plan_flywheel` |
| `improvements_next_batch` | same + `batch_size: Option<u32>` | `next_batch` |
| `improvements_status` | `kind: Option<String>` | `status_report` |
| `improvements_check_pending` | `kind: Option<String>, is_dry_run: bool` | `check_pending` |
| `improvements_detail` | `kind: Option<String>, is_dry_run: bool, instance_id: Option<String>` | `detail_improvements` |

Each command returns `AppResult<DispatchOutcome>`. Default `kind = "all"`, `is_dry_run = false`. Parameter count is wrapped through `FlywheelParams` (no 4+ naked args inside the module).

---

## 5. Frontend

### 5.1 Service — `src/services/improvementsService.ts` (new)

Do not overload `telegramService.ts` (that file is config/daemon-only: `get_telegram_config`, `save_telegram_config`, …). New service:

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export type ImprovementKind = 'ui' | 'code' | 'system' | 'ideas' | 'all';

export interface FlywheelDispatchOutcome { /* mirrors DispatchOutcome */ }

async function invokeFlywheel<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    useErrorStore.getState().captureError(error, { source: 'ImprovementsService', triggerAction: cmd });
    throw error;
  }
}

export const improvementsService = {
  plan: (kind: ImprovementKind, isDryRun: boolean) =>
    invokeFlywheel<FlywheelDispatchOutcome>('improvements_plan', { kind, isDryRun }),
  nextBatch: (kind: ImprovementKind, isDryRun: boolean) =>
    invokeFlywheel<FlywheelDispatchOutcome>('improvements_next_batch', { kind, isDryRun }),
  status: (kind: ImprovementKind = 'all') =>
    invokeFlywheel<FlywheelDispatchOutcome>('improvements_status', { kind }),
  checkPending: (kind: ImprovementKind, isDryRun: boolean) =>
    invokeFlywheel<FlywheelDispatchOutcome>('improvements_check_pending', { kind, isDryRun }),
  detail: (kind: ImprovementKind, isDryRun: boolean) =>
    invokeFlywheel<FlywheelDispatchOutcome>('improvements_detail', { kind, isDryRun }),
};
```

Tauri camelCases `is_dry_run` → `isDryRun` on the JS side; keep the invoke **command** strings snake_case as listed in §4.

Failures: `captureError` for user-facing invoke failures (opens the error modal). Status refresh on mount uses `trackWarning` if the daemon is down, then renders an empty state — never a bare `catch {}`.

### 5.2 Component — `src/components/telegram/ImprovementsFlywheel.tsx` (new, ≤ 100 lines)

Mount in `src/pages/Telegram.tsx` immediately below the header banner card and above the bot-token form. Outer card:

```tsx
<div className="agm-hover-shade rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-xs">
```

No `hover:-translate-y-*`. Amber/yellow hover text stays black (`#090611`) via existing `.agm-hover-shade`.

**Segmented action capsule** (contiguous pill, shared border, subtle dividers — AGENTS.md toolbar rule):

```tsx
<div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611] overflow-hidden">
  {(['plan', 'next-10', 'status', 'check', 'detail'] as const).map((action, i) => (
    <button
      key={action}
      type="button"
      className={`px-3 py-1.5 text-[11px] font-semibold cursor-pointer ${
        i > 0 ? 'border-l border-slate-200 dark:border-[#40285e]' : ''
      }`}
    >
      {action}
    </button>
  ))}
</div>
```

**Kind select** uses `src/components/common/CompactPillSelect.tsx` (already `bg-transparent border-0 focus:ring-0 focus:outline-none`). Options: `ui`, `code`, `system`, `ideas`, `all`. Do not nest a white native `<select>` inside the capsule.

**Dry-run toggle** is a second pill in the same toolbar row (`Dry-run` / `Live`). Default **Dry-run = on** in the UI so a mis-click cannot enqueue. Live mode requires an explicit toggle.

On action click: call `improvementsService.*`, then `showToast`. Render `pending_count` / `claimed_count` / `last_batch_id` as zinc pills. Copyable CLI chips (reuse `Telegram.tsx` `handleCopyCli`):

```text
agm improvements plan --kind ui --dry-run
agm improvements next-10 --dry-run
agm improvements status --json
agm improvements check --dry-run
agm improvements detail --dry-run
```

### 5.3 Page wiring — `src/pages/Telegram.tsx`

- Import `ImprovementsFlywheel`
- Render `<ImprovementsFlywheel />` in the main column
- Apply `.agm-hover-shade` to the new card only (existing cards already have it from sequence 234)

---

## 6. Unit Tests (write only — do not run `cargo test --lib`)

File: `src-tauri/src/modules/improvements/tests.rs` under `#[cfg(test)]`. Use a temp directory (`tempfile` or `std::env::temp_dir()` + unique slug) with fake `.ai-memory/plans/pending/{a,b,c}.md` whose `created_at` is forced via a test helper that accepts injected `PendingItem`s (do not depend on real repo pending plans).

| Test | Assertion |
| :--- | :--- |
| `claim_fifo_created_at_then_id` | Items `(t=2,id=b)`, `(t=1,id=c)`, `(t=1,id=a)` claim as `a`, `c` then `b` |
| `claim_does_not_overlap_second_batch` | First claim takes 10; second claim's `item_ids` disjoint |
| `claim_skips_already_claimed_ids` | Pre-seeded `claimed_ids` are excluded |
| `claim_empty_pending_is_error` | `claim_next_batch` on empty set returns `Err` |
| `dry_run_next_batch_does_not_persist_claim` | After `is_dry_run: true`, `load_flywheel_state().claimed_ids` unchanged |
| `dry_run_sets_is_dispatched_false` | `DispatchOutcome.is_dispatched == false` and `queued_prompt_id.is_none()` |
| `dry_run_never_calls_enqueue` | Next10 dry-run with a stub/hook or by asserting no new `active_prompts` row when repo_db is unreachable — at minimum, `is_dispatched` false and state file mtime unchanged |
| `kind_maps_to_relpath` | `Ui+Plan` → `01-prompts/30-improvements/01-next-100-ui-improvements.md`, `Next10` → `…/07-implement-next-10-improvements.md`, `Check` → `…/06-check-pending-improvements.md`, `Detail` → `…/04-next-detail-out-100-improvements.md` |
| `callback_payloads_fit_60_bytes` | `"im:pl"`, `"im:n10"`, `"im:st"`, `"im:ck"` each `len() <= 60` (and `==` the exact strings) |
| `parse_improvements_callback_roundtrip` | `parse_improvements_callback("im:n10") == Some(FlywheelAction::Next10)`; `"im:xx"` → `None` |
| `status_json_envelope_shape` | `CliEnvelope::ok("improvements status", …)` serializes `success`, `data.state.pending_count`, `meta.command` |
| `flywheel_action_aliases` | `"next10"` and `"next-10"` parse to `Next10`; `"ck"` → `Check` |

Do **not** execute `cargo test --lib`, `gitmap cargo test`, or any compile on this machine as part of spec authoring.

---

## 7. Call graph

```text
UI ImprovementsFlywheel
  → improvementsService.invoke('improvements_*')
    → commands/improvements.rs
      → modules::improvements::run_flywheel
        → templates::load_prompt_template   (01-prompts/30-improvements/0N-*.md)
        → claim::preview_next_batch | claim_next_batch
        → repo_db::enqueue_prompt_for_instance   (SKIPPED when is_dry_run)
        → save_flywheel_state                   (SKIPPED when is_dry_run)

CLI agm improvements | agm next10
  → improvements_cmds::dispatch_improvements_domain
    → same run_flywheel
    → CliEnvelope pretty-print when --json

Telegram /improvements|/improve|/next10
  → processing.rs match arm
    → handle_telegram_verb → run_flywheel → format_improvements_card

Telegram inline buttons im:pl|im:n10|im:st|im:ck
  → daemon.rs callback intercept (both loops)
    → handle_improvements_callback → run_flywheel (live, is_dry_run=false)
```

---

## 8. Invariants & confinement

- **Single module:** claim, template load, and dry-run gating live only in `src-tauri/src/modules/improvements/`. CLI / Telegram / Tauri are thin wrappers.
- **FIFO:** pending scan and claim use `created_at ASC, id ASC`. Dispatch uses `enqueue_prompt_for_instance` (queued FIFO), not `send_prompt_now_for_instance`.
- **Dry-run:** zero enqueue, zero send, zero `flywheel_state.json` writes, `is_dispatched == false`.
- **Relative paths only:** templates loaded as `01-prompts/30-improvements/…` from repo root. State at `.ai-memory/improvements/flywheel_state.json`.
- **Telegram cap:** `im:pl` / `im:n10` / `im:st` / `im:ck` are 5–6 bytes.
- **No silent errors:** I/O and enqueue failures return `AppError`; UI uses `captureError` / `trackWarning`.
- **File size:** new `.rs` files 150–300 lines (mod.rs / cmd file may be shorter). `ImprovementsFlywheel.tsx` ≤ 100 lines.
- **Blast radius:** listed files in §§1–5 plus `src-tauri/src/modules/mod.rs`, `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib/run.rs`, `src-tauri/src/bin/agm/main.rs`, `src-tauri/src/bin/agm/help_text.rs`, `src-tauri/src/modules/telegram_inbound/{processing,config,daemon,reports_b,tests}.rs`, `src-tauri/src/modules/json_envelope/types.rs`. Do not retouch instance cards (`i:` namespace stays exclusive to sequence 234).
- **Caller IDE:** enqueue only; never `agm instances stop` / kill the caller instance.

---

## 9. Acceptance checklist (implementation, not this spec turn)

- [ ] `agm improvements help` prints `plan`, `next-10`, `status`, `check`, `detail`
- [ ] `agm improvements next-10 --dry-run --json` → `data.is_dispatched == false` and `.ai-memory/improvements/flywheel_state.json` unchanged
- [ ] `agm improvements plan --kind ui --dry-run` reads `01-prompts/30-improvements/01-next-100-ui-improvements.md` by relative path
- [ ] Telegram `/improvements` sends HTML card whose four `callback_data` values are exactly `im:pl`, `im:n10`, `im:st`, `im:ck`
- [ ] `setMyCommands` includes `improvements`, `improve`, `next10`
- [ ] `src/pages/Telegram.tsx` renders the segmented `rounded-full` capsule with transparent borderless kind `<select>` and `.agm-hover-shade`
- [ ] Invoke names `improvements_plan` / `improvements_next_batch` / `improvements_status` / `improvements_check_pending` / `improvements_detail` registered in `generate_handler`
- [ ] Unit tests in `src-tauri/src/modules/improvements/tests.rs` cover FIFO claim disjointness and dry-run non-dispatch (written, not executed on this machine)
