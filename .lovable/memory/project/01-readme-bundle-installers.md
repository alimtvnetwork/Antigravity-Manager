---
name: README Bundle Installers + GIFs
description: README rewritten to 243 lines in v3.56.0 (Phases A–E shipped). Auto-stamped via scripts/sync-readme-stats.mjs. Long-form moved to docs/{architecture,principles,author}.md. Bundle matrix + AI Agents section + verify/uninstall on root.
type: feature
---

## Status (as of v3.56.0, 2026-04-22)

**Shipped in v3.55.0:**
- `public/images/coding-guidelines-walkthrough.gif` — 7-slide editorial deck (DejaVuSans + LiberationMono, 960×540, 145 KB, 34s loop). Embedded at top of `readme.md` under the Spec Viewer screenshot.
- `public/images/install-flow.gif` — terminal install demo (144 KB). Embedded inside the new **Bundle Installers** section.
- `readme.md` **Bundle Installers** section — 7-row matrix with bash + PowerShell one-liners, "pick a bundle by goal" table, traits list. Links to `bundles.json`.
- TOC entry added.
- All 7 bundles verified present: error-manage, splitdb, slides, linters, cli, wp, consolidated. Each has `<bundle>-install.sh` + `<bundle>-install.ps1` + spec folders + `bundles.json` registration.
- New consolidated files registered in `02-spec/17-consolidated-guidelines/00-overview.md`: 28 (distribution-and-runner), 29 (blind-ai-audit-v3), 30 (readme improvement suggestions).

**Shipped in v3.56.0 (Phases A–E):**
- `readme.md` rewritten 1397 → **243 lines** (well under 400-line target). Hand-edited content moved into `docs/architecture.md`, `docs/principles.md`, `docs/author.md`.
- Auto-stamp script: `scripts/sync-readme-stats.mjs` rewrites `<!-- STAMP:KEY -->...<!-- /STAMP:KEY -->` markers from `version.json`. Wired into `npm run sync` and `prebuild`.
- Stamp keys: VERSION, UPDATED, FILES, FOLDERS, LINES, BADGES.
- Badges: shields.io with `--` dash escape, AI-Ready uses `%20` space (was malformed before).
- `## For AI Agents` section lists canonical entry points (llm.md, bundles.json, version.json, condensed master, anti-hallucination rules, consolidated overview, .lovable/memory + prompts).
- `Verify your install` section with `sha256sum` example + Windows `Get-FileHash`.
- `Uninstall` section explaining manifest-free removal via `bundles.json`.
- Phase E a11y: all images use `<img>` with `width="960"` and descriptive alt text; centred via `<p align="center">`.

**Known pre-existing issues (not regressions):**
- `02-spec/17-consolidated-guidelines/00-overview.md` references `31-distribution-and-runner.md` and `32-blind-ai-audit-v3.md` — neither file exists yet. These were registered as planned content from prior phases but were never authored. Either create the files or remove the index entries.

## How to regenerate the GIFs

Both render scripts are in `/tmp/gifgen/`. They use:
- Slides: `DejaVuSans.ttf` (the only sans font reliably available on the sandbox; LiberationSans is missing)
- Terminal: `LiberationMono-{Regular,Bold}.ttf`

If a render shows hollow rectangles instead of letters, the font path resolution fell through to the wrong fallback — re-confirm with `fc-match -f "%{file}\n" sans-serif`.

## Why this matters

The README is the single first impression for any human or AI landing on the repo. The bundle matrix turns "what do I install?" from a wiki-crawl into a single table lookup. The walkthrough GIF makes the value of the guidelines visceral in 8 seconds (5 principles → before/after refactor → 7 bundles → install demo).
