# Subtask 01: Rust Toolchain Installation, Windows MSVC Verification & Build Diagnostics

**Slug:** `rust-build-toolchain-installation-and-release`  
**File:** `.ai-memory/plans/subtasks/rust-build-toolchain-installation-and-release/01-toolchain-and-build-diagnostics.md`  
**Target Components:** `scripts/install-rust-toolchain.ps1`, `src-tauri/Cargo.toml`, `src-tauri/`  
**Related Specs:** `02-spec/21-app/rust-build-toolchain-installation-and-release/01-architecture-spec.md`  
**Status:** PENDING EXECUTION & VERIFICATION  
**Lead Author:** Antigravity Architect (Spec 01)  

---

## 1. Objective

Execute the automated Rust build toolchain installation script fixture on Windows x86_64, verify compiler and toolchain versions, execute compilation diagnostics across all workspace targets in `src-tauri/`, and collect structured verification evidence.

Specifically, this plan covers:
1. Executing `scripts/install-rust-toolchain.ps1` with the targeted `-Only rust` parameter.
2. Synchronizing the active PowerShell environment session PATH to immediately expose `.cargo\bin`.
3. Validating installation of `rustc`, `cargo`, `clippy`, and `rustfmt`.
4. Executing `cargo check --all-targets` in `src-tauri/` to verify compiler health across `antigravity_tools_lib`, `agm-alim`, and `agm`.
5. Enforcing repository invariants: strict relative git paths, affirmative booleans, and zero bare unwraps in production code.

---

## 2. Target Files & Artifacts

All file references are strictly relative to the git repository root:
- `scripts/install-rust-toolchain.ps1`: Automated Windows toolchain installation fixture.
- `src-tauri/Cargo.toml`: Package manifest for `agm-alim` and `antigravity_tools_lib`.
- `src-tauri/src/main.rs`: Desktop GUI binary entry point (`agm-alim`).
- `src-tauri/src/bin/agm.rs`: Standalone terminal CLI suite entry point (`agm`).
- `src-tauri/src/lib.rs`: Shared engine library crate root (`antigravity_tools_lib`).

---

## 3. Step-by-Step Instructions: Installation Script Fixture Execution

### Step 3.1: Pre-Installation Environment Assessment

Before triggering installer changes, evaluate the host system to determine whether a Rust toolchain is already present and if the active PATH contains Cargo binaries:

```powershell
# Check command resolution for rustc and cargo
$hasRustc = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
$hasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)
Write-Host "Pre-check: has_rustc=$hasRustc, has_cargo=$hasCargo"

# If already present, inspect active versions
if ($hasRustc) { rustc --version }
if ($hasCargo) { cargo --version }
```

### Step 3.2: Dry-Run Inspection (Optional Validation)

To verify parameter parsing and catalog resolution without making disk or system modifications, execute a dry-run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-rust-toolchain.ps1 -Only rust -DryRun
```

Expected dry-run output indicates that only the `rust` item is resolved in the execution sequence:
```text
Item 'rust': Rust toolchain via rustup (stable MSVC) + clippy/rustfmt components
```

### Step 3.3: Executing the Toolchain Installation Fixture

Execute the installation fixture targeting only the Rust toolchain:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/install-rust-toolchain.ps1 -Only rust
```

The script performs the following automated steps:
1. Verifies existing `rustc` and `cargo` installations.
2. Downloads `rustup-init.exe` from `https://win.rustup.rs/x86_64` to `$env:TEMP\rustup-init.exe` if absent or if `-Force` is supplied.
3. Invokes `rustup-init.exe -y --default-toolchain stable-x86_64-pc-windows-msvc`.
4. Adds the required analysis components: `rustup component add clippy rustfmt`.
5. Automatically removes temporary installer binaries.
6. Invokes `Refresh-SessionPath` to update `$env:Path`.

### Step 3.4: In-Process Environment Session PATH Refresh

If running across different PowerShell sessions or custom runners, manually refresh the active session's PATH environment to ensure immediate access without restarting the shell:

```powershell
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$sysPath  = [Environment]::GetEnvironmentVariable("Path", "Machine")
$env:Path = "$sysPath;$userPath"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
    if ($env:Path -notlike "*$cargoBin*") {
        $env:Path = "$cargoBin;$env:Path"
    }
}
Write-Host "PATH refreshed. Testing cargo resolution: $([bool](Get-Command cargo -ErrorAction SilentlyContinue))"
```

---

## 4. Toolchain Binary Verification Commands

Verify that all essential compiler binaries and linter components respond with valid version signatures and target the Windows MSVC ABI.

### 4.1 Verification Commands Table

