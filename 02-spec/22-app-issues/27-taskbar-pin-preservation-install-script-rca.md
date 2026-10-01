# Issue 27: Taskbar Pin Removed on Reinstallation & Update (4-Part RCA)

> **Canonical Specification:** `02-spec/22-app-issues/27-taskbar-pin-preservation-install-script-rca.md`  
> **Status:** RESOLVED  
> **Target Subsystem:** `install.ps1` (Windows Standalone & In-App Portable Installer)  
> **Classification:** Non-CI/CD Application Bug & Packaging Defect  
> **Resolved In:** `v4.110.0`  

---

## 1. 🔍 Symptom

When a user who has previously pinned **Antigravity Manager Tools** (or its running executable `agm-alim.exe`) to the Windows Taskbar runs `install.ps1` to update or reinstall the application, the pinned taskbar icon is unexpectedly removed / unpinned from the taskbar, forcing the user to manually locate and re-pin the application after every installation.

---

## 2. 🧬 Root Cause Analysis (RCA)

Investigation revealed three compounding mechanisms in `install.ps1`:

1. **Unquoted Path Comparison Triggering Unnecessary Uninstaller (`Remove-PreviousInstallations`)**:
   - In `Remove-PreviousInstallations`, the script inspects Windows Registry uninstall keys to identify foreign or legacy installations. To skip uninstalling when the detected app is already located in the target directory `$InstallDir`, it performed:
     ```powershell
     if ($entry.InstallLocation -and (Resolve-Path $entry.InstallLocation -ErrorAction SilentlyContinue).Path -eq (Resolve-Path $InstallDir -ErrorAction SilentlyContinue).Path)
     ```
   - However, in Windows NSIS and standard registry entries, `$entry.InstallLocation` contains surrounding quotation marks (e.g., `"C:\Users\Administrator\AppData\Local\Programs\agm-alim"`).
   - Because of these quotes, `Resolve-Path $entry.InstallLocation` threw an exception and returned `$null`.
   - The equality comparison evaluated to `$false`, falsely classifying the existing installation as an outdated version from another directory.
   - Consequently, `install.ps1` invoked the previous uninstaller:
     ```powershell
     $uninstExit = Invoke-IndentedCommand -FilePath $uninstClean -ArgumentList @("/S", "/currentuser")
     ```
   - The NSIS uninstaller wiped the existing application files and deleted the Start Menu shortcut. In modern Windows (Windows 10 & 11), Windows Explorer monitors application uninstallations and automatically removes / moves associated taskbar pinned shortcuts into the `Tombstones` directory.

2. **Indiscriminate Deletion of Taskbar Directory Shortcuts**:
   - In `Remove-PreviousInstallations` (lines 527–530) and Step 5 shortcut cleanup (line 1601), `install.ps1` explicitly iterated over `$TaskbarDir` (`$env:APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar`) and called `Remove-Item -Force` on candidate shortcut names, including:
     - `agm-alim.lnk`
     - `Agm - Alim.lnk`
     - `Anti-Gravity Tools by Alim.lnk`
     - `Antigravity Tools.lnk`
     - `antigravity-tools.lnk`
   - When a user right-clicks a running instance of `agm-alim.exe` on their taskbar and selects "Pin to taskbar", Windows automatically names the pinned shortcut `agm-alim.lnk` inside `$TaskbarDir`.
   - Line 1601 directly deleted `$TaskbarDir\agm-alim.lnk` on every reinstallation.

3. **Disabled Taskbar Pin Restoration**:
   - In Step 6, taskbar pinning had been commented out with a note assuming that disabling it would preserve existing pins. However, because the uninstaller ran and shortcuts in `$TaskbarDir` were deleted, there was zero logic to detect existing pins prior to installation and restore / update them afterwards.

---

## 3. 🛠️ Resolution

1. **Pre-flight Taskbar Pin Detection & Backup**:
   - Implemented `Get-PinnedTaskbarShortcuts` and `Backup-PinnedTaskbarShortcuts` prior to any download or uninstallation attempt.
   - Scans `$TaskbarDir` for shortcuts pointing to AGM (`agm-alim.exe`, `agm.exe`, or `$InstallDir`) or carrying AGM branding names. Backs them up safely into `$env:TEMP\agm_taskbar_pin_backup`.

2. **Strict Ban on Deleting Shortcuts in `$TaskbarDir`**:
   - Removed all `$TaskbarDir` deletions from `Remove-PreviousInstallations` and Step 5 shortcut cleanup.
   - Cleanups are strictly restricted to `$DesktopDir` and `$StartMenuDir`. Pinned taskbar shortcuts are never deleted.

3. **Trim Quotes and Normalize Paths in `Remove-PreviousInstallations`**:
   - Stripped enclosing quotes from `$entry.InstallLocation` and derived the installation directory from `$entry.UninstallString` if `InstallLocation` is absent.
   - Accurately checks whether the detected installation is already in `$InstallDir`, preventing `uninstall.exe` from executing during in-place updates or re-installations.

4. **Post-Installation Taskbar Pin Restoration & Target Refresh**:
   - Implemented `Restore-And-Update-PinnedTaskbarShortcuts` in Step 6.
   - If the user previously had the application pinned, it restores the shortcut from backup if missing, updates `TargetPath` to the newly verified `$ExePath`, sets `WorkingDirectory` to `$InstallDir`, sets `IconLocation` to `"$ExePath,0"`, and saves the shortcut.
   - If the user had not pinned the application, no pin is forced, respecting user choices.

---

## 4. 🛡️ Prevention & Learnings

1. **Windows Registry Quoting Convention**: Registry strings for `InstallLocation` and `UninstallString` must always be sanitized with `.Trim().Trim('"').Trim()` before calling filesystem cmdlets like `Resolve-Path` or `Test-Path`.
2. **Never Clean User Pinned Taskbar Directories**: User-pinned directories (`%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar`) belong to the user's desktop environment and must never be subject to brute-force legacy cleanup.
3. **Capture State Before Modification**: Always detect and back up existing desktop/taskbar configuration before applying destructive or uninstallation actions, ensuring lossless restoration.
