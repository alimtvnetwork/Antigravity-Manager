---
plan: 235-add-new-option-for-100-improvements
subtask: "01"
title: CLI module, help, main.rs wiring, flywheel state
domain: backend-rust
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/02-component-spec.md
  ai_instructions: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/03-ai-instructions.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/improvements/mod.rs
  - src-tauri/src/modules/improvements/types.rs
  - src-tauri/src/modules/improvements/engine.rs
  - src-tauri/src/modules/improvements/ledger.rs
  - src-tauri/src/modules/improvements/prompts.rs
  - src-tauri/src/modules/improvements/tests.rs
  - src-tauri/src/modules/mod.rs
  - src-tauri/src/bin/agm/improvements_cmds.rs
  - src-tauri/src/bin/agm/main.rs
  - src-tauri/src/bin/agm/help_text.rs
status: pending
---

# Subtask 01 — CLI engine, help, main.rs wiring, flywheel state

Land the shared flywheel engine and the `agm improvements` command family. UI (02) and Telegram (03) consume this public API and must not duplicate ledger logic. This commit is independently revertable as long as 02 and 03 have not yet patched these exclusive files.

## Exclusive owned files

| File | Role |
| :--- | :--- |
| `src-tauri/src/modules/improvements/mod.rs` | Module root; re-export public API. |
| `src-tauri/src/modules/improvements/types.rs` | `ImprovementKind`, `BatchStatus`, `BatchRecord`, `FlywheelState`, JSON DTO. |
| `src-tauri/src/modules/improvements/engine.rs` | `plan_flywheel`, `next_batch`, `status_report`, `check_pending`. |
| `src-tauri/src/modules/improvements/ledger.rs` | Read/write `.ai-memory/improvements/flywheel.json`. |
| `src-tauri/src/modules/improvements/prompts.rs` | Map kind → `01-prompts/30-improvements/0N-*.md`; load body by relative path. |
| `src-tauri/src/modules/improvements/tests.rs` | Unit tests: dry-run does not dispatch; FIFO batch claim. |
| `src-tauri/src/modules/mod.rs` | **Only** add `pub mod improvements;` — no other edits. |
| `src-tauri/src/bin/agm/improvements_cmds.rs` | CLI dispatcher `cmd_improvements`. |
| `src-tauri/src/bin/agm/main.rs` | **Only** `mod improvements_cmds;` plus match arm `"improvements" \| "improve" \| "imp"`. |
| `src-tauri/src/bin/agm/help_text.rs` | **Only** Continuous Improvement group in `print_help`, `print_help_json`. |

Do not edit Telegram inbound, `src/pages/Telegram.tsx`, Tauri `commands/`, or `src/lib/run.rs`.

## Types

```rust
pub enum ImprovementKind { Ui, Code, System, Ideas, All }
pub enum BatchStatus { Pending, Running, Done }
pub struct BatchRecord {
    pub batch_index: u8,      // 1..=10
    pub status: BatchStatus,
    pub item_start: u16,      // 1, 11, 21, ...
    pub item_end: u16,        // 10, 20, ..., 100
}
pub struct FlywheelState {
    pub wave: u32,
    pub kind: ImprovementKind,
    pub total: u16,           // always 100
    pub batch_size: u8,       // always 10
    pub next_batch: u8,       // 1..=10, first pending index
    pub batches: Vec<BatchRecord>, // length 10, FIFO by batch_index
    pub dry_run: bool,
}
```

Constants, not magic numbers: `TOTAL_IMPROVEMENTS = 100`, `BATCH_SIZE = 10`, `BATCH_COUNT = 10`.

Ledger path (repo-relative): `.ai-memory/improvements/flywheel.json`. Create the directory on first write. IO failures return `AppError::Io` / `AppError::Config` with the relative path in the message.

## Engine contracts