| Tool | Verification Command | Expected Output Pattern | Purpose |
|:---|:---|:---|:---|
| **Rust Compiler** | `rustc --version` | `rustc <version> (<hash> <date>)` | Validates compiler binary availability. |
| **Cargo Manager** | `cargo --version` | `cargo <version> (<hash> <date>)` | Validates build tool availability. |
| **Active Target** | `rustc -vV` | `host: x86_64-pc-windows-msvc` | Validates MSVC ABI target architecture. |
| **Clippy Linter** | `cargo clippy --version` | `clippy <version> (<hash> <date>)` | Validates static analysis tool availability. |
| **Code Formatter** | `rustfmt --version` | `rustfmt <version> (<hash> <date>)` | Validates style formatting tool availability. |
| **Installed Components**| `rustup component list --installed` | Contains `rustc`, `cargo`, `clippy`, `rustfmt` | Validates complete component suite. |

### 4.2 Automated Verification Script

Run the following consolidated PowerShell block to produce structured JSON evidence:

```powershell
$evidence = @{
    has_rustc        = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
    has_cargo        = [bool](Get-Command cargo -ErrorAction SilentlyContinue)
    rustc_version    = (& rustc --version 2>&1).ToString().Trim()
    cargo_version    = (& cargo --version 2>&1).ToString().Trim()
    clippy_version   = (& cargo clippy --version 2>&1).ToString().Trim()
    rustfmt_version  = (& rustfmt --version 2>&1).ToString().Trim()
    is_msvc_target   = ((& rustc -vV 2>&1) -match "x86_64-pc-windows-msvc")
    timestamp_utc    = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
}
$evidence | ConvertTo-Json -Depth 3
```

---

## 5. Compilation Diagnostics: `cargo check --all-targets`

To ensure full workspace build integrity, execute compilation diagnostics in `src-tauri/`.

### 5.1 Command Execution in `src-tauri/`

```powershell
# Navigate to the Rust workspace directory
cd src-tauri

# Execute compilation check across library, binaries, and test harnesses
cargo check --all-targets
```

### 5.2 Expected Targets Checked

The execution must validate three primary compilation artifacts:
1. `antigravity_tools_lib`: Shared library containing proxy, storage, and auto-switcher logic.
2. `agm-alim` (`src/main.rs`): Desktop GUI application binary.
3. `agm` (`src/bin/agm.rs`): Native terminal CLI application binary.

### 5.3 Full Pre-Flight Gate Verification

Prior to release builds or PR submissions, execute the complete pre-flight gate sequence:

```powershell
# 1. Check code formatting
cargo fmt -- --check

# 2. Comprehensive Clippy gate with all features enabled
cargo clippy --all-targets --all-features

# 3. Clean compilation check
cargo check --all-targets
```

### 5.4 Diagnostic Failure Remediation Workflow

If `cargo check --all-targets` fails with compiler errors:
1. Capture the exact error diagnostic from stderr (e.g. `error[E0308]: mismatched types`).
2. Identify the file and line number (using relative git paths, e.g. `src-tauri/src/modules/proxy_pool.rs:184`).
3. Formulate a 4-part Root Cause Analysis (RCA):
   - **Root Cause**: Specific type, borrow, or trait violation.
   - **Key Indicator**: Exact compiler message and code snippet.
   - **Resolution**: Surgical edit applied.
   - **Prevention**: Invariant enforced to avoid regression.
4. Re-run `cargo check --all-targets` until exit code is 0.

---

## 6. Acceptance Criteria & Evidence Collection Requirements

### 6.1 Acceptance Criteria (AC)

| Criterion ID | Condition | Evaluation Method | Target Value |
|:---|:---|:---|:---|
| **AC-1** | `has_rustc` | `[bool](Get-Command rustc)` | `true` |
| **AC-2** | `has_cargo` | `[bool](Get-Command cargo)` | `true` |
| **AC-3** | `is_msvc_target` | `rustc -vV \| Select-String "x86_64-pc-windows-msvc"` | `true` |
| **AC-4** | `has_clippy` | `cargo clippy --version` | Exit code 0 |
| **AC-5** | `has_rustfmt` | `rustfmt --version` | Exit code 0 |
| **AC-6** | `is_check_clean` | `cd src-tauri && cargo check --all-targets` | Exit code 0 |
| **AC-7** | `has_relative_paths` | Review all doc/script paths | Strict relative to repo root |
| **AC-8** | `has_affirmative_booleans` | Verify boolean naming | Affirmative (`is_*`, `has_*`) |

### 6.2 Evidence Collection Protocol

During task execution, the executor must record:
1. Raw stdout and stderr output from `scripts/install-rust-toolchain.ps1 -Only rust`.
2. Output of `rustc --version` and `cargo --version`.
3. Complete output and exit code of `cargo check --all-targets` run inside `src-tauri`.
4. Confirmation that all documentation paths use strict relative formatting.
