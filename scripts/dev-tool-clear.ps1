<#
.SYNOPSIS
    Cleans developer tools, Cargo build cache, build demo artifacts, pycache, temporary logs, and prunes stale conversations.
.DESCRIPTION
    Provides automated developer hygiene by clearing Cargo/Rust intermediate caches,
    build-demo / target-demo leftovers, pycache, and pruning Antigravity conversations across all candidate directories.
.PARAMETER KeepCount
    Number of recent conversations to preserve (default: 10). Supports 'k <N>', '-k <N>', or '<N>'.
#>
param(
    [string]$Arg1 = "",
    [string]$Arg2 = "",
    [switch]$CargoOnly,
    [switch]$Force
)

$ErrorActionPreference = "Continue"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$localAgm = Join-Path $RootDir "src-tauri\target\debug\agm.exe"
$cargoAgm = Join-Path $RootDir "src-tauri\target\release\agm.exe"

# Parse KeepCount from Arg1/Arg2 or $args with support for 'k N', '-k N', '--keep N', 'N'
$KeepCount = 10
$allArgs = @()
if ($Arg1) { $allArgs += $Arg1 }
if ($Arg2) { $allArgs += $Arg2 }
if ($args) { $allArgs += $args }

for ($i = 0; $i -lt $allArgs.Count; $i++) {
    $a = $allArgs[$i]
    if ($a -match '^(?:-k|--keep|k)$' -and ($i + 1 -lt $allArgs.Count) -and ($allArgs[$i + 1] -match '^\d+$')) {
        $KeepCount = [int]$allArgs[$i + 1]
        $i++
    } elseif ($a -match '^-?k(\d+)$') {
        $KeepCount = [int]$Matches[1]
    } elseif ($a -match '^\d+$') {
        $KeepCount = [int]$a
    } elseif ($a -eq '--cargo-only') {
        $CargoOnly = $true
    }
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " AGM Developer Hygiene & Cache Cleaner (Cross-Platform)   " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Clean Cargo/Rust compiler caches, incremental files, and build demo artifacts
Write-Host "[1/3] Cleaning Cargo and build demo caches..." -ForegroundColor Yellow
python "$RootDir/03-ai-scripts/19-artifact-remover.py" --clean-cargo --force
if ($LASTEXITCODE -ne 0) {
    Write-Warning "Artifact remover reported warning or exit code $LASTEXITCODE"
}

# Clean any demo build artifacts if present
$demoDirs = @(
    (Join-Path $RootDir "build-demo"),
    (Join-Path $RootDir "target-demo"),
    (Join-Path $RootDir "src-tauri\build-demo"),
    (Join-Path $RootDir "src-tauri\target-demo")
)
foreach ($dir in $demoDirs) {
    if (Test-Path $dir) {
        Write-Host "  Removing demo directory: $dir" -ForegroundColor DarkGray
        Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
    }
}

if ($CargoOnly) {
    Write-Host "[OK] Cargo and build-demo cleanup completed." -ForegroundColor Green
    exit 0
}

# 2. Clean temporary files and pycache
Write-Host "[2/3] Cleaning temporary test files, pycache, and log dumps..." -ForegroundColor Yellow
python "$RootDir/03-ai-scripts/19-artifact-remover.py" --clean-temp --clean-pycache --force

# 3. Clean AGM application caches and conversations across all candidate directories using agm CLI
Write-Host "[3/3] Pruning old Antigravity conversations on all directories (keeping latest $KeepCount)..." -ForegroundColor Yellow

if (Get-Command "agm" -ErrorAction SilentlyContinue) {
    agm clear k $KeepCount
} elseif (Test-Path $localAgm) {
    & $localAgm clear k $KeepCount
} elseif (Test-Path $cargoAgm) {
    & $cargoAgm clear k $KeepCount
} else {
    Write-Host "  agm.exe not found on PATH; attempting cargo run..." -ForegroundColor DarkGray
    Push-Location (Join-Path $RootDir "src-tauri")
    try {
        cargo run --bin agm -- clear k $KeepCount
    } finally {
        Pop-Location
    }
}

Write-Host ""
Write-Host "[OK] Developer hygiene cleanup completed successfully!" -ForegroundColor Green
