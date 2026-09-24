## Quick Install

### Windows (PowerShell 5.1+)

**Bar 1: Latest Version (Auto-Updating)**
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

**Bar 2: Version-Based Installation (v4.72.0)**
```powershell
& ([scriptblock]::Create((irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.72.0/install.ps1))) -Version "4.72.0"
```

### Linux / macOS (Bash)

**Bar 1: Latest Version (Auto-Updating)**
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

**Bar 2: Version-Based Installation (v4.72.0)**
```bash
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.72.0/install.sh | bash -s -- --version "4.72.0"
```

---

## What's Changed in v4.72.0

* **Release Page Two-Bar Installation Layout (`02-spec/21-app/45-two-bar-installer-and-dwm-blank-ui-fix.md`)**: Replaced merged, single-block installation instructions with two distinct copyable code blocks ("bars") in GitHub release notes and project documentation. Users can now copy **Bar 1 (Latest Version)** for auto-updating bleeding-edge installations or **Bar 2 (Version-Based Installation)** with `-Version "4.72.0"` explicitly bound to that exact tag.
* **PowerShell Interactive History Pinned Version Recovery**: Expanded `Get-InvocationHistoryCandidates` in `install.ps1` to query `Get-History`, `[Microsoft.PowerShell.PSConsoleReadLine]::GetHistoryItems()`, and the PSReadLine history file. When invoked via `irm <url>/vX.Y.Z/install.ps1 | iex`, the installer now accurately extracts the pinned version from the interactive session command line even when `$MyInvocation.Line` is empty.
* **Strict Pinned Version Queue Isolation & Drift Prevention**: In `install.ps1` and `install.sh`, enforced strict queue isolation when `$isPinned` is true. Disabled CDN manifest staleness checks and GitHub latest release replenishment on pinned runs. If GitHub API is unavailable or rate-limited, the installer builds deterministic direct release asset URLs (`/releases/download/v<VER>/...`) for that exact version, eliminating silent fallbacks to unintended builds (such as `v4.71.1`).
* **Win32 DWM Frame Recalculation & WebView2 Blank UI Elimination**: Permanently eradicated the intermittent blank gray window and blank taskbar thumbnail previews on Windows 10/11. Added `--disable-features=CalculateNativeWinOcclusion,CalculateNativeWindowOcclusion` and background throttling flags to `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` in `src-tauri/src/lib.rs`. In `force_restore_and_focus_win32`, invoked Win32 `SetWindowPos` with `SWP_FRAMECHANGED` and `RedrawWindow` with `RDW_INVALIDATE | RDW_INTERNALPAINT | RDW_UPDATENOW | RDW_ALLCHILDREN` to force DWM redirection bitmap composition. Evaluated `window.dispatchEvent(new Event('resize'))` and added frontend window restoration listeners in `src/App.tsx`.
* **Root Cause Analysis & Canonical Spec**: Published canonical specification `02-spec/21-app/45-two-bar-installer-and-dwm-blank-ui-fix.md` and 4-part RCA `02-spec/22-app-issues/10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md`.
