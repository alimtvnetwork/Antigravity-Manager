# Issue 67: Prompt Tree Enqueue stays at 0 after a successful click

Status: FIXED in v4.279.0 (`3f678d03a`)
Raised: 2026-10-29
Related: `.ai-memory/memory/learned/32-prompt-tree-enqueue-count-and-v4-279-0.md`
Related plan: `.ai-memory/plans/completed/237-prompt-tree-enqueue-visibility.md`

## 1. Symptom

Prompt Tree showed **Enqueue 0** and **IDLE 1h** while inspecting completed turn 11 of 13. Cursor already had **Queued Messages 2** for the same prompt. The FIFO write succeeded; the tree header did not.

Installed GUI in the screenshot was `v4.276.0`. Source HEAD was already past that.

## 2. Trigger

Click Enqueue or Rerun ×N on a conversation, then stay on an older completed turn, or wait for the 30s tree refresh / force cache reset.

## 3. Root Cause

1. Force `loadTree(false, true)` and `reset_prompt_tree_cache` after the click dropped optimistic queued turns.
2. Selected-turn overlay copied the completed turn's `is_queued: false` onto the conversation.
3. `tree_collect.rs` required last_modified within 600s to mark queued, so idle conversations never showed FIFO rows.

## 4. Fix (shipped v4.279.0)

Optimistic inject; skip force reload; `applySelectedTurnOverlay` keeps QUEUED; Enqueue badge is live count; queued status has no recency gate. Frontend `queuedTurns` passed. Isolated `test-create` enqueued five `queued-*` ids immediately. Default PID 3804 and `v2-3127` were not stopped.

## 5. E2E rule

Prove enqueue with SQLite / CLI `agm enqueue -i <sandbox>` ids, then `test-clean`. Do not score the live GUI until `agm --version` matches HEAD. Do not stop Default or `v2-3127`.
