# Plan: Fast Forward Algorithm Issues and Confirmation

## Task Metadata
- **ID**: `222-fast-forward-algorithm-issues-and-confirmation`
- **Slug**: `fast-forward-algorithm-issues-and-confirmation`
- **Status**: `In Progress`
- **Priority**: `High`
- **Spec**: `02-spec/21-app/222-fast-forward-algorithm-issues-and-confirmation/spec.md`

## User Request (Verbatim)
> # Fast Forward Algorithm Issues and Confirmation: high priority instruction, non-negotiable task
> 
> A few of the issues in your fast forward algorithm. And you need to confirm that it is working as I am saying. If the weekly quota is less than 15%, and that should be controllable from the settings where we control for the auto switch. Usually by default it would be less than 15% weekly quota. It would be considered as zero. If any account has zero credits, we don't move there. We just skip those accounts. Now, if any accounts has any users in the four hours credit, let's say less than 1%, we never switch to that account. Which accounts we never switch in fast forward? One, if that has, in four hours credit, less than 100%, if any other instance is using it, if any other machine instance is using it, that we know using email or SSH, Gitmap, or Telegram, any one of the sources. If we know that, if anyone on the account in last four hours is using that, then we skip that. Then we move to the next one, next possible best one, which has the closest time to expire and has full 100% four-hour credit. That is our best choice. In case, if we don't have any one of them, then we try to find whichever has the highest weekly and four-hour credits. It would be the same algorithm will judge it, and pro multiplier or ultra multiplier would be already there. This is the conditional logic that you should check based on switching. If you understood this, then your job is to update the logic, update the algorithm, and also have a setting section in the UI, so that we can change the slider for this. Finally, do a minor bump and make a release, and make sure that CICD is green using Gitmap EE space hyphen T.

## Actionable Subtasks
1. `Subtask-01`: Research codebase and design fast-forward account target resolver.
2. `Subtask-02`: Refactor `fast_forward_target_resolver.rs` with weekly floor, 100% 4h headroom, cross-instance exclusion, closest expiry ranking, and degraded fallback scoring.
3. `Subtask-03`: Author comprehensive unit tests in `tests_fast_forward_target_resolver.rs`.
4. `Subtask-04`: Update TypeScript config types and add UI slider in `src/components/settings/AutoSwitcher/`.
5. `Subtask-05`: Run targeted validation, execute minor version bump (`npm run bump minor`), and verify release/CI gates.
