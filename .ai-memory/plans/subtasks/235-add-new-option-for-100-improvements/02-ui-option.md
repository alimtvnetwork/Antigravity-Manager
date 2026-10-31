---
plan: 235-add-new-option-for-100-improvements
subtask: "02"
title: UI ImprovementsFlywheel, Tauri invokes, hover shade
domain: frontend-ui
depends_on:
  - 01-cli-engine
citations:
  architecture_spec: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/02-component-spec.md
  ai_instructions: ../../../../02-spec/21-app/235-add-new-option-for-100-improvements/03-ai-instructions.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/telegram/ImprovementsFlywheel.tsx
  - src/services/improvementsService.ts
  - src/pages/Telegram.tsx
  - src-tauri/src/commands/cmd_improvements.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/lib/run.rs
status: completed
---

# Subtask 02 — UI ImprovementsFlywheel, Tauri invokes, hover shade

Add a compact Plan / Next 10 / Status option on the Telegram page that calls the shared engine from subtask 01. Independently revertable: reverting this commit removes only the UI + Tauri command shim; CLI and Telegram inbound keep working.

## Exclusive owned files

| File | Role |
| :--- | :--- |
| `src/components/telegram/ImprovementsFlywheel.tsx` | Segmented capsule. **≤ 100 lines.** |
| `src/services/improvementsService.ts` | `invoke` wrappers; `captureError` on failure. |
| `src/pages/Telegram.tsx` | **Only** import + mount `<ImprovementsFlywheel />` in the header/actions row. No unrelated refactors. |
| `src-tauri/src/commands/cmd_improvements.rs` | Tauri commands calling `modules::improvements`. |
| `src-tauri/src/commands/mod.rs` | **Only** `mod cmd_improvements;` + `pub use cmd_improvements::*;`. |
| `src-tauri/src/lib/run.rs` | **Only** three `generate_handler!` entries. |

Do not edit `src-tauri/src/modules/improvements/**`, `src-tauri/src/bin/agm/**`, or `src-tauri/src/modules/telegram_inbound/**`. Do not edit `src/App.css` — reuse `.agm-hover-shade`.

## Tauri commands

`src-tauri/src/commands/cmd_improvements.rs`:

```rust
#[tauri::command]
pub async fn improvements_plan(kind: Option<String>, dry_run: Option<bool>) -> Result<FlywheelState, String>

#[tauri::command]
pub async fn improvements_next(dry_run: Option<bool>) -> Result<FlywheelState, String>

#[tauri::command]
pub async fn improvements_status() -> Result<FlywheelState, String>
```

Map `String` errors from `AppError::to_string()`. Default `dry_run` to `true` from the UI unless the user explicitly confirms a live run (safer for the Telegram page). Parse `kind` via `ImprovementKind` (`ui|code|system|ideas|all`); invalid kind returns `AppError::Config`.

Register in `src-tauri/src/lib/run.rs` `generate_handler!`:

```rust
commands::improvements_plan,
commands::improvements_next,
commands::improvements_status,
```

Do not add HTTP mappings in `src/utils/request.ts` unless the page already uses `request()` for Telegram; prefer `@tauri-apps/api/core` `invoke` through `improvementsService.ts` (same pattern as `src/services/telegramService.ts`).

## ImprovementsFlywheel UI

Mount on `src/pages/Telegram.tsx` in the header banner action cluster (next to existing watcher / guide controls), not as a scattered floating button.

Requirements (AGENTS.md UI):

- Contiguous segmented pill capsule: `rounded-full`, shared border, subtle dividers, dark-glass.
- Pills: **Plan**, **Next 10**, **Status**.
- Optional kind `<select>` inside the capsule: `bg-transparent border-0 focus:ring-0 focus:outline-none` (no nested white pill-in-pill).
- Hover: add `agm-hover-shade` on the capsule. **No** `hover:-translate-y-*`, **no** position change. Color / shade / shadow / filter only. Amber/yellow hover text stays `#090611`.
- After each action, show a one-line status (`wave`, `kind`, `next_batch`, dry-run badge).
- Failures: `useErrorStore.getState().captureError(err, { source: 'ImprovementsFlywheel.<action>', triggerAction: 'improvements_plan|next|status' })`. Never empty `catch`.
- `ImprovementsFlywheel.tsx` ≤ 100 lines. Extract types into the service file if needed.

Live Plan / Next 10 must not be the default click. Default the invokes to `dry_run: true`. A secondary confirm (or a `Live` toggle defaulting off) is required before `dry_run: false`.

## Service

```ts
export const improvementsService = {
  plan(kind: ImprovementKind, dryRun: boolean): Promise<FlywheelState>
  next(dryRun: boolean): Promise<FlywheelState>
  status(): Promise<FlywheelState>
}
```

Each method `invoke`s the matching command and `captureError` then rethrows.

## Hover shade

Use existing `src/App.css` `.agm-hover-shade` / `.agm-hover-shade:hover` (`transform: none`). Do not add new translate utilities on the capsule or its buttons.

## Coding rules

- No unused `mut` (TS: no reassignment except loop indices).
- GitMap-only search.
- Relative paths only.
- Do not dispatch prompts from the frontend; Tauri commands call the engine.

## Verification (required)

CLI still works (do not regress 01):

```text
agm improvements --help
agm improvements status --json
agm improvements plan --dry-run
agm improvements next --dry-run
```

UI:

- Telegram page shows the capsule.
- Plan (dry-run) refreshes the one-line status without enqueueing a prompt.
- Hover changes shade only (no jump).
- `npm run build` after TS/TSX edits.

Do not run `cargo test --lib` on this host. Do not stop Default / `v2-3127`.

## Done when

- [ ] Capsule mounted on `src/pages/Telegram.tsx`.
- [ ] Three Tauri commands registered and invoked.
- [ ] Default path is dry-run.
- [ ] `.agm-hover-shade` applied; no translate-on-hover.
- [ ] Exclusive file box respected; independently revertable.
