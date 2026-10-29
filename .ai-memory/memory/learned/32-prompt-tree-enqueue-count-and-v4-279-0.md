# Learned Memory 32: Prompt Tree Enqueue Count, Selected-Turn Overlay, and v4.279.0

- **ID**: `learned-32`
- **Scope**: Prompt Tree Enqueue/Rerun immediate visibility; conversation QUEUED while inspecting a completed turn; FIFO recency gate; isolated sandbox E2E; release `v4.279.0`.
- **Date**: 2026-10-30
- **Related Issue**: `.ai-memory/issues/67-prompt-tree-enqueue-zero-after-click.md`
- **Related Plan**: `.ai-memory/plans/completed/237-prompt-tree-enqueue-visibility.md`
- **Related**: `.ai-memory/memory/learned/31-prompt-tree-copy-skill-refs-telegram-e2e-and-json-switch.md`, `.ai-memory/issues/66-instances-switch-json-fake-success.md`
- **Shipped**: `68622ceef` (`v4.278.0` optimistic merge), `80478a3d1` (backed-up dispatch isolation), `3f678d03a` tagged `v4.279.0`

---

## 1. Last-30-commit progress (2026-10-30)

Newest first:

| Commit | What shipped |
|---|---|
| `3f678d03a` | Enqueue count on click; skip force tree reset; selected-turn overlay keeps QUEUED; no 600s recency gate; tagged `v4.279.0` |
| `80478a3d1` | Isolate backed-up prompt dispatch to the target instance (no foreign fallback) |
| `68622ceef` | Optimistic queued turns survive SWR; `mergeQueuedTurns` counts identical rerun clicks; JSON switch proves `bound_email`; Telegram idle/busy/N queued + Send |
| `df8202752` | Write-memory: Copy, Telegram 236, JSON switch lie |
| `86877cf53` | Prompt Tree Copy is verbatim full text; skill refs are not attachments |
| `b2cfc5596` / `1ce618741` | Telegram 236; tagged `v4.276.0` |

Do not score the live Prompt Tree window against an older installed `agm-ui` than HEAD. On 2026-10-29 the screenshots were `v4.276.0` while source had already passed `v4.278.0`.

---

## 2. Symptom the user showed

Prompt Tree header: **Enqueue 0**, **IDLE 1h**, inspecting **turn 11 of 13** (COMPLETED). Cursor already had **Queued Messages 2** for the same prompt text. Backend enqueue worked. AGM display did not.

---

## 3. Root causes

1. After Enqueue/Rerun, `loadTree(false, true)` plus `reset_prompt_tree_cache` wiped optimistic queued turns. Soft `loadTree(false, false)` also no-ops inside `TREE_CACHE_TTL_MS` (30s), so in-place mutation without `setState` never painted.
2. `loadTree` overlay copied the **selected turn** `is_queued` onto the conversation. Viewing completed turn 11 set `is_queued: false` even when later turns were `QUEUED`.
3. `src-tauri/src/modules/repo_db/tree_collect.rs` required status match **and** last_modified within 600s **and** owning instance alive. An idle 1h conversation never showed FIFO queued prompts.
4. Capsule B Enqueue had no live count. Header showed IDLE when `selectedConversation.is_queued` was false.

Rerun ×N already goes through `enqueue_prompt` (zero `send_prompt_now`) in `dispatchRerunMessages`. The bug was visibility, not dispatch.

---

## 4. Fix (`3f678d03a`, `v4.279.0`)

- Enqueue/rerun inject queued turns into `selectedConversation`, `treeData`, and `patchPromptTreeSwrCache`. Skip force/soft reload after the click.
- `applySelectedTurnOverlay` / `queuedPromptCount` in `src/components/instances/PromptTreeView/usePromptTreeData.ts`: conversation stays QUEUED if any turn is queued.
- DetailHeader Enqueue badge is the live count; QUEUED capsule shows when `enqueuedCount > 0`. Rerun uses local optimistic turns, not in-place mutate.
- Never fall back session id to the literal `current`. Use `conversation_id`.
- `tree_collect.rs` marks queued from status words `queued|queue|pending|in_progress` with no recency gate. `PENDING_STAGES` in `src-tauri/src/modules/repo_db/tree_node_conv.rs` includes those words.

---

## 5. Tests and E2E

Frontend: `queuedTurns` and `rerunActions` passed (`node scripts/run-frontend-tests.mjs`). Overlay test: inspecting turn 11 still reports 2 later queued turns.

Isolated sandbox only:

- `agm instances test-create enqueue-ui --json` → `test-enqueue-ui-1793289596-9596`
- Five immediate FIFO ids: `queued-1793289597305-0000-b68fdd`, `queued-1793289597524-0000-569e8d`, `queued-1793289597623-0000-65c281`, `queued-1793289597724-0000-187b40`, `queued-1793289597832-0000-d91ac9`
- `agm instances test-clean` removed only that sandbox
- Default PID 3804 and `v2-3127` were not stopped

Not proven this round:

- CLI `agm prompt copy` hung on installed `v4.276.0` (killed; not a source regression of `86877cf53`)
- Live GUI click of Enqueue on `v4.279.0` (installer not yet on this node)
- Live Telegram card Send click (source already in `v4.278.0`)
- Autoswitch enable/run on a production instance (status only)

---

## 6. Adjacent facts

- JSON switch honesty shipped in `v4.278.0` (`68622ceef`). Issue 66 is FIXED. Never treat `--json` `"status": "switched"` as bound-email proof.
- Telegram `/status` cards: idle / busy / N queued and Send (`src-tauri/src/modules/telegram_inbound/instance_cards.rs`).
- Backed-up dispatch must stay on the target instance (`80478a3d1`).
- FIFO remains `ORDER BY created_at ASC, id ASC`.
- Do not run `cargo test --lib` / full-crate cargo on this host (OOM). Preflight is `cargo fmt -- --check` + `npm run build`.
- Attribution `@aukgit` only. Relative Git paths only.

---

## 7. Do not break

- After enqueue/rerun click: inject then stop. No `reset_prompt_tree_cache`. No `loadTree(false, true)`.
- Selected completed turn must not clear conversation `is_queued`.
- Do not restore the 600s recency gate on queued status.
- Do not dispatch rerun via `send_prompt_now`.
- Do not stop Default or `v2-3127` during E2E.
- Do not enqueue with session id `current`.
