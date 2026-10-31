# Subtask 04 — Rerun auto-pick (randomized by default)

Repo: Antigravity-Manager. Spec: 01-agm-prompt-restore-spec.md (rerun section)
Status: PENDING. Owner: Worker wave 1.

## Goal
Clicking rerun automatically selects and executes the randomized nudge option — "a bunch more" — without the user re-picking it every time.

## Owned files
- src/components/instances/PromptTreeView/DetailHeader.tsx
- src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
- src/components/instances/PromptTreeView/ConversationContextMenu.tsx

## Steps
1. DetailHeader.tsx: `rerunMessageOption` default changes from `is-it-done` to `randomized`; `rerunCount` default raised (from 2 to 4). Persist both (localStorage or gui config — follow the existing pattern used by neighboring PromptTreeView preferences) and read them on mount instead of resetting.
2. ConversationContextMenu.tsx: the context-menu rerun must use the same persisted option + count instead of the hardcoded `is-it-done` nudge.
3. usePromptTreePromptActions.ts: enlarge `RERUN_RANDOM_MESSAGE_SET` from 5 to at least 12 varied completion-check nudges (keep tone consistent with the existing set; all must read naturally as "are you done / keep going" checks). Keep `resolveRerunMessage` random-per-dispatch behavior.
4. Keep the manual override: the picker UI stays, so a user can still choose `is-it-done` / `is-it-published` explicitly; the change is only the default + persistence + menu parity.

## Verification (deferred to subtask 08)
- Code re-read; E2E scenario 3 in spec 03.
- Errors, if any are introduced in handlers, must route through the error store (house rule) — no bare catches.
