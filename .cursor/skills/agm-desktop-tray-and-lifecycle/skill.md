---
name: agm-desktop-tray-and-lifecycle
description: Specialized skill for managing the native desktop OS integration, System Tray menus, Lightweight Mode memory unloading, dynamic window reconstruction, Linux WebKit/Wayland graphics policies, and in-process Win32 shell shortcut healing in Antigravity-Manager.
---

# AGM Desktop System Integration & Lightweight Lifecycle

This skill provides comprehensive architectural guidance, lifecycle management procedures, and cross-platform OS adaptation rules for the Desktop System Integration tier in **Antigravity-Manager (AGM)**.

---

## 1. Subsystem Architecture Overview

AGM runs as a persistent system background service with an optional rich GUI. To minimize system resource consumption and ensure rock-solid cross-platform desktop integration, it employs dynamic window reconstruction, native OS system trays, and platform-specific graphic/shell workarounds:

```
+----------------------------------------------------------------------------------------------------+
|                                    Application Window Lifecycle                                    |
|    - Normal Mode: Window minimized to System Tray on close (CloseRequested intercepted)           |
|    - Lightweight Mode: Window destroyed completely to free ~400MB WebKit/Chromium RAM             |
|    - Restoration: Dynamically rebuilt from tauri.conf.json via ensure_main_window()                |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                    Cross-Platform System Tray                                      |
|                                  src-tauri/src/modules/tray.rs                                     |
|    - macOS: Dynamic monochrome template icon (tray-icon.png)                                       |
|    - Windows / Linux: Full-color RGBA icon (icon.png)                                              |
|    - Menu: Live Quota status, Next Profile Switch, Refresh, Lightweight Toggle, Quit              |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                   OS-Specific Shell & Graphics Tuning                              |
|    - Windows: In-process COM IShellLinkW icon healer & DWM repaint (src/utils/win_shortcut.rs)     |
|    - Linux: GDK / WebKitGTK Wayland startup policy & wlroots compositor defense (linux_graphics.rs)|
|    - macOS: Background LaunchServices lsregister GC & trash purging daemon                         |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/tray.rs` | System tray setup, localized menu entries (`texts.switch_next`, `texts.quota`), tray event listener (left-click restore, right-click menu), and dynamic text updates. |
| `src-tauri/src/modules/lightweight.rs` | `enter_lightweight_mode` (destroys WebviewWindow), `exit_lightweight_mode`, and `ensure_main_window` (reconstructs window dynamically from `tauri.conf.json` config). |
| `src-tauri/src/linux_graphics.rs` | Linux GDK/WebKit startup policy, `desktop_is_wlroots_family` (Hyprland, sway, niri), `should_force_x11_backend`, and `should_disable_webkit_dmabuf`. |
| `src-tauri/src/utils/win_shortcut.rs` | Native Win32 COM `IShellLinkW` implementation. Heals corrupted Desktop and Start Menu shortcut icons in-process without invoking `powershell.exe`. |
| `src-tauri/src/modules/autostart.rs` | Native autostart on system boot via `tauri-plugin-autostart`. |

---

## 3. Core Invariants & Rules

1. **Lightweight Mode Dynamic Window Restoration**:
   - In Lightweight Mode, `window.close()` or `window.destroy()` drops the webview process to reclaim memory.
   - When the user clicks the tray icon or triggers `open-ui`, `ensure_main_window(app)` must dynamically re-instantiate the window from `tauri.conf.json`, restore saved bounds via `tauri-plugin-window-state`, and re-apply native window icons.

2. **Win32 Focus & Frame Invariant**:
   - `force_restore_and_focus_win32()` in `src-tauri/src/lib.rs` must use `ShowWindow(SW_RESTORE)`, `SetWindowPos(SWP_FRAMECHANGED)`, `InvalidateRect`, and `AttachThreadInput` to defeat Windows foreground lock and prevent gray occluded preview freezes.

3. **Linux Wayland / wlroots Defense**:
   - Never force `GDK_BACKEND=x11` on wlroots compositors (`niri`, `hyprland`, `sway`), as this produces completely black windows in WebKitGTK (Issue #3388).
   - Automatically disable WebKit DMA-BUF rendering when Nvidia proprietary drivers (`/dev/nvidia0`) are detected.

4. **In-Process Shell Healer**:
   - Shell shortcut healing on Windows must strictly use the native C FFI COM interface (`CLSID_SHELL_LINK`, `IID_ISHELL_LINK_W`), never spawning external PowerShell or batch scripts that trigger console flashing.

---

## 4. Verification & Testing

```bash
# 1. Quality pre-flight checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Test linux graphics decision table in isolation
cd src-tauri && rustc --test src/linux_graphics.rs -o target/test_graphics && ./target/test_graphics
```
