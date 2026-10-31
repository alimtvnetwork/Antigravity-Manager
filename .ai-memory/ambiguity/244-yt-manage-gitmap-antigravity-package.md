# Ambiguity Record 244 - yt-manage-gitmap-antigravity-package (Antigravity-Manager leg)

**Date**: 01-Nov-26
**For**: morning review by the owner. None of these items blocked the run;
each records the closest sensible solution chosen and the question left open.

## 1. Brief's prior state superseded by pull (DECIDED, needs owner awareness)

- Brief assumed: tip `88857d224` (duplicate 4.284.0 bump only), CI green.
- Reality after `gitmap pull` (01-Nov-26): tip `073394179` (v4.287.0), with
  intervening commits `5b2ab54bc` (4.285.0 bump, released), `880be99ca`
  and `98137220a` (spec/skill syncs), `78a9078d0` (4.286.0 bump + functional
  changes), `073394179` (4.287.0 bump + functional changes).
- Chosen: decision branch (b) (functional changes + CI red) instead of
  branch (a) (already green, no release).
- Question: none, unless the owner intended this leg to cover only the
  pre-pull state; if so, the extra work (fix + 4.288.0 release) still leaves
  the repo green and released, which was the brief's deliverable.

## 2. Brief's "bump to 4.285.0 ONLY" superseded (DECIDED, needs owner confirmation)

- 4.285.0 is already released (published 31-Oct-26, tag `v4.285.0` on
  commit `5b2ab54bc`, CI run `41214451609` PASS). Re-bumping it would
  violate the brief's own never-re-bump rule.
- Chosen: `npm run bump minor` from tip 4.287.0, yielding 4.288.0.
- Question: does the owner accept 4.288.0 as the next release, or was a
  different version intended?

## 3. Burned version numbers 4.286.0 and 4.287.0 (DECIDED, needs owner confirmation)

- Tags `v4.286.0` and `v4.287.0` exist on the remote and point at
  broken-CI commits (`78a9078d0`, `073394179`). Both have NO release
  (their Release runs were refused by the CI-green gate).
- Chosen: leave both tags exactly where they are (moving a tag rewrites
  history under an existing tag: forbidden) and release from 4.288.0.
- Question: does the owner accept 4.286.0 / 4.287.0 as skipped release
  numbers, or want any other treatment of those tags?

## 4. Original duplicate-bump question (OPEN, carried forward)

- Brief's question: does the owner want duplicate-bump commit `88857d224`
  (changelog/readme-only 4.284.0 bump, 4 files) covered by a 4.285.0
  release, or left as-is under the v4.284.0 release?
- Note: 4.285.0 has since been released from a later commit
  (`5b2ab54bc`), and the duplicate commit's changelog content was carried
  forward by the subsequent bumps, so the question is now historical
  rather than actionable. No action taken on `88857d224`.
- Question: confirm the duplicate commit is accepted as history, no action.

## 5. v5.x release line alongside the v4.x line (OPEN)

- The same repository shows `v5.0.13` (Draft) and `v5.0.14` (Latest)
  releases, while `main`'s version line is v4.287.0 / v4.288.0.
  The v5 releases come from the upstream/beta line, not from this leg.
- Chosen: no change to any v5 release; the v4.288.0 release is created
  through the normal tag-triggered Release workflow without touching the
  Latest pointer by hand.
- Question: is the v5 line intentional, and should a v4.288.0 release
  claim the Latest pointer or leave it on v5.0.14?

## 6. Bare `gh` resolves to the upstream fork in this checkout (NOTE)

- Bare `gh repo view` / `gh release view` / `gh run view` in this
  checkout resolve to `lbjlaq/Antigravity-Manager` (upstream), NOT to
  `alimtvnetwork/Antigravity-Manager` (origin), even though `git remote -v`
  shows origin as alimtvnetwork. Unqualified `gh` queries therefore
  return "release not found" / 404 for this repo's runs.
- Chosen: every `gh` evidence call in this run used
  `-R alimtvnetwork/Antigravity-Manager` explicitly.
- Question: none for this leg; flagging so the coordinator does not
  re-derive "no v4.284.0 release" from a bare `gh` call.

## 7. v7 skill defaults overridden by the task brief (NOTE)

- The execute-parent-task-with-n-steps-v7 skill mandates spawning 3
  subagents (A=3), bans releases unless explicitly requested (R10), and
  bans builds/tests unless explicitly requested (R1).
- The task brief explicitly authorizes releases, requires commit AND push,
  requires CI verification via `gitmap pe -t` and remote runs, and states
  the brief takes precedence over conflicting skill defaults. This run is
  also a depth-2 subagent, which cannot spawn subagents.
- Chosen: brief wins; single-worker execution, release performed, remote
  CI treated as arbiter, no local full build/test (also infeasible here:
  no cargo toolchain on PATH, prior full cargo check OOM-killed).
- Question: none; recorded so the deviation is visible, not silent.

## 8. Fix-shape choice for the E0061 error (DECIDED, one-token alternative noted)

- Chosen: `invalidate_prompt_tree_cache(instance_id)` (pass-through),
  preserving the enclosing function's scoped-vs-global contract and
  matching the same-file precedent at `queue.rs:363`.
- Alternative: `invalidate_prompt_tree_cache(None)` would also compile
  and would globally invalidate on every clear, including scoped clears.
  That over-invalidation was rejected as less precise, not as incorrect.
- Question: if the owner prefers global invalidation on any clear, it is
  a one-token change; otherwise the pass-through stands.
