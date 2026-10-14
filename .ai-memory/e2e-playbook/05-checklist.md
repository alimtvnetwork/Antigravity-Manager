# Master E2E checklist — Antigravity-Manager prompt flows

Run the four flows **in order**. Each flow is its own playbook file with
exact labels, failure branches, and a verdict rule; this file is the
master control sheet: preconditions, the ordered flow list with tick
boxes, how to report, and what to do when blocked.

Flow order: **01-auto-switch → 02-instance-open → 03-prompt-preview →
04-prompt-send**. Later flows depend on earlier ones: 03 and 04 need a
running instance (02), and all of them need a working bot/daemon where
stated.

> **Standing rule across all flows:** the tester must confirm the
> destination **before** sending any prompt text. Destination first, then
> prompt — always.

## Preconditions (whole run)

- [ ] The `agm` binary under test is the build you intend to test.
      Confirm with `agm --version` and compare against the commit/tag
      under test. Run one command at a time (this app is memory-sensitive:
      parallel builds plus heavy scans have crashed WebView2 with
      `0xe0000008`).
- [ ] At least **2 configured instances** on the test machine, verified
      with `agm instances ls`. Record their ids, seq numbers, and names
      (on the reference host: `default` and `default-copy-5597`).
- [ ] The IDE executable the instances point at is installed on this
      machine.
- [ ] At least one instance is bound to an account whose quota state is
      known (fresh vs. near-empty) — needed for the 01 flow's rotation
      decision to be meaningful.
- [ ] The Telegram inbound daemon is running and the bot is configured
      (bot token + allowed chat id in the Telegram inbound config —
      `src-tauri/src/modules/telegram_inbound/config.rs`). Sanity-check:
      send `/target show` in the bot chat and expect a reply, not silence.
- [ ] The tester's Telegram client is open on the bot chat for flows 03
      and 04.
- [ ] The AGM desktop UI can be opened (`agm open-ui`), or note a CLI-only
      run for the UI-marked steps.
- [ ] **Safety gates are decided up front:** is it acceptable for the
      active profile to switch accounts right now (flow 01, Phase D)?
      Is it acceptable to inject a test prompt into the destination
      conversation (flows 03/04)? A "no" gates those phases — record it,
      do not run them silently.
- [ ] A scratch notes file is open to record one line per step:
      command/message, exit code or reply, one observed number/status,
      PASS or FAIL.
- [ ] Do not print tokens, passwords, or prompt bodies in the result
      block. Record ids, counts, timings, and status words only.

## Ordered flow list

### Flow 01 — auto-switch
*(playbook: `.ai-memory/e2e-playbook/01-auto-switch.md`)*

- [ ] 01 baseline: `agm --version`, `agm instances ls`,
      `agm auto-switch status --json` — ids, threshold (`15.0`), and
      enabled state recorded.
- [ ] 01 decision-only: `agm auto-switch test 98` reports a decision with
      no hang; the saved threshold is unchanged afterwards.
- [ ] 01 real rotation (GATED): only if the safety gate was confirmed
      "yes". Threshold back at `15.0` afterwards (hard requirement).
- [ ] Verdict recorded: `01-auto-switch: PASS|FAIL — ...` (see the
      playbook for the exact format).

### Flow 02 — instance open
*(playbook: `.ai-memory/e2e-playbook/02-instance-open.md`)*

- [ ] 02 baseline: `agm --version`, `agm instances ls` — target instance
      id, `#seq`, name, PID-or-idle recorded.
- [ ] 02 launch: `agm instances launch <id>` (or the card's
      **"Launch Instance"** button) → exactly one new PID, OS window
      title begins with `#<seq> <name> - `, workspace matches the
      instance's configured workspace.
- [ ] 02 focus: launching an already-running instance focuses it — no
      second PID.
- [ ] Verdict recorded: `02-instance-open: PASS|FAIL — <id>,
      <window-title-prefix>, <pid>, <workspace>, <notes>`.

### Flow 03 — prompt preview
*(playbook: `.ai-memory/e2e-playbook/03-prompt-preview.md`)*

- [ ] 03 baseline: bot alive (`/target show` replies), instance and
      conversation seq chosen.
- [ ] 03 preview: `/target <instance> <conversation>` → the
      `🎯 Next prompt goes here` message renders the exact instance and
      conversation typed — every element ticked off before any prompt
      text is sent.
