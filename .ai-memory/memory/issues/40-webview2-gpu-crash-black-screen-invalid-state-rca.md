# Issue 40: Desktop Window Black Screen & WebView2 `0x8007139F` Invalid State — Memory & Learnings

> **Cross-Reference:** Comprehensive specification and diagnostic artifacts documented in `02-spec/22-app-issues/35-webview2-gpu-crash-black-screen-invalid-state-rca.md`.

## Summary
In virtual machine environments (specifically VMware Virtual SVGA 3D with 256MB VRAM), Edge WebView2's GPU child process (`msedgewebview2.exe --type=gpu-process`) crashed with unhandled Direct3D 11 device lost exception `0xe0000008` in `KERNELBASE.dll`. This caused a cascading fatal exit (`STATUS_BREAKPOINT 0x80000003`) in the WebView2 browser host process (`msedge.dll`), orphaning the application's outer Win32 HWND with a dead, unpainted black client area (`Antigravity Manager Tools v4.162.0`).

Simultaneously, the background `AutoSwitcher` daemon ticked every 5 seconds, unconditionally invoking `handle.emit("auto-switcher://status-tick")` on the dead `ICoreWebView2` COM object, flooding the application log with `HRESULT 0x8007139F` (`ERROR_INVALID_STATE: The group or resource is not in the correct state to perform the requested operation`).

## Architectural Solutions Applied
1. **VM & Software Graphics Detection (`src-tauri/src/lib/run.rs`, `src-tauri/src/lib/appruntimeflags.rs`)**:
   - Zero-allocation Win32 registry checks against `HARDWARE\DESCRIPTION\System\BIOS` and primary display `DriverDesc` detect VMware, VirtualBox, Hyper-V, and basic display adapters.
   - Automatically appends `--disable-gpu-compositing` to `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, forcing stable CPU software compositing onto the DWM surface and preventing D3D11 device crashes.
2. **Self-Healing Window Reconstruction (`src-tauri/src/modules/lightweight.rs`)**:
   - Added `is_main_window_alive(window)` (`window.eval("true").is_ok()`) and `rebuild_main_window(app)`.
   - `ensure_main_window` automatically destroys a dead/corrupted window and rebuilds a fresh one.
3. **Auto-Recovery on Window Focus & Restore (`src-tauri/src/lib/appruntimeflags.rs`)**:
   - In `restore_and_focus_window`, checks webview liveness. If dead (`0x8007139F`), automatically reconstructs and focuses a fresh window.
4. **Log Storm Suppression (`src-tauri/src/modules/auto_switcher/status.rs`)**:
   - Skips status emissions when the main window's webview is unresponsive.
5. **Tray Menu Recovery (`src-tauri/src/modules/tray.rs`)**:
   - Added "Reload UI" menu item wired to `rebuild_main_window`.
6. **Window State Minimization Hygiene (`src-tauri/src/lib/run.rs`, `src-tauri/src/modules/lightweight.rs`)**:
   - Excluded `StateFlags::MINIMIZED` from window state restore to eliminate offscreen `-32000, -32000` coordinate initialization.
