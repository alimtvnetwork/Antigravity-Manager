# Ambiguity: app theme count (palettes vs daisyUI themes vs slide themes)

Status: proceeding by best judgment (plan D1/D3).

## Question

How many themes does the overhaul ship? Three competing counts exist:

- 14 app palettes in `src/components/common/themePalettes.ts` (mapped to 2 daisyUI
  `data-theme` values by `ThemeManager.tsx`).
- 8 slide switcher ids (`02-spec/07-design-system/40-theme-switch.md` §1) and
  10 catalog presentation themes (`27-slide-canvas-and-themes.md` §2).
- 2 new RiseUp-derived palettes proposed in `docs/ui-overhaul-plan.md`.

## Decision taken

Ship 16 app palettes (14 existing + `riseup-dark` + `riseup-paper`) and 4 daisyUI
themes. Slide ids stay deck-scoped (`.slide-stage` only) and never enter
`THEME_PALETTES` or the slide-agnostic `ThemePicker`.

## Why ambiguous

`40-theme-switch.md` counts 8 while `27-slide-canvas-and-themes.md` catalogs 10,
and the task statement lists "daisyui themes incl RiseUp Asia + slide-system
themes" together, which could be read as merging all three sets into daisyUI.
Merging is rejected: slide vars (`--canvas/--ink/--accent…`) collide semantically
with app vars and the capsule contract forbids cross-use.

## If overturned

Adding a slide id as an app palette requires all six 40-§2 values plus
`surfaceHover/primaryHover/borderHover` twins and a `data-theme` mapping entry;
re-run acceptance T1/T3/T6 for the new id.
