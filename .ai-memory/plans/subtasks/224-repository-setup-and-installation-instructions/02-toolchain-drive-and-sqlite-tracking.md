# Subtask 02: Toolchain Drive Defaulting (C: Drive) & SQLite Tracking Engine

- **Parent Task:** [224-repository-setup-and-installation-instructions](../../pending/224-repository-setup-and-installation-instructions.md)
- **Specification:** [02-spec/21-app/224-repository-setup-and-installation-instructions/02-component-spec.md](../../../02-spec/21-app/224-repository-setup-and-installation-instructions/02-component-spec.md)
- **Master Ledger:** [02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md](../../../02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md)
- **Status:** `COMPLETED`
- **Owner:** Worker 01 (Toolchain & SQLite Specialist)
- **Date:** 2026-10-24

---

## 1. Overview & Objective

This subtask implements the technical requirements specified in `02-spec/21-app/224-repository-setup-and-installation-instructions/02-component-spec.md`:
1. **Eliminate Unconditional `Test-Path "D:\"`:** Remediate `scripts/install-rust-toolchain.ps1` so that toolchain downloads, MSVC BuildTools, and LLVM extract to the system `C:` drive by default (`C:\Program Files\LLVM` or `C:\LLVM`), instead of forcibly choosing `D:\` whenever that drive exists.
2. **Explicit Drive Parameter (`-Drive`):** Add support for `-Drive <Letter>` (supporting `C`, `C:`, `D`, `D:`) and environment variable `$env:AGM_TOOLCHAIN_DRIVE`.
3. **SQLite Toolchain Tracking Engine (`toolchain.db`):** Implement `scripts/toolchain_db.py` and `scripts/toolchain-db.ps1` to track toolchain state, versions, installation drive, and verification status with affirmative boolean `IsVerified` and FIFO ordering.
4. **Auto-Marking & Sub-5ms Fast Verification:** Integrate auto-marking upon toolchain installation and fast verification preflight in `scripts/install-rust-toolchain.ps1`, `install.ps1`, and `run.ps1`.

---

## 2. Implementation Work Breakdown Structure (WBS)

### Step 2.1: Implement Drive Resolution & Normalization
- **File:** `scripts/install-rust-toolchain.ps1`
- **Actions:**
  1. Add `[string]$Drive = ""` parameter to `param(...)` block.
  2. Implement `Resolve-ToolchainDrive` function:
     - Check `$Drive` parameter.
     - Fall back to `$env:AGM_TOOLCHAIN_DRIVE`.
     - Default to `$env:SystemDrive` (or `"C:"`).
     - Normalize to uppercase with trailing colon (`"C:"` or `"D:"`).
  3. Replace line 162 in `Get-ToolchainCacheDir`:
     ```powershell
     # Replace:
     # $target = if (Test-Path "D:\") { "D:\agm-toolchain-cache" } else { Join-Path $env:LOCALAPPDATA "agm-toolchain-cache" }
     # With:
     $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive
     $target = if ($targetDrive -eq "D:" -and (Test-Path "D:\")) {
         "D:\agm-toolchain-cache"
     } else {
         Join-Path $env:LOCALAPPDATA "agm-toolchain-cache"
     }
     ```
  4. Replace line 333 in `Install-MsvcItem`:
     ```powershell
     # Replace:
     # $targetInstallPath = if (Test-Path "D:\") { "D:\BuildTools" } else { "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" }
     # With:
     $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive
     $targetInstallPath = if ($targetDrive -eq "D:") {
         "D:\BuildTools"
     } else {
         "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools"
     }
     ```
  5. Replace lines 440-441 in `Install-LlvmItem`:
     ```powershell
     # Replace:
     # $targetUnpack = if (Test-Path "D:\") { "D:\LLVM" } else { "$env:ProgramFiles\LLVM" }
     # With:
     $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive
     $targetUnpack = if ($targetDrive -eq "D:") {
         "D:\LLVM"
     } else {
         "$env:ProgramFiles\LLVM"
     }
     ```
  6. Update `$candidates` array in `Install-LlvmItem` (around lines 405-418):
     - Dynamically prepend C: candidates (`$env:ProgramFiles\LLVM\bin`, `C:\Program Files\LLVM\bin`, `C:\Program Files (x86)\LLVM\bin`, `C:\LLVM\bin`) when drive preference is `C:`.

---

### Step 2.2: Implement `scripts/toolchain_db.py`
- **File:** `scripts/toolchain_db.py`
- **Dependencies:** Pure Python 3 standard library (`sqlite3`, `argparse`, `pathlib`, `datetime`, `os`, `sys`).
- **Features:**
  1. Default database location: Join-Path `$env:LOCALAPPDATA` `"agm-toolchain\toolchain.db"`, overrideable via `--db` or `$env:AGM_TOOLCHAIN_DB`.
  2. Table DDL:
     ```sql
     CREATE TABLE IF NOT EXISTS ToolchainInstallation (
         ToolchainInstallationId INTEGER PRIMARY KEY AUTOINCREMENT,
         ToolchainName           TEXT NOT NULL,
         Version                 TEXT NOT NULL,
         InstalledPath           TEXT NOT NULL,
         InstallDrive            TEXT NOT NULL,
         InstalledAt             TEXT NOT NULL,
         IsVerified              INTEGER NOT NULL DEFAULT 0,
         LastVerifiedAt          TEXT NOT NULL,
         CONSTRAINT UQ_ToolchainInstallation_Name_Path UNIQUE (ToolchainName, InstalledPath)
     );
     ```
  3. Commands:
     - `init`: Create schema and indices.
     - `upsert`: Insert or update toolchain record, updating `LastVerifiedAt` and `IsVerified`.
     - `verify`: Test disk presence and verified flag; output JSON `{ "status": "VERIFIED", ... }`.
     - `status`: Output tabular or JSON summary of all registered toolchains ordered by `InstalledAt ASC`.
     - `check-all`: Verify essential suite (`rust`, `msvc`, `llvm`, `nasm`). Return exit code 0 if all verified, 1 if any missing.

---

### Step 2.3: Implement PowerShell Helper `scripts/toolchain-db.ps1`
- **File:** `scripts/toolchain-db.ps1`
- **Actions:**
  1. Define cmdlets:
     - `Get-ToolchainDbPath`
     - `Initialize-ToolchainDb`
     - `Set-ToolchainRecord`
     - `Get-ToolchainRecord`
     - `Test-ToolchainDbFast`
  2. Connect to `scripts/toolchain_db.py` CLI or direct SQLite query bridge.
  3. Support non-blocking reads and safe concurrent execution.

---

### Step 2.4: Integrate Auto-Marking in `scripts/install-rust-toolchain.ps1`
- **File:** `scripts/install-rust-toolchain.ps1`
- **Actions:**
  1. Dot-source `scripts/toolchain-db.ps1` (or embed lightweight wrapper functions).
  2. In `Install-MsvcItem`:
     - Once `vswhere` confirms MSVC tools, call `Set-ToolchainRecord -ToolchainName "msvc" -Version $vsVer -InstalledPath $vsPath -InstallDrive $targetDrive -IsVerified $true`.
  3. In `Install-RustItem`:
     - Once `rustup` components are confirmed, call `Set-ToolchainRecord -ToolchainName "rust" -Version $rustcVer -InstalledPath $cargoBin -InstallDrive "C:" -IsVerified $true`.
  4. In `Install-LlvmItem`:
     - Once extraction completes and `libclang.dll` is verified, call `Set-ToolchainRecord -ToolchainName "llvm" -Version "23.1.4" -InstalledPath $binDir -InstallDrive $targetDrive -IsVerified $true`.
  5. In `Install-NasmItem`:
     - Once NASM is confirmed, call `Set-ToolchainRecord -ToolchainName "nasm" -Version $nVer -InstalledPath $nasmPath -InstallDrive $targetDrive -IsVerified $true`.

---

### Step 2.5: Fast Verification Preflight Integration
- **Files:** `scripts/install-rust-toolchain.ps1`, `install.ps1`, `run.ps1`
- **Actions:**
  1. Update `VerifyOnly` path in `scripts/install-rust-toolchain.ps1`:
     - Check `toolchain.db` first.
     - If all 4 core components (`rust`, `msvc`, `llvm`, `nasm`) are marked `IsVerified = 1` and paths exist on disk, report instant pass (<5ms).
  2. Add `-Drive` pass-through to `install.ps1`:
     - Allow callers to run `.\install.ps1 -BuildTools -Drive D` or `.\install.ps1 -BuildTools -Drive C`.
  3. Add fast-path check in `run.ps1`:
     - Inspect `toolchain.db` before triggering full toolchain dependency checks.

---

## 3. Verification & Acceptance Gate

| Checkpoint | Verification Method | Expected Result | Status |
| :--- | :--- | :--- | :--- |
| **1. Drive Defaulting** | Code inspection of lines 162, 333, 440 | Unconditional `Test-Path "D:\"` eliminated; defaults to `C:` drive. | PASSED |
| **2. Drive Override** | Invoke with `-Drive D` in dry-run mode | Resolves target unpack to `D:\LLVM` and `D:\BuildTools`. | PASSED |
| **3. Schema Compliance** | Inspect `ToolchainInstallation` DDL | PascalCase table name, `IsVerified` affirmative boolean, FIFO view. | PASSED |
| **4. Python Helper Syntax** | `python -m py_compile scripts/toolchain_db.py` | Exit code 0, zero syntax errors. | PASSED |
| **5. PowerShell Syntax** | `pwsh -NoProfile -Command "Get-Command scripts\toolchain-db.ps1"` | Valid parsing, zero script errors. | PASSED |
| **6. Relative Path Invariant** | Audit touched files with `gitmap aum search` | Zero absolute workspace paths. | PASSED |
| **7. Zero-Build Discipline** | Process observation | Zero `tauri build` or `cargo build` commands executed. | PASSED |
