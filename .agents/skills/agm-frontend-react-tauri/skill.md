---
name: agm-frontend-react-tauri
description: Specialized skill for developing, styling, and debugging the React 18 + TypeScript frontend, Tailwind CSS design system, Zustand state stores, and Tauri IPC communications in Antigravity-Manager.
---

# AGM Frontend React & Tauri IPC Architecture

This skill guides engineering work on the user interface of Antigravity-Manager. The frontend is built with React 18, Vite, TypeScript, and Tailwind CSS, communicating with the Rust backend via Tauri v2 IPC channels.

## Key Directory Structure

- `src/pages/` — Top-level views: `Accounts.tsx`, `ApiProxy.tsx`, `Instances.tsx`, `Settings.tsx`, `Security.tsx`, `Email.tsx`, `TokenStats.tsx`, `UserToken.tsx`.
- `src/components/` — Modular UI elements:
  - `errors/` — `error-history-drawer.tsx`, `error-modal.tsx`, `error-queue-badge.tsx` (Error diagnostics, per-section clipboard copy, high-contrast slate theme).
  - `layout/` — `Layout.tsx`, `MiniView.tsx` (compact floating desktop widget).
- `src/stores/` — State management via Zustand:
  - `useAccountStore.ts` — Account list, active account, and quota polling.
  - `useInstanceStore.ts` — IDE profile lifecycle and process health.
  - `useConfigStore.ts` — Global proxy configurations, listening ports, and runtime flags.
  - `error-store.ts` — Centralized error queue and diagnostic logs.
- `src/utils/request.ts` — Typed Tauri IPC wrapper (`invoke`).

## Core Frontend Directives

### 1. Tauri IPC Communication
- Always invoke backend commands using the typed wrapper:
  ```typescript
  import { request as invoke } from '../utils/request';
  const result = await invoke<ResponseType>('command_name', { param1, param2 });
  ```
- Listen for asynchronous backend events cleanly, registering teardown unlisteners in `useEffect`:
  ```typescript
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen('event_name', (event) => { /* handle */ }).then((fn) => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }, []);
  ```

### 2. Styling & Theme System
- Use Tailwind CSS utility classes adhering to the project's color palette.
- Support both dark, high-contrast slate, and light themes via CSS variable tokens.
- Maintain responsive layouts supporting both the standard application window and the compact `MiniView.tsx` widget.

### 3. Error Diagnostics & History Drawer
- Global unhandled promise rejections and UI errors are captured into `error-store.ts`.
- The Error History Drawer must support full data copy, per-section copy, and Markdown export for immediate troubleshooting.
