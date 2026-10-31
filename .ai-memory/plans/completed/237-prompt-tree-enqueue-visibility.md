# Master Plan 237: Prompt Tree enqueue visibility

## Status: COMPLETED
Written as completed on 2026-10-30. Shipped `3f678d03a`, tagged `v4.279.0`. Predecessor merge/scheduler work in `68622ceef` (`v4.278.0`) and `80478a3d1`.

Slug: `prompt-tree-enqueue-visibility`.

## User Request (Verbatim)

Prompts enqueue does not show up properly. Find the root cause and fix it with testing and a minor bump. Test autoswitch, prompt restoring after autoswitch on an instance, send prompt, copy prompt, enqueue prompt, rerun should enqueue as many times. Telegram instances and status must stay friendly and prompts can be sent. All enqueue and rerun should enqueue immediately, not after waiting, and must show up after the click on that instance.

## Owned files

- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` — inject queued turns; skip force reload; no session id `current`
- `src/components/instances/PromptTreeView/usePromptTreeData.ts` — `applySelectedTurnOverlay`, `queuedPromptCount`, `mergeQueuedTurns`
- `src/components/instances/PromptTreeView/DetailHeader.tsx` — live Enqueue count; optimistic rerun turns
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx` — no force reload; no `current` fallback
- `src-tauri/src/modules/repo_db/tree_collect.rs` — queued status without 600s recency
- `src-tauri/src/modules/repo_db/tree_node_conv.rs` — `PENDING_STAGES` includes `queued` and `in_progress`

## Acceptance

- [x] Enqueue/rerun show queued turns on the clicked instance immediately (optimistic inject; no force cache reset)
- [x] Header QUEUED while inspecting an older completed turn if later turns are queued
- [x] Enqueue badge is the live count
- [x] Rerun ×N uses `enqueue_prompt` only (`rerunActions` 0 `send_prompt_now`)
- [x] Isolated `agm instances test-create` / `test-clean`; five `queued-*` ids; Default PID 3804 and `v2-3127` not stopped
- [x] Minor bump `4.279.0` with `@aukgit` only and relative Git paths
- [x] Telegram idle/busy/N queued + Send already in `v4.278.0` (`instance_cards.rs`)
- [ ] Live GUI Enqueue click on installed `v4.279.0` (node still on `v4.276.0` binary at write time)
- [ ] CLI `agm prompt copy` on current CLI (hung on `v4.276.0`; Copy source fix is `86877cf53`)
- [ ] Autoswitch enable/run on a sandbox (status only this round; do not rotate production instances)

## Leftovers

Install `v4.279.0` before scoring the Prompt Tree window. JSON switch honesty is issue 66 (`v4.278.0`), not this plan.
