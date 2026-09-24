# 10 Blank UI DWM Occlusion & Versioned Installer Root Cause Analysis (RCA)

> **/goal** Provide a comprehensive 4-part Root Cause Analysis detailing the intermittent blank UI / DWM taskbar thumbnail freeze, the installer version drift and GitHub Actions stamping bug, and the release page two-bar installation formatting.
> **/learn** Reference this document whenever diagnosing WebView2 blank window rendering, PowerShell/Bash installer version resolution without GitHub API drift, and CI/CD release notes generation.

---

## 1. Problem Symptom & Trigger Conditions

### 1.1 Symptom 1: Intermittent Blank Gray UI / DWM Occlusion Freeze
- **Observed Behavior**: When the application runs in the background, is minimized, or stays behind other windows, hovering over the Windows taskbar icon displays a completely blank gray thumbnail rectangle (`assets/screenshots/blank-ui-taskbar-hover-01.png`). In some cases, clicking the taskbar icon to restore the window brings up a blank, white/gray window surface that does not render until manually resized.
- **Trigger Conditions**: Windows 10/11 with DWM (Desktop Window Manager) live preview active; frameless window (`decorations: false`); system entering idle or another window fully occluding the application.

### 1.2 Symptom 2: Version-Pinned Installation Drifting to Broken Release
- **Observed Behavior**: The user executed:
  ```powershell
  irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.70.0/install.ps1 | iex
  ```
  Expecting version `4.70.0` to be installed. However, the installer output:
  ```
  [*] Discovering available release versions from GitHub...
  [*] Discovered releases from CDN manifest (10 versions available, rate-limit free)
  [*] Cached CDN manifest version (v4.65.0) is not newer than current installed version (v4.71.2); querying live GitHub releases...
  [*] Current installed version: v4.71.2
  [*] Target release version   : v4.71.1 (Architecture: x64)
  [*] Migration path           : v4.71.2 -> v4.71.1
  ```
  And proceeded to install `v4.71.1`, which subsequently crashed on launch with `agm-alim.exe - Entry Point Not Found` (`TaskDialogIndirect` missing, `assets/screenshots/entrypoint-not-found-crash-01.png`).
- **Trigger Conditions**: Piped installation via `irm <url>/v4.70.0/install.ps1 | iex` when a newer version was already present locally, and the release workflow had failed to stamp the version placeholder.

### 1.3 Symptom 3: Release Page Lacked Two-Bar Installation Format
- **Observed Behavior**: GitHub release notes merged the latest and pinned install commands into a single block with a comment (`assets/screenshots/release-page-two-bars-install-01.png`), lacking separate copyable code blocks ("bars") for Latest vs Version-Based installation.

---

## 2. Root Cause Analysis

### 2.1 GitHub Actions Variable Scope Subshell Loss
In `.github/workflows/release.yml`, the release asset preparation was split into two steps:
1. `Generate updater.json` ran:
   ```bash
   DOWNLOAD_BASE="https://github.com/${REPO}/releases/download/${VERSION}"
   VER="${VERSION#v}"
   ```
2. The subsequent step `Collect release files and generate checksums` ran in a fresh subshell without `VER` being defined:
   ```bash
   sed "s/__PINNED_VERSION__/$VER/g" install.ps1 > release-files/install.ps1
   ```
   Because `$VER` was unset in that subshell, `sed` replaced `__PINNED_VERSION__` with empty string `""`!
   As a result, all released `install.ps1` and `install.sh` scripts contained `$PinnedVersion = ""` instead of the actual release tag.

### 2.2 Incomplete History Candidate Sources in PowerShell
Inside an interactive PowerShell session where the user ran `irm ... | iex`:
- `$MyInvocation.Line` is empty inside the script block executed by `Invoke-Expression`.
- `[System.Environment]::CommandLine` returns `powershell.exe` without the pipeline arguments.
- `Get-InvocationHistoryCandidates` only checked `$MyInvocation` and process command lines, but failed to query `Get-History` or `[Microsoft.PowerShell.PSConsoleReadLine]::GetHistoryItems()`. Therefore, `Resolve-PinnedVersion` returned `$null`.

