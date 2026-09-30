# Master Plan: Multi-Instance Switching, UI Overlap Fix, Conversation Prune Safety, Telegram Bot Enhancements & Email Telemetry Deduplication

Spec Reference: [02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md](../../../02-spec/21-app/26-instance-switching-ui-telegram-email-fix.md)

## User Request (Verbatim)

```text
Now there is a serious instance bug. That means if you have multiple instances, I cannot switch the instances. It seems like instances are stuck with one with another. So I think you need to fix that. And the Focus, if you did in Focus, Focus does not normally scroll to the position, and it doesn't have a proper coloring for the hover and selected. I asked you several times that it needs to have a different coloring so that it feels like it is highlighted. Okay, fully, this is the one I could understand clearly. So the color combination is very dark with the dark. It does not make any sense. Probably even maybe white background with the dark text, that would make it nice. You can give it a shot, I think. And also at the same time, there are UI bugs. For example, we just click on the instances and then also click on the parts section. Both of these comes up. There is no way to close it. And if you purge the, let's say, purge or clean up the conversation, it should always respect the last few conversation on the running projects. Okay? So that we can understand or get back to the ones that we're working on. Remember that running or queued prompts should not be removed. Remember to update this code in your clear and purge section, prune section. Please confirm that you understood that, and also try to fix the UI bug overlapping issue. There should be a close button to close these panels. Both of these just do not work properly. The instances issue that I cannot switch. I'm clicking on another instance, and again, come back to the worker alpha. I don't know why. So you need to work on it. Also do a git pull before the work, so that there is no inconsistencies. Okay, and make sure your testing is solid. And also there is one more issue I think I need to warn you. That's actually the email. The emails that you provide, the same name is repeated twice, same thing happening in the, let's say, Telegram bot. Same project name multiple times. Don't do that. Create a dictionary to have a distinct names. Okay, and Telegram SMS needs to be improved a lot. The same name repeated twice and many times. I think that is a serious problem that you need to fix. Also at the same time... So in the Telegram, there is same project name mentioned several times. There is a HTML broken down that also you need to fix. I'm just giving you the screenshot, okay, so that you can prioritize this. So I need to see more of the text for each one of the prompt that is running, which I don't see that is kind of bad. And also we should be able to do the AGM update from the Telegram bot. The commands needs to be there, so if there is an update that arrives, it should also put into the status section as well. And too many dark text. I wanted to have the white text, because in the dark, it creates a problem. So try to have this writing properly as much as possible so that the links and other stuff does not become blue under blue. Okay? Depending on the theme, try to understand that as well. Yes. I think I tried to also this stuff. You can have some command example from Git map as well, like the SSH notes. Sending something using Git SSH. Okay, also selecting the machine, how to do that. These type of commands needs to be in the Antigravity selection. Okay? Yeah. So a lot of things you need to improve, try to do that. And also we can merge and prune the conversation. That is the PR command that we could do from the UI. We should be able to do it from the CLI and also from the Telegram bot. Make sure of that. And also try to keep these images saved, okay, so that we can reuse them in the future, I believe. Yeah, I think these are on top of my head. So you can start working on this so that you don't miss anything for sure to respect. None of the image or anything should be missed. It should be UI fix, email fix, Telegram fix task. Okay. And make sure when we get the email, in the email, we should see some of the prompts that these are running, like prompts 200 words at least in the email together. That should be cached in the SQLite database in the future so that it easier to start from the database rather than querying all the time. So we should have query methods using our CLI so that we can do it from the terminal as well. There should be examples of this in the terminal help very clearly. And also this type of example needs to be in the Telegram bot as well so that we can run and test. Do you understand?
```

## Architectural Context & Blast Radius Analysis

### 1. Multi-Instance Switching & Fallback Protection
- **Root Cause Analysis:** When an instance is activated via `switch_instance` or `activate_instance`, if the instance launcher encounters any transient issue or checks `get_current_instance_id()`, it can fall back to the first profile in `instances.json` or `worker-alpha` if the active instance marker in `instances.json` isn't updated atomically prior to launch, or if `active_instance_id` is cached stale in the frontend Zustand store.
- **Remediation:** Ensure atomic update of `active_instance_id` in `instances.json` and split SQLite configuration, emit instant Tauri events, and prevent any unrequested fallback to `worker-alpha`.

### 2. UI Modal Dismissal & Mutual Exclusivity
- **Root Cause Analysis:** `InstanceSelector.tsx` dropdown popover and `agy-clean-modal.tsx` dialog both operate without checking if another overlay is active, and neither provides a prominent close (`X`) button in the title bar or backdrop dismissal.
- **Remediation:** Add top-right close buttons (`X`), click-outside backdrop handlers, and close other active modals when opening one.

### 3. High Contrast Theming & Scroll-Into-View
- **Root Cause Analysis:** Active instance rows use dark amber/red backgrounds with muted grey text, creating poor contrast on dark backgrounds.
- **Remediation:** Switch active selection styling to high-contrast styling (light background with bold dark text in dark mode or vibrant high-contrast badges), and call `.scrollIntoView({ behavior: 'smooth', block: 'nearest' })` on selected items.

### 4. Conversation Cleaner / Prune Safety Gate
- **Root Cause Analysis:** Routine pruning could wipe active workspace sessions if not cross-referenced against currently running tasks.
- **Remediation:** Query `repo_db::get_active_prompts()` for running/queued prompts. For each active project, protect the latest 5 conversation session files (`.vscdb` / `state.vscdb`).

### 5. Email & Telegram Deduplication & 200-Word Previews
- **Root Cause Analysis:** Active prompts query was returning multiple records for the same project name, which was blindly mapped into strings without a `HashSet`/`IndexSet`.
- **Remediation:** Deduplicate project names using `IndexSet`. Format Telegram HTML safely with XML/HTML entity escaping. Render extended 200-word prompt previews cached in split SQLite.

## Deliverable Mapping to Subtasks

- **Subtask 01:** `01-instance-switching-and-modal-dismissal.md` (Task-01, Task-02, Task-03)
- **Subtask 02:** `02-conversation-pruner-safety-gate.md` (Task-04, Task-05)
- **Subtask 03:** `03-email-telegram-dedup-html-update.md` (Task-06, Task-07, Task-08)
- **Subtask 04:** `04-prompt-caching-cli-ssh-parity.md` (Task-09, Task-10)
