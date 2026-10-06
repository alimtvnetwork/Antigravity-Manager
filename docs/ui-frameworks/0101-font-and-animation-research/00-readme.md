# Font and animation research (2026-10-06)

Research dump for **Antigravity Manager** (Tauri desktop app: React 19 + Tailwind + Vite). Goal: fluid, modern UI/UX with animation done mostly by **CSS** (View Transitions, scroll-driven animations, `@starting-style`, container queries, `clamp()`, `:has()`, anchor positioning, `linear()` easing, discrete transitions) rather than JS-driven animation, plus tooling that automates good UI.

**Context constraints:** the team already knows Tailwind, Svelte, Vue, and React â€” those were not â€œsuggested as things to learnâ€; the stack still *uses* React + Tailwind because that is the product stack.

## Files

| # | File | Topic |
|---|------|--------|
| 00 | [00-readme.md](./00-readme.md) | This index |
| 01 | [01-css-first-animation-and-fluid-design.md](./01-css-first-animation-and-fluid-design.md) | CSS-first libs, Utopia, Open Props, scroll-driven demos |
| 02 | [02-js-animation-libraries.md](./02-js-animation-libraries.md) | Motion, GSAP, AutoAnimate, Anime.js, Lottie/Rive |
| 03 | [03-react-tailwind-ui-frameworks.md](./03-react-tailwind-ui-frameworks.md) | shadcn, Base UI, Radix, Ark, HeroUI, Magic UI, etc. |
| 04 | [04-ui-design-automation.md](./04-ui-design-automation.md) | Tokens, v0, Storybook, Chromatic, shadcn CLI |
| 05 | [05-recommended-stack-and-tauri-webview-support.md](./05-recommended-stack-and-tauri-webview-support.md) | Recommended stack + WebView2 / WebKit matrix |
| 06 | [06-fonts.md](./06-fonts.md) | Variable fonts, Fontsource self-host for Tauri offline |

## Cache location

- Parent: `D:\work\antigravity-manager\cache` (**created** â€” no existing research `cache` / `.cache` / `ai-cache` at repo root; `tmp\cache` exists but is a file-index cache, not a research notes store)
- This folder: `cache\0101-font-and-animation-research` (first research entry; four-digit sequence starting at **0101**)

## Verification notes

- Research date: **2026-10-06** (Asia/Singapore, UTC+8).
- Stars verified via live GitHub HTML pages (WebFetch). GitHub REST API was rate-limited â€” `pushed_at` not bulk-verified.
- npm versions / licenses cited where pages were fetched.
- Bundle sizes for Motion / Anime / GSAP mix official docs and 2026 third-party measurements â€” treat as approximate.
- Do **not** invent numbers; items marked **unverified** where data was missing.