# Spec: Windows Installer and Add/Remove Programs (Installed Apps) Branding

## Status: Approved & Active
## Scope: Tauri Bundler NSIS Installer, `install.ps1`, Windows Registry Branding
## Canonical Branding:
- **Title (DisplayName)**: `Antigravity Manager Tools`
- **Below (Publisher)**: `Maintained by Alim, Sponsored by RISEUP ASIA LLC`

---

## 1. Architectural Context & Problem Statement

On Windows systems, applications installed via NSIS, MSI, or standalone installers appear in Windows Settings > Apps > Installed apps ("Add or remove programs") and Control Panel > Programs and Features.

Windows Installed Apps displays two lines:
1. **Title (DisplayName)**: The primary application title shown at the top.
2. **Below (Publisher)**: The publisher metadata line shown underneath the title.

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
During NSIS installation, the `NSIS_HOOK_POSTINSTALL` macro executes after all files and default registry keys are placed, directly stamping the canonical branding:

```nsis
!macro NSIS_HOOK_POSTINSTALL
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "Antigravity Manager Tools"
    WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Maintained by Alim, Sponsored by RISEUP ASIA LLC"
!macroend
```

### 2.3 Standalone / Script Installer Layer (`install.ps1`)
When installed or updated via `install.ps1`:
- `$AppName`: Set to `"Antigravity Manager Tools"`.
- `$FullName`: Set to `"Antigravity Manager Tools"`.
- `$Publisher`: Set to `"Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.
- Function `Sync-InstalledAppsRegistryBranding` scans and updates all relevant uninstall registry keys (`HKCU` and `HKLM`), setting `DisplayName` to `"Antigravity Manager Tools"` and `Publisher` to `"Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.

---

## 3. Verification & Compliance
- Registry inspection verifies that `HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\agm-alim` has `DisplayName` set to `"Antigravity Manager Tools"` and `Publisher` set to `"Maintained by Alim, Sponsored by RISEUP ASIA LLC"`.
- Zero disruption to `.github/workflows/*.yml` or binary asset naming pipelines.
