# App UI — Design System

> **/goal** Master and enforce the architectural standards, specifications, and CI/CD validation rules for 24 App Ui Design System.
> **/learn** Read the sequentially ordered specification files in this directory, follow the actionable CI/CD checklist, and apply mandatory rules before generating code.

## 🎯 Actionable CI/CD & Agent Checklist

- [ ] `/goal` Read and understand all numbered specifications under `24-app-ui-design-system/`.
- [ ] `/learn` Adhere strictly to `.ai-memory/folder-structure.md` and `.ai-memory/strictly-avoid.md`.
- [ ] `/goal` Verify zero explicit `true` boolean evaluations and no mixed-polarity conditionals.
- [ ] `/learn` Run all local verification linters via `python 03-ai-scripts/06-cicd-local-runner.py`.

. **CRITICAL AI INSTRUCTION:** This `01-index.md` file is the primary entry point for this directory. AI agents MUST read this file first before exploring other files in this folder.

**Version:** 3.3.0
**Updated:** 2026-09-10
**AI Confidence:** Verified
**Ambiguity:** None

---

## Keywords

`app-ui` · `app-design-system` · `theming` · `components` · `layout` · `tailwind` · `daisyui`

---

## Scoring

| Criterion | Status |
|---|---|
| `01-index.md` present | ✅ |
| AI Confidence assigned | ✅ |
| Ambiguity assigned | ✅ |
| Keywords present | ✅ |
| Scoring table present | ✅ |

---

## Purpose

Application-specific UI and design-system specifications for Antigravity-Manager. Governs component hierarchy, theme tokens, TailwindCSS 3.4 & DaisyUI 5 conventions, window drag boundaries, responsive layouts, and cross-platform desktop integration.

---

## Document Inventory

