<#
.SYNOPSIS
    Antigravity-Manager: Dedicated Production Build Runner (Windows)

.DESCRIPTION
    Compiles production-ready Tauri desktop bundles and CLI binaries. Checks
    prerequisites (MSVC, Rust/Cargo, LLVM/Clang, NASM, Node.js/npm) in topological
    order, automatically invoking .\install.ps1 -BuildTools if missing. Configures
    parallel worker threads, sccache, and lld-link fast linker.

.PARAMETER Release
    Compile production-optimized release binaries and installers. Default.

.PARAMETER Debug
    Compile unoptimized debug binary with debug symbols.

.PARAMETER CliOnly
    Compile only the standalone CLI binary (agm) via Cargo.

.PARAMETER FrontendOnly
    Compile only the Vite web application frontend (npm run build).

.PARAMETER NoInstall
    Fail immediately if toolchains are missing instead of auto-installing.

.PARAMETER Drive
    Target drive for toolchain installation if self-healing is triggered. Default: "C:"

.PARAMETER Clean
    Perform cargo clean and purge build output directories before compiling.

.PARAMETER VerboseMode
    Output verbose compilation and diagnostic telemetry.

.PARAMETER DryRun
    Validate prerequisites and print build commands without executing them.

.EXAMPLE
    .\build.ps1
    .\build.ps1 -CliOnly
    .\build.ps1 -Debug
    .\build.ps1 -Clean
    .\build.ps1 -DryRun
#>

