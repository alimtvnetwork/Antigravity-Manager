<#
.SYNOPSIS
    Antigravity-Manager Supabase Credential Importer & One-Liner Runner
.DESCRIPTION
    Zero-hardcoding credential importer that dynamically loads Supabase credentials
    into Antigravity-Manager from repo-secrets vault or environment variables.
    Executes the standard runner workflow:
      agm supabase load-secrets; agm supabase test
.EXAMPLE
    pwsh -NoProfile -File scripts/import-supabase-credentials.ps1
.EXAMPLE
    pwsh -NoProfile -Command "agm supabase load-secrets; agm supabase test"
.EXAMPLE
    pwsh -NoProfile -File scripts/import-supabase-credentials.ps1 -SecretsDir "custom/path/to/repo-secrets"
#>

[CmdletBinding()]
param (
    [string]$SecretsDir = "",
    [string]$RootUrl = "",
    [string]$RootKey = "",
    [string]$SecondaryUrl = "",
    [string]$SecondaryKey = "",
    [switch]$SkipTest = $false
)

$ErrorActionPreference = "Stop"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  Antigravity-Manager (AGM) Dynamic Supabase Credential Importer" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

# 1. Apply optional environment variables if provided
if ($SecretsDir) {
    Write-Host "Setting REPO_SECRETS_DIR to: $SecretsDir" -ForegroundColor Yellow
    $env:REPO_SECRETS_DIR = $SecretsDir
}

if ($RootUrl) {
    $env:SUPABASE_ROOT_URL = $RootUrl
}
if ($RootKey) {
    $env:SUPABASE_ROOT_KEY = $RootKey
}
if ($SecondaryUrl) {
    $env:SUPABASE_SECONDARY_URL = $SecondaryUrl
}
if ($SecondaryKey) {
    $env:SUPABASE_SECONDARY_KEY = $SecondaryKey
}

# 2. Locate agm executable or cargo runner
$agmCmd = "agm"
$agmAvailable = $false
try {
    $check = Get-Command "agm" -ErrorAction SilentlyContinue
    if ($check) {
        $agmAvailable = $true
    }
} catch { }

if (-not $agmAvailable) {
    # Check common target binary paths relative to repository root
    $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
    $RootDir = Split-Path -Parent $ScriptDir
    $releaseBin = Join-Path $RootDir "src-tauri\target\release\agm.exe"
    $debugBin = Join-Path $RootDir "src-tauri\target\debug\agm.exe"

    if (Test-Path $releaseBin) {
        $agmCmd = $releaseBin
        $agmAvailable = $true
    } elseif (Test-Path $debugBin) {
        $agmCmd = $debugBin
        $agmAvailable = $true
    }
}

Write-Host "`nImporting credentials via AGM CLI runner..." -ForegroundColor Cyan

if ($agmAvailable) {
    Write-Host "Running: $agmCmd supabase load-secrets" -ForegroundColor Gray
    & $agmCmd supabase load-secrets

    if (-not $SkipTest) {
        Write-Host "`nTesting connectivity: $agmCmd supabase test" -ForegroundColor Gray
        & $agmCmd supabase test
    }
} else {
    Write-Host "AGM executable not found in PATH or target directories." -ForegroundColor Yellow
    Write-Host "You can execute the canonical zero-hardcoding one-liner directly:" -ForegroundColor White
    Write-Host '  pwsh -NoProfile -Command "agm supabase load-secrets; agm supabase test"' -ForegroundColor Green
    Write-Host "Or via Cargo:" -ForegroundColor White
    Write-Host '  cargo run --manifest-path src-tauri/Cargo.toml --bin agm -- supabase load-secrets' -ForegroundColor Green
}

Write-Host "`n================================================================================" -ForegroundColor Cyan
Write-Host "POWERSHELL ZERO-HARDCODING ONE-LINER:" -ForegroundColor Yellow
Write-Host '  pwsh -NoProfile -Command "agm supabase load-secrets; agm supabase test"' -ForegroundColor Green
Write-Host "================================================================================" -ForegroundColor Cyan
