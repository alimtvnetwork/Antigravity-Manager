# Subtask 03: PR Single Problem Scope Classification Linter (`Imp-677`)
Status: IN_PROGRESS
Parent Plan: 210-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md PR Scope & Grouping:
"Single Problem Scope: A PR represents a cohesive collection of fixes or features dedicated to a single problem class. Keep unrelated concerns (such as governance, release tooling, or documentation) in isolated PRs."

Implement `src-tauri/src/modules/release_gate/pr_problem_classification_linter.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_pr_problem_classification_linter.rs`.

## Requirements
- Classify PR changeset touchpoints into functional domains (GatewayProxy, ReleaseTooling, InstanceManagement, DatabaseOrm, UiComponents, Documentation).
- Check PR description and titles against single-problem scope rules.
- Detect scope creep when unrelated domains are modified within the same PR without architectural justification.
- Provide structured report tracking affected functional domains, detected scope entanglement, and compliance status.
