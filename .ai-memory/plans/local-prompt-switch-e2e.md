# Plan: Local prompt, switch, and vault retest

- **Slug**: `local-prompt-switch-e2e`
- **Status**: `COMPLETED`
- **Spec**: `02-spec/21-app/local-cli-e2e/01-retest-cases.md`

Any later AI reruns the spec on this machine. The cases cover prompt entry from the tree sequence, prompt entry on a named instance, account switch on the idle copy, the shared login, the email vault, and the auto-switch threshold. They do not rotate every vault account through the running Default IDE.

## Subtasks

| # | File | Scope | Status |
| :--- | :--- | :--- | :--- |
| `01` | `.ai-memory/plans/subtasks/local-prompt-switch-e2e/01-prompt-send.md` | P1, P2, P3 | `COMPLETED` |
| `02` | `.ai-memory/plans/subtasks/local-prompt-switch-e2e/02-account-switch-and-shared-login.md` | S1, S2 | `COMPLETED` |
| `03` | `.ai-memory/plans/subtasks/local-prompt-switch-e2e/03-email-and-autoswitch.md` | E1, B1 | `COMPLETED` |

## Rules

- Read `02-spec/21-app/local-cli-e2e/01-retest-cases.md` before running a command.
- Search with `gitmap`.
- Build with `cargo build --bin agm -j 1`.
- Relative paths only in tracked files.
- Restore the copy account and stop the copy before finishing.
- Restore auto-switch to 15%.
