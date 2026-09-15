# Session — Docs Viewer Quick-Jump + GitHub Sync Banner

**Date:** 2026-04-27
**Scope:** Frontend-only enhancements to the Docs Viewer (`src/pages/DocsViewer.tsx`) plus repo cleanup. No backend, no spec changes.

---

## What was done

### 1. Command Palette + prominent "Spec Overview" jump (Cmd/Ctrl+J)

- New helper `src/components/docs/specOverviewJump.ts` — exports `findSpecOverviewFile(allFiles)` and `SPEC_OVERVIEW_PATH = "02-spec/00-overview.md"`. Resolves directly from the flat `allFiles` list (which `useSpecData` derives from `src/data/specTree.json`), so it works **even when the sidebar tree is stale or collapsed**. Falls back to a name-based match if the exact path is missing.
- New component `src/components/docs/CommandPalette.tsx` using shadcn `Command`/`CommandDialog`. Top-pinned "Open Spec Overview" item, then per-folder `00-overview.md` quick links, then a fuzzy-searchable file list (capped at 30 for perf).
- New global keybind: `Cmd+J` / `Ctrl+J` opens the palette (`useCommandPaletteShortcut`). Added `KeyboardKeyType.J = "j"` to `src/constants/enums.ts`.
- Prominent header button `SpecOverviewButton` added in `src/pages/DocsViewerComponents.tsx` next to the existing search bar — primary-tinted, displays platform-aware kbd hint (`⌘J` / `Ctrl+J`).
- Threaded an optional `onJumpToOverview?: () => void` callback through `ContentPanelInput`, `DocsHeader`, `ContentPanelMain`, `DocsContentPanel`, `DocsContent`. Wired in `DocsViewer.tsx`'s `useDocsViewerState` (`jumpToOverview` resolves overview directly, falls back to opening the palette).

### 2. GitHub Sync Banner

- New component `src/components/docs/GithubSyncBanner.tsx`, mounted at the very top of `DocsViewerLayout` (above the sidebar/header).
- Reads build-baked `version.json` (`git.branch`, `git.shortSha`, `git.sha`, `updated`) — `tsconfig.app.json` already has `resolveJsonModule: true`.
- Surfaces: `Synced with GitHub` badge, branch + short SHA, repo `updated` date, session "Loaded" timestamp.
- **Reload** button (hard `window.location.reload()` after 100 ms).
- **Dismiss (×)** persists the dismissed SHA in `localStorage` under key `lovable.github-sync-banner.dismissed-sha`. Banner reappears automatically on the next synced commit (different SHA).
- Uses semantic tokens only (`text-success`, `bg-success/10`, `border-border`, `text-foreground`, etc.).

### 3. Repo cleanup

- Removed `/dev-server/.gitmap/` (3 files: `release/latest.json`, `release/v3.85.0.json`, `release/v4.24.0.json`). No source code referenced it; only unrelated text mentions in spec markdown remain.

### 4. Spec folder visibility incident

- User reported `spec/` folder missing from the file tree. Verified via `ls -la spec/` that all 29 entries (modules `01-spec-authoring-guide` … `24-app-ui-design-system`) are intact and bidirectionally synced with GitHub. Diagnosed as a UI/file-tree caching artifact — recommended refresh / hard reload / `Cmd+P` file search.

---

## Files touched

- created `src/components/docs/specOverviewJump.ts`
- created `src/components/docs/CommandPalette.tsx`
- created `src/components/docs/GithubSyncBanner.tsx`
- edited `src/constants/enums.ts` (added `J: "j"`)
- edited `src/pages/DocsViewer.tsx` (palette + banner wiring + `jumpToOverview` callback)
- edited `src/pages/DocsViewerHelpers.ts` (added `onJumpToOverview?` to `ContentPanelInput`)
- edited `src/pages/DocsViewerComponents.tsx` (added `SpecOverviewButton`, threaded `onJumpToOverview` through `DocsHeader` / `ContentPanelMain` / `DocsContentPanel` / `DocsContent`)
- removed `/dev-server/.gitmap/` (whole directory)

## Verification

- `bunx tsc --noEmit -p tsconfig.app.json` → clean (0 errors).
- `bunx vite build` → success, 1830 modules transformed.

---

## Decisions & rationale

- **Why a dedicated `findSpecOverviewFile` helper instead of `allFiles.find(f => f.path === ...)` inline?** The user explicitly framed this as resilience to stale sidebar state. Centralising the lookup with a name-based fallback makes the "even if the tree is stale" guarantee explicit and unit-testable later.
- **Why `Cmd+J` and not extending `Cmd+K`?** `Cmd+K` already opens the existing search dialog (see `useSearchShortcut`); leaving it untouched preserves muscle memory while giving the palette its own slot.
- **Why per-SHA dismissal for the banner?** The banner's value is highest when something *new* arrives. Permanent dismissal would defeat the purpose; per-SHA means it auto-reappears on the next sync.
- **`version.json` is the canonical sync ground truth** (already maintained by `scripts/sync-version.mjs` per existing memory). Reusing it avoided introducing a new metadata file.

## Strictly do NOT regress

- Do not bind `Cmd+J` to anything else. Do not change `Cmd+K` semantics.
- Do not remove the per-SHA dismiss key — making the banner permanently dismissable defeats its purpose.
- Keep `findSpecOverviewFile` independent of the sidebar tree; the name-based fallback is intentional.

## Pickup point for next AI

All changes shipped + verified. No pending follow-ups from this session. Suggested follow-on (if desired): Playwright spec for `Cmd+J → Open Spec Overview → 02-spec/00-overview.md becomes active`.
