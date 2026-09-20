<#
.SYNOPSIS
    Local Development & Quality Verification Runner for Antigravity-Manager.

.DESCRIPTION
    Provides unified execution workflows for local development, fast quality checks,
    frontend builds, and Tauri desktop debugging.

.PARAMETER Dev
    Start frontend Vite dev server.

.PARAMETER Desktop
    Start full Tauri desktop application in development mode with hot-reloading.

.PARAMETER Build
    Run production frontend build (`npm run build`).

.PARAMETER Check
    Execute fast local quality checks: TypeScript validation (`tsc --noEmit`)
    and Rust formatting (`cargo fmt -- --check`).

.PARAMETER Help
    Show usage information.

.EXAMPLE
    .\run.ps1 -Check
    .\run.ps1 -Dev
    .\run.ps1 -Desktop
    .\run.ps1 -Build
#>

[CmdletBinding()]
param(
    [Alias('d')][switch]$Dev,
    [Alias('t')][switch]$Desktop,
    [Alias('b')][switch]$Build,
    [Alias('c')][switch]$Check,
    [Alias('h')][switch]$Help
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "[*] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Err {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RepoRoot

if ($Help) {
    Get-Help $MyInvocation.MyCommand.Path
    return
}

# Default action is -Check if no flags provided
$hasAction = $Dev -or $Desktop -or $Build -or $Check
if (-not $hasAction) {
    $Check = $true
}

if ($Check) {
    Write-Step "Running fast quality verification..."
    
    # 1. TypeScript Verification
    Write-Step "Checking TypeScript types (tsc --noEmit)..."
    & npx tsc --noEmit
    if ($LASTEXITCODE -ne 0) {
        Write-Err "TypeScript type check failed!"
        exit $LASTEXITCODE
    }
    Write-Success "TypeScript checks passed."

    # 2. Rust Formatting Check
    Write-Step "Checking Rust formatting (cargo fmt -- --check)..."
    Push-Location (Join-Path $RepoRoot "src-tauri")
    try {
        & cargo fmt -- --check
        if ($LASTEXITCODE -ne 0) {
            Write-Err "Rust formatting check failed! Run 'cargo fmt' to fix."
            exit $LASTEXITCODE
        }
    } finally {
        Pop-Location
    }
    Write-Success "Rust formatting checks passed."

    Write-Success "All local quality checks passed cleanly!"
    return
}

if ($Build) {
    Write-Step "Building frontend assets (npm run build)..."
    & npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Err "Frontend build failed!"
        exit $LASTEXITCODE
    }
    Write-Success "Frontend build completed successfully."
    return
}

if ($Desktop) {
    Write-Step "Starting Antigravity-Manager in Desktop Tauri Dev mode..."
    & npm run tauri dev
    return
}

if ($Dev) {
    Write-Step "Starting frontend Vite development server..."
    & npm run dev
    return
}
