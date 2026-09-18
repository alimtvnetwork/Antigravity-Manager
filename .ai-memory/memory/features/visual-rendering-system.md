# Memory: features/visual-rendering-system

**Updated:** 2026-04-03
**Version:** 1.0.0
**Status:** Active

---

## Overview

Complete reference for how the docs viewer renders markdown content visually. Covers tree structures, code blocks, heading animations, inline elements, and the TOC scroll-spy system. Full specification at `02-spec/08-docs-viewer-ui/02-features/07-visual-rendering-guide.md`.

---

## Key Rendering Rules

### Tree / Folder Structures

- Auto-detected via box-drawing chars (`├└│─`), trailing `/` (dirs), or file extensions
- Directories prefixed with 📁, files with 📄
- Guides rendered at 50% muted opacity; directories bold white; files at 85% opacity
- Ellipsis (`...`) rendered in accent pink; comments in italic muted
- Neutral color scheme — no red/pink syntax colors on tree content

### Code Blocks

- Always dark background (`hsl(220, 14%, 11%)`) regardless of app theme
- Per-language HSL accent color for badge, dot glow, hover shadow
- Header: language badge + line count + font controls (A-/A/A+) + copy + download + fullscreen
- Line interactions: click-to-pin, Shift+click for ranges, drag across line numbers
- Syntax tokens: keywords=purple, strings=pink, numbers=amber, comments=italic gray
- Font: Ubuntu Mono / JetBrains Mono, default 18px, range 12–32px

### Heading Animations

- H1/H2: gradient text fill (purple→pink), `brightness(1.2) saturate(1.1)` on hover
- H3: left border slide (3px border, +0.2rem padding-left on hover, color shifts to primary)
- H4: muted→foreground color transition on hover

### Inline Elements

- Links: purple text, sweep underline (right-to-left `::after` animation), shifts to pink on hover
- Inline code: pink text on muted background, lifts 1px with glow ring on hover
- Paragraphs: subtle primary-tinted background on hover

### Tables

- Rounded wrapper with subtle shadow, uppercase header labels
- Row hover: background highlight + inset 3px left bar in primary color

### Checklists

- Dedicated container with header + copy button
- Checked: green gradient checkbox; Unchecked: bordered empty box
- Copy preserves raw markdown syntax, not HTML

### TOC & Scroll Spy

- Sticky right sidebar (208px, hidden below xl breakpoint)
- IntersectionObserver with `rootMargin: "-10% 0px -80% 0px"`
- Active item: primary color text + 2px left border
- Auto-scrolls active item into view via `scrollIntoView({ behavior: "smooth", block: "nearest" })`

### Sidebar Navigation & Search

- File tree: recursive `SpecTreeNav` using shadcn `SidebarMenu`/`SidebarMenuSub`
- Folder icons: `Folder` (muted) when closed, `FolderOpen` (primary) when open; `ChevronRight` rotates 90° on expand
- File icons: `FileText` (muted); active file uses shadcn `isActive` state
- Auto-expand: top-level folders always open; folders containing active file auto-expand
- Search: `useSpecSearch` filters by name + content (case-insensitive), max 20 results
- Results show file name (bold) + path (xs muted) with `FileText` icon
- File selection clears search query; only files are selectable (folders toggle expand)
- Sidebar collapse: `collapsible="icon"` hides branding/search via `group-data-[collapsible=icon]:hidden`

### Split-View Editor

- Three view modes: Preview (P), Edit (E), Split (S)
- Monaco editor with JetBrains Mono font, minimap, bracket colorization, theme-aware (vs/vs-dark)
- Split view: draggable divider (6px, col-resize cursor), ratio clamped 20%–80%, default 50%
- Divider hover: background shifts to primary/0.2, grip line grows from 32px to 48px
- Live preview re-renders on every keystroke; no TOC in split mode
- Reading progress bar: gradient (primary→accent), calculated from scroll position

---

## Cross-References

| Reference | Location |
|-----------|----------|
| Full Spec | `02-spec/08-docs-viewer-ui/02-features/07-visual-rendering-guide.md` |
| Theme Spec | `02-spec/08-docs-viewer-ui/02-features/06-ui-theme-animations.md` |
| CSS Source | `src/index.css` |
| Monaco Editor | `src/components/MonacoMarkdownEditor.tsx` |
| Split View | `src/pages/DocsViewerComponents.tsx` |
| Highlighter | `src/components/markdown/highlighter.ts` |
| Code Builder | `src/components/markdown/codeBlockBuilder.ts` |
| Constants | `src/components/markdown/constants.ts` |
| Highlighter | `src/components/markdown/highlighter.ts` |
| Code Builder | `src/components/markdown/codeBlockBuilder.ts` |
| Constants | `src/components/markdown/constants.ts` |
