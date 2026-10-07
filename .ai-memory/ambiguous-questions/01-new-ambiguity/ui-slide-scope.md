# Ambiguity: slide `[data-theme]` selector scope (spec-40 vs daisyUI)

Status: proceeding by best judgment.

## Question

`40-theme-switch.md` §3 writes bare `[data-theme="<id>"]` blocks, but AGM sets
`data-theme` on `<html>` for daisyUI (`light`/`dark` + new `riseup-*`). A bare
selector would still only *match* deck ids in practice (app never sets those
values on `<html>`), yet it leaves the two systems one typo apart.

## Decision taken

Scope every slide block to `.slide-stage[data-theme="<id>"]`. Same six vars
per block, same values, same order as 40-§3; only the selector is narrowed so
app chrome can never match. `white-blue` stays excluded; block count stays 8.

## If overturned

Rewrite the blocks as bare `[data-theme]` selectors and re-run T9 plus a
full-palette daisyUI regression (every `THEME_PALETTES` id × deck render).
