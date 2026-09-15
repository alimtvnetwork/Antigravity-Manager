# Plan 16: Repository Folder Structure, Installers & v4.9.0 Release — Consolidated

> **Header:** Started from user instruction to align folder structure to coding guidelines (`01-prompts/`, `02-spec/21-app/`), create standalone portable installers (`install.ps1`, `install.sh`), fix CI/CD, and bump minor version to 4.9.0 for full release execution.
> **Loops to Complete:** 4 micro-tasks completed in 1 continuous self-loop turn.
> **Completed Date:** 2026-09-15
> **Status:** COMPLETED

---

## 1. Accomplishments & Structural Alignments

1. **Folder Structure Standardization**:
   - Realigned repository root: installed canonical `01-prompts/` library (21 modular categories).
   - Migrated all 8 technical guides from temporary `01-instructions/` into `02-spec/21-app/` (`08-` through `14-`).
   - Removed `01-instructions/`.
   - Updated `02-spec/21-app/01-index.md` to index all 14 application specifications.
   - Updated `02-spec/folder-structure-root.md` and `.lovable/folder-structure.md`.

2. **Standalone Portable Installer Architecture**:
   - Replaced heavy installer execution with clean portable extraction in `install.ps1`:
     - Downloads `Antigravity.Tools_${VERSION}_windows_x64.zip` directly from GitHub releases.
     - Extracts directly into `$env:LOCALAPPDATA\Programs\Antigravity-Tools` without wizard prompts or admin elevation.
     - Adds installation directory to User PATH.
     - Creates Start Menu and Desktop shortcuts.
     - Supports `-Version`, `-InstallDir`, `-Arch`, `-NoPath`, `-NoShortcut`, `-DryRun`, and `-Uninstall`.
   - Updated `install.sh` for Linux and macOS with upstream fallback, multi-architecture detection, and portable deployment to `~/.local/bin`.

3. **CI/CD & Release Workflow Upgrades**:
   - Added `workflow_dispatch:` to `.github/workflows/ci.yml` for manual/CLI dispatch.
   - Added portable ZIP packaging step in `.github/workflows/release.yml` on Windows runner (`Antigravity.Tools_${VERSION}_windows_x64.zip`).
   - Added `*.zip` collection and automatic SHA-256 `checksums.txt` generation in `Collect release files and generate checksums`.

4. **Minor Version Bump (v4.9.0)**:
   - Version advanced from `4.8.1` to `4.9.0` across:
     - `version.json`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `Casks/antigravity-tools.rb`, `README_EN.md`, `readme.md`, `CHANGELOG.md`, `CHANGELOG_EN.md`, `src/pages/Settings.tsx`, and `src/components/layout/MiniView.tsx`.
