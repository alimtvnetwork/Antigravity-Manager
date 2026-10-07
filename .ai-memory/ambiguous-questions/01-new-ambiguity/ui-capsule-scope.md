# Ambiguity: slide capsule contract scope (deck-only vs app pills)

Status: proceeding by best judgment (plan C4).

## Question

`27-slide-canvas-and-themes.md` §3.2 totally bans inline `style.background/color`
on capsules (catastrophic contrast collapse on light themes) and mandates
`.capsule-{tone}` classes. Does this ban extend to AGM app pills/badges?

## Decision taken

Deck-scoped only. App pills keep daisyUI badge/btn classes driven by L1 vars;
the §3.3 audit grep runs against `.slide-stage` subtrees, not the whole app.
Rationale: app badges resolve through daisyUI `data-theme` (already
contrast-mapped per theme), while slide capsules re-purpose `--cream/--white`
tokens across the light/dark inversion — different failure mode.

## If overturned

Introduce `.capsule-{tone}` + per-palette overrides for app badges and extend
the audit grep to `src/`; re-verify T9 and ThemePicker swatch contrast.
