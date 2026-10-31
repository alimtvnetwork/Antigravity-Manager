# Architecture Specification: Overnight Autopilot Check, CI Fix and Release for Antigravity-Manager

**Slug**: `yt-manage-gitmap-antigravity-package`
**Sequence**: `244`
**Spec Path**: `02-spec/21-app/244-yt-manage-gitmap-antigravity-package/01-architecture-spec.md`
**Status**: APPROVED_FOR_EXECUTION
**Target Branch**: `main`
**Attribution**: strictly `@aukgit`
**Date**: 01-Nov-26

---

## 1. Executive Summary

This is the Antigravity-Manager leg of the overnight autopilot task. The task brief
assumed a prior state (tip at the duplicate 4.284.0 bump commit `88857d224`, CI
green, no functional changes since the v4.284.0 release) and prescribed: report
"already green at v4.284.0, no bump needed" in that case, otherwise fix CI and
bump to 4.285.0 only.

A pull at the start of this run superseded that prior state. The tip is now
`073394179` (v4.287.0), six commits ahead of the released v4.284.0 commit, with
real functional changes and RED CI. Decision branch (b) therefore applies, with
the version number adjusted to the next free version (see section 3): fix the
single CI compile error, then bump minor to v4.288.0, tag, release, and verify
CI green on the release tip.

Antigravity-Manager version line (v4.x) is kept strictly separate from the
gitmap v6.x line in every summary in this task.

## 2. Tip-vs-Release Comparison (evidence, 01-Nov-26)

Released v4.284.0 commit: `d5769c5d7` (annotated tag object `b777f56e`,
published 31-Oct-26, 21 assets on the release page).

Commits on `main` after `d5769c5d7`, oldest first:

| Commit | Content | Tag / Release state |
| :--- | :--- | :--- |
| `88857d224` | Duplicate 4.284.0 bump: changelog and readme only (4 files, no version-file change) | No new tag; covered by nothing new |
| `5b2ab54bc` | Bump to 4.285.0 (version files, changelog, readme) | Tag `v4.285.0` on remote; release published 31-Oct-26; CI run `41214451609` PASS |
| `880be99ca` | Sync: mirror canonical prompts, skills, specs (45 files, mostly specs and scripts) | No version change; CI run `41214615412` PASS |
| `98137220a` | Sync follow-up: `version.json` one-line change | No version change; CI run `41217348766` PASS |
| `78a9078d0` | Bump to 4.286.0 PLUS functional changes (prompt-tree queue, detection, PromptTreeView components, fleet rows) | Tag `v4.286.0` on remote; NO release; CI run `41218704243` FAIL |
| `073394179` | Bump to 4.287.0 PLUS functional changes (prompt-tree copy-button fix, attachment normalization, detection) — current tip | Tag `v4.287.0` on remote; NO release; CI run `41221316712` FAIL; Release run `41221319488` FAIL at the release gate |

Working tree after pull: clean (`gitmap pc`: 0 uncommitted, 0 unpushed).
Remote HEAD: `073394179`.

`gitmap pe -t` BEFORE (01-Nov-26): latest `0733941` FAIL
(`Release [FAIL]`, `CI [FAIL]`), `-1` `78a9078` FAIL, `-2` `9813722` PASS,
`-3` `880be99` PASS, `-4` `5b2ab54` PASS.

## 3. Version Decision (closest sensible solution)

- The brief's "bump to 4.285.0 ONLY" instruction predates the 4.285.0, 4.286.0
  and 4.287.0 bumps pushed by other agents. 4.285.0 is already released, so
  re-bumping it is impossible and re-using it is forbidden by the same rule
  that forbids re-bumping 4.284.0.
- The existing `v4.286.0` and `v4.287.0` tags point at broken-CI commits.
  Moving either tag onto a fixed commit would rewrite history under an
  existing tag: FORBIDDEN. Those two version numbers are therefore burned
  as releases; they remain in history as tags only.
- `npm run bump minor` from the tip version 4.287.0 yields 4.288.0, which is
  the next free version and matches the brief's bump mechanism exactly.
- The brief's duplicate-bump question (does the owner want commit `88857d224`
  covered by a 4.285.0 release, or left as-is) is recorded for morning review
  in `.ai-memory/ambiguity/244-yt-manage-gitmap-antigravity-package.md`;
  it does not block the fix, because 4.285.0 has since been released from a
  later commit and the duplicate commit's changelog content was carried
  forward by the subsequent bumps.
- No change is made to the v5.x release line (`v5.0.13` Draft, `v5.0.14`
  Latest on the same repository). Whether the v5 line is intentional, and
  whether the v4.288.0 release should claim the Latest pointer, is a morning
  question for the owner (see ambiguity file).

## 4. Root Cause: Single CI Compile Error E0061

CI failure in both the 4.286.0 and 4.287.0 runs is one error, repeated across
all Rust jobs:

```text
error[E0061]: this function takes 1 argument but 0 arguments were supplied
  --> src-tauri/src/modules/repo_db/queue.rs:516:5
     invalidate_prompt_tree_cache();
note: function defined here
  --> src-tauri/src/modules/repo_db/tree.rs:109:8
     pub fn invalidate_prompt_tree_cache(instance_id: Option<&str>)
error: could not compile `agm-ui` (lib) due to 1 previous error; 1234 warnings emitted
```

