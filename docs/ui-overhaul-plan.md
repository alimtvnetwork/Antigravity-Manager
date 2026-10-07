# UI Overhaul Plan — Token Architecture, Worklist, Acceptance

Branch: `feature/ui-ux-overhaul`. Prior reader results 1–5 carried no inspectable
evidence (summaries only), so every claim below is grounded in direct reads done
for this plan. External `D:/work` repos are READ-ONLY (RiseUp values cited, not copied).

## 1. Ground rules (non-negotiable)

- No invented hex/px/ms: every value comes from `02-spec/07-design-system/`,
  `tailwind.config.js`, `src/App.css`, `themePalettes.ts`, or cited RiseUp files.
- No purple-gradient AI-slop (`bundled:taste`): violet lives only in depth/glow roles.
- Never mix `&&` with `? :null` in JSX; positive booleans implicit.
- Preserve: `request.ts` dual IPC contract (`COMMAND_MAPPING`, `_suppressGlobalModal`,
  `abv-unauthorized` debounce), MiniView bounds logic (`savedBounds`/`priorBounds` in
  `useViewStore`/`windowManager`), Tauri drag regions (`data-tauri-drag-region`,
  36px band), WebView2 guards (`visibilitychange`/`window-restored` reflow in
  `App.tsx`/`Layout.tsx`), Linux fallbacks (`isLinux()` skips View Transitions +
  transparent-window background sync).

## 2. Token architecture (4 layers)

```text
L0 spec sources (read-only inputs)
  03-theme-variable-architecture  HSL-triplet format, semantic names, forbidden patterns
  16 + 17-theme-tokens.json       navy-purple / vscode-* / tokyo-night / one-dark / github-dark …
  tokens/design-tokens.json+less  4 themes, radii, motion (150/300/500ms, emphasized/standard easings)
  27/30/32/40 + 05-bright-gold    slide systems: 10 catalog themes, 8 switcher ids × 6 vars, capsule contract
  riseup-asia (READ-ONLY)         amber #FFAD01 / violet #791AB0 / red #C72D2D, warm-black ramp,
                                  paper light, oklch twins, radii 8/14/20/28, motion 180/320/560/800/1200ms
        │
L1 CSS vars (runtime truth, set by ThemeManager per palette id)
  existing: --bg --surface --primary --fg --surface-hover --primary-hover --border-hover
            --app-* twins, --awan-* brand anchors (src/App.css)
  add (namespaced, no collisions): --rise-* (warm ramp + amber-ink), --slide-* (deck stage only)
        │
L2 tailwind.config.js extend (compile-time utilities)
  existing: fontFamily ubuntu/mono, colors.awan (9 anchors)
  add: colors.rise = {amber,amber-ink,violet,red,ink-…} (hex twins from RiseUp spec/05 §1)
       NO oklch in config (Tailwind 3.4 + WebView2: keep hex/hsl strings)
        │
L3 daisyUI themes (component primitives: btn/badge/toggle/slider)
  existing: light + dark (2 themes; ThemeManager maps every palette → data-theme dark|light)
  add: riseup-dark (warm-black bg, amber primary), riseup-paper (paper bg, amber-ink text)
       ThemeManager maps palette.dark → data-theme, with explicit override ids
       (riseup-dark→dark base + rise tokens, riseup-paper→light base + rise tokens).
```

Key decisions (best judgment; see ambiguity files):

- D1 App palettes ≠ slide themes. The 8 switcher ids (`40-theme-switch.md`) and 10
  catalog themes (`27-slide-canvas-and-themes.md`) target a 1920×1080 deck stage with
  a 6-var (`--canvas/--ink/--ink-muted/--accent/--accent-ink/--card`) contract and a
  capsule className system. They do NOT become app palettes. Only `bright-gold-tech`
  tokens (`--pres-*`, `05-bright-gold-tech/01`) may inform dark-gold app accents.
- D2 Storage format stays hex in `themePalettes.ts` (matches all 14 existing entries
  and `setBackgroundColor`); HSL twins from spec files go in comments/docs, oklch
  twins from RiseUp are reference-only (see `ui-token-format.md`).