| Function | Behavior |
| :--- | :--- |
| `plan_flywheel(kind, dry_run)` | Write 10 `pending` stubs. Increment `wave` if a completed ledger already exists. If `dry_run`: do **not** call prompt send/enqueue; set `dry_run: true` on the returned state. If live: load the kind's discovery prompt and dispatch via existing prompt send **or** FIFO enqueue (`enqueue_prompt_for_instance` / `agm prompt queue` path). |
| `next_batch(dry_run)` | Claim the lowest `batch_index` with `Pending` (FIFO). Live: mark `Running`, dispatch `01-prompts/30-improvements/07-implement-next-10-improvements.md` wrapped by `01-prompts/14-execute/02-execute-parent-task-with-n-steps-v6.md`. Dry-run: report the would-be batch, leave status `Pending`. |
| `status_report()` | Read ledger (default empty pending ledger if missing). Return PascalCase JSON via existing envelope helpers. |
| `check_pending()` | Read-only scan of `.ai-memory/plans/pending/` and `.ai-memory/plans/subtasks/` using GitMap (`gitmap lf`, `gitmap cat`). No mutation. |
| `detail_flywheel(dry_run)` | Load `04-next-detail-out-100-improvements.md`. Dry-run prints path; live dispatches. |

`--dry-run` must never dispatch prompts. Prove this with a unit test that uses a fake dispatcher counting calls == 0.

## CLI (`improvements_cmds.rs`)

Mirror `src-tauri/src/bin/agm/scheduler_cmds.rs` help style.

```text
agm improvements --help
agm improvements help
agm improvements status [--json]
agm improvements plan [--kind ui|code|system|ideas|all] [--dry-run] [--json] [--queue]
agm improvements next [--dry-run] [--json]
agm improvements next-10 [--dry-run] [--json]
agm improvements check [--json]
agm improvements detail [--dry-run] [--json]
```

Aliases at `main.rs`: `improvements`, `improve`, `imp`. Subcommand aliases: `next` = `next-10` = `n10`; `plan` = `plan-100`; `status` default when args empty.

`--queue` live-only: FIFO enqueue instead of immediate send. Queries that write the queue must remain `ORDER BY created_at ASC, id ASC`.

Empty args or `help` / `-h` / `--help` print the module help. Unknown subcommand uses `handle_unknown_domain_command("improvements", ...)`.

## `main.rs` wiring

1. `mod improvements_cmds;` with the other `mod *_cmds`.
2. Match arm (keep alphabetical-ish near other domains, after `ide` or before `install` is fine):

```rust
"improvements" | "improve" | "imp" => {
    crate::improvements_cmds::cmd_improvements(&cmd_args);
}
```

No other `main.rs` edits.

## Help text

New group **Continuous Improvement Flywheel** in `print_help` and `print_help_json` (sibling to Email & Telegram is acceptable). Items must name `plan`, `next-10`, `status`, `check`, `detail`, flags `--dry-run` `--json` `--kind`.

## Coding rules for this box

- No unused `mut` except loop counters (`for batch_index in 1..=BATCH_COUNT`).
- File cap 300 lines; split rather than grow `engine.rs` past the cap.
- No silent errors. Ledger write: `.map_err(|e| AppError::Io(format!("write flywheel ledger .ai-memory/improvements/flywheel.json: {e}")))?`.
- GitMap-only search while implementing.
- Relative paths only in user-facing strings and logs.

## Verification (required)

```text
agm improvements --help
agm improvements status --json
agm improvements plan --dry-run
agm improvements next --dry-run
agm improvements next-10 --dry-run
```

Expect:

- `--help` lists `plan`, `next-10`, `status`, `check`, `detail`.
- `status --json` is valid JSON with `Kind` / `Wave` / `Total` / `BatchSize` / `NextBatch` / `Batches` (PascalCase envelope or nested `Data`).
- `plan --dry-run` creates or prints 10 stubs and does not send a prompt (no new `active_prompts` row whose body is a 30-improvements prompt).
- `next --dry-run` names batch 1 (or the first pending) without flipping it to `running`.
- Default and `v2-3127` still running.

Write unit tests in `tests.rs` (do not run `cargo test --lib` on this host):

- `plan_dry_run_does_not_dispatch`
- `next_batch_claims_lowest_pending_index`
- `fifo_batch_order_is_index_ascending`

Optional fmt: `cd src-tauri && cargo fmt -- --check` on touched Rust files only.

## Done when

- [ ] Engine + ledger + prompt map exist and compile in CI.
- [ ] `agm improvements --help` works.
- [ ] Dry-run plan/next/status JSON work.
- [ ] No Telegram/UI files touched.
- [ ] No unused `mut` except loop counters.
