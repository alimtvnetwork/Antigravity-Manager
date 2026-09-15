# Session — Docs Sidebar Tree Diagnostic Logging

**Date:** 2026-04-27 (later same day as quick-jump + sync-banner session)
**Scope:** Frontend-only observability. No spec, no backend, no business logic touched.

---

## User intent (verbatim)

> "Add diagnostic logging for file-tree rendering and refreshing so I can see why spec/ isn't appearing in the editor sidebar."

## Important clarification given to user

The Lovable **IDE file tree** (the sidebar inside the Lovable editor that lists `src/`, `spec/`, etc.) is **not part of this project's code** — it is a Lovable platform UI feature that we cannot instrument. The earlier `spec/` visibility incident was about that IDE tree.

What we **can** instrument is the **in-app docs sidebar** rendered by `src/pages/DocsViewer.tsx` (the spec navigation tree the deployed app shows). That is what this session adds logging to.

---

## What was done

### 1. Diagnostics ring buffer

- New module `src/lib/treeDiagnostics.ts`:
  - Bounded ring buffer (default 200 entries) to avoid memory growth.
  - Categorised log entries prefixed `[tree:<category>]` in console.
  - Exposes `window.__treeDiagnostics` (read API) for ad-hoc devtools inspection.
  - Toggle via `localStorage["lovable.tree-diagnostics.enabled"] = "1"` (also toggleable from the panel).

### 2. Instrumentation points

- `src/hooks/useSpecData.ts` — spec-tree load lifecycle (`tree:load.start`, `tree:load.success`, `tree:load.error`, file count, root folder names).
- `src/components/docs/DocsSidebar.tsx` — render counts, expand/collapse refresh actions, mount/unmount.
- `src/components/SpecTreeNav.tsx` — node renders, expansion-state changes, missing-children edge cases.

### 3. Floating diagnostics panel

- New component `src/components/docs/TreeDiagnosticsPanel.tsx`:
  - Floating button (bottom-right) shown only when diagnostics are enabled.
  - Panel: live log stream, Copy logs, Clear, Disable buttons.
  - Hidden entirely when off — zero visual noise for end users.

---

## Files touched

- created `src/lib/treeDiagnostics.ts`
- created `src/components/docs/TreeDiagnosticsPanel.tsx`
- edited `src/hooks/useSpecData.ts`
- edited `src/components/docs/DocsSidebar.tsx`
- edited `src/components/SpecTreeNav.tsx`
- edited `src/pages/DocsViewer.tsx` (mount the panel)

## Verification

- TypeScript clean.
- Vite build succeeded.
- Diagnostics off by default — production users see no change.

---

## Decisions & rationale

- **Opt-in via localStorage**, not a build flag, so the user can flip it on in the live preview without a rebuild.
- **Ring buffer** (not unbounded array) so a long-running session cannot OOM the tab.
- **Console prefix `[tree:<category>]`** so a devtools filter on `tree:` shows only relevant entries.
- **`window.__treeDiagnostics`** is a read-only snapshot accessor — no foot-guns for accidental mutation.
- **Panel hidden when disabled** — no visual debt for normal users.

## Strictly do NOT regress

- Do not enable diagnostics by default. The toggle must remain opt-in.
- Do not log secrets, file contents, or user input — categories are render/load/state only.
- Do not remove the bound on the ring buffer.
- Do not conflate the Lovable IDE file tree with the in-app docs sidebar in any future explanation.

## Pickup point for next AI

All shipped + verified. If the user reports `spec/` still missing from the **in-app** sidebar, ask them to:
1. Enable diagnostics: `localStorage.setItem("lovable.tree-diagnostics.enabled","1")` and reload.
2. Open the Docs Viewer.
3. Click "Copy logs" in the floating panel and paste back.

If they report it missing from the **Lovable IDE** sidebar (not the app), restate that it is platform UI outside project code and recommend a hard refresh of the IDE.

## Related sessions

- `2026-04-27-docs-viewer-quick-jump-and-sync-banner.md` — earlier same-day work on Cmd/Ctrl+J palette and GithubSyncBanner; same Docs Viewer surface.
