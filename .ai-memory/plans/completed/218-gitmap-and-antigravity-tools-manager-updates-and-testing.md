# Plan 218: GitMap and Antigravity Tools Manager Updates, Prompt Restore & E2E Testing
Status: COMPLETED

## User Request (Verbatim)
```
# GitMap and Antigravity Tools Manager Updates and Testing: high priority instruction, non-negotiable task

Can you please run GitMap PA first to get all the updates? And then I want you to understand in the GitMap and Antigravity, both of those, let's say last 50 to 80 commits and last few changes. Plans and how many pending plans are there in both of these repositories, try to make sure that those are completed. That's the first thing. Also, try to understand the new changes in the coding guideline, the prompts, and skill. So based on that, make sure the recent changes and recent updates, these are pending in the Antigravity Tools Manager, `Antigravity Manager or Antigravity Tools`. Make sure that you finalize this. And whatever the pending improvements are, make sure you complete it with as detail as possible with end-to-end test cases. Once you complete all these things with the GitMap and the `Antigravity Manager or Antigravity Tools` package, every pending thing is done, then I want you to do some end-to-end testing for the Antigravity Tools Manager and GitMap. Check all the cases that it can run. Most of the commands try to do the end-to-end testing for GitMap and `Antigravity Manager or Antigravity Tools` both. In `Antigravity Manager or Antigravity Tools`, the most important part I want you to test is the account switching, along with the prompt backup and restore that the auto-switching works, but the prompt restore does not work properly. So I want you to focus on that part very seriously. It has been asked several times, but still, it's not working. And also sometimes what happens is that if the account has the credit finished, then some of the quota got finished. All the prompts, those are in pending mode, that gets closed. So in those cases, our job is to understand which prompts actually did not get completed after the auto-switch for the quota finish, so that we can restore those prompt and rerun those prompt. Also in the rerun, when we click on it, rerun should have the random option automatically picked. Is it finished? Is it released? So there should be a bunch of more that would be automatically selected and executed, which is not happening right now. Can you please follow through and do all these things? And also check the code quality in GitMap. Make sure that we have the top-notch code quality following the coding guideline and also the CG execute prompts. I think we were executing the CG execute prompts folder halfway there. You can also execute from zero to end for the CG execute, and also at the end, you can do the end-to-end testing. During the CG execute implementation or fixing, do not do any testing or building. At the end, when every improvement and everything is done, then and only then you are going to build, fix. You are going to run end-to-end test and finalize it. Same goes for the `Antigravity Manager or Antigravity Tools` package. Do not run anything when you are coding. Make sure you fix all the improvements and everything. At the end, when things are all done, then and only then you do the build testing. You do all these end-to-end test cases. Does this make sense? Can you please follow through? Please confirm as soon as you understand with your reporting what you are progressing.
```

## Non-Negotiable Architectural Invariants
1. Git pull BEFORE starting any work via `gitmap pa`.
2. Strictly relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`). Zero absolute paths.
3. Search codebase exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`). Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
4. Two-Phase Build/Test Invariant: Strictly ZERO builds (`cargo build`, `npm run build`, `vite build`) or test runs during active code modification; execute builds and test cases only after all improvements are finalized.
5. Strict FIFO database ordering (`ORDER BY created_at ASC, id ASC`).
6. Enforce No Silent Errors: all fallible paths propagate via `AppError` or record using `crate::error::record_ignored` with a `// Justification:` comment.

## Subtask Breakdown

- [x] **Subtask 01**: Fleet Sync & Historical Commit Audit (Steps 50–80)
  - Run `gitmap pa` across all workspace repositories (completed via task-1587).
  - Audit last 50–80 commits in GitMap and Antigravity-Manager.
  - Review pending and completed plans in both repositories.
  - Ingest recent updates in coding guidelines, canonical prompts, and Antigravity skills.

- [x] **Subtask 02**: Antigravity Manager Account Switch & Prompt Restore Verification
  - Ensure account switching correctly preserves in-flight and pending prompts upon quota exhaustion.
  - Ensure `is_live_status_preserved` only preserves `Some("completed")`.
  - Ensure interrupted prompts are swept into `backed_up` and re-dispatched.
  - Verify CDP connection retry ladder for restarted IDE processes.

- [x] **Subtask 03**: Rerun Randomizer & Up-Next Queue Visibility
  - Ensure clicking Rerun auto-picks from diverse random follow-up nudges ("Is it finished?", "Is it released?", "Is it completed?", etc.).
  - Strip recursive template delimiters so rerun never nests headers.
  - Ensure backed-up prompts appear in the Up-Next queue.

- [x] **Subtask 04**: GitMap Code Quality Audit & CG Execute Pass
  - Audit GitMap codebase against coding guidelines (`02-spec/02-coding-guidelines/`).
  - Verify AppError handling, bounded functions (8–15 lines), positive booleans, and PascalCase enums.
  - Execute CG execute guidelines across touched modules with zero builds and zero tests during modification.

- [x] **Subtask 05**: End-to-End Testing & Verification Ceremony
  - Execute E2E command suite for GitMap (`gitmap ls`, `gitmap aum search`, `gitmap rc heatmap status`).
  - Execute E2E command suite for Antigravity-Manager (`agm status`, `agm accounts`, `agm instances list`, `agm auto-switch status`, `agm backup ls`, `agm restore running --keep`, `agm running`).
  - Verify all test cases pass with 100% success.
