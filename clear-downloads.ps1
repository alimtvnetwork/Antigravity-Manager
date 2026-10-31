<#
.SYNOPSIS
    Purge cached installer downloads for Antigravity-Manager build toolchain.

.DESCRIPTION
    Scans and purges cached installer executables, MSIs, and archives downloaded
    during toolchain installation (e.g. vs_BuildTools.exe, LLVM MSIs, rustup-init).
    Reclaims disk space while keeping installed software intact.

.EXAMPLE
    .\clear-downloads.ps1
#>

[CmdletBinding()]
param(
    [string]$CacheDir = ""
)

$ErrorActionPreference = "Stop"

$script = Join-Path $PSScriptRoot "scripts\install-rust-toolchain.ps1"
if (Test-Path $script) {
    if ($CacheDir) {
        & $script -ClearDownloads -CacheDir $CacheDir
    } else {
        & $script -ClearDownloads
    }
} else {
    Write-Host "Toolchain installer script not found at: $script" -ForegroundColor Red
    exit 1
}
