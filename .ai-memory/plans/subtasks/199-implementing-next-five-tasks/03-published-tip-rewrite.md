# Subtask 199-03: Published-Tip Rewrite Guard

Status: IN PROGRESS (TAKEN)
Owner: Agent-199-Lead
Spec: `02-spec/21-app/199-implementing-next-five-tasks/spec.md` (Task 3)

## Target
- Create `src-tauri/src/modules/release_gate/published_tip_rewrite_guard.rs`
- Create `src-tauri/src/modules/release_gate/tests_published_tip_rewrite.rs`
- Do NOT edit `src-tauri/src/modules/release_gate/mod.rs` (lead wires registration).

## Work
Implement a rewrite-safety evaluator. Input `RewriteOperation`: target branch name, whether the tip is published, open PRs based on the branch (count + ids), whether a `backup/*` rollback ref exists for the current tip, and the drift check result (files differing between backup ref and HEAD, or `NotChecked`). Output `RewriteVerdict`:
- `Allowed` only when: tip unpublished, OR (PRs inspected and acknowledged AND backup ref present AND drift confirmed zero).
- `Blocked` with every missing precondition named (open PRs not inspected, no backup ref, drift unchecked, drift non-zero).
Pure functions, no I/O, house guard style.

## Tests (in the tests file, house style)
- Open PR + no backup ref → `Blocked` naming both reasons.
- Backup ref present but drift `NotChecked` → `Blocked`.
- No open PRs + backup ref + zero drift → `Allowed`.
- Unpublished local branch → `Allowed`.

## Invariants
Zero builds/tests during implementation. Relative paths only. GitMap-only search. No silent errors.
