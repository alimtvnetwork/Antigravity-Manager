---
name: agm-installer-distribution
description: Specialized skill for managing cross-platform installers (install.ps1, install.sh), multi-version fallback ladder, aria2c acceleration, NSIS branding, Homebrew Casks, and release distribution in Antigravity-Manager.
---

# AGM Installer & Distribution Architecture

This skill provides comprehensive architectural guidance, fallback strategies, packaging scripts, and release distribution procedures for Antigravity-Manager across Windows, macOS, and Linux platforms.

---

## 1. Subsystem Architecture Overview

The installation and distribution toolchain ensures zero-friction portable deployments with multi-tier redundancy:

```
+-------------------------------------------------------------------------+
|                       GitHub Releases & CDN Mirrors                     |
|  - alimtvnetwork/Antigravity-Manager / gh-proxy / fastgit               |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                     Decoupled Release Asset Ladder                      |
|  - releases-manifest.json: 10-release fallback ladder                   |
|  - Standalone installers fetch assets without hardcoded single links    |
+------------------------------------+------------------------------------+
           |                                              |
           v                                              v
+------------------------------------+ +----------------------------------+
|      Windows: install.ps1          | |      Linux / macOS: install.sh   |
|  - 10-version fallback ladder      | |  - Architecture auto-detection   |
|  - aria2c 16-conn acceleration     | |  - deb / appimage / tarball      |
|  - NSIS hooks.nsh DisplayName vX.Y | |  - Systemd / desktop shortcuts   |
|  - Desktop & Start Menu healing    | |  - Non-root user auto-switch     |
+------------------------------------+ +----------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                  Homebrew Cask (Casks/antigravity-tools.rb)             |
|  - macOS brew install --cask antigravity-tools                          |
|  - Automated SHA256 checksum updates via release scripts                |
+-------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `install.ps1` | Standalone Windows installer: downloads latest or fallback releases, handles aria2c acceleration, creates shortcuts, configures PATH. |
| `install.sh` | Standalone Linux/macOS installer: detects platform/arch (`x86_64`, `aarch64`), handles permissions, installs desktop entries. |
| `releases-manifest.json` | JSON manifest detailing asset URLs, versions, SHA256 hashes, and fallback release ladders. |
| `Casks/antigravity-tools.rb` | Homebrew Cask formula for macOS distribution. |
| `src-tauri/hooks.nsh` | NSIS installer script: Windows registry keys, Add/Remove Programs DisplayName with version, install paths. |
| `03-ai-scripts/29-release-orchestrator.py` | Automated release preparation: synchronizes manifests, updates changelogs, builds tags. |
| `03-ai-scripts/watch-release-notes.ps1` | GitHub Actions release notes formatting watchdog: reapplies split release notes upon workflow completion. |
| `version.json` | Single source of truth for versioning across all manifests. |

---

## 3. The 10-Version Multi-Tier Fallback Ladder

To prevent installation failures when a release asset is temporarily missing, rate-limited, or being published:
1. `install.ps1` and `install.sh` query `releases-manifest.json` or iterate over an array of 10 recent release versions (e.g. `4.69.0`, `4.68.0`, `4.67.0`, ...).
2. For each version, download attempts test multiple mirrors:
   - Primary: GitHub Releases direct link.
   - Secondary: Accelerated CDN mirrors.
3. If the latest tag 404s (e.g. during an in-flight GitHub Actions build), the installer seamlessly falls back to the previous verified release without exiting in failure.

---

## 4. Accelerated Multi-Connection Downloading (`aria2c`)

- Both `install.ps1` and `install.sh` check for `aria2c`.
- If available, download is executed with multi-connection acceleration (`-x 16 -s 16 -k 1M -j 16`) in quiet mode (`--quiet=true`).
- If `aria2c` is not installed, the scripts fall back seamlessly to `Invoke-WebRequest` / `curl` / `wget`.

---

## 5. Windows Add/Remove Programs DisplayName Synchronization

In `src-tauri/hooks.nsh` and `install.ps1`:
- The registry key `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Antigravity Manager` must set:
  - `DisplayName`: `"Antigravity Manager Tools v<version>"`
  - `DisplayVersion`: `"<version>"`
- This ensures users can unambiguously identify the installed version in Windows Settings.
