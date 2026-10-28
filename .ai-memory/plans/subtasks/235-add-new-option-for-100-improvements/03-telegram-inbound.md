---
plan: 235-add-new-option-for-100-improvements
subtask: "03"
title: Telegram verbs, bot commands, callback buttons, help text
domain: backend-rust
depends_on:
  - 01-cli-engine
citations:
  architecture_spec: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/02-component-spec.md
  ai_instructions: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/03-ai-instructions.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/telegram_inbound/improvements.rs
  - src-tauri/src/modules/telegram_inbound/mod.rs
  - src-tauri/src/modules/telegram_inbound/processing.rs
  - src-tauri/src/modules/telegram_inbound/config.rs
  - src-tauri/src/modules/telegram_inbound/daemon.rs
  - src-tauri/src/modules/telegram_inbound/reports_b.rs
  - src-tauri/src/modules/telegram_inbound/tests.rs
status: pending
---

# Subtask 03 — Telegram verbs, bot commands, callback buttons, help text

Remote operators drive the same flywheel through `/improvements`, `/improve`, and `/next10`. Independently revertable: reverting this commit removes only inbound Telegram wiring; CLI and UI keep working.

## Exclusive owned files

| File | Role |
| :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/improvements.rs` | HTML card + `im:` keyboard. Keep ≤ 300 lines. |
| `src-tauri/src/modules/telegram_inbound/mod.rs` | **Only** `pub mod improvements;` (+ re-export if required). |
| `src-tauri/src/modules/telegram_inbound/processing.rs` | **Only** match arms `improvements` \| `improve` \| `next10` (and `imp`). |
| `src-tauri/src/modules/telegram_inbound/config.rs` | **Only** `setMyCommands` entries. |
| `src-tauri/src/modules/telegram_inbound/daemon.rs` | **Only** `im:` callback normalize + markup match arm. |
| `src-tauri/src/modules/telegram_inbound/reports_b.rs` | **Only** flywheel section inside `format_help_manual`. |
| `src-tauri/src/modules/telegram_inbound/tests.rs` | Help snippet + callback byte-cap tests. |

Do not edit `src-tauri/src/modules/improvements/**`, `src-tauri/src/bin/agm/**`, `src/pages/Telegram.tsx`, or Tauri `commands/`. Call `crate::modules::improvements::{status_report, plan_flywheel, next_batch, check_pending}`.

## Verbs (`processing.rs`)

In `process_telegram_command_text`, add:

```rust
"improvements" | "improve" | "imp" => Some(super::improvements::format_improvements_card(rest).await),
"next10" | "next-10" | "n10" => Some(super::improvements::format_next10_card(rest).await),
```

`rest` flags: `plan`, `next`, `next-10`, `status` (default), `check`, `detail`, `--dry-run`, `--kind ui|code|system|ideas|all`. Default Telegram `status` is read-only. `plan` / `next10` without `--go` behave as `--dry-run` so a tap cannot surprise-dispatch. `--go` is required for live dispatch.

Live enqueue (only with `--go`) uses the existing FIFO prompt queue (`ORDER BY created_at ASC, id ASC`). Never invent a second pipeline.

## Callback protocol (`im:` namespace, ≤ 60 bytes)

| `callback_data` | Action | Maps to |
| :--- | :--- | :--- |
| `im:pl` | Plan (dry-run) | `/improvements plan --dry-run` |
| `im:n10` | Next 10 (dry-run) | `/improvements next-10 --dry-run` |
| `im:st` | Status | `/improvements status` |
| `im:ck` | Check pending | `/improvements check` |

All four strings are well under 60 bytes. Do not put kind, wave, or prompt text in `callback_data`.

Extend `normalize_cmd_callback_payload` (or a sibling `normalize_improvements_callback`) in `daemon.rs`:

- `im:pl` → `/improvements plan --dry-run`
- `im:n10` → `/improvements next-10 --dry-run`
- `im:st` → `/improvements status`
- `im:ck` → `/improvements check`

Non-`im:` payloads unchanged. Then `process_telegram_command_text` runs as today.

## HTML card + buttons (`improvements.rs`)

One compact HTML message:

- Title: Continuous Improvement Flywheel
- Lines: kind, wave, next batch `N/10`, pending/running/done counts, dry-run badge
- `pre` JSON omitted unless `--json` rest flag; keep the card short

Inline keyboard (attach from `daemon.rs` markup match, same pattern as `/backup`):

```text
[ 📋 Plan ] [ ⏭ Next 10 ]
[ 📊 Status ] [ 🗂 Check ]
```

`callback_data` values exactly `im:pl`, `im:n10`, `im:st`, `im:ck`.

Daemon match arm:

```rust
"improvements" | "improve" | "imp" | "next10" | "next-10" | "n10" => {
    Some(super::improvements::improvements_reply_markup())
}
```

## Bot commands (`config.rs`)

Inside `register_telegram_bot_commands` `commands` array, add (keep descriptions ≤ Telegram limits):

```json
{ "command": "improvements", "description": "100-improvements flywheel: plan, next-10, status" }
{ "command": "improve", "description": "Alias for /improvements" }
{ "command": "next10", "description": "Claim the next 10 improvements (dry-run card)" }
```

Place near `help` / `update`. Do not remove existing commands.

## Help text (`reports_b.rs`)

Append a section to `format_help_manual()` (before the closing theme/target block is fine):

```text
🔧 <b>Continuous Improvement Flywheel</b>
• <code>/improvements</code> — Status card with Plan / Next 10 / Status / Check buttons
• <code>/improve</code> · <code>/next10</code> — Aliases
• <code>/improvements plan --dry-run</code> — Materialize 10 batch stubs without dispatch
• <code>/improvements next-10 --dry-run</code> — Show the next FIFO batch without dispatch
• <code>/improvements status --json</code> — Ledger envelope
```

Add the same `<code>/improvements</code>` (and ideally `plan --dry-run` / `next-10 --dry-run`) to the required-snippet list in `src-tauri/src/modules/telegram_inbound/tests.rs` (`test` that walks `format_help_manual()`).

## Tests (write, do not cargo-test --lib here)

In `tests.rs`:

- `improvements_callback_payloads_fit_60_bytes` — each `im:pl|n10|st|ck` `.len() <= 60`
- `help_manual_includes_improvements` — `format_help_manual()` contains `<code>/improvements</code>`
- `normalize_im_pl_maps_to_dry_run_plan`

No unused `mut` except loop counters. Errors from engine calls surface in the HTML card (`⚠️`); do not swallow.

## Verification (required)

CLI regression (engine from 01):

```text
agm improvements --help
agm improvements status --json
agm improvements plan --dry-run
agm improvements next --dry-run
```

Telegram:

- `/improvements` returns the card + four buttons.
- Tapping Plan/Next 10 is dry-run (no prompt dispatch).
- `/help` contains `/improvements`.
- Default and `v2-3127` never stopped.

## Done when

- [ ] Verbs wired in `processing.rs`.
- [ ] `setMyCommands` lists `improvements`, `improve`, `next10`.
- [ ] `im:` buttons ≤ 60 bytes and daemon-normalized.
- [ ] Help section + unit assertions.
- [ ] Exclusive file box respected; independently revertable from 02.
