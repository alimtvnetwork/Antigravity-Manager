# Plan: Windows Add/Remove Programs Branding & v4.54.0 Release Synchronization

## Status: Completed
## Verification Score: 100/100
## Scope: Tauri Bundler NSIS Hooks, Windows Registry Branding, Manifest Synchronization, Version Bump to v4.54.0

---

## 1. Executive Summary & Root Cause Analysis

In Windows Settings > Apps > Installed apps ("Add or remove programs"), the application was displaying:
- **DisplayName**: `agm-alim`
- **Publisher**: `lbjlaq`

### Root Cause
1. In `src-tauri/tauri.conf.json`, `productName` was `"agm-alim"`. Tauri's default NSIS installer template uses `productName` as the Windows Registry `DisplayName`.
2. `bundle.publisher` was omitted in `tauri.conf.json`. In Tauri v2, when `publisher` is omitted, the bundler defaults the installer publisher metadata to the second token of `identifier` (`com.lbjlaq.antigravity-tools` -> `lbjlaq`).

---

## 2. Implemented Architecture & Solutions

### 2.1 Tauri NSIS Post-Install Hook (`src-tauri/hooks.nsh`)
Injected an NSIS macro that executes at the conclusion of installation:
```nsis
!macro NSIS_HOOK_POSTINSTALL
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
    WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
!macroend
```
Connected via `src-tauri/tauri.conf.json`:
- `bundle.publisher`: `"Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"`
- `bundle.copyright`: `"Copyright © 2026 MD Alim Ul Karim. Sponsored by RISE UP ASIA LLC. All rights reserved."`
- `bundle.windows.nsis.installerHooks`: `"hooks.nsh"`

### 2.2 Version & Manifest Synchronization (`v4.54.0`)
Synchronized across all manifests:
- `src-tauri/Cargo.toml`: `version = "4.54.0"`, `authors`, `description`
- `src-tauri/tauri.conf.json`: `version = "4.54.0"`, window title, short/long descriptions
- `package.json`: `version = "4.54.0"`, `author`, `description`
- `version.json`: `Version = "4.54.0"`, `version = "4.54.0"`, `FullName`, `Title`, `releaseDate`
- `index.html`: Window title updated
- `src-tauri/src/modules/email_sender.rs`: Footer attribution updated

### 2.3 Installer Script Alignment (`install.ps1` & `install.sh`)
- `install.ps1`: `$FullName` updated; previous version detection regexes updated; added `Sync-InstalledAppsRegistryBranding` function to verify and set registry branding in `HKCU` and `HKLM`.
- `install.sh`: `FULL_NAME` updated.

### 2.4 Live System Registry Synchronization
- Directly updated `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\agm-alim` and `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Antigravity Tools` on the live Windows host.
- Both `DisplayName` and `Publisher` now display: `"Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"`.

---

## 3. Specifications Created
- `02-spec/20-instance-management/05-smart-profile-multiplicative-scoring-spec.md`
- `02-spec/15-distribution-and-runner/07-windows-installer-and-add-remove-programs-branding.md`
