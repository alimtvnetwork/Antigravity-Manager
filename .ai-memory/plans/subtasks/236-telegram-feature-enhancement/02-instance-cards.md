---
plan: 236-telegram-feature-enhancement
subtask: "02"
title: Copy and Full send full prompt .txt via i:cp
domain: backend-rust
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/236-telegram-feature-enhancement/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/236-telegram-feature-enhancement/02-component-spec.md
  ai_instructions: ../../../../02-spec/21-app/236-telegram-feature-enhancement/03-ai-instructions.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/telegram_inbound/instance_cards.rs
status: completed
---

# Subtask 02 — Instance cards Copy/Full document

Telegram `copy_text` is capped at 256 characters and rejects the whole keyboard when exceeded. Copy and Full both send the complete prompt as a `.txt` document. Independently revertable from 01 (crash/CSV) and 03 (hover/E2E/release).

## Exclusive owned files

| File | Role |
| :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/instance_cards.rs` | Markup, preview cap, `send_full_prompt_document`, `i:cp:` handler, unit tests. |

Do not edit `open_shot.rs`, `media.rs`, `daemon.rs`, `tests.rs`, `src/App.css`, changelog, or README. Daemon already forwards `i:` to `handle_instance_callback`. Do not bump.

## Copy / Full

- Constants: `PREVIEW_WORDS = 100`, `MAX_PROMPTS_SHOWN = 2`, `MAX_CALLBACK_BYTES = 60`.
- `build_instance_card_markup`: 📋 Copy and 📄 Full both `callback_data` from `cb("cp", &snip.id)` → `i:cp:<pid8>`. **No** `copy_text` key on this keyboard.
- `handle_instance_callback` `"cp"` → `send_full_prompt_document` (`agm-prompt-<id8>.txt`, full `prompt_content` bytes, markup `None`).
- HTML card: expandable ~100-word preview. At most 2 running + 2 queued snippets so the message stays under ~3900 chars.
- Fleet header blurb must not claim Copy is a 256-char native clipboard button. Copy and Full both mean the `.txt` document.
- Keep existing `i:rr:` / `i:ad:` / `i:ap:` / `i:ax:` / `i:ff:` / `i:sc:` / `i:st:` rows. Do not embed prompt text in `callback_data`.
- Prompt lists for enqueue/rerun stay FIFO (`created_at ASC, id ASC`). Do not invent a second queue or flywheel.

## Tests (write, do not `cargo test --lib` here)

In `instance_cards.rs`:

- `instance_card_copy_sends_document_not_copy_text` — markup has no `copy_text`; contains `i:cp:`; both Copy and Full labels present.
- `callback_payloads_fit_budget` — `i:cp:<pid8>` `.len() <= 60`.

## Coding rules

- GitMap-only search. Relative paths only.
- No unused `mut` except loop counters.
- Document send failures surface in chat (caption + error + expandable excerpt). Ack follow-up uses `record_ignored` + `// Justification:`.

## Verification (required)

- [ ] Copy and Full share `i:cp:` and send `.txt` of the full prompt.
- [ ] Instance-card markup has zero `copy_text`.
- [ ] Preview ~100 words; `MAX_PROMPTS_SHOWN=2`.
- [ ] Callbacks ≤ 60 bytes.
- [ ] Default and `v2-3127` still running.
- [ ] Exclusive file box respected.

## Done when

- [ ] Tapping Copy or Full uploads `agm-prompt-<id8>.txt`.
- [ ] Keyboard can no longer be rejected by the 256-char `copy_text` cap.
- [ ] No version bump. No crash/CSV/hover files touched.
