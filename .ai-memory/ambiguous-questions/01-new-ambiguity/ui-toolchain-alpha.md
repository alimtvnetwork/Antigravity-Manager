# Ambiguity: daisyUI opacity modifiers dead under Tailwind v3 (toolchain fix)

Status: proceeding by best judgment. VERIFIED by compiling `src/App.css`
with the project `tailwind.config.js` and grepping the emitted CSS.

## Question

Batch A maps hundreds of literal utilities onto daisyUI semantics, many with
opacity modifiers (`bg-success/10`, `text-base-content/60`, `border-warning/30`,
`ring-primary/20`, `bg-primary-content/25`, `shadow-success/50`, …).
Do these compile?

## Finding

No — not before the config fix. daisyUI v5 under Tailwind 3.4 registers theme
colors as bare `var(--color-*)` (verified: `.bg-success{ background-color:
var(--color-success) }`), which carries no `<alpha-value>` channel, so every
`/NN` semantic utility is silently dropped. Solid semantics (`bg-base-200`,
`text-primary`, `divide-base-300`, `dark:`/`motion-safe:`/arbitrary-value
variants) DO emit. Pre-existing code (`bg-primary/10` in `NetworkMonitor`,
`dark:border-base-300/60`, `bg-error/10`, …) was already affected — those
washes never rendered.

## Decision taken

Override the 20 daisyUI theme slots in `tailwind.config.js`
`theme.extend.colors` (user config wins over the plugin) with
`color-mix(in srgb, var(--color-<slot>) calc(<alpha-value> * 100%),
transparent)`. Verified post-fix: `.bg-success\/10`, `.bg-success\/15`,
and the solid (via `--tw-bg-opacity`) all emit with correct color-mix
bodies, still resolving per `data-theme` (stock + both RiseUp themes).
No values invented: percentages come from Tailwind's own opacity scale at
usage sites; hues still come from the live `--color-*` vars. WebView2
(Chromium 111+) supports `color-mix`; the repo already uses it in
`src/App.css` (`--surface-hover`, `.theme-glass-*`).

## If overturned

Revert the `tailwind.config.js` color block; then every `/NN` semantic
utility in batch A (and the pre-existing ones) must be rewritten — either
as explicit `color-mix(...)` arbitrary values per usage site, or as custom
`@layer utilities` wash classes in `src/App.css` — and re-verified with the
same standalone compile + grep procedure.
