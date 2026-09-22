<#
.SYNOPSIS
    Automated Rust, Cargo, LLVM & Build Toolchain Installer for Windows.

.DESCRIPTION
    Checks and installs the required build toolchain for Antigravity-Manager:
    - Rustup & Cargo (stable-x86_64-pc-windows-msvc)
    - LLVM & Clang (compiler toolchain & libclang)
    - Visual Studio C++ Build Tools detection & guidance
    - User PATH configuration and environment verification

.PARAMETER Force
    Reinstall or upgrade toolchain components even if already present.

.PARAMETER Quiet
    Suppress interactive prompts and execute non-interactively.

.PARAMETER SkipLlvm
    Skip LLVM/Clang installation.

.EXAMPLE
    .\scripts\install-rust-toolchain.ps1
    .\scripts\install-rust-toolchain.ps1 -Force
#>

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$Quiet,
    [switch]$SkipLlvm
)

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

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

function Refresh-SessionPath {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $sysPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $env:Path = "$sysPath;$userPath"
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        if ($env:Path -notlike "*$cargoBin*") {
            $env:Path = "$cargoBin;$env:Path"
        }
    }
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   Antigravity-Manager: Windows Build Toolchain Installer   " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Inspect Cargo & Rustup
Write-Step "Checking Rust toolchain (rustup & cargo)..."
Refresh-SessionPath
$hasRustc = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
$hasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)

if ($hasRustc -and $hasCargo -and -not $Force) {
    $rVer = & rustc --version 2>$null
    $cVer = & cargo --version 2>$null
    Write-Success "Rust toolchain already installed: $rVer | $cVer"
} else {
    Write-Step "Downloading and installing Rustup (MSVC 64-bit)..."
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    $rustupUrl = "https://win.rustup.rs/x86_64"
    try {
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe -UseBasicParsing
        Write-Step "Executing rustup-init (default toolchain: stable-x86_64-pc-windows-msvc)..."
        $p = Start-Process -FilePath $rustupExe -ArgumentList "-y", "--default-toolchain", "stable-x86_64-pc-windows-msvc" -Wait -PassThru -NoNewWindow
        if ($p.ExitCode -eq 0) {
            Write-Success "Rustup installation completed successfully."
        } else {
            Write-Warn "Rustup returned exit code: $($p.ExitCode)"
        }
    } catch {
        Write-Err "Failed to download/install Rustup: $_"
    } finally {
        if (Test-Path $rustupExe) { Remove-Item $rustupExe -Force -ErrorAction SilentlyContinue }
    }
    Refresh-SessionPath
}

# 2. Inspect LLVM & Clang
if (-not $SkipLlvm) {
    Write-Step "Checking LLVM and Clang compiler..."
    $hasClang = [bool](Get-Command clang -ErrorAction SilentlyContinue)
    if ($hasClang -and -not $Force) {
        $clangVer = (& clang --version 2>$null | Select-Object -First 1)
        Write-Success "LLVM/Clang already installed: $clangVer"
    } else {
        Write-Step "Attempting to install LLVM via Winget..."
        $hasWinget = [bool](Get-Command winget -ErrorAction SilentlyContinue)
        $installedLlvm = $false
        if ($hasWinget) {
            try {
                $wArgs = @("install", "--id", "LLVM.LLVM", "--exact", "--accept-package-agreements", "--accept-source-agreements")
                if ($Quiet) { $wArgs += "--silent" }
                $wp = Start-Process -FilePath "winget" -ArgumentList $wArgs -Wait -PassThru -NoNewWindow
                if ($wp.ExitCode -eq 0) {
                    $installedLlvm = $true
                    Write-Success "LLVM installed via Winget."
                }
            } catch {
                Write-Warn "Winget installation of LLVM failed: $_"
            }
        }
        if (-not $installedLlvm) {
            $hasChoco = [bool](Get-Command choco -ErrorAction SilentlyContinue)
            if ($hasChoco) {
                Write-Step "Attempting to install LLVM via Chocolatey..."
                try {
                    $cp = Start-Process -FilePath "choco" -ArgumentList "install", "llvm", "-y" -Wait -PassThru -NoNewWindow
                    if ($cp.ExitCode -eq 0) {
                        $installedLlvm = $true
                        Write-Success "LLVM installed via Chocolatey."
                    }
                } catch {
                    Write-Warn "Chocolatey installation of LLVM failed: $_"
                }
            }
        }
        if (-not $installedLlvm) {
            Write-Warn "Could not automatically install LLVM via winget or choco."
            Write-Warn "You can download the Windows installer manually from: https://github.com/llvm/llvm-project/releases"
        }
        Refresh-SessionPath
    }
}

# 3. Inspect MSVC C++ Build Tools
Write-Step "Checking Visual Studio C++ Build Tools / Linker..."
$hasLink = [bool](Get-Command link -ErrorAction SilentlyContinue)
if ($hasLink) {
    Write-Success "MSVC Linker (link.exe) is available in PATH."
} else {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    $hasVs = $false
    if (Test-Path $vswhere) {
        $vsInstall = & $vswhere -latest -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsInstall) {
            $hasVs = $true
            Write-Success "Visual Studio C++ Build Tools detected at: $vsInstall"
        }
    }
    if (-not $hasVs) {
        Write-Warn "Visual Studio C++ Build Tools were not detected."
        Write-Warn "Install using: winget install Microsoft.VisualStudio.2022.BuildTools --override `"--passive --wait --add Microsoft.VisualStudio.Workload.VCTools`""
    }
}

# 4. Final Toolchain Verification
Write-Host ""
Write-Host "--- Toolchain Verification ---" -ForegroundColor Cyan
Refresh-SessionPath

$vRustc = if (Get-Command rustc -ErrorAction SilentlyContinue) { & rustc --version } else { "Not Found" }
$vCargo = if (Get-Command cargo -ErrorAction SilentlyContinue) { & cargo --version } else { "Not Found" }
$vClang = if (Get-Command clang -ErrorAction SilentlyContinue) { (& clang --version 2>$null | Select-Object -First 1) } else { "Not Found (Optional for standard builds)" }

Write-Host "  Rust Compiler : $vRustc" -ForegroundColor $(if ($vRustc -ne "Not Found") { "Green" } else { "Red" })
Write-Host "  Cargo Package : $vCargo" -ForegroundColor $(if ($vCargo -ne "Not Found") { "Green" } else { "Red" })
Write-Host "  LLVM / Clang  : $vClang" -ForegroundColor $(if ($vClang -ne "Not Found (Optional for standard builds)") { "Green" } else { "Yellow" })
Write-Host ""

if ($vRustc -ne "Not Found" -and $vCargo -ne "Not Found") {
    Write-Success "Build toolchain is ready. You can now build or test Antigravity-Manager!"
} else {
    Write-Warn "Toolchain setup completed with warnings. You may need to restart your terminal or shell to reload PATH."
}
