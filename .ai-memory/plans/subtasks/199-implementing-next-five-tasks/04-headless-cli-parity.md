# Subtask 199-04: Headless & CLI Parity Guard

Status: IN PROGRESS (TAKEN)
Owner: Agent-199-Lead
Spec: `02-spec/21-app/199-implementing-next-five-tasks/spec.md` (Task 4)

## Target
- Create `src-tauri/src/modules/release_gate/headless_cli_parity_guard.rs`
- Create `src-tauri/src/modules/release_gate/tests_headless_cli_parity.rs`
- Do NOT edit `src-tauri/src/modules/release_gate/mod.rs` (lead wires registration).

## Work
Implement a parity analyzer over an operation registry. Input: `&[OperationEntry]` where each entry has a name, `has_gui_entry: bool`, `has_cli_entry: bool`, and `service_route: Option<String>` (the shared underlying service both surfaces must call, e.g. `copy_instance_with_options`). Output `ParityReport` listing:
- GUI-only operations (no CLI entry),
- CLI-only operations (no GUI entry),
- operations whose surfaces do not share one service route (missing route or divergent routes),
with `is_clean()` when the registry is fully paired and shared. Pure functions, no I/O, house guard style.

## Tests (in the tests file, house style)
- GUI-only lifecycle operation is flagged.
- CLI-only operation is flagged.
- Entry with no shared service route is flagged.
- Fully paired registry with shared routes passes clean.

## Invariants
Zero builds/tests during implementation. Relative paths only. GitMap-only search. No silent errors.
