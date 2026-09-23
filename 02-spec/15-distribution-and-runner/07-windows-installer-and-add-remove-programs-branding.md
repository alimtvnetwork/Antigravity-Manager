# Spec: Windows Installer and Add/Remove Programs (Installed Apps) Branding

## Status: Approved & Active
## Scope: Tauri Bundler NSIS Installer, `install.ps1`, Windows Registry Branding
## Canonical Branding:
- **Title (DisplayName)**: `Antigravity Manager Tools <Version>` (e.g. `Antigravity Manager Tools 4.65.1`)
- **Below (Publisher)**: `Maintained by Alim, Sponsored by RISEUP ASIA LLC`

---

## 1. Architectural Context & Problem Statement

On Windows systems, applications installed via NSIS, MSI, or standalone installers appear in Windows Settings > Apps > Installed apps ("Add or remove programs") and Control Panel > Programs and Features.

Windows Installed Apps displays:
1. **Title (DisplayName)**: The primary application title shown at the top, formatted as `Antigravity Manager Tools <Version>` so users and administrators can instantly identify the exact version installed alongside other standard applications.
2. **Below (Publisher)**: The publisher metadata line shown underneath the title (`Maintained by Alim, Sponsored by RISEUP ASIA LLC`).

---

## 2. Solution Architecture

To permanently enforce this across all deployment channels while preserving release asset naming pipelines, the branding is configured at three distinct layers:

### 2.1 Tauri Configuration Layer (`src-tauri/tauri.conf.json`)
- `app.windows[0].title`: `"Antigravity Manager Tools"`
- `bundle.publisher`: Configured explicitly to `"Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.
- `bundle.copyright`: Configured explicitly to `"Copyright © 2026 Alim. Sponsored by RISEUP ASIA LLC. All rights reserved."`.
- `bundle.shortDescription` & `bundle.longDescription`: Updated to reflect `"Antigravity Manager Tools - Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.
- `bundle.windows.nsis.installerHooks`: Set to `"hooks.nsh"`.

### 2.2 NSIS Hook Layer (`src-tauri/hooks.nsh`)
During NSIS installation, the `NSIS_HOOK_POSTINSTALL` macro executes after all files and default registry keys are placed, dynamically reading the installed `DisplayVersion` (with static fallback) and writing `DisplayName`:

```nsis
!macro NSIS_HOOK_POSTINSTALL
    ReadRegStr $0 SHCTX "${UNINSTKEY}" "DisplayVersion"
    StrCmp $0 "" 0 +2
    StrCpy $0 "4.65.1"
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "Antigravity Manager Tools $0"
    WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Maintained by Alim, Sponsored by RISEUP ASIA LLC"
!macroend
```

### 2.3 Standalone / Script Installer Layer (`install.ps1`)
When installed or updated via `install.ps1`:
- `$AppName`: Set to `"Antigravity Manager Tools"`.
- `$FullName`: Set to `"Antigravity Manager Tools"`.
- `$Publisher`: Set to `"Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.
- Function `Sync-InstalledAppsRegistryBranding` dynamically resolves the target/installed version (from `$TargetVersion`, `$Version`, the executable's `FileVersionInfo`, or the existing registry `DisplayVersion`), setting `DisplayName` to `"Antigravity Manager Tools $ver"` and `Publisher` to `"Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.
- Regex in uninstall detection (`install.ps1`) supports optional version suffix `Antigravity Manager Tools(\s+[0-9A-Za-z.-]+)?`.

### 2.4 Version Synchronization (`scripts/bump-version.mjs`)
- `src-tauri/hooks.nsh` is included in `TARGET_FILES` so all version bumps automatically update the fallback version string in `hooks.nsh`.

---

## 3. Verification & Compliance
- Registry inspection verifies that `HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\agm-alim` has `DisplayName` set to `"Antigravity Manager Tools <Version>"` and `Publisher` set to `"Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.
- Zero disruption to `.github/workflows/*.yml` or binary asset naming pipelines.
