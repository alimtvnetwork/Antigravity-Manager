<#
.SYNOPSIS
    Automated Rust, Cargo, LLVM & Build Toolchain Installer for Windows.

.DESCRIPTION
    Checks and installs the required build toolchain for Antigravity-Manager:
    - Rustup & Cargo (stable-x86_64-pc-windows-msvc)
    - LLVM & Clang (compiler toolchain & libclang)
    - Visual Studio C++ Build Tools detection & guidance
    - Optional items: Node.js (winget), pnpm, Tauri CLI, sccache,
      cargo-watch, GitHub CLI (winget)
    - Install profiles (repeatable, stacked): minimal, rust-dev,
      frontend, full
    - Remote install over SSH (key auth only)

.PARAMETER Force
    Reinstall or upgrade toolchain components even if already present.

.PARAMETER Quiet
    Suppress interactive prompts and execute non-interactively.

.PARAMETER SkipLlvm
    Skip LLVM/Clang installation.

.PARAMETER Profile
    Install profile (repeatable, stacked):
      minimal  = rust toolchain + clippy/rustfmt
      rust-dev = minimal + sccache + cargo-watch
      frontend = nodejs (winget) + pnpm + tauri-cli
      full     = rust-dev + frontend + gh

.PARAMETER Only
    Install exactly these items (repeatable). See -ListItems for ids.

.PARAMETER DryRun
    Print the resolved item list, install nothing.

.PARAMETER ListItems
    Print installable items (id + description), then exit.

.PARAMETER Troubleshoot
    Print common issues and fixes, then exit.

.PARAMETER SshTarget
    Run install on a remote host via SSH: "user@host" (key auth only,
    never passwords). The script is copied over scp and executed remotely.

.EXAMPLE
    .\scripts\install-rust-toolchain.ps1
    .\scripts\install-rust-toolchain.ps1 -Force
    .\scripts\install-rust-toolchain.ps1 -Profile full
    .\scripts\install-rust-toolchain.ps1 -Profile rust-dev -Profile frontend -DryRun
    .\scripts\install-rust-toolchain.ps1 -Only nodejs -Only pnpm
    .\scripts\install-rust-toolchain.ps1 -ListItems
    .\scripts\install-rust-toolchain.ps1 -Troubleshoot
    .\scripts\install-rust-toolchain.ps1 -Profile full -SshTarget dev@buildbox.local
#>

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$Quiet,
    [switch]$SkipLlvm,
    [string[]]$Profile = @(),
    [string[]]$Only = @(),
    [switch]$DryRun,
    [switch]$ListItems,
    [switch]$Troubleshoot,
    [string]$SshTarget = ""
)

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# --- item catalog ------------------------------------------------------------
$ItemCatalog = @(
    @{ Id = "rust";        Desc = "Rust toolchain via rustup (stable MSVC) + clippy/rustfmt components" },
    @{ Id = "llvm";        Desc = "LLVM/Clang via winget (native build dependency)" },
    @{ Id = "nodejs";      Desc = "Node.js LTS via winget" },
    @{ Id = "pnpm";        Desc = "pnpm package manager via npm (requires nodejs)" },
    @{ Id = "tauri-cli";   Desc = "Tauri CLI via cargo install (requires rust)" },
    @{ Id = "sccache";     Desc = "sccache compilation cache via cargo install (requires rust)" },
    @{ Id = "cargo-watch"; Desc = "cargo-watch file watcher via cargo install (requires rust)" },
    @{ Id = "gh";          Desc = "GitHub CLI via winget" }
)
$ItemOrder = @("rust", "llvm", "nodejs", "pnpm", "sccache", "cargo-watch", "tauri-cli", "gh")
$ProfileMap = @{
    "minimal"  = @("rust")
    "rust-dev" = @("rust", "sccache", "cargo-watch")
    "frontend" = @("nodejs", "pnpm", "tauri-cli")
    "full"     = @("rust", "sccache", "cargo-watch", "nodejs", "pnpm", "tauri-cli", "gh")
}

