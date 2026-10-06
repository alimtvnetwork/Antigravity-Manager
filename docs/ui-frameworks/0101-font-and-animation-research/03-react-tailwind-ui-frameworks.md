# 03 â€” React + Tailwind UI / UX frameworks

Research date: **2026-10-06**. Stars from live GitHub HTML. Focus: headless vs styled, accessibility, Tailwind v4.

## Comparison table

| Framework | URL | Stars | Headless vs styled | Accessibility | Tailwind v4 |
|-----------|-----|-------|--------------------|---------------|-------------|
| **shadcn/ui** | https://ui.shadcn.com Â· https://github.com/shadcn-ui/ui | **125,194** | **Copy-paste owned code**; styled with Tailwind | Via primitives (Radix / Base UI / etc.) | **Yes** (React 19 + `@theme`) â€” https://ui.shadcn.com/docs/tailwind-v4 |
| **Base UI** | https://base-ui.com Â· https://github.com/mui/base-ui | **11,076** | **Headless** | Strong (team includes Radix alumni) | Bring your own styles; **shadcn default base as of July 2026** â€” https://ui.shadcn.com/docs/changelog/2026-07-base-ui-default |
| **Radix Primitives** | https://www.radix-ui.com Â· https://github.com/radix-ui/primitives | **19,363** | **Headless** | Excellent | Works with TW; still fully supported by shadcn |
| **React Aria / Spectrum** | https://react-spectrum.adobe.com Â· https://github.com/adobe/react-spectrum | **15,916** | Aria = headless; Spectrum = styled Adobe system | **Best-in-class a11y** | Aria pairs with Tailwind; Spectrum is its own design system |
| **Ark UI** | https://ark-ui.com Â· https://github.com/chakra-ui/ark | **5,408** | **Headless** (Zag.js state machines) | Strong | Style yourself or use Park UI |
| **Headless UI** | https://headlessui.com Â· https://github.com/tailwindlabs/headlessui | **28,768** | **Headless** | Good | Tailwind-oriented; **v4 specifics not fully verified** in this pass |
| **Park UI** | https://park-ui.com Â· https://github.com/chakra-ui/park-ui | **2,369** | Styled on **Ark + Panda CSS** (not Tailwind-first) | Via Ark | Tailwind plugin historically TW3 â€” **TW4 unverified** |
| **Mantine** | https://mantine.dev Â· https://github.com/mantinedev/mantine | **31,799** | **Styled** (CSS modules / PostCSS) | Good | Not Tailwind-native |
| **HeroUI** (ex-NextUI) | https://www.heroui.com Â· https://github.com/heroui-inc/heroui | **30,876** | **Styled** + Tailwind | Good | **Yes** from **â‰¥2.8.0** (docs: Tailwind v4 guide on heroui / beta docs) |
| **daisyUI** | https://daisyui.com Â· https://github.com/saadeghi/daisyui | **42,547** | **Styled** Tailwind plugin | Decent; less compositional than headless+shadcn | Check daisyUI docs for current v4 status |
| **Magic UI** | https://magicui.design Â· https://github.com/magicuidesign/magicui | **22,480** | Copy-paste **animated** blocks (often Motion) | Varies by component | Tailwind + Motion |
| **Origin UI â†’ coss** | https://www.originui.com Â· https://github.com/cosscom/coss | **10,651** | Copy-paste / design-system style | Varies | Tailwind; **AGPL on much of coss**; legacy Origin paths reported MIT in secondary sources â€” **verify LICENSE before use** |
| **Aceternity UI** | https://ui.aceternity.com | No single OSS monolith with star count | Copy-paste React + Tailwind + **Motion** showpieces | Marketing-oriented | Free components; Pro templates separately licensed â€” https://ui.aceternity.com/licence |
| **Tremor** | https://www.tremor.so Â· https://github.com/tremorlabs/tremor-npm | **16,484** | Styled charts / dashboards | OK | Tailwind-based |

## Practical pick for Antigravity Manager

1. **Primary:** `shadcn/ui` on **Base UI** (new projects default since July 2026) **or** keep **Radix** if already mid-migration â€” do not churn libraries casually.
2. **Variants:** `cva` (Class Variance Authority) â€” https://github.com/joe-bell/cva (**6,904â˜…**) pairs well with shadcn.
3. **Marketing / flourish only:** Magic UI / Aceternity (Motion-heavy; use sparingly if CSS-first is the goal).
4. **Skip as primary system** if staying Tailwind-owned + CSS-first: Mantine, daisyUI (unless you want opinionated class themes), Park UI (Panda-oriented).

## shadcn Base UI default (July 2026)

- Docs: https://ui.shadcn.com/docs/changelog/2026-07-base-ui-default
- New `npx shadcn init` defaults to Base UI; Radix via `-b radix`.
- Both bases remain supported; migrate only when ready (skill-based migration available per docs).