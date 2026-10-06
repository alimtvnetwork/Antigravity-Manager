# 05 â€” Recommended stack and Tauri webview CSS support

Research date: **2026-10-06**.

## Recommended stack (CSS-first, React + Tailwind, Tauri)

1. **UI primitives:** [shadcn/ui](https://ui.shadcn.com) on **Base UI** (or Radix if already invested).
2. **Animation utilities:** [tw-animate-css](https://github.com/Wombosvideo/tw-animate-css) (Tailwind v4).
3. **Fluid type/space:** [Utopia](https://utopia.fyi) `clamp()` scales in CSS / `@theme`.
4. **Optional token layer:** [Open Props](https://open-props.style) or Style Dictionary from Tokens Studio.
5. **Thin JS motion:** [@formkit/auto-animate](https://auto-animate.formkit.com) for lists/panels; [Motion](https://motion.dev) (`motion/react`) only for layout / gestures / shared-element needs.
6. **Fonts:** self-host via Fontsource â€” see `06-fonts.md`.
7. **Avoid by default:** full GSAP / Lottie / Rive; daisyUI / HeroUI / Mantine as *primary* systems if you want copy-owned Tailwind + CSS-first.

### Minimal â€œautomate good UIâ€ workflow

Tokens (Figma Tokens Studio â†’ Style Dictionary / Open Props) â†’ Tailwind `@theme` + Utopia clamps â†’ shadcn CLI â†’ Storybook + Playwright screenshots â†’ optional v0 for drafts only.

---

## Tauri webview engines

Authoritative Tauri reference: https://v2.tauri.app/reference/webview-versions/ (page noted last updated **2026-05-17**).

| Platform | Engine | Notes |
|----------|--------|-------|
| **Windows** | **WebView2 (Chromium / Edge)** | Self-updating; relatively recent Chromium on supported Windows. Best modern-CSS story. |
| **macOS** | **WKWebView = system WebKit** | Tracks Safari via OS updates. Unsupported macOS versions **freeze** WebKit. |
| **Linux** | **WebKitGTK** (`webkit2gtk`) | Distro packages vary widely; often **years behind** Safari. Lowest common denominator. |

---

## Modern CSS support matrix (CanIUse / WebKit, fetched 2026-10-06)

| Feature | Safari / WebKit | Windows WebView2 | macOS Tauri | Linux WebKitGTK |
|---------|-----------------|------------------|-------------|-----------------|
| Container queries | **Safari 16+** â€” https://caniuse.com/css-container-queries | Yes | Yes on modern macOS | Likely if WebKit â‰¥ ~16-era; verify distro |
| `linear()` easing | **Safari 17.2+** â€” https://caniuse.com/mdn-css_types_easing-function_linear-function | Yes | Yes if webview â‰¥ 17.2 | Unreliable â€” `@supports` |
| `@starting-style` | **Safari 17.5+** (display anim synergy stronger in **18**) â€” https://caniuse.com/mdn-css_at-rules_starting-style | Yes | Yes on Sequoia-era+ | Unreliable |
| View Transitions (same-document) | **Safari 18.0+** â€” https://caniuse.com/view-transitions | Yes (Chromium 111+) | Yes if OS WebKit â‰¥ Safari 18 | Often **no** |
| Scroll-driven (`animation-timeline: scroll()`) | **Safari 26.0+** (not 18.x) â€” https://caniuse.com/mdn-css_properties_animation-timeline_scroll | Yes (Chromium 115+) | Only very new Safari/macOS | Likely **no** |
| Anchor positioning | **Safari 26 partial / 27 fuller**; not in 18.x â€” https://caniuse.com/css-anchor-positioning | Partial (Chromium) | New Safari only | Likely **no** |
| `:has()`, `clamp()` | Broad modern support | Yes | Yes | Usually yes |

Safari version numbering on CanIUse now shows **26 / 27** after the 18.x line â€” use CanIUse / MDN for the target OS, not assumptions from Chromium alone.

WebKit Safari 18 feature write-up (View Transitions, `@starting-style` + `display` transitions, style queries, etc.): https://webkit.org/blog/15865/webkit-features-in-safari-18-0/

---

## Implications for Antigravity Manager

- Design **progressive enhancement**: animations degrade to opacity/instant on older WebKitGTK.
- Put **scroll-driven animations** and **anchor positioning** behind `@supports` (or treat as Windows polish).
- Cross-platform â€œfluidâ€ feel should rely on: **View Transitions + `@starting-style` + tw-animate-css + AutoAnimate + Utopia clamps + solid fonts**.
- Always test installers on **Windows WebView2**, **current macOS WKWebView**, and **target Linux distrosâ€™ WebKitGTK**.