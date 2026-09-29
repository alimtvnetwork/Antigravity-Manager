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
    [switch]$CleanInstances,
    [switch]$InstancesOnly,
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
    } elseif ($a -in @('--clean-instances', '-i', '--instances', 'clean-instances')) {
        $CleanInstances = $true
    } elseif ($a -in @('--instances-only')) {
        $InstancesOnly = $true
        $CleanInstances = $true
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

if (Get-Command "adm" -ErrorAction SilentlyContinue) {
    adm clear k $KeepCount
} elseif (Get-Command "agm" -ErrorAction SilentlyContinue) {
    agm clear k $KeepCount
} elseif (Test-Path (Join-Path $RootDir "adm.cmd")) {
    & (Join-Path $RootDir "adm.cmd") clear k $KeepCount
} elseif (Test-Path $localAgm) {
    & $localAgm clear k $KeepCount
} elseif (Test-Path $cargoAgm) {
    & $cargoAgm clear k $KeepCount
} else {
    Write-Host "  agm/adm not found on PATH; attempting cargo run..." -ForegroundColor DarkGray
    Push-Location (Join-Path $RootDir "src-tauri")
    try {
        cargo run --bin agm -- clear k $KeepCount
    } finally {
        Pop-Location
    }
}

# 4. Clean temporary/test instances and stale instance lockfiles
$instancesRoot = Join-Path $env:USERPROFILE ".antigravity_tools\instances"
$hasInstances = Test-Path $instancesRoot
if ($CleanInstances -or $hasInstances) {
    Write-Host "[4/4] Cleaning test instances and stale lockfiles..." -ForegroundColor Yellow
    
    # Clean stale lockfiles across all instances
    if ($hasInstances) {
        Get-ChildItem -Path $instancesRoot -Recurse -File -Include "lockfile", "code.lock", "singleton*" -ErrorAction SilentlyContinue | ForEach-Object {
            try {
                Remove-Item -Force $_.FullName -ErrorAction SilentlyContinue
                Write-Host "  Removed lockfile: $($_.FullName)" -ForegroundColor DarkGray
            } catch {}
        }
    }

    # Clean orphaned test instances (test-* or tmp-*)
    $agmCmd = if (Test-Path $localAgm) { $localAgm } elseif (Test-Path $cargoAgm) { $cargoAgm } elseif (Get-Command "agm" -ErrorAction SilentlyContinue) { "agm" } else { $null }
    if ($agmCmd -and (Test-Path $instancesRoot)) {
        Get-ChildItem -Path $instancesRoot -Directory -ErrorAction SilentlyContinue | Where-Object {
            $_.Name -match '^(?:test[-_]|tmp[-_]|testdiag)'
        } | ForEach-Object {
            $instId = $_.Name
            Write-Host "  Purging test instance: $instId" -ForegroundColor Yellow
            try {
                & $agmCmd instances rm $instId --force 2>$null
            } catch {}
            if (Test-Path $_.FullName) {
                Remove-Item -Recurse -Force $_.FullName -ErrorAction SilentlyContinue
            }
        }
    }
}

if ($InstancesOnly) {
    Write-Host ""
    Write-Host "[OK] Instance hygiene cleanup completed successfully!" -ForegroundColor Green
    exit 0
}

Write-Host ""
Write-Host "[OK] Developer hygiene cleanup completed successfully!" -ForegroundColor Green
