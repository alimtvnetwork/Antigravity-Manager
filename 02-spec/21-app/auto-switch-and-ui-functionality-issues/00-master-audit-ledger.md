# Master Audit Ledger: Auto-Switch and UI Functionality Issues

- **Slug**: `auto-switch-and-ui-functionality-issues`
- **Specification Directory**: `02-spec/21-app/auto-switch-and-ui-functionality-issues/`
- **Master Plan**: `.ai-memory/plans/auto-switch-and-ui-functionality-issues.md`
- **Benchmark Branch**: `backup/auto-switch-and-ui-functionality-issues-benchmark`
- **Target Release Version**: `v4.208.0` (Minor bump)
- **Status**: `[IN PROGRESS]`

---

## User Request (Verbatim)

```text
# Auto-Switch and UI Functionality Issues: high priority instruction, non-negotiable task

A couple of serious problems. In this case, the account auto-switch did not work, and I clicked on fast forward manually, also did not work. This is what we're talking about, the version 4.206. Auto-switch is not working properly. You need to fix it. You need to find the root cause why it is not working. When we click on auto-switch, it pops up many PowerShell or terminal windows, which should be hidden and then do the work, which is not happening. The log section in the top right is not very friendly. If I click on JSON, I don't get a copy of the JSON. The buttons for the copy do not work, where it should have been copied properly, JSON NL and JSON. The click and the UI of the audit need to have more information, like where it is clicking, how it is clicking in the log section. Even though the JSON contains it, the JSON should contain more information, a stack trace of which function and how it worked, which line of the code. That was much more important in the logs, which is not there. Auto-switch did not happen where it should actually run for the auto-switch algorithm. There is no log of it, why it didn't work. I asked to retest for you and other AI to verify this by using end-to-end testing by running the cargo, and build, and testing, and opening new instances, putting it to 98, putting some prompt in to check whether it is working or not. That is something that we requested several times. It's looking very bad now. You need to put your full focus on why the auto-switch does not work, why the fast forward manual click also does not work. When we keep the information for the prompts, make sure that for each one of the prompts, we should not have more than 100 prompts as a backup, but it can be changed from the settings. There should be a setting that would allow us to change that. So far the changes you have done, I really do think that you have not done the end-to-end testing. You did not run the code. You did not open using the CLI. Each case didn't run the auto-switch mechanism. It was very bad work so far. You need to be more careful in this. I requested in the past that in the drop-down button, we do have the update. I do see that. The update icon, when I hover over, looks a little bit bad and terrible. Can we improve that? The updater, which shows up in the terminal, looks very terrible. It has several broken UI in the terminal. You can see I have taken the screenshot. You can improve the UI visualization. Why the fonts are not bigger in the EXE what is trying to run. Make sure that this has bigger fonts, at least 18. The UI and everything needs to be bigger. A couple of serious things, and also I don't think that you have the synchronization. The reason I'm saying this is because in the riseteamasiaTeam@gmail.com, do not put the email into the logs or anywhere, just for your understanding. Remember to put masking for the email. It does not synchronize. Even the synchronize runs in 30 seconds, right? That should be 30, 50 seconds, but it did not. If you see the screenshot on the instance section, it actually has the wrong quota information. That makes the whole thing broken. It does not work. Clicking on fast forward does not work. Everything looks really, really bad. You should focus on this. This is a very serious issue you need to fix immediately. Is it clear?
```

---

## Visual Evidence & UI Artifacts

- **Image 1**: `assets/screenshots/auto-switch-and-ui-functionality-issues-01.png`
  - Top: Antigravity IDE showing **5-Hour Limit Remaining: 0%** (Resets in 1h 18m).
  - Middle & Bottom: "Interaction Audit Log (60/60)" table displaying empty `METHOD` and `TARGET` cells, lacking function names, line numbers, and stack traces.
- **Image 2**: `assets/screenshots/auto-switch-and-ui-functionality-issues-02.png`
  - Top: Account identity `riseup.asia.team@gmail.com` requiring strict masking.
  - Bottom: Antigravity Tools v4.207.0 showing **Gemini 2.5 Pro (4H): 100%** (falsely desynchronized from IDE 0% reality).

---

## Traceable Discrete Tasks

| Task ID | Module / Area | Description | Status |
|---|---|---|---|
| **Task-01** | Git Branching | Preserve engineering baseline in `backup/auto-switch-and-ui-functionality-issues-benchmark` | `PENDING` |
| **Task-02** | Auto-Switch & Quota Sync | Fix 403 quota summary drop, add active 30s-50s background quota refresh, emit `accounts://refreshed` | `PENDING` |
| **Task-03** | Fast-Forward Pipeline | Unblock fast-forward execution, remove blocking timeouts, ensure candidate pool unblocking | `PENDING` |
| **Task-04** | Process Window Suppression | Apply `CREATE_NO_WINDOW` (`0x08000000`) across all 14 Windows subprocess spawn sites | `PENDING` |
| **Task-05** | Audit Log Copy Buttons | Implement true clipboard copying with visual confirmation and toast for `JSON` and `JSONL` | `PENDING` |
| **Task-06** | Audit Log Metadata & Traces | Fix `rename_all = "camelCase"` on `InteractionEntry`, capture caller/target DOM, add stack traces | `PENDING` |
| **Task-07** | Auto-Switch Audit Logging | Record candidate evaluations, scoring, rejections, and rotation decisions into audit DB | `PENDING` |
| **Task-08** | Max Backup Prompts Cap | Enforce FIFO 100-prompt backup cap in `backup_prompts_db` and add Settings retention slider | `PENDING` |
| **Task-09** | Strict Email Masking | Remove hover email leaks in `InstanceCard.tsx`, standardize `maskEmail`, mask backend logs | `PENDING` |
| **Task-10** | Updater UI & Terminal Font | Remove `animate-bounce` on NavMenu update icon, set 18pt font and ANSI processing in Windows console | `PENDING` |
| **Task-11** | E2E Testing Verification | Execute end-to-end simulation of quota sync, auto-switch trigger, audit copy, and backup cap | `PENDING` |
| **Task-12** | Release Ceremony | Pre-flight checks (`cargo fmt`, `cargo check`, `npm run build`), bump minor `v4.208.0`, sync docs | `PENDING` |
