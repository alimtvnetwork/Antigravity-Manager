# Parent Plan — 195 Implementing Next Five Tasks

Status: IN PROGRESS (TAKEN)
Phase: 1 (claim committed & spec authored)
Owner: Agent-195-Lead
Specs: `02-spec/21-app/195-implementing-next-five-tasks/spec.md`
Subtasks: `.ai-memory/plans/subtasks/195-implementing-next-five-tasks/`
Backlog Delta: 65 steps -> 60 steps remaining (-5 steps claimed)

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `risky-path-fallback` | `src-tauri/src/modules/release_gate/risky_path_fallback_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce explicit fallback on risky path replacements (e.g. 0 matches -> full restart; reject silent no-ops). |
| 02 | `side-effect-ordering` | `src-tauri/src/modules/release_gate/side_effect_ordering_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce ordering side effects to fail before point of no return (write credentials before kill, validate before delete). |
| 03 | `bounded-matcher-timeout` | `src-tauri/src/modules/release_gate/bounded_matcher_timeout_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce detect broadly/act narrowly matching constraints and bound all asynchronous waits with explicit timeouts. |
| 04 | `remote-branch-base` | `src-tauri/src/modules/release_gate/remote_branch_base_verifier.rs` | `IN PROGRESS (TAKEN)` | Enforce branching from remote tracking bases (`origin/main`, `origin/beta`) to eliminate untracked ancestor commits. |
| 05 | `commit-convergence` | `src-tauri/src/modules/release_gate/commit_convergence_auditor.rs` | `IN PROGRESS (TAKEN)` | Audit commit history against trial-and-error noise and enforce consolidated final-state commit convergence. |

## 2. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly preserved; zero compilers invoked.
- **Repository-Relative Paths**: 100% compliant.
- **Anti-Collision**: Initial `IN PROGRESS (TAKEN)` commitment registered and synchronized upstream so parallel AI agents do not duplicate.
