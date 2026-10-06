# 02 â€” JS animation libraries

Research date: **2026-10-06**. Ranked for a **CSS-first** Tauri app: prefer CSS â†’ WAAPI-leaning â†’ heavy JS.

## Ranking

| Rank | Library | URL | Stars | Bundle / perf model | License / activity |
|------|---------|-----|-------|---------------------|-------------------|
| **1 (default add)** | **AutoAnimate** | https://github.com/formkit/auto-animate Â· https://auto-animate.formkit.com | **13,926** | **&lt;3 KB**; automatic DOM enter/move/leave; almost no API | MIT Â· npm **@formkit/auto-animate@0.10.0** updated **2026-07-10** |
| **2 (UI / React)** | **Motion** (formerly Framer Motion) | https://github.com/motiondivision/motion Â· https://motion.dev | **33,844** | Hybrid: **WAAPI / hardware-accelerated** path + JS springs/layout. Official comparison sizes: `animate` mini **~2.6 KB**, full animate **~18 KB**; React entry adds more (third-party 2026 figures ~15â€“44 KB depending on import). Prefer `LazyMotion` / `motion/mini`. | MIT Â· npm **motion@14.0.0** updated **2026-10-02** |
| **3 (timelines / SVG)** | **Anime.js v4** | https://github.com/juliangarnier/anime Â· https://animejs.com | **73,381** | Modular ESM; **`waapi` module ~3 KB**; full `animate` ~13 KB (third-party 2026 writeups). Timelines, SVG, scroll, layout. | MIT Â· GitHub release **v4.5.0** dated **22 Jun** (year not on release page header; recent activity verified via release list) |
| **4 (pro choreography)** | **GSAP** | https://github.com/greensock/GSAP Â· https://gsap.com | **28,837** | Own ticker (not primarily WAAPI). Core ~23â€“28 KB; ScrollTrigger adds more. **All plugins free** since Webflow acquisition era. | **Standard â€œno chargeâ€ license** (not MIT) â€” restricts tools competing with Webflow; Webflow can terminate. npm **gsap@3.15.0** updated **2026-04-13** |
| **5** | **React Spring** | https://github.com/pmndrs/react-spring | **29,165** | Spring physics, React-first; JS-driven. Heavier mental model for UI chrome. | MIT Â· last-push **unverified** |
| **Special** | **Lottie** | https://github.com/airbnb/lottie-web | **32,145** | After Effects â†’ JSON; canvas/SVG runtime. Designer-driven, not CSS. | Apache-2.0 (typical; confirm LICENSE) |
| **Special** | **Rive** | https://github.com/rive-app/rive-react | **1,168** | Interactive state-machine vectors; WASM runtime. | Check Rive license for commercial use |

## Perf model cheat sheet

1. **CSS / compositor** (`transform`, `opacity`, View Transitions, `@starting-style`)
2. **WAAPI-leaning** â€” AutoAnimate, Motion mini, Anime `waapi`
3. **Hybrid UI** â€” Motion full (`motion/react`) for layout / shared element / gestures
4. **Timeline engines** â€” Anime core, GSAP (max control; larger; license nuance)
5. **Asset runtimes** â€” Lottie / Rive

## Recommendation for this app

- **Always:** CSS transitions + `tw-animate-css` + View Transitions where supported.
- **Thin JS:** `@formkit/auto-animate` for lists, trees, panel insert/remove.
- **Reach for Motion** only when you need layout animations, drag, or AnimatePresence-style exit that CSS cannot cover cleanly.
- **Avoid by default:** full GSAP/Lottie/Rive unless a specific feature needs timelines, AE exports, or Rive state machines.

## Official comparison references

- Motion vs GSAP feature comparison: https://motion.dev/docs/feature-comparison
- GSAP homepage (free announcement): https://gsap.com/