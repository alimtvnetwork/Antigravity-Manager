# Subtask 02 — Progress-bar layout correction

> SUPERSEDED 2026-10-13: covered by the completed plan (5-slot `#1af18d` progress bar, right-aligned compaction, shipped v4.195.0). Kept for record only.

## Goal

Quota bars respect their designed column widths — no left-squish — with `#1af18d` as the progress green.

## Steps

1. Inspect the accounts-table quota columns (`src/components/accounts/`) and the `compact` prop added in v4.179.0; identify why the bars collapse left (width allocation / flex / min-width).
2. Restore proper widths: bars fill their column, 4H and Weekly bars keep designed inter-bar spacing.
3. Set progress green to exactly `#1af18d` (keep the red→green scale semantics).
4. Layout only — gradient, checkpoints, behavior unchanged. Verify with `npm run build`.

## Done when

Bars render at full designed widths with correct spacing and `#1af18d` green.
