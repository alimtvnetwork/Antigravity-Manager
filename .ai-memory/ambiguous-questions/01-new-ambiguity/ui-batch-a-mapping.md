# Ambiguity: batch A token mapping + deferred radii

Status: proceeding by best judgment.

## Mapping applied (all targets are L1 vars or L3 daisyUI semantics)

- Navy hex ramp → depth slots: `#070b10/#050f18/#040e16/#051320/#061421` ⇒
  `base-100`; `#071a27/#081a29/#081a2b/#0a1e30/#092033/#061724/#0c283f/#0c2438/
  #102a40/#122e47` ⇒ `base-200`; `#15334d/#153a54/#14344d/#173d5c/#1e4a6d` ⇒
  `base-300`; gold `#f5d76e` text ⇒ `warning`.
- Status hues → state language (spec-02 §9): emerald/green ⇒ `success`,
  rose/red ⇒ `error`, amber/orange/yellow ⇒ `warning`.
- Accent blue ⇒ `primary`; decorative cyan/sky/teal ⇒ `info`; decorative
  purple/indigo ⇒ `accent` (amber-family on RiseUp, single-accent safe).
- Neutrals: slate/gray text ⇒ `base-content` + fixed opacity (`800+` solid,
  `700` /80, `600` /70, `500` /60, `400` /50); `bg white/slate-100/gray-100` ⇒
  `bg-base-100/200`; light borders/dividers ⇒ `base-300` (opacity kept);
  `dark:` slate/gray surfaces/borders ⇒ `base-200/300`.
- Wash normalization: light `*-50/*-100` ⇒ `{sem}/10`, dark `*-900/*-950` ⇒
  `dark:{sem}/20`, then collapsed to a single `{sem}/10` wash.
- Solid accent hovers use `hover:bg-[var(--primary-hover)]` (L1, set per
  palette by `ThemeManager`, correct hover direction on paper per T10).
- `animate-*` ⇒ `motion-safe:animate-*` (plan T5; `motion-safe` exists in
  Tailwind 3.4). `hover:bg-black/5 dark:hover:bg-white/5` veils kept.
- Loading veil `dark:bg-[#071a27]/85` ⇒ `dark:bg-black/70` (`base-100` would
  invert to a light veil on stock dark).

## Deferred: `rounded-[4px]` / `rounded-[5px]` radii

`Instances.tsx`/`Accounts.tsx` use 4px/5px arbitrary radii throughout. No
migration applied: spec-06 base is 8px and RiseUp chips are 8px, but a blind
global radius rewrite without visual verification risks making dense
toolbar/card UI look worse. Follow-up with screenshots.

## Deferred: `shadow-2xs` / `shadow-xs`

Not Tailwind 3.4 utilities (v4 names); currently no-ops. Left untouched —
removal is a separate cleanup, not a token fix.
