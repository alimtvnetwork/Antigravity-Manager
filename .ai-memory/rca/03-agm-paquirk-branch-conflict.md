# RCA-03 — `gitmap pa` failed on `gitmap-v28` with merge conflicts from a stale duplicate branch

Repo: gitmap-v28 (Go CLI) · Command: `gitmap pa` (pull-all / sync helper) · Stale branch: `fix/ci-gates-green` · Canonical fix already pushed: `83939b8ae`

## Symptom

Running `gitmap pa` against the `gitmap-v28` repo failed with **merge
conflicts**. `pa` is GitMap's "pull all" helper: it updates the local repo
from its remote. Instead of a clean fast-forward, the operation stopped and
reported conflicting changes that had to be resolved by hand.

The conflict list showed the same logical fix present on *two* branches: the
local work branch `fix/ci-gates-green` contained a **duplicate of the fix
that had already been pushed to the remote as commit `83939b8ae`**. When `pa`
tried to merge the remote's state into the local branch, git saw two
different commits making the same changes and refused to auto-merge —
identical content, different commit identities, therefore a conflict.

Zero-context primer: a *branch* is a movable label pointing at a commit. If
you fix something on a branch, push the fix from `main`, and then keep
working on the old branch, the branch still points at its own private copy of
the fix. Pulling then tries to unite the branch's private copy with the
remote's canonical copy. Git compares *commits*, not "intent", so two copies
of the same fix = a merge with overlapping hunks = conflicts.

## Root cause

**One line:** the local branch `fix/ci-gates-green` held a stale private
duplicate of a fix that had already landed on the remote as `83939b8ae`, and
`gitmap pa` tried to merge the two copies.

Step by step:

1. A CI-gates fix was developed on the branch `fix/ci-gates-green`.
2. The *canonical* version of that fix was pushed to the remote (commit
   `83939b8ae`) — from `main`, not from the work branch. At that moment the
   work branch became redundant: its reason to exist was already on the
   remote under a different commit SHA.
3. Nobody deleted or abandoned the local `fix/ci-gates-green` branch. It sat
   there holding its own duplicate copy of the fix.
4. Later, `gitmap pa` was run while the repo was sitting on
   `fix/ci-gates-green` (or while `pa` tried to reconcile it). `pa` fetched
   the remote, found `83939b8ae` with the same changes, and attempted a
   merge — producing conflicts between the two duplicate copies.

The bug class is "stale work branch": the branch outlived its purpose, and a
routine sync operation tripped over the corpse.

## How it was found (exact commands)

All commands from the repo root (`~/workspace/repos/Antigravity-Manager`);
the `gitmap-v28` repo is addressed by its GitMap label. `gitmap pe` is
GitMap's read-only diagnostic helper.

```bash
export PATH="$HOME/.local/bin:$PATH"

# 1. Surface the repo state (read-only diagnostics)
gitmap pe

# 2. Ask GitMap what went wrong with the last pull/sync on gitmap-v28.
#    This printed the conflict list and named the colliding commits.
gitmap pull-error gitmap-v28
#    -> conflicts between local fix/ci-gates-green and remote 83939b8ae

# 3. Inspect branch state: which branch is checked out, and does the stale
#    branch still exist locally?
git status
git branch --list 'fix/ci-gates-green'
#    -> currently on fix/ci-gates-green; the branch exists with its duplicate copy
```

Step 2 named the collision (`83939b8ae` = the already-pushed canonical fix);
step 3 confirmed the local branch was both checked out and stale. Together
they proved the conflict was duplicate-content, not a genuine divergence —
which is why the resolution was "abandon the branch", not "resolve hunk by
hunk".

## Fix

Do **not** resolve the conflicts hunk-by-hunk — the local branch's content is
redundant. Abandon it and re-sync `main`:

```bash
# 1. Leave the stale branch
git switch main            # (or: git checkout main)

# 2. Fast-forward main to the remote, which already contains the canonical fix 83939b8ae.
#    --ff-only = refuse to merge; only move forward. If this fails, STOP and investigate —
#    it means main has local commits the remote doesn't know about.
git pull --ff-only

# 3. The stale branch fix/ci-gates-green is left alone — NOT deleted.
```

Why not delete the stale branch? Standing house rule: **never delete a repo
or any files/branches without the user explicitly asking.** Deleting feels
like cleanup, but an unmerged branch might hold work someone cares about.
Leaving it is the safe default; the owner deletes it when they say so. The
conflict disappears because `main` never contained the duplicate — only the
canonical `83939b8ae`.

Result: `main` fast-forwarded cleanly to include `83939b8ae`; no merge, no
conflict resolution, no lost work.

## Verification evidence

1. `git pull --ff-only` on `main` succeeded with no conflicts — proof that
   `main`'s history was a clean ancestor of the remote and that the canonical
   fix `83939b8ae` is now the local `main` tip content.
2. A subsequent `gitmap pa` on `gitmap-v28` completed without conflicts —
   the duplicate-copy collision is gone because the stale branch is no longer
   in the sync path.
3. The stale branch `fix/ci-gates-green` still exists untouched (per the
   no-deletion rule) — it simply no longer participates in any pull/merge.

## Prevention — checklist item

**Before every `gitmap pa`, check where you are and what's stale:**

```bash
export PATH="$HOME/.local/bin:$PATH"

# 1. Which branch am I on, and is it the branch I mean to sync?
git status        # read the "On branch ..." line
git branch        # list local branches; look for finished work branches

# 2. If you're on a work branch whose fix already landed on main/remote
#    (compare: does the remote already contain this work?),
#    switch to main FIRST, then pa:
git switch main
gitmap pa gitmap-v28
```

Two one-line rules for the dummy AI:

1. **Never `pa` from a work branch you haven't thought about.** `pa` syncs
   *the current branch* — if the current branch is a leftover, you inherit its
   leftover problems. `main` first, then sync.
2. **Retire work branches the moment their fix is pushed canonically.**
   Once the fix is on the remote (here: `83939b8ae`), the work branch's job
   is done. If you didn't create a deletion request for it, at minimum stop
   standing on it. A branch you forgot is a merge conflict waiting for its
   appointment.
