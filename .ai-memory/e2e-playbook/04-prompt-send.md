# 04 — Prompt send end-to-end

## Purpose

Verify that entering a prompt and sending it reaches the **right**
instance's conversation in the Antigravity IDE: the prompt is dispatched
to the exact destination the tester chose, the IDE shows it verbatim in
that conversation, and the bot acknowledges the send. This flow covers the
Telegram `/target` aim flow
(`src-tauri/src/modules/telegram_inbound/ui_remote.rs`) into the prompt
injection path
(`src-tauri/src/modules/telegram_inbound/injection.rs`), and ends with
the human-visible check inside the IDE.

> **Mandatory ordering — destination first, then prompt.** This is the
> standing rule of this flow and mirrors the Telegram `/send` flow: the
> tester must set the destination (`/target <instance> <conversation>`)
> and see the bot confirm it BEFORE any prompt text is sent. Never type
> the prompt first and hope the destination is right. A prompt sent with
> no live aim is silently dropped — the tester only learns this by
> checking `/target show`.

> **Build-variant note.** If the build under test ships Task-07's
> `/send` two-step (destination picker machine → instance →
> conversation), the destination step is the picker's explicit
> confirmation ("Prompt will go to <machine> → <instance> →
> <conversation>") instead of the `/target` preview. The ordering rule is
> unchanged: destination confirmed first, prompt text second.

## Preconditions

- [ ] The `agm` binary under test is the build you intend to test
      (`agm --version`; must match the commit/tag under test). Run one
      command at a time.
- [ ] At least one configured instance exists, verified with
      `agm instances ls`. Record its id, seq number, and name.
- [ ] The destination instance's IDE window is **open** on this machine
      (use `.ai-memory/e2e-playbook/02-instance-open.md` if it is not).
      Record the OS window title prefix: it begins with
      `#<seq> <name> - ` (e.g. `#2 default-copy-5597 - `).
- [ ] The destination conversation seq (e.g. `C001`) is known and belongs
      to that instance's workspace/project.
- [ ] The Telegram inbound daemon is running; the bot replies to the
      tester's chat (sanity-check with `/target show` — expect a reply,
      not silence).
- [ ] The tester's Telegram client is open on the bot chat.
- [ ] A scratch notes file is open to record one line per step:
      message sent, reply received (verbatim), PASS or FAIL.
- [ ] Do not run this on the tester's live working conversation without
      their confirmation: the send actually injects a prompt into the IDE
      session.

## Key facts (verified in source)

- The aim is set with `/target <instance> <conversation>` and stays live
  for **10 minutes**; `/target clear` forgets it
  (`src-tauri/src/modules/telegram_inbound/ui_remote.rs`).
- The next plain message after a live aim is dispatched through
  `execute_prompt_injection` and the bot answers
  `🎯 Prompt sent to <conversation> on <instance>` plus the
  `🚀 Prompt Injected Locally!` block with `Prompt ID:`, `Instance:`,
  `Target Workspace:`, `Path:`, `Status:` (`🟢 Executing via agy` /
  `⚪ Enqueued in Split DB`), and the verbatim `Prompt Content:`.
- Prompt text is wrapped by `wrap_telegram_prompt`
  (`src-tauri/src/modules/telegram_inbound/reports_cluster_b.rs`): use a
  **multi-word** test prompt with no `--prefix`/`--suffix` template tokens
  so the IDE-side text is verbatim.
- The direct form `/prompt C001 --instance <id> <text>` sends without an
  aim. Missing pieces reply
  `⚠️ Missing Arguments:` (usage) or `⚠️ Empty Prompt:`.
- Do not print tokens, passwords, or prompt bodies in the result block.
  Record ids, counts, timings, and status words only.

## Procedure

### Phase A — baseline

- [ ] **A1.** Run `agm --version`.
      **Expected:** version matches the build under test.
      **Evidence:** the version string.
      **If it fails:** do not continue; find the correct binary first.

- [ ] **A2.** Run `agm instances ls`.
      **Expected:** exit 0; the destination instance listed with id, seq,
      name, and a running PID.
      **Evidence:** record id, `#seq`, name, PID.
      **If it fails:** the instance is idle → open it first
      (02-instance-open); missing → choose another instance.

- [ ] **A3.** In the bot chat, send `/target show`.
      **Expected:** a reply (current aim or
      `🎯 No aim set. Send /target default C001 first, then the prompt text.`).
      This is the bot-alive check.
      **Evidence:** the exact reply.
      **If it fails:** no reply → the bot/daemon is down or the chat is
      not in the allowed list; inspect
      `~/.antigravity_tools/logs/app.log` (Windows:
      `%USERPROFILE%\.antigravity_tools\logs\app.log`; the `ABV_DATA_DIR`
      env var or the location pointer file may override the data dir —
      see `src-tauri/src/modules/account/paths.rs`).

### Phase B — destination FIRST (mandatory)

- [ ] **B1.** In the bot chat, send
      `/target <instance> <conversation>`.
      **Expected:** the bot replies `🎯 Next prompt goes here` with
      `• Instance: <instance>` and `• Conversation: <conversation>`
      matching exactly what was typed.
      **Evidence:** screenshot of the confirmation; record both values.
      **If it fails:**
      - No reply → see A3 failure branch.
      - Rendered values differ from the typed values → STOP. Do not send
        the prompt; the destination is wrong. Record both strings and
        FAIL the case here.

