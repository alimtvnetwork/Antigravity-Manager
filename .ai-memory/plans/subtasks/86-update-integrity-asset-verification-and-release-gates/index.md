# Subtasks — Plan 86 update integrity and release gates

Parent plan: [86-update-integrity-asset-verification-and-release-gates.md](../../pending/86-update-integrity-asset-verification-and-release-gates.md)

Order follows data flow: contracts and scaffolds first, callers after, UI and scripts last. Execute **one step per run**; mark the row `done` in the parent plan's status table, then self-loop.

| # | File | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-update-info-contract.md](./001-update-info-contract.md) | Extend UpdateInfo with asset verification fields | `pending` |
| `002` | [002-release-asset-resolver.md](./002-release-asset-resolver.md) | Resolve the newest release that has a platform asset | `pending` |
| `003` | [003-aggregate-source-errors-and-error-code.md](./003-aggregate-source-errors-and-error-code.md) | Aggregate update source failures and use a registered code | `pending` |
| `004` | [004-update-ui-skipped-versions-notice.md](./004-update-ui-skipped-versions-notice.md) | Tell the user which versions were skipped and install the resolved one | `pending` |
| `005` | [005-install-ps1-precheck-and-ladder.md](./005-install-ps1-precheck-and-ladder.md) | install.ps1 verifies assets before selecting a version | `pending` |
| `006` | [006-install-sh-precheck-parity.md](./006-install-sh-precheck-parity.md) | install.sh gets the same pre-check contract | `pending` |
| `007` | [007-agm-update-uses-resolved-version.md](./007-agm-update-uses-resolved-version.md) | agm update and the delegate updater use the resolved version | `pending` |
| `008` | [008-release-workflow-artifact-gate.md](./008-release-workflow-artifact-gate.md) | release.yml publishes only with every expected artifact | `pending` |
| `009` | [009-post-publish-asset-url-verification.md](./009-post-publish-asset-url-verification.md) | Post-publish job re-fetches every URL in updater.json | `pending` |

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
