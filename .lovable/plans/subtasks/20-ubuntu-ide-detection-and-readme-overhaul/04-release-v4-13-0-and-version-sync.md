# Subtask 04: Release v4.13.0 and Version Synchronization

> **Parent Plan:** [.lovable/plans/pending/20-ubuntu-ide-detection-and-readme-overhaul.md](../../pending/20-ubuntu-ide-detection-and-readme-overhaul.md)
> **Target Files:** `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `CHANGELOG.md`
> **Status:** Completed

---

## Scope & Purpose

Synchronize project version to `4.13.0` across all metadata manifests, update changelog, prepare release commit, and tag `v4.13.0` for CI/CD automated deployment.

---

## Requirements

1. **Version Manifests**:
   - `package.json`: `"version": "4.13.0"`
   - `src-tauri/Cargo.toml`: `version = "4.13.0"`
   - `src-tauri/tauri.conf.json`: `"version": "4.13.0"`

2. **Changelog Update (`CHANGELOG.md`)**:
   - Document `v4.13.0` release notes highlighting:
     - Ubuntu IDE auto-detection via `which`, Snap, Flatpak, and desktop entries.
     - Storage JSON auto-healing and initial synthesis on Ubuntu/Linux.
     - Full Error Management Architecture and AI-sharable Compact Markdown Error Modal.
     - Root README English-only overhaul with Animal Experiments and Rivera attribution.

3. **Release Execution**:
   - Atomic commit, git push, and tag creation `v4.13.0`.
