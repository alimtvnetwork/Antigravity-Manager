---
plan: 236-telegram-feature-enhancement
subtask: "03"
title: Hover shade animation, isolated E2E, minor 4.276.0
domain: frontend-ui
depends_on:
  - 01-crash-csv
  - 02-instance-cards
citations:
  architecture_spec: ../../../../02-spec/21-app/236-telegram-feature-enhancement/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/236-telegram-feature-enhancement/02-component-spec.md
  ai_instructions: ../../../../02-spec/21-app/236-telegram-feature-enhancement/03-ai-instructions.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/App.css
  - changelog.md
  - changelog_en.md
  - README.md
  - README_EN.md
status: completed
---

# Subtask 03 — Hover animation, isolated E2E, minor release

Enhance `.agm-hover-shade` animation only (no position change). Prove switch+restore on an isolated sandbox. After crash/CSV/copy (01–02) land, minor-bump to `4.276.0`. Independently revertable as CSS + changelog/README; E2E is evidence only.

## Exclusive owned files

| File | Role |
| :--- | :--- |
| `src/App.css` | **Only** `.agm-hover-shade` / `:hover` / `@keyframes agm-hover-wash` (and existing amber hover color). |
| `changelog.md` / `changelog_en.md` | `v4.276.0` notes after bump. Attribution `@aukgit` only. |
| `README.md` / `README_EN.md` | Stable-release changelog sync after bump. |

Do not edit `open_shot.rs`, `media.rs`, `daemon.rs`, `instance_cards.rs`, or `tests.rs`. Do not sweep unrelated `hover:-translate-y-*` on navbar / Instances header.

Bump may also rewrite version mirrors (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, …) via `npm run bump 4.276.0` — those are the bump script's files, not a second feature box.

## Hover

`src/App.css` `.agm-hover-shade` is already color/shade only (`transform: none`). Enhance the wash animation only:

- Transition / keyframes: `background-color`, `border-color`, `box-shadow`, `filter`, `color`.
- `:hover { transform: none; }` must remain.
- Never `hover:-translate-y-*`, never `translate`, never layout position change.
- Amber/yellow hover text stays `#090611`.

## E2E (isolated; never Default / `v2-3127`)

```text
agm instances test-create
# NEW_ID from JSON
# enqueue a distinct prompt onto NEW_ID (FIFO)
# switch NEW_ID to a standby account (--force only on NEW_ID)
# assert backup source is NEW_ID, restore/dispatch same prompt id
# ORDER BY created_at ASC, id ASC
agm instances test-clean NEW_ID
```

Pass criteria:

- Default PID and `v2-3127` still running after cleanup.
- No `agm instances stop` / `rm` / `switch` against the caller.
- Sandbox gone; zero orphan processes for `NEW_ID`.
- Do not run `cargo test --lib` / `cargo check --bin agm` / `tauri build` on this host.

## Release (after 01–02 land)

1. `npm run bump 4.276.0` (or `npm run bump minor` if HEAD is `4.275.x`).
2. Fill `changelog.md` / `changelog_en.md` for `v4.276.0`: screenshot no-focus, CSV file retry, Copy/Full `.txt`. `(Thanks to @aukgit)` only. Relative Git paths only.
3. Sync the same summary under `README.md` / `README_EN.md` `## 📝 Changelog`.
4. Do not tag in this subtask unless the parent explicitly asks. Tag, if later requested, must match the heading `v4.276.0`.

## Coding rules

- GitMap-only search. Relative paths only.
- No second flywheel. No new Telegram pipeline.

## Verification (required)

- [ ] `.agm-hover-shade:hover` has `transform: none` and no translate utilities.
- [ ] Isolated `test-create` → switch+restore same prompt id → `test-clean NEW_ID`.
- [ ] Default and `v2-3127` never stopped.
- [ ] Version `4.276.0` with `@aukgit` only in changelogs and README.

## Done when

- [ ] Hover animation enhanced without position change.
- [ ] E2E evidence recorded for `NEW_ID` only.
- [ ] Minor bump `4.276.0` after crash/CSV/copy.
