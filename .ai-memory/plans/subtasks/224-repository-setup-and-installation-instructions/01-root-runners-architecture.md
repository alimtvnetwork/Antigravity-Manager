# Subtask 01: Root Runners Architecture & Script Contracts Implementation

**Slug**: `224-repository-setup-and-installation-instructions`  
**Parent Spec**: `02-spec/21-app/224-repository-setup-and-installation-instructions/01-architecture-spec.md`  
**Master Ledger**: `02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md`  
**Status**: `DONE`  
**Owner**: Worker 02 (Runners & Build Specialist)  

---

## 1. Overview & Objective

This subtask implements the architectural contracts for the repository's root runners:
1. Authoring the dedicated production build runner: `build.ps1` (Windows) and `build.sh` (Linux/macOS).
2. Refactoring `run.ps1` and `run.sh` to delegate `-Build` / `--build` flags to the dedicated build scripts and harmonize toolchain self-healing.
3. Establishing topological prerequisite verification (`MSVC -> Rust/Cargo -> LLVM/Clang -> NASM -> Node.js/npm -> Frontend Packages`) and automatic in-session environment variable refresh.

---

## 2. Target File Manifest

| Target File | Status | Action Required |
| :--- | :--- | :--- |
| `build.ps1` | New File | Implement Windows dedicated production build runner with self-healing, sccache, and lld-link. |
| `build.sh` | New File | Implement Linux/macOS dedicated production build runner with toolchain self-healing. |
| `run.ps1` | Existing File | Refactor to offload build logic to `build.ps1`, synchronize toolchain detection, and preserve CLI routing. |
| `run.sh` | Existing File | Refactor to offload build logic to `build.sh` and synchronize toolchain checks. |

---

## 3. Step-by-Step Implementation Phases

### Phase 1: Parameter Specification & CLI Contract

#### 3.1 `build.ps1` Parameters
```powershell
param(
    [switch]$Release = $true,
    [switch]$Debug,
    [switch]$CliOnly,
    [switch]$FrontendOnly,
    [switch]$NoInstall,
    [string]$Drive = "C:",
    [switch]$Clean,
    [switch]$VerboseMode,
    [switch]$DryRun
)
```

#### 3.2 `build.sh` Flags
```bash
MODE="release"
CLI_ONLY=0
FRONTEND_ONLY=0
NO_INSTALL=0
CLEAN=0
VERBOSE=0
DRY_RUN=0
```

---

### Phase 2: Topological Toolchain Probe & Self-Healing Ladder

1. **Topological Probe Sequence**:
   - Check Node.js (`node -v`) and npm (`npm -v`).
   - Check Rust compiler (`rustc --version`) and Cargo (`cargo --version`).
   - On Windows:
     - Check MSVC Linker (`link.exe` via Visual Studio `vswhere.exe` or PATH).
     - Check LLVM Clang (`clang --version`) and `libclang.dll` at `$env:LIBCLANG_PATH`.
     - Check NASM (`nasm -v`).
2. **Self-Healing Trigger**:
   - If any required component is missing and `-NoInstall` is not set:
     - Display structured yellow warning: `[!] Build toolchain is incomplete. Launching automated installer...`
     - On Windows: Execute `powershell -ExecutionPolicy Bypass -File .\install.ps1 -BuildTools`
     - On Linux/macOS: Execute `bash ./install.sh --build-tools`
3. **In-Session Environment Refresh**:
   - Reload `$env:Path` from `HKCU\Environment` and `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`.
   - Ensure `$env:USERPROFILE\.cargo\bin` is injected into `$env:Path`.
   - Update `$env:LIBCLANG_PATH` dynamically.

---

### Phase 3: Compiler Optimizations & Build Dispatch

1. **Optimization Configuration**:
   - Detect CPU logical core count and set `$env:CARGO_BUILD_JOBS = [System.Environment]::ProcessorCount`.
   - Check for `sccache` in PATH; set `$env:RUSTC_WRAPPER = "sccache"` if present.
   - Check for `lld-link` (Windows); configure `$env:RUSTFLAGS = "-C link-arg=-fuse-ld=lld"`.
2. **Frontend Dependencies**:
   - Ensure `node_modules` exists; if missing, invoke `npm install --legacy-peer-deps`.
3. **Compilation Command**:
   - Full Tauri bundle: `npm run tauri build`
   - Standalone CLI: `cargo build --bin agm --release --manifest-path src-tauri/Cargo.toml`
   - Debug binary: `cargo build --bin agm --manifest-path src-tauri/Cargo.toml`

---

### Phase 4: Alignment of `run.ps1` and `run.sh`

1. When `run.ps1 -Build` is passed:
   ```powershell
   if ($Build) {
       $buildScript = Join-Path $ScriptDir "build.ps1"
       & $buildScript @PSBoundParameters
       exit $LASTEXITCODE
   }
   ```
2. When `run.sh --build` is passed:
   ```bash
   if [ "${MODE}" = "build" ]; then
       exec "${SCRIPT_DIR}/build.sh" "$@"
   fi
   ```
3. Retain existing CLI subcommand fast-routing (`agm ...`) without changes.

---

## 4. Acceptance Criteria & Guardrails

- [x] `build.ps1` runs successfully in non-interactive environments without hanging on user prompts.
- [x] If toolchains are missing, `build.ps1` automatically repairs the toolchain via `install.ps1 -BuildTools` and continues the build in the same session.
- [x] `run.ps1 -Build` cleanly delegates to `build.ps1`.
- [x] Zero hardcoded absolute filesystem paths (`D:\...`, `C:\...`) in code, specs, or logs.
- [x] Zero builds or test executions performed during specification authoring.
