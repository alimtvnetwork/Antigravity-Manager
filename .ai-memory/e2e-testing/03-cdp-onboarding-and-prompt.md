# 03 — CDP onboarding, live prompt, backup (v4.202.0)

Release `v4.202.0`, commit `e8b0fb49`. The AI's required live checks for that release were: create or use an instance, switch, fast-forward, auto-switch, confirm the Security Notice share box is unchecked and accepted, send and enqueue a prompt so the IDE conversation actually receives it, then backup and restore that prompt.

Why: a fresh Antigravity window blocks on "Security Notice & Data Use". CLI `agm prompt` also lost `running_projects` rows because the next process purged them. Typing into the composer is a separate step from writing the SQLite row.

## Prepare

- [ ] Build `agm` and the GUI from the commit under test, `-j 1`, one at a time.
- [ ] `agm instances ls`. Reuse `default-copy-5597` if it exists. Create a copy only when no idle copy exists.
- [ ] Do not close Cursor or `agm-alim.exe`.

## Security Notice

- [ ] Switch the account on the copy, or launch the copy, so the IDE window opens.
- [ ] Pass: the dialog "Security Notice & Data Use" is dismissed without a human click.
- [ ] Pass: the share checkbox "Yes, I agree to help improve Antigravity..." is unchecked before Accept.
- [ ] Pass: the button clicked is "Accept and continue".
- [ ] Pass: instance storage has `hasAcceptedTermsOfService = true` and `shareDataToImproveAntigravity = false` in `app_storage.json` and `state.vscdb`.
- [ ] Fail: the dialog is still up, or the share checkbox was left checked.
- [ ] Decision: this is CDP against `DevToolsActivePort` from `--remote-debugging-port=0` in `src-tauri/src/modules/instance/cdp_onboarding.rs`. Do not call it passed from reading that file.

## Prompt reaches the IDE

- [ ] `agm prompt` (or `agm prompts send`) to the copy, with a repo path already bound to that instance.
- [ ] Pass: `running_projects` and `active_prompts` still exist after a second CLI invocation. The row id is `<project>__<instance>` and `workspace_storage_path` is set. `purge_corrupted_running_projects` must not delete it.
- [ ] Pass: the open IDE conversation composer received the text and it was submitted. That is `submit_prompt_via_cdp_for_instance`, not only a SQLite insert.
- [ ] Enqueue a second short prompt. Pass: status stays `queued` until the running worker finishes, then FIFO (`created_at`, then `id`) drains it.
- [ ] Fail: success toast while the composer is empty, or the queued row disappears on the next `agm` start.

## Backup and restore

- [ ] From the Prompt Tree, Backup writes a `.json` through the native save dialog.
- [ ] Restore opens the native open dialog, reads that file, and dispatches the backed-up prompts.
- [ ] CLI `agm prompts backup` and `agm prompts restore` are extra. They do not replace the two button checks.
- [ ] Cancel on either dialog must not show success.

## Settings route

- [ ] `agm ui goto settings/<tab>` opens that tab. Pass: no React Router 404.
- [ ] Tabs exercised with the v4.202 change: the settings sub-route in `src/App.tsx` and `src/pages/Settings/useSettingsState.ts`.

## Stop

- [ ] Screenshot the Instances view and the IDE after the prompt is visible, under `assets/screenshots/`, only if you are keeping evidence.
- [ ] Threshold is 15.0.