Introduced by commit `78a9078d0`, which added
`clear_queued_prompts_for_instance(instance_id: Option<&str>)` in
`src-tauri/src/modules/repo_db/queue.rs` and ended it with a zero-argument
call. The 4.287.0 commit did not touch that file, so it inherited the error.
The Release run `41221319488` then failed at its gate, correctly refusing to
release a commit whose CI concluded `failure`:
`[Release Gate Intercept] CI for commit 073394179... concluded 'failure',
not 'success'. Release refused.`

## 5. Fix Specification

File: `src-tauri/src/modules/repo_db/queue.rs`, function
`clear_queued_prompts_for_instance`, final statement before `Ok(count)`.

Change:

```rust
// WHY: pass the caller's scope through, so a scoped clear invalidates only
// that instance's prompt-tree cache and an all-instances clear invalidates
// everything, matching the function's own contract and tree.rs handling.
invalidate_prompt_tree_cache(instance_id);
```

Rationale:

- `tree.rs` `invalidate_prompt_tree_cache` treats `Some(id)` (non-empty,
  not `all`) as a scoped delete (instance id, normalized id, `all` rows and
  matching cache keys) and treats `None` or `all` as a full cache delete.
  Passing `instance_id` through preserves both branches of the enclosing
  function's own contract (scoped update vs. all-instances update, and the
  matching in-memory map pruning directly above the call).
- Precedent in the same file: `queue.rs:363` passes `target_instance`
  (an `Option<&str>`) straight through. Same pattern at
  `src-tauri/src/modules/repo_db/import.rs:509`,
  `src-tauri/src/modules/repo_db/resend.rs:321` and
  `src-tauri/src/modules/repo_db/sqlite_export.rs:558`. Every other call
  site in the codebase passes exactly one argument; `queue.rs:516` was the
  only zero-argument call.
- Alternatives rejected: `None` would over-invalidate every other
  instance's cache on a scoped clear; `Some("all")` would be wrong for a
  scoped clear (it happens to behave like the global branch in `tree.rs`,
  but it misstates intent and bypasses the scoped path).

No other code change is required for CI. The 1234 warnings are pre-existing
and are not errors; they are left untouched.

## 6. Release Specification

1. Fix commit on `main` (semantic gitmap fix verb, hyphen-format message,
   no colon in the message argument), pushed via `gitmap push`.
2. `gitmap pe -t` until CI is green on the fix commit.
3. `npm run bump minor` (no `--commit` flag; the commit is made via gitmap).
   Expected result: 4.287.0 -> 4.288.0 across `package.json`,
   `package-lock.json`, `version.json`, `src-tauri/Cargo.toml`,
   `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`,
   `src-tauri/hooks.nsh`, `Casks/antigravity-tools.rb`,
   `releases-manifest.json`, `changelog.md`, `changelog_en.md`,
   `readme.md`, `readme_en.md`, `src/components/layout/MiniView.tsx`.
   Verify the diff before committing; changelog attribution strictly
   `@aukgit` and no other handle; stable-release README changelog sync
   required (see repo `AGENTS.md` release discipline).
4. Bump commit (semantic gitmap chore verb, hyphen-format message), pushed
   via `gitmap push`; `gitmap pe -t` until CI is green on the bump commit.
5. Tag `v4.288.0` on the bump commit and push the tag with raw git
   (`git tag v4.288.0`, `git push origin v4.288.0`). Declared raw-git
   exception: `gitmap release` creates a release branch plus metadata that
   violates this repo's main-only convention, and `gitmap push` treats its
   first argument as a repo name and errors on a tag. Raw git is used for
   the tag steps only.
6. The tag-triggered Release workflow runs with its gate-first check (CI
   must be green for the tagged commit). Watch it to success.
7. Verify on the remote: tag `v4.288.0` resolves to the bump commit,
   the release page exists with the expected asset set (v4.284.0 shipped
   21 assets), and `gitmap pe -t` is green on the release tip.

## 7. Non-Goals and Guardrails

- No force-push, no history rewrite, no moving `v4.286.0` / `v4.287.0` tags,
  no re-bump of 4.284.0 or 4.285.0, no file deletions.
- No `.github/workflows/` changes in this task. (If a fix ever needed one,
  the VM `gh` token lacks `workflow` scope and the change would be committed
  locally and reported BLOCKED with its SHA; this task needs none.)
- No local full build or test suite: this VM previously could not complete
  a full cargo/clippy check (SIGKILL/OOM on 7.7 GiB), and no cargo toolchain
  is on PATH here. Remote CI is the arbiter; local verification is limited
  to targeted inspection of the one changed file.
- No changes to the v5.x release line, no other repository touched
  (disjoint ownership: Antigravity-Manager only).
- Strictly relative Git paths in all artifacts; no absolute paths or
  `file:///` URIs. Dates in DD-MMM-YY form.

## 8. Verification Evidence Required Before Reporting Done

- `gitmap pe -t` BEFORE (FAIL, runs `41221316712` / `41221319488`) and
  AFTER (PASS on the release tip, with run IDs).
- Fix commit SHA, bump commit SHA, tag object / remote resolution for
  `v4.288.0`, release page URL and asset count.
- `gitmap pc` clean (0 uncommitted, 0 unpushed) at the end.
