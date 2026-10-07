# Ambiguity: RiseUp daisyUI slot mapping (accent/info/warning/success)

Status: proceeding by best judgment. REVISED after daisyUI v5 inspection.

## Question

RiseUp `spec/05-brand-system.md` §1.1–1.2 fully specifies a dark ramp, a paper
ramp, amber `#FFAD01`, violet `#791AB0`, red `#C72D2D`, and success `#33A96B`
— but daisyUI needs `primary/secondary/accent/neutral/base-100..300/info/
success/warning/error` slots. Some slots have no 1:1 RiseUp row.

## Finding: `daisyui.themes` config objects are dead under v5

daisyUI v5 (verified: `daisyui@5.7.47/index.js` +
`functions/pluginOptionsHandler.js`) reads themes ONLY from its static
`theme/object.js` built-ins. The `daisyui.themes` key in `tailwind.config.js`
is never referenced, so the pre-existing custom `light`/`dark` objects — and
the `riseup-*` objects the plan's B2 step assumed — emit nothing. The app runs
on v5 built-in `light`/`dark` plus the `html[class*="palette-"]` override
block in `src/App.css`.

## Decision taken

- Live RiseUp themes are `--color-*` var blocks in `src/App.css`
  (`[data-theme="riseup-dark"]` / `[data-theme="riseup-paper"]`) — the exact
  shape v5 emits for built-ins (see `theme/object.js` key set).
- No new objects added to `tailwind.config.js` (would be dead config); the
  pre-existing `light`/`dark` objects stay untouched with a pointer comment.
- Every slot value is still a cited RiseUp value (no invention):

| Slot (dark / paper) | Value | Cite |
|---|---|---|
| base-100/200/300 | `#0B0A09/#151312/#1D1A18` · `#FFFFFF/#FBF9F6/#F4F0EA` | §1.1 ramp / §1.2 ramp |
| primary (+content) | `#FFAD01` on `#0B0A09` (both) | §1.1 accent+accent-fg; §1.2 fill-behind-dark-text rule |
| secondary (+content) | `#ADA8A2`/`#0B0A09` · `#57514B`/`#FBF9F6` | fg-muted rows + dark/paper ink |
| accent (+content) | `#FFC24A`/`#0B0A09` · `#8A5A00`/`#FBF9F6` | accent-hover / accent-ink (single-accent discipline §1.4) |
| neutral (+content) | `#262220`/`#F7F5F2` · `#141210`/`#FBF9F6` | surface-hover+fg / paper-fg+paper |
| info (+content) | `#9B3FD1`/`#0B0A09` · `#791AB0`/`#F7F5F2` | violet-soft / violet (only cited cool tones) |
| success (+content) | `#33A96B`/`#0B0A09` (both) | success row |
| warning (+content) | `#FF9346`/`#0B0A09` (both) | amber→orange mark transition (§1 table) |
| error (+content) | `#C72D2D`/`#F7F5F2` (both) | red row |
| radius selector/field/box | `8px/14px/20px` | §4 radii (r-sm chips, r-md buttons, r-lg cards) |
| border | `1px` | §5 button/card borders |
| depth | `0` dark / `1` paper | §4: dark lifts with light-edges (no shadow); light uses soft shadows |
| noise | `0` (both) | no grain asset in app |
| size-selector/size-field | `0.25rem` (both) | daisyUI v5 built-in default (`theme/object.js`) |
| `riseup-paper` palette `primaryHover` | `#8A5A00` | only cited darker amber (darkening hover direction on paper) |

## If overturned

Any substitute value must come from `02-spec/07-design-system/` or RiseUp
`spec/05` + `spec/15`; then re-verify T3 (contrast) on both RiseUp themes.
If daisyUI is ever downgraded to v4 (which DOES read config objects), the
`tailwind.config.js` objects must be re-added in v4 shape and the CSS blocks
re-checked for double-definition.
