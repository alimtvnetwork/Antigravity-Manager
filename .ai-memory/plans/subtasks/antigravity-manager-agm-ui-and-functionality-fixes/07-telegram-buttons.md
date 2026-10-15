# Task-07: Telegram buttons — send prompt, find instance, credits check

## Status
PENDING (needs Task-01 gap report)

## Objective
E2E-test the Telegram buttons for sending prompts, finding instances, and
checking credits; root-cause the failures; fix them.

## Requirements
- Trace each button end-to-end: button tap → callback payload → handler →
  backend action → user-visible result. The user reports all three are
  broken ("do not work properly").
- Likely areas (verify, don't assume):
  - `src-tauri/src/modules/telegram_inbound/` — callback dispatch
    (`processing.rs`, `commands.rs`), inline keyboard construction
    (`reports_b.rs`, `send_flow.rs`), instance lookup, credits/quotacheck
    paths.
  - Callback payload encoding (`|`-separated; entries containing `|` or
    `:` are rejected — check the picker-safety logic).
- For each broken button: write the RCA (symptom → root cause → fix) and
  fix at the root. Add/extend unit tests for the fixed paths.
- Keep `actor=telegram` in audit entries for bot-driven actions.
- "No Silent Errors"; `cargo fmt` + `clippy` clean.

## Acceptance
- [ ] Send-prompt button works E2E (prompt reaches the conversation).
- [ ] Find-instance button resolves and selects the instance.
- [ ] Credits-check button returns real quota numbers.
- [ ] RCAs written for each root cause found.

## Constraints
- GitMap search only. Relative paths only. No secrets.
