# Plan 221: GitMap and Antigravity Tools Manager Updates and Testing
Status: IN_PROGRESS
Sequence: 221
Batch: Steps 316–320
Improvements: `Imp-730` through `Imp-734`

## User Request (Verbatim)
```text
# GitMap and Antigravity Tools Manager Updates and Testing: high priority instruction, non-negotiable task

Can you please run GitMap PA first to get all the updates? And then I want you to understand in the GitMap and Antigravity, both of those, let's say last 50 to 80 commits and last few changes. Plans and how many pending plans are there in both of these repositories, try to make sure that those are completed. That's the first thing. Also, try to understand the new changes in the coding guideline, the prompts, and skill. So based on that, make sure the recent changes and recent updates, these are pending in the Antigravity Tools Manager, `Antigravity Manager or Antigravity Tools`. Make sure that you finalize this. And whatever the pending improvements are, make sure you complete it with as detail as possible with end-to-end test cases. Once you complete all these things with the GitMap and the `Antigravity Manager or Antigravity Tools` package, every pending thing is done, then I want you to do some end-to-end testing for the Antigravity Tools Manager and GitMap. Check all the cases that it can run. Most of the commands try to do the end-to-end testing for GitMap and `Antigravity Manager or Antigravity Tools` both. In `Antigravity Manager or Antigravity Tools`, the most important part I want you to test is the account switching, along with the prompt backup and restore that the auto-switching works, but the prompt restore does not work properly. So I want you to focus on that part very seriously. It has been asked several times, but still, it's not working. And also sometimes what happens is that if the account has the credit finished, then some of the quota got finished. All the prompts, those are in pending mode, that gets closed. So in those cases, our job is to understand which prompts actually did not get completed after the auto-switch for the quota finish, so that we can restore those prompt and rerun those prompt. Also in the rerun, when we click on it, rerun should have the random option automatically picked. Is it finished? Is it released? So there should be a bunch of more that would be automatically selected and executed, which is not happening right now. Can you please follow through and do all these things? And also check the code quality in GitMap. Make sure that we have the top-notch code quality following the coding guideline and also the CG execute prompts. I think we were executing the CG execute prompts folder halfway there. You can also execute from zero to end for the CG execute, and also at the end, you can do the end-to-end testing. During the CG execute implementation or fixing, do not do any testing or building. At the end, when every improvement and everything is done, then and only then you are going to build, fix. You are going to run end-to-end test and finalize it. Same goes for the `Antigravity Manager or Antigravity Tools` package. Do not run anything when you are coding. Make sure you fix all the improvements and everything. At the end, when things are all done, then and only then you do the build testing. You do all these end-to-end test cases. Does this make sense? Can you please follow through? Please confirm as soon as you understand with your reporting what you are progressing.
```

## Subtasks
1. `01-gitmap-pa-audit-and-fleet-sync.md`: GitMap PA Fleet State & Coding Guidelines Ingestion (`Imp-730`) — `IN_PROGRESS`
2. `02-prompt-restore-reconnect-sweep-engine.md`: Prompt Restore Reconnect Sweep & Quota Interruption Recovery (`Imp-731`) — `COMPLETED`
3. `03-rerun-randomizer-history-and-anti-repeat.md`: Rerun Randomizer UX & Multi-Turn Anti-Repeat Engine (`Imp-732`) — `PENDING`
4. `04-gitmap-code-quality-cmdpull-modularization.md`: GitMap cmdpull Modularization & 8-Pillars Clean Architecture (`Imp-733`) — `PENDING`
5. `05-master-deferred-e2e-plan-and-verification.md`: Master Deferred E2E Plan & Final Live Verification Execution (`Imp-734`) — `PENDING`

## Implementation Invariants
- **FIFO Query Rule**: Every query on prompt tables uses `ORDER BY created_at ASC, id ASC`.
- **Zero Builds / Zero Tests Invariant**: No testing, compiling, or linting during authoring; deferred strictly to Phase 3.
- **Worker Git Ban**: Subagents perform edits only; lead agent performs git operations.
- **Strictly Relative Paths**: All path references must remain relative.
