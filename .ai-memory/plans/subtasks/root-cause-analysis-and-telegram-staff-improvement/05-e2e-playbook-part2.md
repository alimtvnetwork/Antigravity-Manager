# Task-05: E2E playbook — prompt flows + master checklist

## Status
DONE

## Objective
Write `.ai-memory/e2e-playbook/03-prompt-preview.md`,
`.ai-memory/e2e-playbook/04-prompt-send.md`, and
`.ai-memory/e2e-playbook/05-checklist.md`.

## 03-prompt-preview.md must contain
- Purpose: verify the prompt preview renders exactly what will be sent
  (instance target, prompt text, attachments/context).
- Steps: compose prompt → open preview → verify each rendered element
  matches input → confirm or cancel paths.

## 04-prompt-send.md must contain
- Purpose: verify entering a prompt and sending it reaches the right
  instance conversation in the IDE.
- Steps: select destination instance → enter prompt text → send → switch to
  IDE → open the conversation → confirm the prompt arrived verbatim →
  confirm any auto-response/acknowledgment.
- The "destination first, then prompt" ordering is mandatory and must be
  called out explicitly (mirrors the Telegram `/send` flow).

## 05-checklist.md must contain
- The master checklist: preconditions (versions, machine state, accounts),
  then the ordered flow list (auto-switch → instance open → prompt preview
  → prompt send), each with tick boxes.
- A "how to report" section: what evidence to attach per flow
  (screenshots, log excerpts with paths), and the exact PASS/FAIL verdict
  format.
- A "when blocked" section: the single question to ask the user when a step
  cannot proceed (e.g. credentials, machine access).

## Style rules
- Dummy-AI proof: exact labels/commands, no assumed knowledge.
- `[ ]` checkboxes throughout.

## Constraints
- GitMap search only. Relative paths only.