- [ ] **B2.** Double-check the destination against the IDE.
      The instance id in B1 must be the instance whose IDE window is open
      (title begins with `#<seq> <name> - `); the conversation must be
      the seq the tester intends to hit.
      **Expected:** both match.
      **Evidence:** tick both off in the notes file.
      **If it fails:** re-run B1 with the corrected values. Never
      proceed with a destination you cannot tie to a visible IDE window.

### Phase C — enter the prompt and send

- [ ] **C1.** Send the prompt as a plain message (multi-word, no template
      tokens), e.g. `e2e send check: confirm you see this message`.
      **Expected:** the bot replies
      `🎯 Prompt sent to <conversation> on <instance>` followed by the
      `🚀 Prompt Injected Locally!` block with `Prompt ID: <p-...>`,
      `Instance:` matching B1, `Status:` either `🟢 Executing via agy` or
      `⚪ Enqueued in Split DB`, and `Prompt Content:` echoing the prompt
      verbatim.
      **Evidence:** screenshot of the full receipt; record the prompt id.
      **If it fails:**
      - No receipt → check whether the prompt row exists anyway with
        `agm prompts ls -i <instance>` (or `--json`); if present, the
        dispatch worked but the reply failed — inspect the log tail.
        If absent, the dispatch failed — inspect the log tail.
      - `⚠️ Missing Arguments:` / `⚠️ Empty Prompt:` → the message was
        consumed by a different parser; record the exact reply and stop.

- [ ] **C2.** Verify the receipt's `Instance:` and `Prompt Content:`.
      **Expected:** `Instance:` equals the B1 destination; the content
      echo is verbatim (compare character for character).
      **Evidence:** tick both off.
      **If it fails:** `Instance:` differs → **wrong instance received
      the prompt**, FAIL the case. Content differs → record the
      divergence (suspect template expansion); FAIL the case.

### Phase D — IDE verification (the human-visible proof)

- [ ] **D1.** Switch to the destination instance's IDE window.
      **Expected:** an OS window whose title begins with
      `#<seq> <name> - ` matching the B1 instance is in the foreground.
      **Evidence:** note the full title prefix.
      **If it fails:** no such window → the instance is not running; do
      not continue to D2.

- [ ] **D2.** In the IDE, open the destination conversation
      (the `<conversation>` seq from B1).
      **Expected:** the conversation opens and shows the test prompt as a
      new user message.
      **Evidence:** screenshot of the conversation showing the message.
      **If it fails:** the conversation is empty or shows an old state →
      the prompt went elsewhere; check `agm prompts ls -i <instance>`
      for the prompt id and its project binding.

- [ ] **D3.** Compare the IDE-side message text with the C1 prompt,
      character for character.
      **Expected:** verbatim match.
      **Evidence:** note "IDE text verbatim" or the exact divergence.
      **If it fails:** the text differs → record both; template wrapping
      or an intermediate transform altered it — FAIL the case.

- [ ] **D4.** Confirm the IDE acknowledges the prompt (the agent starts
      working on it, or the conversation shows it queued/running).
      **Expected:** some visible activity or queued state for the new
      message — not silence.
      **Evidence:** note what the IDE shows.
      **If it fails:** the message sits with no activity → record the
      state; the dispatch saved but the execution path (agy) did not
      pick it up — inspect the log.

### Phase E — CLI cross-check

- [ ] **E1.** Run `agm prompts ls -i <instance>` (or `--json`).
      **Expected:** the prompt row from C1 (its `p-...` id) is listed
      under this instance with status `running` or queued.
      **Evidence:** the row's id and status.
      **If it fails:** no row → the dispatch never persisted; inspect the
      log. Row under a different instance → **wrong destination**,
      FAIL the case.

### Evidence to capture

1. Screenshot: the B1 destination confirmation
   (`🎯 Next prompt goes here`).
2. Screenshot: the full C1 delivery receipt
   (`🚀 Prompt Injected Locally!`).
3. Screenshot: the IDE conversation showing the prompt message.
4. CLI transcript: `agm instances ls`, `agm prompts ls -i <instance>`
   with exit codes.
5. Log excerpt: `app.log` lines around the C1 timestamp
   (`~/.antigravity_tools/logs/app.log`; daily rolling).

### PASS/FAIL verdict rule

- **PASS** iff: the destination was set and confirmed before any prompt
  text was sent (B1/B2); the receipt arrived with the prompt id and
  `Instance:` matching the confirmed destination (C1/C2); the IDE
  conversation for the exact `<conversation>` seq shows the prompt
  message verbatim (D2/D3); the IDE shows activity/ack for it (D4); and
  the CLI lists the prompt row under the destination instance (E1).
- **FAIL** if any of these hold: prompt text was sent before the
  destination was confirmed; the receipt's instance differed from the
  confirmed destination; the IDE conversation does not show the message
  verbatim; the prompt row is missing or under a different instance; or
  the bot went silent with no log entry for the dispatch.
- Record the verdict as one line:
  `04-prompt-send: PASS|FAIL — <instance>, <conversation>, <prompt-id>, <ide-title-prefix>, <notes>`.
