# Spec: Windows Installer and Add/Remove Programs (Installed Apps) Branding

## Status: Approved & Active
## Scope: Tauri Bundler NSIS Installer, `install.ps1`, Windows Registry Branding
## Canonical Branding String: `Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC`

---

## 1. Architectural Context & Problem Statement

On Windows systems, applications installed via NSIS, MSI, or standalone installers appear in Windows Settings > Apps > Installed apps ("Add or remove programs") and Control Panel > Programs and Features.

Previously, the entries in the Windows registry for the tool displayed:
- **DisplayName**: `agm-alim`
- **Publisher**: `lbjlaq`

This was caused by two factors:
1. `src-tauri/tauri.conf.json` had `"productName": "agm-alim"`, causing NSIS to default `DisplayName` to `"agm-alim"`.
2. `bundle.publisher` was not explicitly set in `tauri.conf.json`, causing Tauri's bundler to default the publisher to the second token of `identifier` (`com.lbjlaq.antigravity-tools` -> `lbjlaq`).

---

## 2. Solution Architecture

To permanently resolve this across all deployment channels while preserving release asset naming pipelines, the branding is enforced at three distinct layers:

### 2.1 Tauri Configuration Layer (`src-tauri/tauri.conf.json`)
- `bundle.publisher`: Configured explicitly to `"Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"`.
- `bundle.copyright`: Configured explicitly to `"Copyright © 2026 MD Alim Ul Karim. Sponsored by RISE UP ASIA LLC. All rights reserved."`.
- `bundle.shortDescription` & `bundle.longDescription`: Updated to reflect full branding.
- `bundle.windows.nsis.installerHooks`: Set to `"hooks.nsh"`.

### 2.2 NSIS Hook Layer (`src-tauri/hooks.nsh`)
During NSIS installation, the `NSIS_HOOK_POSTINSTALL` macro executes after all files and default registry keys are placed, directly stamping the canonical branding:

```nsis
!macro NSIS_HOOK_POSTINSTALL
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
    WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
!macroend
```

### 2.3 Standalone / Script Installer Layer (`install.ps1`)
When installed or updated via `install.ps1`:
- `$FullName` is set to `"Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"`.
- Function `Sync-InstalledAppsRegistryBranding` scans and updates all relevant uninstall registry keys (`HKCU` and `HKLM`) to ensure `DisplayName` and `Publisher` are fully aligned.

---

## 3. Verification & Compliance
- Registry inspection verifies that `HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\agm-alim` has `DisplayName` and `Publisher` set to `"Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"`.
- Zero disruption to `.github/workflows/*.yml` or binary asset naming pipelines.
