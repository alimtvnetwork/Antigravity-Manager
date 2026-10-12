# Subtask 03 — Update-box color fix

> SUPERSEDED 2026-10-13: covered by the completed plan (theming contrast fixes, shipped v4.195.0). Kept for record only.

## Goal

The update notification box uses the correct color per the app's semantic design tokens.

## Steps

1. Locate the update notification UI (`src/components/` update dialog/notification).
2. Align its surface/border/accent colors with `src/styles/ui-tokens.css` semantic tokens.
3. Verify with `npm run build`.

## Done when

Update box colors match the design-token system; no off-palette surfaces.