- [ ] 03 confirm: send the test prompt → `🎯 Prompt sent to ...` receipt
      + `🚀 Prompt Injected Locally!` block; `Prompt Content:` echo is
      verbatim; attachment caption checked.
- [ ] 03 cancel: `/target clear` → `🧹 Aim cleared.`; a following plain
      message dispatches nothing.
- [ ] Verdict recorded: `03-prompt-preview: PASS|FAIL — <instance>,
      <conversation>, <prompt-id>, <notes>`.

### Flow 04 — prompt send
*(playbook: `.ai-memory/e2e-playbook/04-prompt-send.md`)*

- [ ] 04 destination FIRST: `/target <instance> <conversation>`
      confirmed and tied to the visible IDE window (title prefix
      `#<seq> <name> - `) before any prompt text.
- [ ] 04 send: prompt dispatched → receipt with prompt id;
      `Instance:` matches the confirmed destination; content echo
      verbatim.
- [ ] 04 IDE proof: the destination IDE window shows the prompt message
      verbatim in the right conversation; the IDE acknowledges it;
      `agm prompts ls -i <instance>` lists the prompt row.
- [ ] Verdict recorded: `04-prompt-send: PASS|FAIL — <instance>,
      <conversation>, <prompt-id>, <ide-title-prefix>, <notes>`.

## How to report

Attach, per flow:

1. **CLI transcript** — every `agm` command with exit code and output
   (`--version`, `instances ls`, `auto-switch status --json`,
   `instances launch <id>`, `prompts ls -i <instance>`).
2. **Bot transcript** — every Telegram message sent and the bot's exact
   reply (copy-paste the text; screenshots for the receipt blocks).
3. **Screenshots** —
   - Settings → **"Auto Profile Switcher & Task Resumption"** panel
     (flow 01).
   - Instances page: target card idle (gray dot) and running
     (green pulsing dot); the **"Launch Instance"** tooltip (flow 02).
   - Opened IDE window with its `#<seq> <name> - ` title prefix visible
     (flow 02).
   - `🎯 Next prompt goes here` preview message (flow 03).
   - Full `🚀 Prompt Injected Locally!` receipt (flows 03, 04).
   - IDE conversation showing the prompt message (flow 04).
4. **Log excerpts with paths** — `app.log` lines around each dispatch /
   rotation / launch timestamp:
   - Linux/macOS default: `~/.antigravity_tools/logs/app.log`
   - Windows: `%USERPROFILE%\.antigravity_tools\logs\app.log`
   - If the data dir was moved, the `ABV_DATA_DIR` env var or the
     location pointer file takes precedence (see
     `src-tauri/src/modules/account/paths.rs`).

Record each flow's verdict as exactly one line (PASS or FAIL, no other
words between the marker and the dash):

```
01-auto-switch: PASS|FAIL — <notes>
02-instance-open: PASS|FAIL — <id>, <window-title-prefix>, <pid>, <workspace>, <notes>
03-prompt-preview: PASS|FAIL — <instance>, <conversation>, <prompt-id>, <notes>
04-prompt-send: PASS|FAIL — <instance>, <conversation>, <prompt-id>, <ide-title-prefix>, <notes>
```

A run summary is four verdict lines plus the preconditions sheet. If a
flow was skipped (gate declined, CLI-only run), say so in its `<notes>`
field — e.g. `PASS (decision-only) — real rotation not attempted (gate
declined)`.

## When blocked

If a step cannot proceed (missing credential, no machine access, bot
silent, gate declined), stop that flow and ask the tester this single
question — fill in the brackets and ask nothing else until they answer:

> **Blocked at `<flow 01/02/03/04>` step `<step id>` because
> `<what is missing or failing — exact command, reply, or error>`. Do
> you want me to (a) wait while you `<the one action only the tester can
> do, e.g. approve the rotation / reconfigure the bot token / open the
> IDE on the target machine>` and then continue, or (b) stop here and
> report the verdict on what is done so far?**

Rules for the blocked report:

- Name the exact step, the exact last observed output (command + exit
  code, or bot message + reply), and the exact log path checked.
- Never guess the missing piece — if the tester must supply it (bot
  token, machine access, on-screen error text), say so in the question.
- A declined gate is not a failure: record `PASS (decision-only)` /
  `not attempted (gate declined)` and move on.
- Two consecutive blocked steps in the same flow ends the run for that
  flow — report partial verdicts for the completed flows and stop.