function Write-Step {
    param([string]$Message)
    if (-not $Quiet) { Write-Host "  [*] $Message" -ForegroundColor Cyan }
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

function Get-ItemDesc {
    param([string]$Id)
    $entry = $ItemCatalog | Where-Object { $_.Id -eq $Id } | Select-Object -First 1
    if ($entry) { return $entry.Desc }
    return ""
}

function Resolve-InstallItems {
    # Returns ordered, deduplicated item id array.
    $selected = @()
    if ($Only.Count -gt 0) {
        $selected = $Only
    } elseif ($Profile.Count -gt 0) {
        foreach ($p in $Profile) {
            $key = $p.Trim().ToLower()
            if (-not $ProfileMap.ContainsKey($key)) {
                Write-Err "unknown profile: '$p' (minimal|rust-dev|frontend|full)"
                exit 2
            }
            $selected += $ProfileMap[$key]
        }
    } else {
        $selected = @("rust", "llvm")  # default install set
    }
    foreach ($id in $selected) {
        if ($ItemOrder -notcontains $id) {
            Write-Err "unknown item: '$id' (see -ListItems)"
            exit 2
        }
    }
    if ($SkipLlvm) {
        $selected = $selected | Where-Object { $_ -ne "llvm" }
    }
    $ordered = @()
    foreach ($want in $ItemOrder) {
        if ($selected -contains $want -and $ordered -notcontains $want) {
            $ordered += $want
        }
    }
    return $ordered
}

# --- early exits --------------------------------------------------------------
if ($ListItems) {
    foreach ($entry in $ItemCatalog) {
        Write-Host "$($entry.Id)`t$($entry.Desc)"
    }
    exit 0
}

if ($Troubleshoot) {
    Write-Host @"
Troubleshooting — common issues and fixes:

1. "cargo: command not found" (right after install)
   Restart the terminal so the updated PATH loads. rustup also adds
   %USERPROFILE%\.cargo\bin to the user PATH automatically.

2. "link.exe not found" / MSVC linker errors
   Install Visual Studio C++ Build Tools:
   winget install Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools"

3. rustup download fails (offline or proxy)
   Check network connectivity and proxy env vars, then retry.

4. winget package not found
   Run: winget source update, then retry. Some packages need an
   updated winget client from the Microsoft Store.

5. SSH remote install fails with "Permission denied (publickey)"
   This script never handles passwords — pre-configure SSH key auth
   (ssh-copy-id equivalent), then retry.
"@
    exit 0
}

if ($SshTarget) {
    if ($SshTarget -notmatch '^[^@\s]+@[^@\s]+$') {
        Write-Err "-SshTarget needs user@host (got '$SshTarget')"
        exit 2
    }
    # Rebuild forwarded args (everything except -SshTarget itself).
    $fwd = @()
    if ($Profile.Count -gt 0)  { $fwd += "-Profile"; $fwd += ($Profile -join ",") }
    if ($Only.Count -gt 0)     { $fwd += "-Only";    $fwd += ($Only -join ",") }
    if ($DryRun)               { $fwd += "-DryRun" }
    if ($Force)                { $fwd += "-Force" }
    if ($Quiet)                { $fwd += "-Quiet" }
    if ($SkipLlvm)             { $fwd += "-SkipLlvm" }
    Write-Step "Copying installer to $SshTarget via SCP (key auth only)..."
    $remotePath = "~/install-rust-toolchain.ps1"
    & scp -o BatchMode=yes $PSCommandPath "${SshTarget}:$remotePath"
    if ($LASTEXITCODE -ne 0) {
        Write-Err "scp to $SshTarget failed (exit $LASTEXITCODE). Check SSH key auth."
        exit $LASTEXITCODE
    }
    Write-Step "Running installer on $SshTarget via SSH..."
    $remoteCmd = "powershell -NoProfile -NonInteractive -File $remotePath $($fwd -join ' ')"
    & ssh -o BatchMode=yes $SshTarget $remoteCmd
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   Antigravity-Manager: Windows Build Toolchain Installer   " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

# --- item installers ----------------------------------------------------------
function Install-RustItem {
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

    Write-Step "Ensuring clippy + rustfmt components..."
    try {
        & rustup component add clippy rustfmt 2>$null
        Write-Success "clippy + rustfmt ready."
    } catch {
        Write-Warn "Could not add clippy/rustfmt (offline?)."
    }
}

function Install-LlvmItem {
    Write-Step "Checking LLVM and Clang compiler..."
    $hasClang = [bool](Get-Command clang -ErrorAction SilentlyContinue)
    if ($hasClang -and -not $Force) {
        $clangVer = (& clang --version 2>$null | Select-Object -First 1)
        Write-Success "LLVM/Clang already installed: $clangVer"
        return
    }
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

function Install-NodeJsItem {
    $hasNode = [bool](Get-Command node -ErrorAction SilentlyContinue)
    if ($hasNode -and -not $Force) {
        Write-Success "Node.js already installed: $(& node --version 2>$null)"
        return
    }
    Write-Step "Installing Node.js LTS via winget..."
    $hasWinget = [bool](Get-Command winget -ErrorAction SilentlyContinue)
    if (-not $hasWinget) {
        Write-Err "winget not found; install Node.js LTS manually from https://nodejs.org"
        return
    }
    $wArgs = @("install", "--id", "OpenJS.NodeJS.LTS", "--exact", "--accept-package-agreements", "--accept-source-agreements")
    if ($Quiet) { $wArgs += "--silent" }
    $wp = Start-Process -FilePath "winget" -ArgumentList $wArgs -Wait -PassThru -NoNewWindow
    if ($wp.ExitCode -eq 0) {
        Write-Success "Node.js LTS installed via winget."
    } else {
        Write-Warn "winget Node.js install returned exit code: $($wp.ExitCode)"
    }
    Refresh-SessionPath
}

function Install-PnpmItem {
    $hasPnpm = [bool](Get-Command pnpm -ErrorAction SilentlyContinue)
    if ($hasPnpm -and -not $Force) {
        Write-Success "pnpm already installed: $(& pnpm --version 2>$null)"
        return
    }
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        Write-Err "pnpm needs the nodejs item first (npm not found)"
        return
    }
    Write-Step "Installing pnpm via npm..."
    & npm install -g pnpm
    Write-Success "pnpm installed."
    Refresh-SessionPath
}

function Install-CargoPackageItem {
    param(
        [string]$Crate,
        [string]$Binary
    )
    $hasBin = [bool](Get-Command $Binary -ErrorAction SilentlyContinue)
    if ($hasBin -and -not $Force) {
        Write-Success "$Binary already installed."
        return
    }
    Refresh-SessionPath
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Err "$Binary needs the rust item first (cargo not found)"
        return
    }
    Write-Step "Installing $Crate via cargo install (this may take a while)..."
    & cargo install $Crate --locked
    Write-Success "$Binary installed."
}

function Install-GhItem {
    $hasGh = [bool](Get-Command gh -ErrorAction SilentlyContinue)
    if ($hasGh -and -not $Force) {
        Write-Success "gh already installed: $((& gh --version 2>$null | Select-Object -First 1))"
        return
    }
    Write-Step "Installing GitHub CLI via winget..."
    $hasWinget = [bool](Get-Command winget -ErrorAction SilentlyContinue)
    if (-not $hasWinget) {
        Write-Err "winget not found; install gh manually from https://cli.github.com"
        return
    }
    $wArgs = @("install", "--id", "GitHub.cli", "--exact", "--accept-package-agreements", "--accept-source-agreements")
    if ($Quiet) { $wArgs += "--silent" }
    $wp = Start-Process -FilePath "winget" -ArgumentList $wArgs -Wait -PassThru -NoNewWindow
    if ($wp.ExitCode -eq 0) {
        Write-Success "gh installed via winget."
    } else {
        Write-Warn "winget gh install returned exit code: $($wp.ExitCode)"
    }
    Refresh-SessionPath
}

function Install-MsvcCheck {
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
}

# --- resolve + dispatch -------------------------------------------------------
$installItems = Resolve-InstallItems

if ($DryRun) {
    $profLabel = if ($Profile.Count -gt 0) { $Profile -join " " } else { "<none>" }
    Write-Host "Dry run — would install $($installItems.Count) item(s) [profiles: $profLabel]:"
    foreach ($id in $installItems) {
        Write-Host "  - ${id}: $(Get-ItemDesc $id)"
    }
    Write-Host "(nothing installed)"
    exit 0
}

Write-Step "Items: $($installItems -join ' ')"
foreach ($item in $installItems) {
    switch ($item) {
        "rust"        { Install-RustItem }
        "llvm"        { Install-LlvmItem }
        "nodejs"      { Install-NodeJsItem }
        "pnpm"        { Install-PnpmItem }
        "tauri-cli"   { Install-CargoPackageItem -Crate "tauri-cli" -Binary "cargo-tauri" }
        "sccache"     { Install-CargoPackageItem -Crate "sccache" -Binary "sccache" }
        "cargo-watch" { Install-CargoPackageItem -Crate "cargo-watch" -Binary "cargo-watch" }
        "gh"          { Install-GhItem }
    }
}

# MSVC linker check rides along with any rust install (informational only)
if ($installItems -contains "rust") {
    Install-MsvcCheck
}

# --- final verification -------------------------------------------------------
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
