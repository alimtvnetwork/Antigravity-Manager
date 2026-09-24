# Subtask 03: Win32 WebView2 DWM Blank UI Fix

> Status: DONE
> Owner: Antigravity Agent
> Spec: 02-spec/21-app/45-two-bar-installer-and-dwm-blank-ui-fix.md
> RCA: 02-spec/22-app-issues/10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md

## Objectives
1. In `src-tauri/src/lib.rs`, add `--disable-features=CalculateNativeWinOcclusion,CalculateNativeWindowOcclusion` and background throttling flags to `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`.
2. In `force_restore_and_focus_win32`, call `SetWindowPos` with `SWP_FRAMECHANGED` and `RedrawWindow` / `InvalidateRect` with full invalidation flags so DWM updates the live taskbar thumbnail preview immediately.
3. In `restore_and_focus_window`, evaluate JavaScript `window.dispatchEvent(new Event('resize'))` and emit `window-restored`.
4. In `src-tauri/src/modules/lightweight.rs`, route `exit_lightweight_mode` through `crate::restore_and_focus_window(&window)`.
5. In `src/App.tsx`, listen to `window-restored`, `focus`, and `visibilitychange` events to force layout recalculation and eliminate frozen paint buffers.