param(
    [switch]$Release = $true,
    [switch]$Build,
    [switch]$Debug,
    [switch]$CliOnly,
    [switch]$FrontendOnly,
    [switch]$NoInstall,
    [string]$Drive = "C:",
    [switch]$Clean,
    [switch]$VerboseMode,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "  [*] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "  [OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "  [!] $Message" -ForegroundColor Yellow
}

function Write-Err {
    param([string]$Message)
    Write-Host "  [ERROR] $Message" -ForegroundColor Red
}

$ScriptDir = $PSScriptRoot

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "            Antigravity-Manager Build Runner                " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$targetMode = if ($Debug) { "debug" } else { "release" }

# In-session environment refresh
function Update-SessionEnvironment {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $env:Path = "$userPath;$machinePath"

    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if ((Test-Path $cargoBin) -and ($env:Path -notlike "*$cargoBin*")) {
        $env:Path = "$cargoBin;$env:Path"
    }

    $userLibClang = [Environment]::GetEnvironmentVariable("LIBCLANG_PATH", "User")
    if ($userLibClang -and (Test-Path $userLibClang)) {
        $env:LIBCLANG_PATH = $userLibClang
    }

    $chocoBin = "C:\ProgramData\chocolatey\bin"
    if ((Test-Path $chocoBin) -and ($env:Path -notlike "*$chocoBin*")) {
        $env:Path = "$chocoBin;$env:Path"
    }
}

# 1. Check prerequisites in topological order
Update-SessionEnvironment

$toolchainDbHelper = Join-Path $ScriptDir "scripts\toolchain-db.ps1"
$dbVerified = $false
if (Test-Path $toolchainDbHelper) {
    try {
        . $toolchainDbHelper
        if (Get-Command Test-ToolchainDbAllVerified -ErrorAction SilentlyContinue) {
            $dbVerified = Test-ToolchainDbAllVerified
        }
    } catch {
        $dbVerified = $false
    }
}

$missingPrereqs = @()

if (-not $dbVerified) {
    Write-Step "Checking Node.js & npm..."
    $hasNode = [bool](Get-Command node -ErrorAction SilentlyContinue)
    $hasNpm = [bool](Get-Command npm -ErrorAction SilentlyContinue)
    if (-not $hasNode -or -not $hasNpm) {
        $missingPrereqs += "nodejs"
        Write-Warn "Node.js or npm is missing."
    } else {
        Write-Success "Node.js: $(node -v) | npm: v$(npm -v)"
    }

    if (-not $FrontendOnly) {
        Write-Step "Checking Rust & Cargo toolchain..."
        $hasRustc = $false
        $hasCargo = $false
        $rustcVer = ""
        try {
            $rawRustc = (& rustc --version 2>&1 | Out-String).Trim()
            if ($rawRustc -match 'rustc \d+\.\d+') {
                $hasRustc = $true
                $rustcVer = $rawRustc
            }
        } catch {
            $hasRustc = $false
        }

        try {
            $rawCargo = (& cargo --version 2>&1 | Out-String).Trim()
            if ($rawCargo -match 'cargo \d+\.\d+') {
                $hasCargo = $true
            }
        } catch {
            $hasCargo = $false
        }

        if (-not $hasRustc -or -not $hasCargo) {
            $missingPrereqs += "rust"
            Write-Warn "Rust compiler (rustc) or Cargo is not operational or default toolchain is not set."
        } else {
            Write-Success "Rust: $rustcVer"
        }

        Write-Step "Checking MSVC Linker (link.exe)..."
        $hasLink = [bool](Get-Command link -ErrorAction SilentlyContinue)
        if (-not $hasLink) {
            $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
            if (Test-Path $vswhere) {
                $vsInstall = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
                if ($vsInstall -and (Test-Path $vsInstall)) {
                    $hasLink = $true
                }
            }
        }
        if (-not $hasLink) {
            $missingPrereqs += "msvc"
            Write-Warn "MSVC C++ Build Tools (link.exe) not found."
        } else {
            Write-Success "MSVC Linker verified."
        }

        Write-Step "Checking LLVM & Clang (libclang.dll)..."
        $hasClang = [bool](Get-Command clang -ErrorAction SilentlyContinue)
        $hasLibClang = [bool]($env:LIBCLANG_PATH -and (Test-Path (Join-Path $env:LIBCLANG_PATH "libclang.dll")))
        if (-not $hasLibClang) {
            $llvmCandidates = @(
                "C:\Program Files\LLVM\bin\libclang.dll",
                "$env:ProgramFiles\LLVM\bin\libclang.dll",
                "$env:LOCALAPPDATA\Programs\LLVM\bin\libclang.dll",
                "C:\LLVM\bin\libclang.dll",
                "D:\LLVM\bin\libclang.dll"
            )
            foreach ($cand in $llvmCandidates) {
                if (Test-Path $cand) {
                    $hasLibClang = $true
                    $env:LIBCLANG_PATH = Split-Path -Parent $cand
                    break
                }
            }
        }
        if (-not $hasClang -and -not $hasLibClang) {
            $missingPrereqs += "llvm"
            Write-Warn "LLVM / Clang (libclang.dll) not found."
        } else {
            Write-Success "LLVM / Clang verified (LIBCLANG_PATH: $env:LIBCLANG_PATH)."
        }

        Write-Step "Checking NASM Assembler..."
        $hasNasm = [bool](Get-Command nasm -ErrorAction SilentlyContinue)
        if (-not $hasNasm) {
            $nasmCandidates = @(
                "C:\Program Files\NASM\nasm.exe",
                "C:\ProgramData\chocolatey\bin\nasm.exe"
            )
            foreach ($cand in $nasmCandidates) {
                if (Test-Path $cand) {
                    $hasNasm = $true
                    $nasmDir = Split-Path -Parent $cand
                    if ($env:Path -notlike "*$nasmDir*") {
                        $env:Path = "$nasmDir;$env:Path"
                    }
                    break
                }
            }
        }
        if (-not $hasNasm) {
            $missingPrereqs += "nasm"
            Write-Warn "NASM assembler not found."
        } else {
            Write-Success "NASM assembler verified."
        }
    }
} else {
    Write-Success "Toolchain verified via toolchain cache."
}

# 2. Self-Healing Toolchain Ladder
if ($missingPrereqs.Count -gt 0) {
    if ($NoInstall) {
        Write-Err "Missing prerequisites: $($missingPrereqs -join ', ')"
        Write-Err "-NoInstall specified. Aborting compilation."
        exit 1
    }

    Write-Warn "Build toolchain is incomplete. Invoking automated installer..."
    $installer = Join-Path $ScriptDir "install.ps1"
    if (-not (Test-Path $installer)) {
        Write-Err "Installer not found at $installer"
        exit 1
    }

    Write-Step "Executing: install.ps1 -BuildTools -Drive $Drive"
    if (-not $DryRun) {
        & powershell -ExecutionPolicy Bypass -File $installer -BuildTools
        Update-SessionEnvironment
    } else {
        Write-Host "  [DRY-RUN] Would run: powershell -ExecutionPolicy Bypass -File .\install.ps1 -BuildTools"
    }
}

# 3. Handle -Clean
if ($Clean) {
    Write-Step "Cleaning build artifacts..."
    if (-not $DryRun) {
        if (-not $FrontendOnly) {
            Set-Location (Join-Path $ScriptDir "src-tauri")
            cargo clean
            Set-Location $ScriptDir
        }
        $distDir = Join-Path $ScriptDir "dist"
        if (Test-Path $distDir) {
            Remove-Item -Path $distDir -Recurse -Force -ErrorAction SilentlyContinue
        }
        Write-Success "Build directories cleaned."
    } else {
        Write-Host "  [DRY-RUN] Would clean cargo build target and dist/ directory."
    }
}

# 4. Handle -FrontendOnly
if ($FrontendOnly) {
    Write-Step "Executing frontend-only build (npm run build)..."
    if (-not $DryRun) {
        Set-Location $ScriptDir
        npm run build
        exit $LASTEXITCODE
    } else {
        Write-Host "  [DRY-RUN] Would execute: npm run build"
        exit 0
    }
}

# 5. Ensure frontend dependencies are installed
$nodeModulesDir = Join-Path $ScriptDir "node_modules"
if (-not (Test-Path $nodeModulesDir)) {
    Write-Step "node_modules missing. Installing dependencies (npm install --legacy-peer-deps)..."
    if (-not $DryRun) {
        Set-Location $ScriptDir
        npm install --legacy-peer-deps
        Write-Success "Frontend dependencies installed successfully."
    } else {
        Write-Host "  [DRY-RUN] Would execute: npm install --legacy-peer-deps"
    }
}

# 6. Configure compiler parallelization & cache optimizations
$cores = [math]::Min(6, [System.Environment]::ProcessorCount)
$env:CARGO_BUILD_JOBS = $cores
$env:RUST_MIN_STACK = "16777216"
Write-Step "Configured worker threads: $cores (RUST_MIN_STACK: 16MB)"

$hasSccache = [bool](Get-Command sccache -ErrorAction SilentlyContinue)
if ($hasSccache) {
    $env:RUSTC_WRAPPER = "sccache"
    Write-Success "Shared compiler cache (sccache) enabled."
}

# Fast linker (lld-link) is optional and disabled by default on MSVC to prevent ICE
# if ($hasLld -and -not $env:RUSTFLAGS) { ... }

# 7. Execute Compilation
Set-Location $ScriptDir

if ($CliOnly) {
    $cargoArgs = @("build", "--bin", "agm", "--manifest-path", (Join-Path $ScriptDir "src-tauri\Cargo.toml"))
    if ($targetMode -eq "release") {
        $cargoArgs += "--release"
    }
    if ($VerboseMode) {
        $cargoArgs += "--verbose"
    }
    Write-Step "Executing CLI binary compilation: cargo $($cargoArgs -join ' ')"
    if (-not $DryRun) {
        & cargo @cargoArgs
        exit $LASTEXITCODE
    } else {
        Write-Host "  [DRY-RUN] Would execute: cargo $($cargoArgs -join ' ')"
        exit 0
    }
} else {
    Write-Step "Executing Tauri bundle compilation ($targetMode mode)..."
    if (-not $DryRun) {
        if ($targetMode -eq "debug") {
            npm run tauri build -- --debug
        } else {
            npm run tauri build
        }
        exit $LASTEXITCODE
    } else {
        Write-Host "  [DRY-RUN] Would execute: npm run tauri build $(if ($targetMode -eq 'debug') { '-- --debug' })"
        exit 0
    }
}
