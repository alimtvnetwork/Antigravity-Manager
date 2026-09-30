---
name: agm-frontend-react-tauri
description: Specialized skill for managing Antigravity-Manager React frontend architecture, Zustand state management, dual-mode Tauri IPC / Web REST gateway, MiniView widget, and Tailwind/DaisyUI theming.
---

# AGM Frontend React & Tauri IPC Architecture

This skill provides comprehensive architectural guidance, state management conventions, Tauri IPC contracts, and UI design standards for the Frontend in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

The Antigravity-Manager frontend is built with React, TypeScript, TailwindCSS, DaisyUI, and Zustand, operating seamlessly across native desktop and headless web environments:

```
+-----------------------------------------------------------------------------------------+
|                                    React Frontend                                       |
|  +--------------------+  +----------------------+  +---------------------------------+  |
|  |   Full View (HUD)  |  | Mini View (Floating) |  |   Global Overlays / Modals      |  |
|  |  Navbar + Outlets  |  | 300px Compact Widget |  | ErrorModal, DebugConsole, etc.  |  |
|  +--------------------+  +----------------------+  +---------------------------------+  |
|           |                         |                                |                  |
|           +-------------------------+--------------------------------+                  |
|                                     v                                                   |
|                        Zustand Global State Stores                                      |
|    (useAccountStore, useInstanceStore, useConfigStore, useDebugConsole, error-store)   |
|                                     |                                                   |
|                                     v                                                   |
|                         Unified IPC Client (request.ts)                                 |
+-------------------------------------+---------------------------------------------------+
                                      |
                 +--------------------+--------------------+
                 | isTauri()                               | !isTauri() (Web Mode)
                 v                                         v
   +---------------------------+             +---------------------------+
   |  Tauri Native IPC Engine  |             |  HTTP REST Gateway API    |
   |   @tauri-apps/api/core    |             |   /api/accounts, /proxy   |
   |   invoke<T>(cmd, args)    |             |   Authorization: Bearer   |
   +---------------------------+             +---------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src/App.tsx` | Top-level router, auth guards, theme provider, and WebView2 window restore / unfreeze event listeners. |
| `src/components/layout/Layout.tsx` | View mode routing (Full View vs Mini View), toast notification container, and background daemon mounter. |
| `src/components/layout/TitleBar.tsx` | Frameless window titlebar, drag regions (`data-tauri-drag-region`), OS traffic lights vs Windows caption buttons. |
| `src/components/layout/MiniView.tsx` | 300px compact floating widget, live model quota progress bars, and real-time proxy request ticker. |
| `src/components/navbar/Navbar.tsx` | Responsive header, navigation links, instance quick-switcher, debug console toggle, and theme switchers. |
| `src/components/common/BackgroundTaskRunner.tsx` | Headless timer engine for periodic quota scraping, account sync, and reactive auto-switcher threshold triggers. |
| `src/utils/request.ts` | Unified IPC client mapping frontend commands to either native Tauri `invoke` or web HTTP REST endpoints. |
| `src/stores/error-store.ts` | Error tracking with user click breadcrumbs ring buffer, stack frame parsing, and AI diagnostic prompt generation. |

---

## 3. Dual-Mode Runtime Architecture (Tauri vs. Web)

All backend communication must pass through `src/utils/request.ts`:

- **Desktop Tauri Mode (`isTauri() == true`)**:
  - Dynamically imports `@tauri-apps/api/core`.
  - Executes `invoke<T>(command, args)`.
  - Automatically catches errors and routes them into `error-store.captureError`.
- **Web / Docker Headless Mode (`isTauri() == false`)**:
  - Looks up `command` in `COMMAND_MAPPING`.
  - Dispatches `fetch()` to `/api/<endpoint>` with Bearer authorization token from `sessionStorage`.
  - Interpolates dynamic route parameters (e.g. `:accountId`).

---

## 4. WebView2 DWM Canvas Freeze Prevention

Windows WebView2 can encounter background window canvas freezes under DWM occlusion. The frontend implements defensive wakeups in `App.tsx`:

