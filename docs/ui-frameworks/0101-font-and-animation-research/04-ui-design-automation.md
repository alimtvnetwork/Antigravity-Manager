# 04 â€” UI design automation

Research date: **2026-10-06**. Tools that generate tokens, scaffold components, document UI, and catch visual regressions.

## Tools

| Tool | URL | Stars / note | Role |
|------|-----|--------------|------|
| **Style Dictionary** | https://github.com/style-dictionary/style-dictionary | **4,854** | Design tokens â†’ CSS / JS / platform outputs. Pairs with Open Props token JSON exports. |
| **Tokens Studio** | https://github.com/tokens-studio/figma-plugin Â· https://tokens.studio | **1,618** | Figma tokens â†” Git / W3C DTCG JSON. |
| **shadcn CLI / registry** | https://ui.shadcn.com/docs/cli | â€” (part of shadcn/ui **125,194â˜…** repo) | `npx shadcn init` / `add`; registries distribute components, CSS variables, dependencies. Automates â€œgood defaults.â€ |
| **v0** | https://v0.dev | â€” | AI UI generator; React / Tailwind / shadcn-aware prototypes. Hand-refine for Tauri (no Next.js assumptions in final app). |
| **Storybook** | https://storybook.js.org Â· https://github.com/storybookjs/storybook | **91,205** | Component workshop + docs. |
| **Chromatic** | https://www.chromatic.com | â€” | Visual regression on Storybook PRs (SaaS). |
| **Playwright** | https://playwright.dev | â€” | E2E + screenshot diffs; self-hosted alternative / complement to Chromatic. |
| **CVA** | https://github.com/joe-bell/cva | **6,904** | Type-safe variant class authoring (pairs with shadcn). |

## Suggested automation pipeline

```text
Figma (Tokens Studio)
        â†“ sync JSON
Style Dictionary / Open Props tokens
        â†“ emit CSS vars
Tailwind @theme + Utopia clamp scales
        â†“
shadcn CLI components (owned source)
        â†“
Storybook stories
        â†“
Playwright / Chromatic visual checks
```

Optional: **v0** (or similar) only for **first drafts** â€” not the source of truth for the design system.

## Open Props token exports (related)

From Open Props docs / npm readme:

- https://unpkg.com/open-props/open-props.tokens.json
- https://unpkg.com/open-props/open-props.figma-tokens.sync.json
- https://unpkg.com/open-props/open-props.style-dictionary-tokens.json

## Notes

- Figma-to-code one-shots drift; **tokens + owned components** stay maintainable in a desktop app.
- For Tauri offline use, generated CSS/fonts must be **bundled** (no CDN at runtime) â€” see `06-fonts.md`.