# Learned Memory 31: Prompt Tree Copy, Skill Refs, Telegram 236 E2E, and JSON Switch Lie

- **ID**: `learned-31`
- **Scope**: Prompt Tree Copy, attachment vs skill-ref parsing, Telegram inbound 236, isolated switch+restore E2E, CLI `--json` honesty.
- **Date**: 2026-10-29
- **Related Specs**: `02-spec/21-app/236-telegram-feature-enhancement/`, `02-spec/21-app/235-add-new-option-for-100-improvements/`
- **Related Plans**: `.ai-memory/plans/completed/236-telegram-feature-enhancement.md`, `.ai-memory/plans/completed/235-add-new-option-for-100-improvements.md`
- **Related Issue**: `.ai-memory/issues/66-instances-switch-json-fake-success.md`
- **Version Baseline**: source HEAD `86877cf53` on `v4.277.4` tree; installed GUI/CLI on this node was still `v4.276.0`

---

## 1. Last-30-commit progress (2026-10-29)

Newest first, what this session must remember:

| Commit | What shipped |
|---|---|
| `86877cf53` | Prompt Tree Copy copies full prompt as plain text; skill refs are not rewritten into `Attachments:` |
| `ebc55ae2e` | chore bump `v4.277.4` |
| `59f65a222` | Create Profile modal restyle; Instances header **100 Steps** + Prompt Tree suffix `run-100-steps` |
| `b2cfc5596` / `1ce618741` | Telegram 236: screenshot no self-focus, CSV `sendDocument`, Copy/Full `i:cp:` `.txt`, hover shade, tagged `v4.276.0` |
| `01b7e1840` | Shared plan-100 / next-10 flywheel on CLI, UI, Telegram |
| `982e3e284` | CI: drop macOS from required checks so cancelled hosted runners cannot block Release |

Do not score live GUI behavior against an older installed `agm-ui` than HEAD. This node’s installed binary stayed `v4.276.0` after `86877cf53` landed in git.

---

## 2. Prompt Tree Copy (user: “Copy button is not working properly”)

Inspected turn was GitMap User Change Command (23 words) containing:

```text
@[.agents/skills/execute-parent-task-with-n-steps-v7]
@[execute-parent-task-with-n-steps-v7](file;.agents/skills/execute-parent-task-with-n-steps-v7)
```

### Root causes

1. Primary Copy called `copyPromptWithRichImages`, which ran `formatPromptWithAttachments` on **every** `@[...]` token. Skill refs became fake `Attachments:` lines with absolute paths.
2. `convertImageUriToBase64` then treated those skill paths as images (up to 5s timeout each). UI already showed **Copied!** because `handleCopyAll` did not await and always set success.
3. Native `copy_image_to_clipboard` could replace the clipboard with an image and return success without the prompt text.
4. Truncated `prompt_preview_200w` could win over `full_prompt_text`.

### Fix (`86877cf53`)

- Primary Copy: `copyToClipboard(resolvePromptBody(...))`, await, **Copied!** only on `true`.
- `isFileAttachmentTarget` rejects `.agents/skills/` and extension-less skill names; image `@[file;path.png]` still extracts.
- Rich HTML stays in the Copy dropdown, not the primary button.
- `resolvePromptBody` prefers full text over a truncated preview unless the user edited a different body.
- Tests: `src/utils/__tests__/promptAttachment.test.ts`, `src/components/instances/PromptTreeView/__tests__/promptCopy.test.ts`.

Live source proof (2026-10-29): `extractAttachmentsFromPrompt` on the GitMap prompt returned `[]`; formatted text kept both skill refs and had no `Attachments:` block.

Live GUI click of the **new** Copy button was not possible: Computer Use sidecar was `sidecar_unavailable`, and installed `agm-ui` was still `v4.276.0`. `agm ui click prompt-tree-btn-default` opened the tree on the old binary.

---

## 3. Telegram 236 live proof (installed `v4.276.0`)

Proven on this node, chat id configured, Default PID left running:

- `agm telegram ping` delivered.
- `agm telegram observe` delivered per-instance status cards.
- `agm shot` sent an AGM window photo (~80 KB); `agm-ui` stayed `Responding`.
- A real CSV file arrived via `sendDocument` (`agm-instances-e2e.csv`, empty caption).

Not proven (needs a human Telegram click; no Computer Use):

- Card **Copy** / **Full** (`i:cp:` → whole-prompt `.txt`).
- Inbound `/screenshot` mosaic (IDEs + Instances + Accounts). `agm shot` is manager-window only.

---

## 4. Isolated switch + restore (proven without `--json`)

Sandbox `test-create` → enqueue `queued-1793282428503-0000-2837d0` (`e2e-236-restore-marker`, `fifo_ordered: true`) → real `agm instances switch <id> <email>` (no `--json`).

- Bound email **did** change (`rokixshohag1@gmail.com` → `david.copper.riseup@gmail.com`).
- Prompt swept to `backed_up`, restored FIFO, dispatched via CDP (`1 restored, 1 dispatched, 0 failed`).
- Default stayed the original bound account and PID. `v2-3127` stayed idle.
- `agm instances test-clean` removed the sandbox.

`--json` on the same switch is a fake success. See issue 66. Never use it as E2E evidence of a bound-email change.

---

## 5. Create Profile + 100-steps UI (`59f65a222`)

Create New Profile must not use DaisyUI `.input` / `.select` (magenta focus ring). Dark-glass fields, capsule Cancel/Create, `.agm-hover-shade` (color/shadow only).

100-step breakdown is not Telegram-only: Instances header **100 Steps** scrolls to `#agm-improvements-flywheel`; Prompt Tree Suffix has **100 Steps** (`run-100-steps`) wrapping execute-parent-task-with-n-steps.

---

## 6. Conventions for the next agent

- Copy means verbatim full prompt text. Skill refs are not attachments.
- Prove switch with `agm instances ls` bound_email, never with switch `--json` alone.
- Isolated E2E: `test-create` → operate → `test-clean`. Never stop Default or `v2-3127`.
- Do not run `cargo test --lib` or `tauri build` on the 12 GB host (OOM).
- Relative Git paths only in memory, plans, changelogs.
- Attribution `@aukgit` only in release notes.
