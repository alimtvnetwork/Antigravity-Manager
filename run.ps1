<#
.SYNOPSIS
    Antigravity-Manager: Windows Application Runner

.DESCRIPTION
    Verifies build prerequisites (Node.js, npm, Rust/Cargo), installs dependencies
    if missing, and runs the application in development or build mode.

.PARAMETER Dev
    Launch development mode with hot-reloading (npm run tauri dev). Default.

.PARAMETER Build
    Compile production binary and installer (npm run tauri build).

.PARAMETER Debug
    Launch development mode with RUST_LOG=debug for verbose logging.

.PARAMETER FrontendOnly
    Launch Vite frontend dev server only (npm run dev).

.PARAMETER InstallToolchain
    Run the Rust, Cargo, LLVM toolchain installer script.

.EXAMPLE
    .\run.ps1
    .\run.ps1 -Build
    .\run.ps1 -Debug
    .\run.ps1 -FrontendOnly
    .\run.ps1 -InstallToolchain
#>

param(
    [switch]$Dev,
    [switch]$Build,
    [Alias("Debug")]
    [switch]$DebugMode,
    [switch]$FrontendOnly,
    [switch]$InstallToolchain
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
Write-Host "                Antigravity-Manager Runner                 " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

if ($InstallToolchain) {
    $installer = Join-Path $ScriptDir "scripts\install-rust-toolchain.ps1"
    if (Test-Path $installer) {
        & $installer
    } else {
        Write-Err "Toolchain installer not found at $installer"
    }
    exit 0
}

# 1. Check Node.js and npm
Write-Step "Checking Node.js & npm..."
$hasNode = [bool](Get-Command node -ErrorAction SilentlyContinue)
$hasNpm = [bool](Get-Command npm -ErrorAction SilentlyContinue)

if (-not $hasNode -or -not $hasNpm) {
    Write-Err "Node.js and npm are required to run Antigravity-Manager."
    Write-Err "Please install Node.js (v20+ LTS) from: https://nodejs.org/"
    exit 1
}

$nodeVer = & node -v
$npmVer = & npm -v
Write-Success "Node.js: $nodeVer | npm: v$npmVer"

# 2. Check Rust & Cargo (if running Tauri)
if (-not $FrontendOnly) {
    Write-Step "Checking Rust & Cargo toolchain..."
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        if ($env:Path -notlike "*$cargoBin*") {
            $env:Path = "$cargoBin;$env:Path"
        }
    }

    $hasRustc = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
    $hasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)

    if (-not $hasRustc -or -not $hasCargo) {
        Write-Warn "Rust or Cargo was not found in PATH."
        Write-Step "Running toolchain installer now..."
        $installer = Join-Path $ScriptDir "scripts\install-rust-toolchain.ps1"
        if (Test-Path $installer) {
            & $installer
        }
        # Re-check after installer run
        $hasRustc = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
        if (-not $hasRustc) {
            Write-Err "Rust toolchain is missing. Please restart your terminal or install Rustup from https://rustup.rs"
            exit 1
        }
    } else {
        $rustcVer = & rustc --version
        Write-Success "Rust: $rustcVer"
    }
}

# 3. Check frontend dependencies
$nodeModulesDir = Join-Path $ScriptDir "node_modules"
if (-not (Test-Path $nodeModulesDir)) {
    Write-Step "node_modules directory missing. Installing frontend dependencies..."
    npm install --legacy-peer-deps
    Write-Success "Frontend dependencies installed successfully."
}

# 4. Dispatch Run Mode
Set-Location -Path $ScriptDir

if ($Build) {
    Write-Step "Executing production build (npm run tauri build)..."
    npm run tauri build
} elseif ($DebugMode) {
    Write-Step "Executing debug mode with verbose logs (RUST_LOG=debug npm run tauri dev)..."
    $env:RUST_LOG = "debug"
    npm run tauri dev
} elseif ($FrontendOnly) {
    Write-Step "Launching Vite frontend dev server (npm run dev)..."
    npm run dev
} else {
    # Default to development mode
    Write-Step "Launching Antigravity-Manager development mode (npm run tauri dev)..."
    npm run tauri dev
}
