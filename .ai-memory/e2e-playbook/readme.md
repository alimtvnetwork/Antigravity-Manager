# E2E Testing Playbook (`.ai-memory/e2e-playbook/`)

Step-by-step, checklist-driven procedures for end-to-end testing of
Antigravity Manager. Written for an AI with **zero prior context**: every
step names the exact button, label, command, or log path — nothing is
assumed. Each flow ends with an explicit PASS/FAIL verdict rule.

When the user says *"perform the auto-switch end-to-end testing"* or
*"perform the instance test testing"*, start here and work the flows in
order. "Instance test testing" means: auto-switch → instance open →
prompt preview → prompt send (all four).

## Flows (run in this order)

| # | File | What it proves |
|---|---|---|
| 1 | [01-auto-switch.md](./01-auto-switch.md) | AGM automatically switches to the correct instance/account when quota or focus conditions trigger |
| 2 | [02-instance-open.md](./02-instance-open.md) | Opening an instance from AGM launches/focuses the correct IDE window, process, and workspace |
| 3 | [03-prompt-preview.md](./03-prompt-preview.md) | The prompt preview renders exactly what will be sent (instance target, prompt text, attachments/context) |
| 4 | [04-prompt-send.md](./04-prompt-send.md) | Entering a prompt and sending it reaches the right instance conversation in the IDE, verbatim |

## Master checklist

[05-checklist.md](./05-checklist.md) is the single page to print and tick
through: preconditions (versions, machine state, accounts), the ordered
flow list with checkboxes, the evidence to attach per flow (screenshots,
log excerpts with paths), the exact PASS/FAIL verdict format, and the
"when blocked" section naming the one question to ask the user when a
step cannot proceed.

## Rules for the tester

1. Work the flows in order — each builds on the previous.
2. Tick every `[ ]` box as you go; do not skip.
3. The **destination-first** ordering is mandatory for prompt flows: the
   destination (machine → instance → conversation) must be explicit
   before any prompt text is accepted. Never guess the destination.
4. On any failed step, follow the flow's **failure branch** (exact
   commands/paths to check) — do not improvise a workaround.
5. Report with the verdict format from `05-checklist.md`, attaching the
   required evidence. A flow without evidence is not PASS.

## Notes

- These procedures are for on-machine runs (the app GUI cannot be driven
  from a headless sandbox). The tester must have AGM installed and the
  test machine available.
- Related: [voice-command feasibility note](../voice-command-feasibility.md)
  (analysis only — voice transcription is not implemented).
