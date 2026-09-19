---
name: ubuntu-installer-hardening
description: Autonomously hardens install.sh with aria2c parallel download acceleration, safe uninstallation order, migration path UI, and temporary CI/CD verification.
---

# Ubuntu Installer Hardening & Temporary CI/CD Protocol

This skill guides the automated hardening and validation of `install.sh`:

## 1. Core Principles
- **Accelerated Download:** Prioritize `aria2c` with 16 parallel split connections (`-x 16 -s 16 -k 1M`) with graceful fallback to `curl`.
- **Safe Lifecycle Order:** Never uninstall or remove old tools until the new package has been completely downloaded and verified in the temporary directory.
- **Cleanup Guarantee:** Always register `trap cleanup EXIT INT TERM` to delete temporary files.
- **Polished CLI UI:** Enforce top padding, 4-space left padding, and migration progression display (`Current -> Target`).
- **Temporary CI/CD Verification:** When adding temporary E2E tests in CI/CD, verify their execution, then remove them in the subsequent turn to avoid bloating pipeline duration.
