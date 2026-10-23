# Subtask 199-02: Maintainer Staging Protocol Guard

Status: IN PROGRESS (TAKEN)
Owner: Agent-199-Lead
Spec: `02-spec/21-app/199-implementing-next-five-tasks/spec.md` (Task 2)

## Target
- Create `src-tauri/src/modules/release_gate/staging_protocol_guard.rs`
- Create `src-tauri/src/modules/release_gate/tests_staging_protocol.rs`
- Do NOT edit `src-tauri/src/modules/release_gate/mod.rs` (lead wires registration).

## Work
Implement a change-set classifier: input is a `ChangeSetSummary` (files changed, lines added/removed, flags: touches schema/migrations, adds public command or IPC surface, adds feature flag, touches release machinery, is docs-only, is a one-line fix). Output `StagingVerdict`:
- `Required` — features, major refactors, schema changes, non-trivial fixes (maintainer confirmation + `beta` validation before `main`),
- `Recommended` — multi-module changes without the hard triggers,
- `Unnecessary` — trivial/docs/one-line fixes.
The verdict must list the deciding factors (no silent classification). Pure functions, no I/O, house guard style.

## Tests (in the tests file, house style)
- Schema migration changeset → `Required` with the schema factor listed.
- One-line typo fix → `Unnecessary`.
- Multi-module feature without schema change → at least `Recommended`.
- Docs-only change → `Unnecessary`.

## Invariants
Zero builds/tests during implementation. Relative paths only. GitMap-only search. No silent errors.
