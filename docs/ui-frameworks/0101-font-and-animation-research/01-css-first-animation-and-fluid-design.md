# 01 â€” CSS-first animation and fluid design

Research date: **2026-10-06**. Stars verified via live GitHub HTML (WebFetch). GitHub REST API was rate-limited.

## Libraries and resources

| Item | URL | Stars (verified) | License / activity | Notes |
|------|-----|------------------|--------------------|-------|
| **Open Props** | https://github.com/argyleink/open-props Â· https://open-props.style | **5,531** | MIT Â· npm **v1.7.23** updated **2026-01-31** | CSS custom properties: colors, shadows, easings, animations, sizes; Style Dictionary / Figma token exports. Ideal CSS-first design tokens. |
| **tw-animate-css** | https://github.com/Wombosvideo/tw-animate-css | **806** | MIT Â· **v1.4.0** (2025-09-24) | Tailwind **v4 CSS-first** replacement for `tailwindcss-animate`. Prefer this over the legacy plugin for shadcn / modern Tailwind. |
| **tailwindcss-animate** | https://github.com/jamiebuilds/tailwindcss-animate | **3,014** | MIT | Legacy JS plugin; buggy on TW v4 â€” migrate to `tw-animate-css`. |
| **Animate.css** | https://github.com/animate-css/animate.css Â· https://animate.style | **82,853** | MIT (confirm in repo LICENSE) | Class-based CSS animations; `prefers-reduced-motion` support. Cherry-pick; do not import entire library blindly. Last-push **unverified** via API. |
| **Utopia** | https://utopia.fyi Â· calc: https://github.com/trys/utopia-core (**139â˜…**) Â· PostCSS: https://github.com/trys/postcss-utopia | â€” | utopia-core stars verified | Fluid `clamp()` type/space scales; no framework lock-in. Best â€œautomate fluid designâ€ tool. |
| **Scroll-driven demos** | https://scroll-driven-animations.style (Bramus; Apache-2.0 source noted on site) | â€” | Active demo site | Gold-standard CSS/WAAPI scroll demos + polyfill note. **Weak on WebKit** â€” see `05-recommended-stack-and-tauri-webview-support.md`. |
| **Every Layout** | https://every-layout.dev | Paid book/site | â€” | Intrinsic layout primitives (Stack, Cluster, Gridâ€¦). Not a GitHub-stars product; philosophy fits CSS-first. |
| **Pico CSS** | https://github.com/picocss/pico Â· https://picocss.com | **16,864** | License unclear in fetch summary | Semantic HTML + CSS vars; classless. Reference for CSS-only UI; less ideal if deep in Tailwind. |

## Showcase / learning URLs

- Open Props site: https://open-props.style
- Scroll-driven animations: https://scroll-driven-animations.style
- Utopia type calculator: https://utopia.fyi/type/calculator/
- Utopia space calculator: https://utopia.fyi/space/calculator/
- WebKit View Transitions (Safari 18): https://webkit.org/blog/15865/webkit-features-in-safari-18-0/
- Cross-document View Transitions tips: https://webkit.org/blog/16967/two-lines-of-cross-document-view-transitions-code-you-can-use-on-every-website-today/

## Native CSS features to lean on (no library)

Prefer these over JS animation where the webview supports them:

- **View Transitions API** (`document.startViewTransition`, `view-transition-name`)
- **`@starting-style`** + `transition-behavior: allow-discrete` (enter/exit including `display`)
- **Container queries** (`@container`)
- **Fluid type/space** via `clamp()` / Utopia
- **`:has()`** for parent-aware styling
- **`linear()`** easing
- **Scroll-driven animations** (`animation-timeline: scroll()` / `view()`)
- **CSS anchor positioning**

Support differs sharply by Tauri platform â€” see section 05.

## Practical guidance for Antigravity Manager

1. Put shared tokens (easing, shadow, duration) in CSS custom properties (Open Props or hand-rolled `@theme`).
2. Generate type/space with Utopia `clamp()` into Tailwind `@theme` / CSS variables.
3. Use `tw-animate-css` for accordion/caret and utility animation classes under Tailwind v4.
4. Treat scroll-driven demos as **Windows/WebView2 progressive enhancement**, not a macOS/Linux hard dependency.