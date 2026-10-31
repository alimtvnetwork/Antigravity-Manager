# Plan 220: GitMap and Antigravity Tools Manager Updates and Testing
Status: COMPLETED
Sequence: 220
Batch: Steps 311–315
Improvements: `Imp-725` through `Imp-729`

## User Request (Verbatim)
```text
# GitMap and Antigravity Tools Manager Updates and Testing: high priority instruction, non-negotiable task

Can you please run GitMap PA first to get all the updates? And then I want you to understand in the GitMap and Antigravity, both of those, let's say last 50 to 80 commits and last few changes. Plans and how many pending plans are there in both of these repositories, try to make sure that those are completed. That's the first thing. Also, try to understand the new changes in the coding guideline, the prompts, and skill. So based on that, make sure the recent changes and recent updates, these are pending in the Antigravity Tools Manager, `Antigravity Manager or Antigravity Tools`. Make sure that you finalize this. And whatever the pending improvements are, make sure you complete it with as detail as possible with end-to-end test cases. Once you complete all these things with the GitMap and the `Antigravity Manager or Antigravity Tools` package, every pending thing is done, then I want you to do some end-to-end testing for the Antigravity Tools Manager and GitMap. Check all the cases that it can run. Most of the commands try to do the end-to-end testing for GitMap and `Antigravity Manager or Antigravity Tools` both. In `Antigravity Manager or Antigravity Tools`, the most important part I want you to test is the account switching, along with the prompt backup and restore that the auto-switching works, but the prompt restore does not work properly. So I want you to focus on that part very seriously. It has been asked several times, but still, it's not working. And also sometimes what happens is that if the account has the credit finished, then some of the quota got finished. All the prompts, those are in pending mode, that gets closed. So in those cases, our job is to understand which prompts actually did not get completed after the auto-switch for the quota finish, so that we can restore those prompt and rerun those prompt. Also in the rerun, when we click on it, rerun should have the random option automatically picked. Is it finished? Is it released? So there should be a bunch of more that would be automatically selected and executed, which is not happening right now. Can you please follow through and do all these things? And also check the code quality in GitMap. Make sure that we have the top-notch code quality following the coding guideline and also the CG execute prompts. I think we were executing the CG execute prompts folder halfway there. You can also execute from zero to end for the CG execute, and also at the end, you can do the end-to-end testing. During the CG execute implementation or fixing, do not do any testing or building. At the end, when every improvement and everything is done, then and only then you are going to build, fix. You are going to run end-to-end test and finalize it. Same goes for the `Antigravity Manager or Antigravity Tools` package. Do not run anything when you are coding. Make sure you fix all the improvements and everything. At the end, when things are all done, then and only then you do the build testing. You do all these end-to-end test cases. Does this make sense? Can you please follow through? Please confirm as soon as you understand with your reporting what you are progressing.
```

## Subtasks
1. `01-gitmap-pa-audit-and-recent-commits-analysis.md`: GitMap PA Fleet Status Audit & Coding Guidelines Ingestion (`Imp-725`) — `COMPLETED`
2. `02-account-auto-switch-prompt-restore-engine.md`: Account Auto-Switching & Prompt Restore Resiliency (`Imp-726`) — `COMPLETED`
3. `03-rerun-randomizer-and-persistence-ux.md`: Rerun Randomizer UX & Persistent Execution Engine (`Imp-727`) — `COMPLETED`
4. `04-gitmap-code-quality-cg-execute.md`: GitMap Code Quality & CG Execute Suite (`Imp-728`) — `COMPLETED`
5. `05-master-deferred-e2e-plan-and-verification.md`: Master Deferred E2E Plan & Final Live Testing Execution (`Imp-729`) — `COMPLETED`

## Verification Summary
- Rust unit tests in `tests_prompt_restore_honesty.rs` pass.
- Frontend tests in `rerunActions.test.ts` pass (9/9 test suites passed).
- `npm run build` completed cleanly.
- Go unit tests in `cmdfix`, `heatmap`, `cmdcommit` passed with 0 failures.
- Master E2E integration test suite (`scripts/test-agm-gitmap-running-backup-e2e.ps1`) passed all 25 gates (25/25 passed, 0 failed).
- All files $\le 300$ lines, zero builds/tests during refactoring observed.
