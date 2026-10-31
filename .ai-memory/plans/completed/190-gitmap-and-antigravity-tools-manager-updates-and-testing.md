# Parent Plan — 190 GitMap and Antigravity Tools Manager Updates and Testing

Status: ACTIVE
Phase: 2 (execution) — research complete, spec written
Specs: 02-spec/21-app/190-gitmap-and-antigravity-tools-manager-updates-and-testing/ (01 restore, 02 CG, 03 E2E+release)
Budget: N=400 steps, A=3 agents, H=2

## User Request (Verbatim)

Can you please run GitMap PA first to get all the updates? And then I want you to understand in the GitMap and Antigravity, both of those, let's say last 50 to 80 commits and last few changes. Plans and how many pending plans are there in both of these repositories, try to make sure that those are completed. That's the first thing. Also, try to understand the new changes in the coding guideline, the prompts, and skill. So based on that, make sure the recent changes and recent updates, these are pending in the Antigravity Tools Manager, `Antigravity Manager or Antigravity Tools`. Make sure that you finalize this. And whatever the pending improvements are, make sure you complete it with as detail as possible with end-to-end test cases. Once you complete all these things with the GitMap and the `Antigravity Manager or Antigravity Tools` package, every pending thing is done, then I want you to do some end-to-end testing for the Antigravity Tools Manager and GitMap. Check all the cases that it can run. Most of the commands try to do the end-to-end testing for GitMap and `Antigravity Manager or Antigravity Tools` both. In `Antigravity Manager or Antigravity Tools`, the most important part I want you to test is the account switching, along with the prompt backup and restore that the auto-switching works, but the prompt restore does not work properly. So I want you to focus on that part very seriously. It has been asked several times, but still, it's not working. And also sometimes what happens is that if the account has the credit finished, then some of the quota got finished. All the prompts, those are in pending mode, that gets closed. So in those cases, our job is to understand which prompts actually did not get completed after the auto-switch for the quota finish, so that we can restore those prompt and rerun those prompt. Also in the rerun, when we click on it, rerun should have the random option automatically picked. Is it finished? Is it released? So there should be a bunch of more that would be automatically selected and executed, which is not happening right now. Can you please follow through and do all these things? And also check the code quality in GitMap. Make sure that we have the top-notch code quality following the coding guideline and also the CG execute prompts. I think we were executing the CG execute prompts folder halfway there. You can also execute from zero to end for the CG execute, and also at the end, you can do the end-to-end testing. During the CG execute implementation or fixing, do not do any testing or building. At the end, when every improvement and everything is done, then and only then you are going to build, fix. You are going to run end-to-end test and finalize it. Same goes for the `Antigravity Manager or Antigravity Tools` package. Do not run anything when you are coding. Make sure you fix all the improvements and everything. At the end, when things are all done, then and only then you do the build testing. You do all these end-to-end test cases. Does this make sense? Can you please follow through? Please confirm as soon as you understand with your reporting what you are progressing.

Follow-up (same turn): "At end release both and make sure cicd is green"

## State found (research, 2026-10-23)

- `gitmap pa`: 9 repos pulled, 0 conflicts. AGM @ b4a9b9d3 (v4.250.22), gitmap-v28 @ 8b1dd0743, both clean.
- AGM pending plans: 19 → 7 DONE-IN-CODE, 2 SUPERSEDED (09, 75), 10 STILL-PENDING (04, 105, 139, 86, 87, 88, 91, 92-cli-instance-switch-e2e, 92-e2e-tests-plan, 93).
- gitmap pending plans: 26 → 24 substantively complete (folder never drained), genuinely open: 01 (re-triage), 56 (go/no-go).
- AGM prompt restore: 8 root causes (spec 01). Primary: 120s dispatched window (RC-1) + no incomplete-prompt sweep; manual-switch prompt_backups orphaning (RC-2); fake-success reporting (RC-3).
- Rerun: `randomized` option exists but is never default/persisted; context menu hardcodes `is-it-done`; random set = 5 messages; count default 2.
- CG execute: 12/39 prompts campaigned; residue 109 `== true/false`, 1,506 bare `fmt.Errorf`; prompt 22 (function size) not started (root.go dispatch = 116 lines).

## Subtasks