- D3 New app palettes: `riseup-dark` (warm-black #0B0A09 ramp, amber #FFAD01 primary)
  and `riseup-paper` (paper #FBF9F6, amber-ink #8A5A00 for text). Everything else is
  a token re-map of the existing 14, not new palettes (see `ui-theme-count.md`).
- D4 Typography: keep Ubuntu headings + Ubuntu Mono code (already shipped). Poppins
  body (spec-04 / RiseUp) is deferred — new webfont weight in WebView2/offline needs
  a separate decision (see `ui-font-poppins.md`).
- D5 Motion: adopt spec motion tokens where they already agree with shipped values
  (250ms universal color transition in App.css ≈ 300ms normal; 200ms micro per
  `24-app-ui-design-system`; slide stage 0.45s stays deck-only). `prefers-reduced-motion`
  0.01ms rule from `40-theme-switch.md` §6 applies to any new animation.

## 3. Worklist per implementer

### A. Token layer (`src/App.css`, `tailwind.config.js`)

- A1 Add `--rise-*` var block under `:root`/`.dark` (values: RiseUp spec/05 §1.1–1.2).
- A2 Add `colors.rise` to tailwind extend (same hex twins). No other config change.
- A3 Add `--slide-*` 6-var deck contract as opt-in classes only (`.slide-stage`
  + `[data-theme=…]` blocks from `40-theme-switch.md` §3), scoped so app chrome
  never matches. No layout file edits during this step (checklist §6).

### B. Theme engine (`themePalettes.ts`, `ThemeManager.tsx`, `ThemePicker.tsx`)

- B1 Append `riseup-dark` + `riseup-paper` entries (bg/surface/primary/fg + 3 hovers).
  Source: RiseUp spec/05 §1.1/1.2 + spec/15 §2 planes.
- B2 Extend daisyUI `themes` in `tailwind.config.js` with `riseup-dark`/`riseup-paper`
  full token sets (primary/secondary/accent/neutral/base-100..300/info/success/
  warning/error); keep `darkTheme: "dark"`.
- B3 ThemeManager: map override ids to the new `data-theme` values; keep
  `palette-{id}` class loop, `setBackgroundColor`, `set_window_theme`, system
  media-query path, and Linux guards byte-identical in behavior.
- B4 ThemePicker: no structural change; verify 16 swatches render (grid already maps
  `THEME_PALETTES`). Selected ring keeps `border-[var(--primary)]`.

### C. Components (cards, pills, header, MiniView)

- C1 Replace literal color classes with semantic tokens per AC-001/AC-002
  (`bg-primary`, `var(--app-surface)` …); code-block dark exception stays (spec-09).
- C2 Single-accent Von Restorff audit per viewport: max one amber/primary-filled
  element; demote the rest to outline/hairline (RiseUp spec/15 §1, spec-16 §1).
- C3 Header/drag band untouched geometrically (36px, `data-tauri-drag-region`,
  `startDragging` fallback). MiniView bounds logic untouched.
- C4 Capsule rule is deck-scoped: app pills keep daisyUI badge classes; forbid
  inline `style={{background…}}` on pills only inside `.slide-stage`
  (audit grep from `27-slide-canvas-and-themes.md` §3.3).

### D. Verification owner

- D1 `npm run lint && tsc --noEmit && npm run build` → exit 0 (AC-DS-001/002 gate).
- D2 `cargo fmt -- --check` + `cargo clippy --all-targets --all-features` if Rust touched.
- D3 Run §4 acceptance checks; record results in PR + `pull_request_template.md`.

## 4. Acceptance checks

| # | Check | Source |
|---|---|---|
| T1 | Zero hardcoded hex/rgb/literal Tailwind colors in touched components (`rg "#[0-9a-fA-F]{3,6}" src --glob '!themePalettes.ts' --glob '!App.css'` reviewed; only token files + code-block exception) | AC-001/AC-002 |
| T2 | Flipping `--primary` updates all primary elements; `:root`+`.dark` define all tokens | AC-003..005 |
| T3 | Text-on-accent ≥ 4.5:1 (AA); RiseUp pairs per spec/05 §1.3 table | AC-006 |
| T4 | Headings Ubuntu, code Ubuntu Mono; no new font files shipped | AC-007/009, D4 |
| T5 | Hovers ≤ 300ms; `prefers-reduced-motion` kills new animations (0.01ms) | AC-012/014 |
| T6 | 36px drag band + dialog drag divs keep `data-tauri-drag-region`; window drags in Tauri mode | AC-DS-002 |
| T7 | Background occlude/restore repaints (WebView2 `visibilitychange` path); MiniView enter/exit restores `savedBounds` | skill invariants |
| T8 | `request.ts` web/Tauri parity spot-check (one GET + one POST command each mode) | skill invariants |
| T9 | Slide audit grep (spec-27 §3.3) zero matches; `[data-theme]` block count = 8; `white-blue` absent from slide control | spec-40 §6 |
| T10 | Light/paper palettes: amber never carries body text (`--accent-ink` #8A5A00 instead) | RiseUp spec/05 §1.2 |

## Sources inspected

`tailwind.config.js`, `package.json` (daisyUI 5.5.13, Tailwind 3.4.19),
`src/App.css`, `src/components/common/themePalettes.ts`,
`src/components/common/ThemeManager.tsx`, `src/components/settings/ThemePicker.tsx`,
`src/utils/request.ts` (COMMAND_MAPPING), `src/stores/useViewStore.ts` +
`src/utils/windowManager.ts` (bounds), `src/App.tsx` + `Layout.tsx`
(visibilitychange), `TitleBar`/`MiniView`/`Navbar`/dialogs (drag regions),
`02-spec/07-design-system/` {02,03,04*,05*,16§1-3,17§navy/vscode/tokyo,24*,27§1-4,
30§1-2,32§1-3,40,97}, `tokens/{design-tokens.json,theme-palette.less}`,
`05-bright-gold-tech/01`, `02-spec/24-app-ui-design-system/01-index.md`,
RiseUp `spec/{05-brand-system.md (§1-3),15-colour-and-materiality.md (§1-4)}`,
`src/styles.css` (tokens `:root`+`.dark`). (* = mandatory-sequence skim.)