### 2.3 Destructive Fallback Queue on Pinned Version Failure
When `install.ps1` failed to detect the pinned version:
1. It evaluated `$isPinned = $false`.
2. Because the current installed version (`v4.71.2`) was greater than the CDN manifest version (`v4.65.0`), it flagged the CDN manifest as stale and queried `https://api.github.com/repos/$Repo/releases?per_page=30`.
3. GitHub API returned `v4.71.1` as the first entry.
4. The installer targeted `v4.71.1` and performed a downgrade to the broken build.
5. Even if `$isPinned` had been `$true`, lines 1163-1174 replenished the queue with latest releases if GitHub API queries failed, breaking the strict version contract.

### 2.4 Chromium / WebView2 Windows Native Occlusion Throttling
1. In `src-tauri/src/lib.rs`, `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` disabled `CalculateNativeWindowOcclusion`, but Chromium's internal flag name on Windows is `CalculateNativeWinOcclusion`.
2. When a frameless window (`decorations: false`) is occluded or minimized, Windows DWM stops updating the redirection bitmap. Calling `ShowWindow` without `SetWindowPos(SWP_FRAMECHANGED)` and `RedrawWindow` leaves DWM with a stale or blank gray thumbnail preview.

---

## 3. Corrective Actions & Architectural Remediation

### 3.1 Step Variable Export in Release Workflow
In `.github/workflows/release.yml`, pass `env: VERSION: ${{ github.ref_name }}` directly to the `Collect release files and generate checksums` step and compute `VER="${VERSION#v}"` within that exact step.

### 3.2 Two-Bar Release Notes Formatting
In `.github/workflows/release.yml`, format `release_notes.md` with two distinct markdown code blocks:
- **Bar 1**: Latest Version (PowerShell / Bash)
- **Bar 2**: Version-Based Installation (passing `-Version "<VER>"` for PowerShell, and `--version "<VER>"` for Bash).

### 3.3 Strict Installer Version Resolution & History Extraction
In `install.ps1`:
1. Expand `Get-InvocationHistoryCandidates` to query `Get-History`, `[Microsoft.PowerShell.PSConsoleReadLine]::GetHistoryItems()`, and the PSReadLine history file.
2. Expand `Resolve-PinnedVersion` regex to extract version numbers from `-Version`, `--version`, and `irm .../releases/download/vX.Y.Z/...` patterns.
3. When `$isPinned` is `$true`:
   - Never flag manifest as stale.
   - Never replenish the candidate queue with unrequested versions.
   - Target ONLY `$cleanPinned`. If metadata queries fail, construct direct GitHub release asset URLs for `$cleanPinned` directly.

### 3.4 Win32 Frame Change, Invalidation & Redraw Pipeline
1. In `src-tauri/src/lib.rs`:
   - Add `--disable-features=CalculateNativeWinOcclusion,CalculateNativeWindowOcclusion` and background throttling flags.
   - In `force_restore_and_focus_win32`, call `SetWindowPos` with `SWP_FRAMECHANGED` and `RedrawWindow` with `RDW_INVALIDATE | RDW_INTERNALPAINT | RDW_UPDATENOW | RDW_ALLCHILDREN`.
   - In `restore_and_focus_window`, evaluate `window.dispatchEvent(new Event('resize'))`.
   - In `src-tauri/src/modules/lightweight.rs`, route `exit_lightweight_mode` through `crate::restore_and_focus_window(&window)`.
2. In `src/App.tsx`:
   - Listen to `window-restored`, `focus`, and `visibilitychange` events to force layout recalculation and eliminate frozen paint buffers.

---

## 4. Verification & Conformance Evidence

| Test Scenario | Input / Trigger | Expected Outcome | Conformance Status |
|---|---|---|:---:|
| Release Stamping | Tag build `v4.72.0` | `install.ps1` has `$PinnedVersion = "4.72.0"` | Verified |
| Interactive History Pin | `irm .../v4.70.0/install.ps1 \| iex` | Resolves `4.70.0` from `Get-History` | Verified |
| Direct Argument Pin | `& (...) -Version "4.72.0"` | Strictly targets `4.72.0` | Verified |
| Zero Version Drift | Pinned version with network timeout | Fails or downloads direct asset; NEVER installs latest | Verified |
| Two-Bar Release Notes | Release workflow run | Two distinct code blocks with copy buttons | Verified |
| DWM Thumbnail Preview | Hover taskbar while minimized | Window contents render cleanly without blank gray box | Verified |
