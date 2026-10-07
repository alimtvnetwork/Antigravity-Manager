# Ambiguity: batch A scope (page files vs sub-components)

Status: proceeding by best judgment.

## Question

Batch A names "Accounts, Instances, Dashboard". The Accounts view renders
mostly through sub-components (`AccountTable`, `AccountGrid`/`AccountCard`,
`AccountRow`, `AccountDetailsDialog`, `AddAccountDialog`) that carry their own
hardcoded hex (`#070b10`-family navy ramp, gold `#f5d76e`) and literal palette
classes; Dashboard renders `CurrentAccount` + `BestAccounts` (no hex found,
literals not yet audited).

## Decision taken

This batch touches ONLY the three page files
(`src/pages/Accounts.tsx`, `src/pages/Instances.tsx`, `src/pages/Dashboard.tsx`).
Sub-component literals stay untouched pending a components batch, so plan-T1
(`rg "#[0-9a-fA-F]{3,6}"`) still flags `src/components/accounts/*` after this
batch. Rationale: page files alone hold ~400 literal utilities; mixing
component rewrites into the same diff risks unreviewable churn and behavior
drift in quota/switch logic.

## If overturned

Extend the mapping in `ui-batch-a-mapping.md` to `src/components/accounts/*`
and `src/components/dashboard/*`, then re-run T1 plus a visual pass on both
RiseUp themes.