| # | Specification / Module | Purpose |
|---|---|---|
| 01 | [Design Tokens & Themes](#1-design-tokens--theming-palettes) | Tailwind 3.4 & DaisyUI 5 color tokens, dark/light/dim palettes, background sync |
| 02 | [Component & Layout Standards](#2-component-and-layout-standards) | Desktop shell, top header window drag region, cards, modals, form controls |
| 03 | [Responsive Container Queries](#3-responsive-layout--container-queries) | `@container` query integration and desktop breakpoint behaviors |
| 04 | [Accessibility & Interaction](#4-motion-and-interaction-standards) | View Transition API animations, Linux fallback, and focus states |

---

## 1. Design Tokens & Theming Palettes

The UI styling system is built on **TailwindCSS 3.4** and **DaisyUI 5**, configured in `tailwind.config.js`.

### 1.1 Semantic Color Palettes

| Token | Light Theme (`light`) | Dark Theme (`dark`) | Role / Usage |
|---|---|---|---|
| `primary` | `#3b82f6` (Blue-500) | `#3b82f6` (Blue-500) | Primary actions, active navigation tabs, buttons |
| `secondary` | `#64748b` (Slate-500) | `#94a3b8` (Slate-400) | Secondary badges, descriptive subtitles, icons |
| `accent` | `#10b981` (Emerald-500) | `#10b981` (Emerald-500) | Success highlights, active proxy status pills |
| `neutral` | `#1f2937` (Gray-800) | `#1f2937` (Gray-800) | Neutral card surfaces and dropdown headers |
| `base-100` | `#ffffff` (White) | `#0f172a` (Slate-900) | Primary card and modal background surfaces |
| `base-200` | `#f8fafc` (Slate-50) | `#1e293b` (Slate-800) | Secondary nested cards and table headers |
| `base-300` | `#f1f5f9` (Slate-100) | `#334155` (Slate-700) | Main viewport canvas background |
| `info` | `#0ea5e9` (Sky-500) | `#0ea5e9` (Sky-500) | Information banners and quota badges |
| `success` | `#10b981` (Emerald-500) | `#10b981` (Emerald-500) | Operation confirmations and healthy endpoints |
| `warning` | `#f59e0b` (Amber-500) | `#f59e0b` (Amber-500) | Quota threshold alerts and rate limit warnings |
| `error` | `#ef4444` (Red-500) | `#ef4444` (Red-500) | Circuit breaker trips and IPC failure toasts |

### 1.2 OS Window & Canvas Background Synchronization (`ThemeManager.tsx`)
Desktop window framing synchronizes theme states across Webview and host OS:
- **Light Canvas Background:** `#FAFBFC` (HTML root and window background).
- **Dark Canvas Background:** `#1d232a` (HTML root and window background).
- **Native Titlebar Sync:** On Windows/macOS, invokes `set_window_theme` and `getCurrentWindow().setBackgroundColor(...)`.
- **System Mode:** Follows host OS via `window.matchMedia('(prefers-color-scheme: dark)')` when theme is set to `system`.

---

## 2. Component and Layout Standards

### 2.1 Desktop Application Shell (`src/components/layout/Layout.tsx`)
- **Full View Mode:** Fixed viewport height (`h-screen flex flex-col bg-[#FAFBFC] dark:bg-base-300`).
- **Window Drag Zone:** Fixed 36px top drag band (`h-9`, `z-index: 9999`, `data-tauri-drag-region`) with JavaScript fallback `getCurrentWindow().startDragging()` on mouse down.
- **Top Sticky Header:** `Navbar.tsx` positioned with `sticky top-0 z-50 pt-9`, eliminating side navigation bars.
- **Content Area:** `<main className="flex-1 overflow-hidden flex flex-col relative"><Outlet /></main>`.

### 2.2 Compact Floating Widget (`src/components/layout/MiniView.tsx`)
- Specialized borderless mode activated via `useViewStore.isMiniView`.
- Renders minimal status badges, active account switcher, and live token throughput.

### 2.3 Card & Container Conventions
- **Collapsible Cards (`CollapsibleCard`):** `bg-white dark:bg-base-100 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700/50`. Includes collapsible accordion headers and toggle switches.
- **Dialogs & Modals (`ModalDialog`):** Centered viewport overlay with backdrop blur, focus trapping, and ESC keyboard handling.
- **Form Controls:** DaisyUI toggle switches (`toggle toggle-sm checked:bg-blue-500`), sliders (`DebouncedSlider`), and grouped selectors (`GroupedSelect`).

---

## 3. Responsive Layout & Container Queries

- Uses `@tailwindcss/container-queries` plugin.
- **Navbar Responsive Logo:** Container query `@container/logo basis-[200px] shrink min-w-0` dynamically scales logo text and version badges.
- **Responsive Navigation Links (`NavDropdowns.tsx`):** High-priority items stay pinned to the header; medium and low-priority routes collapse into categorized dropdown menus on narrow screens.

---

## 4. Motion and Interaction Standards

- **Theme Transition Animation:** Uses the modern View Transition API (`document.startViewTransition`) with radial clip-path animation expanding from the toggle button's click coordinate.
- **Linux Safety Fallback:** Circumvents View Transitions and window background transparency on Linux (`isLinux()`) to prevent WebKitGTK / softbuffer crashes.
- **Micro-Interactions:** 200ms ease-in-out transitions on button hover, border colors, and card elevation.

---

## Cross-References

- [Frontend UI & State Management](../21-app/05-frontend-ui-and-state-management.md) — Frontend architecture & Zustand stores
- [API Contracts & IPC Registry](../21-app/06-api-contracts-and-ipc-registry.md) — Native Tauri IPC commands
- [Consolidated Design System](../17-consolidated-guidelines/10-design-system.md) — Meta-repository design guidelines

---

## Verification & Acceptance Criteria

### AC-DS-001: DaisyUI Theme Token Resolution
- **Executable Test:** `tests::test_daisyui_theme_token_resolution`
- **Given** TailwindCSS 3.4 and DaisyUI 5 semantic color tokens defined in `tailwind.config.js` and `ThemeManager.tsx`.
- **When** Frontend assets and styles are compiled and validated via verification commands.
- **Then** All components consume semantic theme tokens (`primary`, `secondary`, `accent`, `neutral`, `base-100/200/300`); untokenized raw colors are rejected and color schemes resolve across light and dark themes.

### AC-DS-002: Titlebar Drag Region & Layout Attributes
- **Executable Test:** `tests::test_titlebar_drag_region_attributes`
- **Given** The desktop shell component hierarchy in `src/components/layout/Layout.tsx` and `Navbar.tsx`.
- **When** RENDER DOM tree mounts in full view or compact mode.
- **Then** The top 36px drag band contains `data-tauri-drag-region`, window drag listeners attach correctly, and fixed layout constraints prevent viewport scrolling outside the content area.

**Verification command:**

```bash
npm run lint && tsc --noEmit && npm run build
```

**Expected:** exit 0. Any non-zero exit is a hard fail and blocks merge.

_Verification section last updated: 2026-09-10_