| ID | Subtask | Repo | Status |
|---|---|---|---|
| 01 | Incomplete-prompt sweep + backup durability (RC-1, RC-5) | AGM | DONE 2026-10-23 (worker; sweep fn + 120s removal + cap exemption) |
| 02 | Re-attribution parity + honest restore outcomes (RC-2, RC-3, RC-6) | AGM | DONE 2026-10-23 (worker; RestoreOutcome; lead wired Stage-5 outcome logging in workspace.rs) |
| 03 | Queue advancement + persisted resume + single backup DB (RC-4, RC-7, RC-8) | AGM | DONE 2026-10-23 (worker; lead wired rearm_persisted_resumes at startup in setup_app.rs + instance.rs re-export) |
| 04 | Rerun auto-pick (randomized default, persisted, larger set) | AGM | DONE 2026-10-23 (worker: set 5→14, count 2→4, localStorage persistence; lead extended parity to PromptTreeContextMenu.tsx + PromptTreeViewModal.tsx) |
| 05 | AGM pending-plan closeout (91 workflows, 88 push-settings, 139 UI, drain done plans, 105/86/04 triage) | AGM | DONE 2026-10-23 (worker; 9 plans filed to completed, 91 mostly done, 88 steps 001-006, 139 UI+CLI, 86 core - remainder + open PID question logged in-plan; most files swept into sync commit e431b369, remainder committed by lead) |
| 06 | gitmap pending drain + CG residue sweeps | gitmap-v28 | DONE 2026-10-23 (worker; committed e08910720 - 24 plans drained, == false 106→1, cli/store fmt.Errorf 202→0, cli/cmd production 121→0) |
| 07 | gitmap CG execute zero-to-end (prompts 24,12,13,14,18,22,29,25, efficiency, surfaces, meta) | gitmap-v28 | DONE 2026-10-23 (waves W3a e65be2c0a, W3b 0ff1da47a, W3c 4b047dd82, W4a 4eaa3123e, W4b 9d2ee5960, W5 e5205bd2e; all 39 prompts at a terminal state - applied / partial with named remainders / documented closure; open items for 08 - 10 !IsSuccess linter residuals, 2 TS any sites, cmd_sends.go 628-line split, Python renames to port to coding-guidelines-v24 canonical) |
| 08 | Build + E2E both repos + release both + CI green | both | DONE 2026-10-23 (gitmap: go build + go vet fully green after a 132-file repair - the tree did not compile at task start: duplicate constants and store types, stale Result-envelope callers, corrupted imports; binary E2E pass - help/version/typo-exit-1/scan/ls; AGM: vite build green, tsc shows exactly the 18 pre-existing errors also present at task start, zero new; live switch/restore E2E still requires a Windows/IDE machine. Released gitmap-v28 -> v6.590.0 and Antigravity-Manager -> v4.251.0, both pipelines 100% green) |

Subtask files: .ai-memory/plans/subtasks/190-gitmap-and-antigravity-tools-manager-updates-and-testing/

## Constraints (non-negotiable)

- GitMap-only search; relative paths only (incl. release pages/notes); lowercase filenames.
- No builds, no tests during implementation (subtasks 01-07). Build/test/E2E only in subtask 08.
- Workers never run git; lead commits via GitMap hyphen-format messages.
- FIFO invariant in all dispatch/restore queries. No silent errors.
- Release at the end: both repos, CI green (`gitmap pe -t`) before and after.

## Closure (2026-10-23)
All 8 subtasks DONE. Releases: gitmap-v28 -> v6.590.0, Antigravity-Manager -> v4.251.0; `gitmap pe -t` 100% green on both mains. Carried remainders (documented in the subtask files): live AGM switch/restore E2E on a Windows/IDE machine (plans 92/93 runbooks ready; plan 88 step 007 and plan 139 step 3 run there); plan 139 saved-PID decision (options A/B/C presented to the user, awaiting the pick); repo-wide prompt 22 decomposition backlog (7,044 functions over 15 lines) and prompt 18 DTO conversion as future programs; 10 boolean-linter `!IsSuccess` guard residuals; 2 TypeScript `any` sites in TerminalDrawer.tsx for a tsc-verified pass; Python script renames from prompt 26 to port into the coding-guidelines-v24 canonical copies before the next fleet sync; prompt 36 and 34/39 closures to revisit with a compiler available.
