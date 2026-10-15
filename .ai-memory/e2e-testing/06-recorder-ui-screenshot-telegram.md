# 06 — Recorder, CLI UI, screenshots, Telegram cards

A later AI runs this file without prior chat. Compare the live Telegram chat to the two images in the spec. Do not invent a pass from reading source.

Spec: `02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md`

Images:

- Broken card: `assets/screenshots/enhancing-telegram-ui-and-gitmap-integration-01.png`
- Target button style: `assets/screenshots/enhancing-telegram-ui-and-gitmap-integration-02.png`

## What image 1 showed, and the fix

Image 1 is a Telegram chat with `agm_v1_bot` after a switch. Two defects:

- [ ] The balance lines print the characters `&nbsp;&nbsp;` before `└ Balances`. Telegram HTML only keeps `&lt;`, `&gt;`, `&amp;`, and `&quot;`. `&nbsp;` is not one of them, so it stays as text.
- [ ] The rotation card and the "Post-Switch Liveness Confirmed" card have no buttons under them.

Fix that must already be in the binary under test:

- [ ] Those spaces are Unicode figure spaces, not `&nbsp;`.
- [ ] Bullets are `•`, not `&bull;`.
- [ ] Both cards have inline buttons: Check Balance, Export CSV, Prompt Instance, Refresh Status.

Image 2 is the layout target: one dark message, then a full-width action button under it. AGM uses several buttons, not one green stock button. Pass means the buttons are under the card and a tap runs the command. Fail means raw `&nbsp;` is still visible.

## Recorder

The recorder writes UI and CLI actions into `ui_audit.db`. It does not click for you. Start it before the actions you want to prove.

- [ ] `agm ui record start` prints `[OK] UI audit recording started`.
- [ ] `agm ui record status` prints `ACTIVE` and an entry count.
- [ ] Do one real action, for example `agm ui goto settings/ssh` or `agm ff -i default-copy-5597 --force` on the copy only.
- [ ] `agm ui record list --limit 20` shows a row with actor, trigger, method, target, and result. Do not require the prompt body.
- [ ] `agm ui record list --limit 20 --json` is JSON on stdout only.
- [ ] `agm ui record export` writes a file and prints the path. Do not commit that file.
- [ ] `agm ui record stop` prints stopped.
- [ ] `agm ui record clear` only after the export is saved. Pass: status count returns to 0.
- [ ] Fail: `start` errors, or the action you just ran is missing from `list`.

## CLI drives the window

`agm ui ls` is a static manifest. It does not read the live DOM. A screen pass still needs the window or `agm shot`.

- [ ] `agm ui open` (same as `agm open-ui`). Pass: stdout contains a PID, or the window was already open and stays open.
- [ ] `agm ui goto settings/ssh`. Pass: no "run agm ui open first", no React Router 404. The SSH & GitMap page is the visible page.
- [ ] `agm ui goto settings/email`. Pass: Telegram settings are on that page.
- [ ] `agm ui ls settings/ssh`. Pass: prints the element manifest for that route. Fail: you invent an element id that is not in this list.
- [ ] `agm ui scroll <id> settings/ssh` using an id from `ls`. Pass: the command exits 0. Fail: unknown id.
- [ ] `agm ui click <id> settings/ssh` only for a safe control (copy a command, refresh status). Do not click install-gitmap or delete-node unless the user asked.
- [ ] `agm ui goto` while the app is closed. Pass: exit 1 and the hint to run `agm ui open` first.

## Screenshots

- [ ] With the window open on the page you care about, run `agm shot` (alias `agm screenshot`).
- [ ] Pass: `[OK] AGM window screenshot captured (<n> bytes) and sent to the saved Telegram chat.`
- [ ] Fail: error because the bot token or chat id is missing. Write `NOT RUN` and do not paste a token.
- [ ] The PNG in Telegram must show the page you opened with `agm ui goto`, not the desktop.
- [ ] Keep a copy under `assets/screenshots/` only when you are saving evidence for a commit the user asked for.

## Telegram notifications to re-check against image 1

The bot must already be configured. Send these in the bot chat, or trigger them from CLI.

- [ ] Switch or fast-forward the copy instance so a rotation card is posted.
- [ ] The new card does not contain the text `&nbsp;` or `&bull;`.
- [ ] Tap Check Balance. The reply is the `/balance` credit table (machine, instance, account, 4h, weekly).
- [ ] Tap Export CSV. A `.csv` document arrives, not only HTML text.
- [ ] Tap Prompt Instance. The bot enters the send or prompt flow and asks for the instance, then the text. Do not send a long prompt. A short line is enough.
- [ ] Tap Refresh Status. A status card returns.
- [ ] `/help` shows copyable examples for `/balance`, `/export accounts csv`, `/prompt <machine:instance> <text>`, `/ssh nodes`.
- [ ] `/export instances csv` and `/export balance json` each arrive as a file.
- [ ] If no bot is configured, mark this section `NOT RUN`. Do not call the HTML fix passed.

## Stop

- [ ] Recorder is stopped.
- [ ] Auto-switch threshold is 15.0 if you changed it.
- [ ] Result lines name the image check as PASS or FAIL, not "code looks fixed".
