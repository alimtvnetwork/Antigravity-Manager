# Subtask 03: Pure EXE Installer Hardening

> **Parent Plan:** `.ai-memory/plans/pending/27-single-menu-button-navbar-and-pure-exe-installer.md`
> **Target File:** `install.ps1`

## Objectives
1. Eliminate all references to `.zip` archives from `install.ps1`.
2. Target setup EXE exclusively (`*setup.exe`, `*.exe`).
3. Maintain silent installation with `/S` and `/D=$InstallDir`.
4. Run dry-run validation to verify clean execution without zip fallback.
