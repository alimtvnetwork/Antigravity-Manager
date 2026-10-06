# 06 - Rust Pre-push Gate and Instance Key Regression Test

- Added: 2026-10-06
- Status: pending (tracked in `.ai-memory/suggestions.md`)

## Suggestion A: pre-push rustfmt and clippy gate

Evidence from `git log -n 30 --oneline` on 2026-10-06 (HEAD `ccf336b1`): these commits only fixed rustfmt or compile errors after a feature commit: `ccf336b1`, `6228e437`, `5bb32c51`, `01b48a06`, `4c28ecf8`, `d56a384e`, `5de41444`, `a19644a7`, `76493525`, `65800177`, `e0ce4f7c`, `8ce647c7`. That is 12 of 30.

Proposal: the tracked `.githooks/pre-commit` runs `cargo fmt -- --check` when staged files include `src-tauri/**/*.rs`. Release docs keep `cargo clippy --all-targets --all-features` as the manual gate. CI gates stay on.

## Suggestion B: instance id vs IDE flavor regression test

Issue 62: `switch_account` read the prompt snapshot with `target_ide` instead of `"default"`. Add a test that stores a prompt under `"default"`, calls the audit snapshot path with `target_ide = Some("ide")`, and asserts the payload has `prompt_text` and `conversation_id`.
