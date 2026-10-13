# E2E Test Cases For An AI Runner

- **Slug:** `supabase-connection-and-ui-ux-enhancement`
- **Status:** `PENDING` — these cases have not been executed against a running desktop window
- **Code under test:** `844348f0` and `1548757c` on `main`
- **Audience:** an AI with a desktop window and a browser, not a unit-test harness

Do not mark any case passed from a code reading. A pass requires the action, the observation, and the evidence listed for that case.

## How to run

1. Start the desktop app from a build that contains the two commits above. Leave the window in the foreground.
2. Use one real instance that already has at least one project and one conversation in Prompts and History.
3. Record each case as `PASS` or `FAIL` plus one sentence of what you saw. Save a screenshot for every visual case.
4. Do not paste API keys, tokens, or prompt bodies into the result note. Name the screen and the error title only.
5. Stop on the first crash. The window closing, a blank webview, or the process exiting is a `FAIL` even if a later case would have passed.

## Evidence to keep

- Screenshot of the Instances quota column after the layout case.
- Screenshot of the update card after the color case.
- Screenshot of the error modal after Send and after Enqueue, with the stack tab visible and the prompt text hidden.
- If the window dies, copy the newest lines from the app crash log `crash_stacktrace.log` in the app log directory. Do not invent a path. Say whether the file existed.

## Case 01 — Send does not close the window

1. Open Instances.
2. Open Prompts and History for a running or stopped instance.
3. Select a conversation that has prompt text.
4. Click the green **Send** button in the detail header. The hotkey is `N`. Do this once.
5. Wait 5 seconds.

**Pass:** the Antigravity Manager window is still open and still on the prompt tree. Either a short success line appears, or one error modal opens. The process is still running.

**Fail:** the window closes, the webview goes white, or the OS shows the app has stopped.

## Case 02 — Enqueue does not close the window

1. Stay in the same prompt tree.
2. Open the chevron next to Send.
3. Click **Enqueue Prompt** once.
4. Wait 5 seconds.

**Pass:** the window stays open. A success line says the prompt was enqueued, or one error modal opens.

**Fail:** same crash signals as Case 01.

## Case 03 — One error modal, with a stack

Run this only if Case 01 or Case 02 opened an error modal. If both succeeded with no error, mark this case `SKIP` and say both dispatches succeeded.

1. Count the error modals on screen. There must be one, not two stacked dialogs.
2. Open the stack tab in that modal.

**Pass:** one modal. The stack tab shows a stack or a Rust panic backtrace. The message names `send_prompt_now` or `enqueue_prompt`. The prompt body is not required in the message.

**Fail:** two modals, an empty stack tab, or only `console` output with no modal.

## Case 04 — The error is still there after restart

1. If a dispatch error occurred, close the app with the window control. Do not clear site data.
2. Start the app again.

**Pass:** the error modal opens once on startup and the stored message matches the previous failure. The storage key is `agm.lastClientError`.

**Fail:** the app starts with no record of the previous failure, or it crash-loops on startup.

If Case 01 and Case 02 both succeeded, mark this case `SKIP`.

## Case 05 — Instance quota bars are not crushed to the left

Compare with `https://prnt.sc/1x97cQ1IiNIP`.

1. Open Instances in table view.
2. Find a row whose bound account has both a 4-hour quota and a weekly quota.
3. Look at the quota cell.

**Pass:**

- The cell is wide enough for a full track. It is not a thin strip stuck on the left of the row.
- The 4-hour bar is on its own row. The weekly bar is the next row, under it, with a gap.
- Each track fills the cell width. Checkpoint dots sit on the track, not piled on the left edge.
- A healthy fill uses green `#1af18d`.

**Fail:** the track width collapses to the label, both bars share one crushed line, or the healthy color is blue or purple.

## Case 06 — Accounts table bars fill their columns

1. Open Accounts.
2. Look at the 4-hour column and the weekly column for an enabled account.

**Pass:** each `QuotaProgressBar` fills its own column. The bar is not packed into the right or left edge with empty track missing.

**Fail:** the bar is only a few pixels wide inside a wide cell.

## Case 07 — Update card uses the same green

Compare with `https://prnt.sc/1_fpQFgQqneF`.

1. Open the update card. Use the in-app check if the card is not already visible. If no update exists, mark this case `SKIP` and say the card never appeared.
2. Inspect the icon tile, the version text, the download fill if it appears, and the install button.

**Pass:** those accents are `#1af18d` or the darker green `#0f8f55` on the version text. The install button text is dark on the green fill.

**Fail:** the card icon or install button is still a blue-to-purple gradient.

## Case 08 — Supabase test does not crash Settings

Credentials stay in repo-secrets. Read them from `02-antigravity-manager/01-lovable-supabase-url.txt`, `02-lovable-supabase-publishable-key.txt`, `03-personal-supabase-url.txt`, and `04-personal-supabase-publishable-key.txt`. Do not copy the key values into this file or into the test result.

1. Open Settings, Supabase section.
2. Confirm the two endpoints are listed, or add them from those secret files.
3. Click Test on the Lovable endpoint. Wait for the result.
4. Click Test on the personal endpoint. Wait for the result.

**Pass:** Settings stays open. Each test ends in a visible success or a visible error. A DNS failure (`No such host is known`) is a recorded error, not a crash, and not a pass for connectivity.

**Fail:** Settings closes, or the test button does nothing with no error.

## Case 09 — Send with no conversation selected

1. Open Prompts and History.
2. Clear the conversation selection if the UI allows it. If a conversation is always selected, mark `SKIP`.
3. Click Send.

**Pass:** the window stays open and the UI says to select a prompt first, or the error store records `resolve_send_target`.

**Fail:** the window closes.

## Result block

Copy this block into the run note and fill it. Do not leave a bracket empty. Use `SKIP` with a reason when the case says you may skip.

```text
Build:
Window stayed open after Send:
Window stayed open after Enqueue:
Error modal count:
Stack visible:
Restart replay:
Instance bars:
Accounts bars:
Update card color:
Supabase Lovable:
Supabase personal:
No-selection Send:
```
