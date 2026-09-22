# Plan: Fast-Forward Button Switch Delegation to Proven Switch Pipeline

## User Request & Visual Reference
![Screenshot](assets/screenshots/fast-forward-switch-delegation-01.png)

> The first button, number one, how I have highlighted, this button is not working as expected, so you need to fix it. However, the button two, which I have selected, that is working fine. So the idea here in the first button, which is the transfer button, it just finds the logically which workspace should work, and then it would basically going to delegate the command to that switch button, which I have highlighted in section two, so that it works properly. Okay? Do you understand the task? First, write the issues, why it happened, how it happened, and then you fix it properly, and then make a minor bump and release. Is it clear? Do you have any question and confusion?

## Goals & Boundaries
1. **Document Issues:** Write 4-part RCA in `.ai-memory/memory/issues/31-fast-forward-switch-delegation.md` and `.ai-memory/cicd-issues/31-fast-forward-switch-delegation-rca.md`.
2. **Refactor Button 1:** Refactor `smartRotateProfileAccount` in `src/stores/useInstanceStore.ts` to logically identify the candidate account, and then delegate directly to `useAccountStore.getState().switchAccount(verified.account.id, targetIdeParam)`.
3. **Harmonize Play Action:** Update `smartPlayInstance` in `src/stores/useInstanceStore.ts` to delegate to `switchAccount` similarly.
4. **UI Event Propagation:** Ensure button handlers in `src/components/navbar/InstanceSelector.tsx` call `e.stopPropagation()`.
5. **Quality Gates & Verification:** Pass TypeScript typechecks, lint checks, and local quality gates.
6. **Minor Release Ceremony:** Consolidate plan and trigger minor release bump to `v4.57.0`.

## Subtasks
- [x] `01-write-rca-issues`: Create RCA issue documentation (.ai-memory/memory/issues/31-fast-forward-switch-delegation.md and .ai-memory/cicd-issues/31-fast-forward-switch-delegation-rca.md).
- [x] `02-delegate-smart-rotate`: Refactor smartRotateProfileAccount and smartPlayInstance in useInstanceStore.ts to delegate to switchAccount.
- [x] `03-verify-and-quality-gates`: Execute TypeScript verification, markdown gap fixer, and local quality runner.
- [x] `04-release-orchestration`: Execute minor release bump to v4.57.0, commit, tag, and push.

## Status
- [x] completed
