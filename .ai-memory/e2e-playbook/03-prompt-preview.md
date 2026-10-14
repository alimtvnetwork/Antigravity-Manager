# 03 — Prompt preview end-to-end

## Purpose

Verify that the prompt preview renders exactly what will be sent before
dispatch: the instance target, the prompt text, and any attachments or
context. In this app, "preview" is a Telegram-bot concept. The flow that
exists today is the **aim** flow in
`src-tauri/src/modules/telegram_inbound/ui_remote.rs`:

1. The tester sets a destination with
   `/target <instance> <conversation>` (e.g. `/target default C001`).
2. The bot replies with a **preview message** —
   `🎯 Next prompt goes here` — rendering the exact instance and
   conversation the next prompt will go to.
3. The tester composes the prompt as the next plain message.
4. The bot dispatches it and replies with a **delivery receipt** that
   echoes the prompt content verbatim, so the tester can verify what was
   actually sent.

What "preview" means here: the bot's `🎯 Next prompt goes here` message
is the preview. There is no separate preview screen in the current build.
Confirm = send the prompt text. Cancel = `/target clear`.

> **Build-variant note.** Task-07 ("Telegram — `/send` flow + colorful
> `/help`") will replace or supplement `/target` with a `/send` two-step
> destination picker (machine → instance → conversation). If the build
> under test shows a `/send` picker instead of the `/target` replies
> below, use the picker's exact rendered labels for the same checks and
> record the swap in your notes. Do not invent labels — quote what the
> bot actually prints.

## Preconditions

- [ ] The `agm` binary under test is the build you intend to test
      (`agm --version`; must match the commit/tag under test). Run one
      command at a time.
- [ ] At least one configured instance exists, verified with
      `agm instances ls`. Record its id and a conversation seq on it
      (e.g. `C001` — a `C`-seq from the prompt tree, or a `P`-seq; the
      injection path resolves `P001`/`GM:#1`/`C001`/conv-UUID-prefix in
      `src-tauri/src/modules/telegram_inbound/injection.rs`).
- [ ] The Telegram inbound daemon is running and the bot is configured
      (bot token + allowed chat id set in the Telegram inbound config —
      see `src-tauri/src/modules/telegram_inbound/config.rs`). The bot
      must reply in the tester's chat; if the bot is silent at any step,
      the failure branch of P2 applies.
- [ ] The tester's Telegram client is open on the bot chat and they can
      send and read messages.
- [ ] A scratch notes file is open to record one line per step:
      message sent, reply received (verbatim), PASS or FAIL.

## Key facts (verified in source)

- The aim lives in a `PROMPT_AIM` mutex in `ui_remote.rs` and expires
  after **10 minutes** (`TARGET_TTL_SECS: 600`). After expiry, a plain
  message is **not** treated as a prompt.
- `/target` (bare), `/target show`, and `/target ls` show the current
  aim. `/target clear` and `/target reset` forget it. Aliases: `/where`,
  `/aim`, `/agm target`.
- A plain message arriving with **no live aim** does nothing — no
  dispatch, no error. The tester learns this only by checking
  `/target show`, which prints
  `🎯 No aim set. Send /target default C001 first, then the prompt text.`
- The dispatch ack echoes **Prompt Content:** verbatim (capped at 800
  chars) inside the `🚀 Prompt Injected Locally!` block, and attaches a
  screenshot of the **manager window** (caption
  `AGM after prompt to {conversation} on {instance}`) — not a screenshot
  of the IDE conversation.
- Prompt text is wrapped by `wrap_telegram_prompt`
  (`src-tauri/src/modules/telegram_inbound/reports_cluster_b.rs`): a
  **single-word** message that matches a prompt-template slug is expanded
  to the template content, and `--prefix`/`--suffix` tokens add template
  content. For verbatim preview checks, always use a **multi-word** prompt
  with no template tokens.
- Never paste tokens, passwords, or prompt bodies beyond the test prompt
  into the result block. Record ids, counts, timings, and status words
  only.

## Procedure

### Phase A — baseline (read-only)

- [ ] **A1.** Run `agm --version`.
      **Expected:** version matches the build under test.
      **Evidence:** the version string.
      **If it fails:** do not continue; find the correct binary first.

- [ ] **A2.** Run `agm instances ls`.
      **Expected:** exit 0; the target instance is listed with its id.
      **Evidence:** record the instance id.
      **If it fails:** exit 1 or the instance is missing → choose a
      different instance or stop; do not invent the id.

- [ ] **A3.** In the Telegram bot chat, send `/target show`.
      **Expected:** a reply showing either the current aim
      (`🎯 Current aim — Instance: ..., Conversation: ...`) or
      `🎯 No aim set. Send /target default C001 first, then the prompt text.`
      This proves the bot is alive and the aim state is readable.
      **Evidence:** the exact reply.
      **If it fails:** no reply at all → the bot/daemon is down or the
      chat id is not in the allowed list. Inspect
      `~/.antigravity_tools/logs/app.log` (Windows:
      `%USERPROFILE%\.antigravity_tools\logs\app.log`; if the data dir
      was moved, the `ABV_DATA_DIR` env var or the location pointer file
      takes precedence — see
      `src-tauri/src/modules/account/paths.rs`) and do not continue.

### Phase B — open the preview (set destination)

- [ ] **B1.** In the bot chat, send
      `/target <instance> <conversation>` (e.g. `/target default C001`).
      **Expected:** the bot replies
      `🎯 Next prompt goes here`
      with lines
      `• Instance: <instance>`
      `• Conversation: <conversation>`
      and the note `Send the prompt as the next plain message. It stays
      aimed for 10 minutes. /target clear forgets it.`
      **Evidence:** screenshot of the preview message.
      **If it fails:**
      - No reply → see A3 failure branch.
      - The rendered instance or conversation does not match what was
        typed → **preview mismatch**, FAIL this phase; record both
        strings exactly.

- [ ] **B2.** Verify each rendered element of the preview against input.
      **Expected:** the `Instance:` code block contains exactly the
      instance id typed in B1; the `Conversation:` code block contains
      exactly the conversation seq typed in B1; the TTL note says
      10 minutes.
      **Evidence:** tick each element off in the notes file.
      **If it fails:** any element differs → do not send the prompt;
      record the mismatch.

### Phase C — confirm path (send and verify the echo)

- [ ] **C1.** Immediately after B2, send the prompt as a plain message.
      Use a fixed, multi-word test prompt with no template tokens, e.g.
      `e2e preview check: reply with the word ok`.
      **Expected:** the bot replies
      `🎯 Prompt sent to <conversation> on <instance>`
      followed by the `🚀 Prompt Injected Locally!` block containing
      `• Prompt ID: <p-...>`
      `• Instance: <instance>`
      `• Target Workspace: <project>`
      `• Path: <repo-path>`
      `• Status: 🟢 Executing via agy` (or `⚪ Enqueued in Split DB`)
      `• Prompt Content:` with the prompt text verbatim in a `<pre>`
      block.
      **Evidence:** screenshot of the full receipt; record the prompt id.
      **If it fails:**
      - The receipt never arrives → the dispatch may have failed after
        the plain message was consumed; run
        `agm prompts ls -i <instance>` to see whether the prompt row
        exists anyway, then inspect the log tail.
      - `⚠️ Missing Prompt Content:` or `⚠️ Empty Prompt:` → the message
        was consumed by a different command parser; record the exact
        text and do not retry the same text.

- [ ] **C2.** Compare the `Prompt Content:` echo against the prompt typed
      in C1, character for character.
      **Expected:** verbatim match (case, punctuation, spacing).
      **Evidence:** note "echo verbatim" or the exact divergence.
      **If it fails:** the text differs → record the before/after strings;
      suspect template expansion (`wrap_telegram_prompt`) or HTML
      escaping. FAIL the case.

- [ ] **C3.** Check the attachments/context on the receipt.
      **Expected:** a screenshot is attached with the caption
      `AGM after prompt to <conversation> on <instance>`; it shows the
      **manager window**, not the IDE conversation. If the receipt says
      `Screenshot was not attached: ...`, record the reason text.
      **Evidence:** note attached/not-attached and the caption.
      **If it fails:** no attachment and no reason line → record
      "attachment missing, no reason given". (Note: the capture helper
      is Windows-only — `capture_once` in `ui_remote.rs` returns
      `"window screenshot is implemented for Windows"` elsewhere.)

- [ ] **C4.** Verify the instance/context rendered in the receipt.
      **Expected:** `Instance:` matches the B1 preview; `Target
      Workspace:` / `Path:` match the workspace the instance is bound
      to (compare with the instance card / `agm instances ls` data);
      `Status:` is either `🟢 Executing via agy` or
      `⚪ Enqueued in Split DB` — both are valid, record which.
      **Evidence:** the four values.
      **If it fails:** `Instance:` differs from the B1 preview → **wrong
      destination**, FAIL the case. `⚠️ Failed to Save Prompt: ...` →
      record verbatim; the dispatch path is broken.

### Phase D — cancel path

- [ ] **D1.** Set an aim again (`/target <instance> <conversation>`),
      then send `/target clear`.
      **Expected:** the bot replies
      `🧹 Aim cleared. The next plain message is no longer sent as a prompt.`
      **Evidence:** the exact reply.
      **If it fails:** no clear confirmation → record; verify with
      `/target show` whether the aim is still set.

- [ ] **D2.** Send a plain-text sentence (e.g. `e2e cancel check: ignore
      this`).
      **Expected:** nothing is dispatched — no `🎯 Prompt sent to ...`
      receipt; `/target show` still reports no aim; `agm prompts ls -i
      <instance>` shows no new row.
      **Evidence:** the absence (note timestamps).
      **If it fails:** a dispatch receipt appears → the clear did not
      take effect; FAIL the case and record the prompt id it created.

### Phase E — expiry path (optional, time-boxed)

- [ ] **E1.** Set an aim, wait 10+ minutes without sending a prompt, then
      send a plain-text sentence.
      **Expected:** no dispatch receipt (the aim expired silently);
      `/target show` reports `🎯 No aim set.`.
      **Evidence:** note the timestamps.
      **If it fails:** a receipt appears → the TTL is not enforced;
      record the elapsed time.

### Evidence to capture

1. Screenshot: the `🎯 Next prompt goes here` preview message.
2. Screenshot: the full `🚀 Prompt Injected Locally!` receipt.
3. The prompt id from C1; the exact test prompt text.
4. Screenshot: the `🧹 Aim cleared.` reply.
5. Log excerpt: `app.log` lines around the C1 dispatch timestamp
   (`~/.antigravity_tools/logs/app.log`; daily rolling).

### PASS/FAIL verdict rule

- **PASS** iff: the preview rendered the exact instance and conversation
  typed (B1/B2); the receipt arrived with a verbatim `Prompt Content:`
  echo (C1/C2); the receipt's `Instance:` matched the preview (C4); and
  the cancel path left no dispatch behind (D2).
- **FAIL** if any of these hold: the preview misrendered the destination;
  the echo diverged from the composed prompt; the receipt's instance
  differed from the preview; a prompt was dispatched after
  `/target clear`; or the bot went silent and the log shows no dispatch
  entry.
- Record the verdict as one line:
  `03-prompt-preview: PASS|FAIL — <instance>, <conversation>, <prompt-id>, <notes>`.
