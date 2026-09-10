# Consolidated: App Design System & Desktop UI

**Version:** 4.7.0
**Updated:** 2026-09-10
**Source Spec:** [`02-spec/21-app/05-frontend-ui-and-state-management.md`](../21-app/05-frontend-ui-and-state-management.md)

---

## 1. Purpose & Architecture

This is the **standalone consolidated design system** for **Antigravity-Manager** v4.7.0. It defines desktop shell layout, component tokens, theme configurations, and visual standards for the React 19 frontend powered by **TailwindCSS 3.4** and **DaisyUI 5**.

---

## 2. Desktop App Shell Layout Architecture

Unlike traditional web applications with vertical sidebars, Antigravity-Manager employs a **desktop top navigation bar** layout with an integrated native window drag region and support for an ultra-compact floating mini-view.

### Shell Hierarchy

```
┌────────────────────────────────────────────────────────────────────────┐
│ Native Tauri Window Drag Region (height: 36px / h-9, z-index: 9999)    │
├────────────────────────────────────────────────────────────────────────┤
│ Top Navigation Bar (Navbar)                                            │
│ [Logo + Status]  [Navigation Tabs / Menus]  [Sync Modals] [Theme/Lang] │
├────────────────────────────────────────────────────────────────────────┤
│ Main Content Area (flex-1, overflow-hidden, relative)                 │
│                                                                        │
│   <Outlet /> (Dashboard / Accounts / Logs / Security / Settings)       │
│                                                                        │
├────────────────────────────────────────────────────────────────────────┤
│ Background Runners & Notification Overlays (ToastContainer, Modals)    │
└────────────────────────────────────────────────────────────────────────┘
```

### Layout Tokens

```css
:root {
  --app-drag-height: 36px;
  --app-navbar-height: 56px;
  --app-window-min-width: 960px;
  --app-window-min-height: 640px;
  --app-content-padding: 1.5rem;
}
```

### Mini-View Floating Mode

When `useViewStore.isMiniView` is active:
- Standard window decorations and navbar are hidden.
- Window resizes to compact desktop dimensions (e.g. 360px x 180px).
- Renders `<MiniView />` displaying real-time proxy traffic rate, current account, and emergency failover toggle.

---

## 3. DaisyUI 5 & TailwindCSS 3.4 Design Tokens

Color tokens are declared in `tailwind.config.js` using DaisyUI 5 semantic color roles mapped to light and dark themes.

### Semantic Color Matrix

| Role | Light Theme (`light`) | Dark Theme (`dark`) | Purpose |
|---|---|---|---|
| `primary` | `#3b82f6` (Blue 500) | `#3b82f6` (Blue 500) | Primary actions, active navigation, focus rings |
| `secondary` | `#64748b` (Slate 500) | `#94a3b8` (Slate 400) | Secondary text, muted badges, subtle borders |
| `accent` | `#10b981` (Emerald 500) | `#10b981` (Emerald 500) | Positive indicators, quota healthy state |
| `neutral` | `#1f2937` (Gray 800) | `#1f2937` (Gray 800) | Contrasting chip elements, tooltips |
| `base-100` | `#ffffff` (Pure White) | `#0f172a` (Slate 900) | App surface background, cards, modal containers |
| `base-200` | `#f1f5f9` (Slate 100) | `#1e293b` (Slate 800) | Secondary background, table headers, hover rows |
| `base-300` | `#e2e8f0` (Slate 200) | `#334155` (Slate 700) | Borders, divider lines, disabled control tracks |
| `info` | `#0ea5e9` (Sky 500) | `#0ea5e9` (Sky 500) | Informational callouts, telemetry stats |
| `success` | `#10b981` (Emerald 500) | `#10b981` (Emerald 500) | Account valid, proxy running, test passed |
| `warning` | `#f59e0b` (Amber 500) | `#f59e0b` (Amber 500) | Quota near ceiling, account warming, rate limit |
| `error` | `#ef4444` (Red 500) | `#ef4444` (Red 500) | Account banned, upstream 429/503, invalid token |

### Theme Mode Enforcement
- **Storage:** Synced via `localStorage` key `theme` (`light` / `dark`) and `data-theme` attribute on root document element.
- **Class Strategy:** Tailwind `darkMode: 'class'` enables fine-grained utility classes (`dark:bg-base-300`, `dark:text-slate-200`).

---

## 4. Component Patterns & Specifications

### 4.1 Top Navigation Bar (`Navbar.tsx`)
- **Structure:** Split flex container containing `<NavLogo />`, `<NavMenu />`, and `<NavSettings />`.
- **Drag Region:** Upper 36px overlay intercepts `onMouseDown` to invoke Tauri native window dragging via `getCurrentWindow().startDragging()`.
- **Responsive Behavior:** Navigation items collapse into grouped dropdowns on compact viewports without breaking desktop layout.

### 4.2 Account Cards & Tables
- **Account Card (`AccountCard.tsx`):** Container displaying account email, avatar badge, active status pill, live quota percentage ring, and action menu.
- **Quota Bars (`QuotaItem.tsx`):** Dynamic multi-model quota gauges (Gemini 2.5 Pro, Flash, Claude) with progress bars shifting from `bg-success` (>30%) to `bg-warning` (10-30%) to `bg-error` (<10%).

### 4.3 Proxy Telemetry & Analytics
- **Live Feed Table:** High-density log viewer with monospace protocol badges (`OpenAI`, `Claude`, `Gemini`), status codes, latency in ms, and expandable request/response inspection drawers.
- **Recharts Integration:** Responsive token volume charts rendered with CSS variable color bindings.

### 4.4 Modals and Dialogs
- **Backdrop:** Native modal with `backdrop-blur-sm bg-black/50`.
- **Z-Index Layering:**
  - Base content: `z-0`
  - Sticky sub-headers: `z-10`
  - Dropdown popovers: `z-30`
  - Modal dialogues: `z-50`
  - Global toasts: `z-60`
  - Window drag overlay: `z-[9999]`

---

## 5. Typography & Spacing

### Font Stack
```css
font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji";
font-family-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
```

### Density Scale
Antigravity-Manager enforces a compact desktop information density:
- **Table cell padding:** `py-2.5 px-3` (compact desktop ergonomics).
- **Form input height:** `h-9` (36px standard controls).
- **Icon size standard:** 16px (`w-4 h-4`) inline, 20px (`w-5 h-5`) buttons, 24px (`w-6 h-6`) header.

---

## 6. Verification & Conformance

1. **AC-UI-001 (Layout Conformance):** Shell must never render legacy vertical sidebar navigation; desktop top navbar is mandatory.
2. **AC-UI-002 (Design Token Conformance):** All colors must bind to DaisyUI 5 semantic tokens or Tailwind color classes; no arbitrary hex codes in component templates.
3. **AC-UI-003 (Drag Window Conformance):** Native window drag region (`h-9` with `data-tauri-drag-region`) must remain unobstructed across all full-view pages.

---

*Consolidated app design system & UI — Antigravity-Manager v4.7.0*