```typescript
// Listen for window restore, focus, and visibility changes to force repaint
window.addEventListener('focus', () => window.dispatchEvent(new Event('resize')));
document.addEventListener('visibilitychange', () => {
  if (document.visibilityState === 'visible') {
    window.dispatchEvent(new Event('resize'));
  }
});
```

---

## 5. State Management & Reactive Polling Invariants

1. **Reactive Auto-Switcher Trigger**:
   - In `BackgroundTaskRunner.tsx`, adjustments to the quota threshold slider in `Settings.tsx` trigger immediate quota re-evaluation within 500ms rather than waiting for the next periodic cycle.
2. **Optimistic Store Updates**:
   - Account reordering and status updates in `useAccountStore.ts` apply optimistic UI state immediately and synchronize with backend persistence asynchronously.
3. **Mismatched JSX Conditional Rendering — TOTAL BAN**:
   - Never mix logical AND (`&&`) with ternary null returns (`) : null}`).
   - Correct patterns: `{condition && <div>...</div>}` or `{condition ? <div>...</div> : null}`.

---

## 6. MiniView Dynamic Sizing, Bounds Memory & Self-Healing

The compact 300px floating HUD (`src/components/layout/MiniView.tsx` & `src/utils/windowManager.ts`) implements:
- **Dynamic Content Height**: Measures `containerRef.current.scrollHeight` and invokes `enterMiniMode(contentHeight)`. Sets Tauri window to `LogicalSize(300, contentHeight + 2)` with `alwaysOnTop: true`, `resizable: false`, and `decorations: false`.
- **Prior Bounds Memory (`priorBounds`)**: Before entering mini mode, captures the user's outer window position and dimensions (`width > 350 && height > 250`) and persists to `useViewStore.savedBounds`. Upon `exitMiniMode`, restores exact coordinates and dimensions (or falls back to `1200x800` centered).
- **Startup Self-Healing (`ensureFullViewState`)**: On mount in `App.tsx`, checks window coordinates. If the window was previously closed in mini mode or launched with negative/offscreen coordinates, centers and restores the window to standard full view.
- **Real-Time Proxy Event Stream**: Listens to native Tauri event `proxy://request` via `@tauri-apps/api/event`, displaying the active model, input/output tokens, and request latency.

---

## 7. Unified Request Gateway & Modal Suppression (`src/utils/request.ts`)

- **Route Parameter Interpolation**: Replaces tokens like `:accountId` with matching argument values, stripping the key from the request payload to prevent duplicate fields in the request body.
- **Modal Suppression Flag (`_suppressGlobalModal`)**: When background polling runs (e.g. periodic quota checks), passing `_suppressGlobalModal: true` prevents error toast/modal popups from interrupting user interactions on non-critical failures.
- **Debounced 401 Unauthorized Event**: On HTTP 401 responses, dispatches `abv-unauthorized` custom event with a 2,000ms debounce cooldown to prevent UI refresh storms.

---

## 8. Division of Labor in Background Tasks (`BackgroundTaskRunner.tsx`)

- **Desktop Tauri Mode**: Guarded by `if (isTauri()) return;`. The native Rust background daemon (`start_auto_switcher` in `auto_switcher.rs`) acts as the single source of truth, performing quota evaluation, conscious PID matching, and emitting `account://auto-switched` events.
- **Headless Web Mode**: `BackgroundTaskRunner.tsx` executes the client-side single-circle loop, evaluating `instancesToEvaluate`, applying 15s cooldowns, and calling `smartRotateProfileAccount`.

---

## 9. Account Focus & Highlight Pattern (`src/pages/Accounts.tsx`)

`handleFocusActiveAccount()` locates the active account with high accessibility:
1. Clears existing search filters (`setFilter("all")`, `setSearchQuery("")`).
2. Jumps to the exact pagination page containing the active account.
3. Dispatches `scrollIntoView({ behavior: "smooth", block: "center" })`.
4. Applies a high-visibility amber highlight ring (`ring-4 ring-amber-400 dark:ring-amber-500 ring-offset-2`) that auto-fades after 2.5 seconds.

