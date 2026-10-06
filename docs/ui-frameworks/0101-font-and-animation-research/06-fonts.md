# 06 â€” Fonts for React + Tailwind + Tauri

Research date: **2026-10-06**. URLs verified via WebFetch / npm where noted. No invented download counts beyond npm pages fetched.

## Goals for a desktop Tauri app

- **Self-host fonts** (offline / no CDN dependency at runtime).
- Prefer **variable fonts** where available (fewer files, continuous weight axis).
- Pair with **Utopia `clamp()`** fluid type â€” https://utopia.fyi
- Set sensible **`font-display`** (Fontsource CSS typically uses a sensible default; confirm in generated CSS).
- Bundle through **Vite** so fonts ship inside the app webview assets.

## Fontsource (primary recommendation)

- Site: https://fontsource.org/ (browse 2,100+ families; variable filter available)
- Self-host via npm packages; works with Vite / React.
- Docs pattern: install package â†’ `import "@fontsource/<family>/400.css"` (or variable package) â†’ set `font-family` in CSS / Tailwind theme.

### Recommended UI families (verified pages)

| Role | Family | Fontsource page | npm package (verified) | License (npm) | Notes |
|------|--------|-----------------|------------------------|---------------|-------|
| UI sans (default) | **Inter** | https://fontsource.org/fonts/inter | [@fontsource/inter](https://www.npmjs.com/package/@fontsource/inter) **5.3.0** (updated **2026-07-19**) | OFL-1.1 | Variable on Fontsource; weights 100â€“900 |
| UI sans (modern / Vercel aesthetic) | **Geist Sans** | https://fontsource.org/fonts/geist-sans | [@fontsource/geist-sans](https://www.npmjs.com/package/@fontsource/geist-sans) **5.3.0** (updated **2026-07-19**) | OFL-1.1 | Latin subset on package page |
| UI sans (IBM) | **IBM Plex Sans** | https://fontsource.org/fonts/ibm-plex-sans | [@fontsource/ibm-plex-sans](https://www.npmjs.com/package/@fontsource/ibm-plex-sans) **5.3.0** (updated **2026-07-19**) | OFL-1.1 | Strong for dense admin UIs |
| Code / mono | **JetBrains Mono** | https://fontsource.org/fonts/jetbrains-mono | Use [@fontsource/jetbrains-mono](https://www.npmjs.com/package/@fontsource/jetbrains-mono) or **@fontsource-variable/jetbrains-mono** (listed on npm search) | OFL-1.1 (typical Fontsource) | Variable listed on Fontsource |
| Code / mono alt | **Geist Mono** | https://fontsource.org/fonts/geist-mono | [@fontsource/geist-mono](https://www.npmjs.com/package/@fontsource/geist-mono) **5.3.0** (updated **2026-07-19**) | OFL-1.1 | Pairs with Geist Sans |

Also visible as popular on https://fontsource.org/: Inter Variable, Geist Variable, JetBrains Mono Variable, IBM Plex Sans Variable, IBM Plex Mono.

### Variable vs static packages

- Prefer **`@fontsource-variable/<name>`** when you need one file covering a weight axis (smaller runtime surface for many weights).
- Static `@fontsource/<name>/<weight>.css` imports let you ship **only** the weights you use (e.g. 400 + 500 + 600 + 700).

Exact variable package names: confirm on the Fontsource font page â€œGet fontâ€ / npm before locking `package.json`.

## Example install (illustrative)

```bash
npm install @fontsource-variable/inter @fontsource/jetbrains-mono
```

```ts
// e.g. src/main.tsx
import "@fontsource-variable/inter"
import "@fontsource/jetbrains-mono/400.css"
import "@fontsource/jetbrains-mono/500.css"
```

```css
/* Tailwind v4 @theme or global CSS */
:root {
  --font-sans: "Inter Variable", "Inter", system-ui, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
}
```

Wire `--font-sans` / `--font-mono` into Tailwind `font-sans` / `font-mono` theme tokens.

## Fluid type with Utopia

- Generators: https://utopia.fyi/type/calculator/ Â· https://utopia.fyi/space/calculator/
- Core calc library: https://github.com/trys/utopia-core (**139â˜…** verified)
- Emit `--step-*` clamps into `:root` / `@theme`, then use those tokens for `text-*` sizes instead of fixed `px` scales only.

## `font-display` and loading

- For desktop apps, **FOIT is less catastrophic** than on the web, but still avoid invisible text flashes on first paint.
- Prefer `font-display: swap` or `optional` depending on brand strictness; inspect Fontsourceâ€™s emitted `@font-face` rules.
- Preload only the **primary** UI face if needed (`<link rel="preload" as="font" â€¦ crossorigin>` in `index.html`) â€” usually unnecessary if fonts are local and small.

## Tauri offline bundling checklist

1. **Do not** rely on Google Fonts CDN or `fonts.bunny.net` at runtime.
2. Install Fontsource (or commit `woff2` under `src/assets/fonts` + manual `@font-face`).
3. Confirm Vite emits font files into `dist/assets` and Tauri packages them.
4. Smoke-test **airplane mode** / blocked network: UI text must still render.
5. Subset if binary size matters (latin-only packages; avoid unused scripts).

## Pairing with the rest of this research

- Tokens / type scale: Utopia + Open Props (see `01-â€¦` and `04-â€¦`).
- Component text styles: shadcn / CVA variants (see `03-â€¦`).
- Keep motion on compositor-friendly properties; font swaps should not be animated via JS weight thrashing.