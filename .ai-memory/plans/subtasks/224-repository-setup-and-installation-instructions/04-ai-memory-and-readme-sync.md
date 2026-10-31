# Subtask 04: AI Memory, What-To-Read & Readme Synchronization

**Task ID:** `224`  
**Step ID:** `04`  
**Slug:** `224-repository-setup-and-installation-instructions`  
**Parent Plan:** [.ai-memory/plans/pending/224-repository-setup-and-installation-instructions.md](../../pending/224-repository-setup-and-installation-instructions.md)  
**Specification:** [02-spec/21-app/224-repository-setup-and-installation-instructions/04-memory-and-readme-spec.md](../../../02-spec/21-app/224-repository-setup-and-installation-instructions/04-memory-and-readme-spec.md)  
**Status:** `COMPLETED`  
**Owner:** Worker 03 (Documentation & AI Memory Specialist)  
**Date:** 2026-10-24  

---

## 1. Overview & Objective

This subtask executes the complete documentation, AI memory, and README synchronization for Task 224. It ensures that the root runner suite (`run.ps1` / `run.sh`, `build.ps1` / `build.sh`, `e2e.ps1` / `e2e.sh`, `install.ps1` / `install.sh`), the C: drive default toolchain invariant, and SQLite toolchain tracking (`.ai-memory/toolchain.db`) are thoroughly and authoritatively documented in `README.md`, `README_EN.md`, and `.ai-memory/what-to-read.md`.

---

## 2. Action Items & Execution Checklist

- [x] **1. Synchronize `README.md` (Primary Documentation)**:
  - [x] Add `## 🚀 Root Runner Suite & Developer Automation` section detailing:
    - `run.ps1` / `run.sh`: Hot-reload development server, Vite frontend server, CLI passthrough router (`agm ...`).
    - `build.ps1` / `build.sh`: Dedicated production compilation (`npm run tauri build` / `cargo build --release`) with automatic `install.ps1 -BuildTools` self-healing.
    - `e2e.ps1` / `e2e.sh`: Dedicated end-to-end testing harness with caller IDE self-preservation and sandboxed test isolation.
    - `install.ps1` / `install.sh`: Universal toolchain provisioner defaulting LLVM to C: drive with `.ai-memory/toolchain.db` verification tracking.
  - [x] Update Option A (Bar 2) version-based installation example from stale `v4.72.0` to canonical `v4.257.2`.
  - [x] Prepend recent verified stable releases (`v4.257.2`, `v4.257.1`, `v4.257.0`, `v4.256.4`) to the version pinning table (`<!-- STAMP:VERSION_PIN_TABLE_START -->`).

- [x] **2. Synchronize `README_EN.md` (English Documentation)**:
  - [x] Update title subtitle to `(v4.257.2)`.
  - [x] Update version badge to `Version-4.257.2-blue`.
  - [x] Add matching `## 🚀 Root Runner Suite & Developer Automation` section in clear American English.
  - [x] Update Option A (Bar 2) version-based installation commands from `v4.72.0` to `v4.257.2`.

- [x] **3. Update `.ai-memory/what-to-read.md` (AI Agent Onboarding Memory)**:
  - [x] Update header timestamp: `Last updated: 2026-10-24T17:00:00Z`.
  - [x] Prepend Task 224 changelog entry to `## Changelog`:
    - Explaining the root runner suite contracts (`run.ps1`, `build.ps1`, `e2e.ps1`, `install.ps1`).
    - Codifying the C: drive default toolchain unpack invariant (`C:\Program Files\LLVM` or `%LOCALAPPDATA%\Programs\LLVM`) with `-Drive` parameter override.
    - Documenting SQLite verification tracking in `.ai-memory/toolchain.db`.
  - [x] Add `02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md` and `.ai-memory/toolchain.db` under `## Before any task (always)`.

- [x] **4. Synchronize Plans & Memory Indices**:
  - [x] Update `.ai-memory/plans/pending/224-repository-setup-and-installation-instructions.md` with Task 224 documentation completion status.
  - [x] Update `00-master-audit-ledger.md` with step 04 completion status.

---

## 3. Detailed Documentation Text Templates

### 3.1 Template: Root Runner Suite Table for READMEs

```markdown
## 🚀 Root Runner Suite & Developer Automation

Antigravity Tools provides dedicated single-responsibility root runners for development, production compilation, end-to-end testing, and toolchain provisioning across Windows, Linux, and macOS:

| Script (Windows) | Script (POSIX / Bash) | Primary Responsibility | Auto Self-Healing? |
| :--- | :--- | :--- | :---: |
| **`run.ps1`** | **`run.sh`** | Hot-reload development server (`npm run tauri dev`), Vite frontend server, or CLI passthrough router (`agm ...`). | ✅ Yes |
| **`build.ps1`** | **`build.sh`** | Dedicated production compiler (`npm run tauri build` / `cargo build --release`). Never launches UI. | ✅ Yes (via `install.ps1 -BuildTools`) |
| **`e2e.ps1`** | **`e2e.sh`** | Dedicated end-to-end testing harness with caller IDE self-preservation and sandboxed test isolation. | N/A (Runs on prebuilt binary) |
| **`install.ps1`** | **`install.sh`** | Universal toolchain and application provisioner. Defaults LLVM and compilers to **C: drive**. | ✅ Native Provisioner |

#### Quick Runner Commands

```powershell
# 1. Launch development server with hot-reloading
.\run.ps1

# 2. Compile release package (auto-installs missing toolchains to C: drive)
.\build.ps1 -Release

# 3. Fast compile CLI binary only (agm.exe)
.\build.ps1 -CliOnly

# 4. Run end-to-end test suite with sandbox isolation
.\e2e.ps1 -Suite all

# 5. Full toolchain provisioning (MSVC, Rust, LLVM, NASM, Node.js)
.\install.ps1 -BuildTools -Drive "C:"
```
```

---

## 4. Acceptance Criteria & Verification Gates

1. **Bilingual Parity**: `README.md` and `README_EN.md` both contain the Root Runner Suite section, correct version badges (`v4.257.2`), and updated installation commands.
2. **AI Memory Invariants Recorded**: `.ai-memory/what-to-read.md` contains the authoritative changelog and "Before any task" entries.
3. **Strictly Relative Git Paths**: Zero absolute filesystem paths (`D:\...`, `C:\...`) are written to documentation files.
4. **Zero Builds / Zero Tests**: No build or test commands are executed during this documentation synchronization phase.
5. **Clean Diff**: File diffs show surgical additions and updates without unintended line-ending churn or whitespace regression.
