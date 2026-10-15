# E2E testing the AI already ran

Read this folder when the user says "run the same end-to-end tests", "repeat the E2E", or "how did the AI test this".

These files are checklists of tests this session actually created or executed. They are not a design catalog. Older generic flows stay in `.ai-memory/e2e-playbook/`. The long decision log for the prompt-tree crash stays in `.ai-memory/memory/learned/27-e2e-instance-autoswitch-prompt-playbook.md`.

## Order

| # | File | Release the run proved | What it covers |
|---|---|---|---|
| 0 | [00-decisions.md](./00-decisions.md) | all | How the AI chose the binary, the instance, and what counts as pass |
| 1 | [01-prompt-tree-cli.md](./01-prompt-tree-cli.md) | v4.197.1 then v4.198.0 | Tree, reset, send, queue, backup, restore, window, auto-switch 15% |
| 2 | [02-live-switch-and-resend.md](./02-live-switch-and-resend.md) | v4.201.0 (`f9b7d218`) | Isolated copy, ff, validation block, live prompt resend |
| 3 | [03-cdp-onboarding-and-prompt.md](./03-cdp-onboarding-and-prompt.md) | v4.202.0 (`e8b0fb49`) | Security Notice dialog, CDP send, backup restore, settings route |
| 4 | [04-telegram-balance-ssh.md](./04-telegram-balance-ssh.md) | v4.203.0 (`876dc50b`) | Balance table, CSV, SSH status, RSA round-trip |
| 5 | [05-master-checklist.md](./05-master-checklist.md) | all | One page to tick |

## Rules for the next AI

- Use the binary you just built. Do not score the installed `agm` if it is an older version.
- One command at a time. Build with `-j 1`.
- Test account switch, fast-forward, and prompt send on the copy instance, not on the user's live Default IDE, unless the user named Default.
- Put the auto-switch threshold back to 15% before you finish.
- Do not print tokens, passwords, or prompt bodies. Print ids, counts, exit codes, and PASS or FAIL.
- Relative paths only in the result you write back into this folder.
