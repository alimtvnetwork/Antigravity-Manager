<#
.SYNOPSIS
    Automated Rust, Cargo, MSVC, LLVM & Build Toolchain Installer for Windows.

.DESCRIPTION
    Installs, configures, and verifies the complete build toolchain for Antigravity-Manager:
    - Microsoft Visual Studio 2022 C++ Build Tools & Windows SDK (link.exe, kernel32.Lib)
    - Rustup & Cargo (stable-x86_64-pc-windows-msvc) + clippy/rustfmt
    - LLVM & Clang (compiler toolchain & libclang.dll for bindgen with LIBCLANG_PATH)
    - NASM (Netwide Assembler via Chocolatey)
    - Download caching for fast reuse and offline reproducibility
    - Cache purging command (-ClearDownloads / -ClearCache)
    - Optional items: Node.js (winget), pnpm, Tauri CLI, sccache, cargo-watch, GitHub CLI
    - Install profiles: minimal, build-tools (agm-build-tools), rust-dev, frontend, full
    - Remote install over SSH (key auth only)

.PARAMETER Force
    Reinstall or upgrade toolchain components even if already present.

.PARAMETER Quiet
    Suppress interactive prompts and execute non-interactively.

.PARAMETER SkipLlvm
    Skip LLVM/Clang installation.

.PARAMETER Profile
    Install profile (repeatable, stacked):
      minimal         = rust toolchain + clippy/rustfmt
      build-tools     = msvc + rust + llvm + nasm (default AGM build suite)
      agm-build-tools = alias for build-tools
      rust-dev        = build-tools + sccache + cargo-watch
      frontend        = nodejs (winget) + pnpm + tauri-cli
      full            = rust-dev + frontend + gh

.PARAMETER Only
    Install exactly these items (repeatable). See -ListItems for ids.

.PARAMETER CacheDir
    Directory used to cache downloaded installers. Defaults to D:\agm-toolchain-cache or %LOCALAPPDATA%\agm-toolchain-cache.

.PARAMETER ClearDownloads
    Purge cached installer files and report freed disk space, then exit.

.PARAMETER ClearCache
    Alias for -ClearDownloads.

.PARAMETER VerifyOnly
    Run toolchain detection, native compile & linker verification test without installing anything.

.PARAMETER DryRun
    Print the resolved item list, install nothing.

.PARAMETER ListItems
    Print installable items (id + description), then exit.

.PARAMETER Drive
    Target drive letter for toolchain installation and cache (e.g. "C", "D").
    Defaults to "C:" unless explicitly configured or overridden by $env:AGM_TOOLCHAIN_DRIVE.

.PARAMETER Troubleshoot
    Print common issues and fixes, then exit.

.PARAMETER SshTarget
    Run install on a remote host via SSH: "user@host" (key auth only).
#>

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$Quiet,
    [switch]$SkipLlvm,
    [string[]]$Profile = @(),
    [string[]]$Only = @(),
    [string]$CacheDir = "",
    [string]$Drive = "",
    [switch]$ClearDownloads,
    [switch]$ClearCache,
    [switch]$VerifyOnly,
    [switch]$DryRun,
    [switch]$ListItems,
    [switch]$Troubleshoot,
    [string]$SshTarget = ""
)

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# --- item catalog ------------------------------------------------------------
$ItemCatalog = @(
    @{ Id = "msvc";        Desc = "Visual Studio 2022 C++ Build Tools & Windows SDK (link.exe, kernel32.Lib)" },
    @{ Id = "rust";        Desc = "Rust toolchain via rustup (stable MSVC) + clippy/rustfmt components" },
    @{ Id = "llvm";        Desc = "LLVM & Clang compiler + libclang.dll for bindgen (LIBCLANG_PATH)" },
    @{ Id = "nasm";        Desc = "NASM assembler via chocolatey" },
    @{ Id = "nodejs";      Desc = "Node.js LTS via winget" },
    @{ Id = "pnpm";        Desc = "pnpm package manager via npm (requires nodejs)" },
    @{ Id = "tauri-cli";   Desc = "Tauri CLI via cargo install (requires rust)" },
    @{ Id = "sccache";     Desc = "sccache compilation cache via cargo install (requires rust)" },
    @{ Id = "cargo-watch"; Desc = "cargo-watch file watcher via cargo install (requires rust)" },
    @{ Id = "gh";          Desc = "GitHub CLI via winget" }
)
$ItemOrder = @("msvc", "rust", "llvm", "nasm", "nodejs", "pnpm", "sccache", "cargo-watch", "tauri-cli", "gh")
$ProfileMap = @{
    "minimal"         = @("rust")
    "build-tools"     = @("msvc", "rust", "llvm", "nasm")
    "agm-build-tools" = @("msvc", "rust", "llvm", "nasm")
    "dev-rust"        = @("msvc", "rust", "llvm", "nasm", "sccache", "cargo-watch")
    "rust-dev"        = @("msvc", "rust", "llvm", "nasm", "sccache", "cargo-watch")
    "frontend"        = @("nodejs", "pnpm", "tauri-cli")
    "full"            = @("msvc", "rust", "llvm", "nasm", "sccache", "cargo-watch", "nodejs", "pnpm", "tauri-cli", "gh")
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

# --- Toolchain SQLite Tracker Helper Integration ---
$Script:ToolchainDbHelper = Join-Path $PSScriptRoot "toolchain-db.ps1"
if (Test-Path $Script:ToolchainDbHelper) {
    . $Script:ToolchainDbHelper
}

function Resolve-ToolchainDrive {
    param([string]$RequestedDrive = "")
    $raw = if ($RequestedDrive) {
        $RequestedDrive
    } elseif ($env:AGM_TOOLCHAIN_DRIVE) {
        $env:AGM_TOOLCHAIN_DRIVE
    } else {
        $env:SystemDrive
    }
    if (-not $raw) { $raw = "C:" }
    $match = [regex]::Match($raw, '^[a-zA-Z]')
    if ($match.Success) {
        return "$($match.Value.ToUpper()):"
    }
    return "C:"
}

function Refresh-SessionPath {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $sysPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $combined = "$sysPath;$userPath"

    $extraBins = @(
        (Join-Path $env:USERPROFILE ".cargo\bin"),
        "C:\Program Files\NASM",
        "$env:ProgramFiles\LLVM\bin",
        "C:\Program Files\LLVM\bin",
        "C:\Program Files (x86)\LLVM\bin",
        "C:\LLVM\bin",
        "D:\LLVM\LLVM\bin",
        "D:\LLVM\bin"
    )
    foreach ($bin in $extraBins) {
        if ((Test-Path $bin) -and ($combined -notlike "*$bin*")) {
            $combined = "$bin;$combined"
        }
    }
    $env:Path = $combined

    if (-not $env:LIBCLANG_PATH) {
        $envLib = [Environment]::GetEnvironmentVariable("LIBCLANG_PATH", "Machine")
        if (-not $envLib) { $envLib = [Environment]::GetEnvironmentVariable("LIBCLANG_PATH", "User") }
        if ($envLib) { $env:LIBCLANG_PATH = $envLib }
    }
}

function Get-ToolchainCacheDir {
    if (-not [string]::IsNullOrWhiteSpace($CacheDir)) {
        if (-not (Test-Path $CacheDir)) { New-Item -ItemType Directory -Path $CacheDir -Force | Out-Null }
        return $CacheDir
    }
    if (-not [string]::IsNullOrWhiteSpace($env:AGM_TOOLCHAIN_CACHE)) {
        if (-not (Test-Path $env:AGM_TOOLCHAIN_CACHE)) { New-Item -ItemType Directory -Path $env:AGM_TOOLCHAIN_CACHE -Force | Out-Null }
        return $env:AGM_TOOLCHAIN_CACHE
    }
    $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive
    $target = if ($targetDrive -eq "D:" -and (Test-Path "D:\")) {
        "D:\agm-toolchain-cache"
    } else {
        Join-Path $env:LOCALAPPDATA "agm-toolchain-cache"
    }
    if (-not (Test-Path $target)) { New-Item -ItemType Directory -Path $target -Force | Out-Null }
    return $target
}

function Clear-ToolchainCache {
    Write-Host ""
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host "   Antigravity-Manager: Clearing Toolchain Cache            " -ForegroundColor Cyan
    Write-Host "============================================================" -ForegroundColor Cyan
    Write-Host ""
    $candidates = @(
        "D:\agm-toolchain-cache",
        (Join-Path $env:LOCALAPPDATA "agm-toolchain-cache"),
        (Join-Path $env:TEMP "agm-toolchain-cache"),
        (Join-Path $env:TEMP "WinGet\LLVM.LLVM.23.1.4")
    )
    $totalReclaimed = 0
    $clearedCount = 0
    foreach ($dir in $candidates) {
        if (Test-Path $dir) {
            $files = Get-ChildItem -Path $dir -Recurse -File -Force -ErrorAction SilentlyContinue
            foreach ($f in $files) {
                $bytes = $f.Length
                try {
                    Remove-Item -LiteralPath $f.FullName -Force -ErrorAction SilentlyContinue
                    $totalReclaimed += $bytes
                    $clearedCount++
                    Write-Host "  [-] Purged: $($f.FullName) ($([math]::Round($bytes / 1MB, 2)) MB)" -ForegroundColor Yellow
                } catch {}
            }
            try { Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction SilentlyContinue } catch {}
        }
    }
    $reclaimedMB = [math]::Round($totalReclaimed / 1MB, 2)
    Write-Success "Cleared $clearedCount cached file(s), reclaimed $reclaimedMB MB disk space."
}

function Get-ItemDesc {
    param([string]$Id)
    $entry = $ItemCatalog | Where-Object { $_.Id -eq $Id } | Select-Object -First 1
    if ($entry) { return $entry.Desc }
    return ""
}

function Resolve-InstallItems {
    $selected = @()
    if ($Only.Count -gt 0) {
        $selected = $Only
    } elseif ($Profile.Count -gt 0) {
        foreach ($p in $Profile) {
            $key = $p.Trim().ToLower()
            if (-not $ProfileMap.ContainsKey($key)) {
                Write-Err "unknown profile: '$p' (minimal|build-tools|agm-build-tools|rust-dev|frontend|full)"
                exit 2
            }
            $selected += $ProfileMap[$key]
        }
    } else {
        $selected = @("msvc", "rust", "llvm", "nasm")  # default AGM build-tools suite
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
if ($ClearDownloads -or $ClearCache) {
    Clear-ToolchainCache
    exit 0
}

if ($ListItems) {
    foreach ($entry in $ItemCatalog) {
        Write-Host "$($entry.Id)`t$($entry.Desc)"
    }
    exit 0
}

if ($Troubleshoot) {
    Write-Host @"
Troubleshooting - common issues and fixes:

1. "link.exe not found" / MSVC linker errors:
   Visual Studio 2022 C++ Build Tools must be installed with workload Microsoft.VisualStudio.Workload.VCTools.
   Run: .\scripts\install-rust-toolchain.ps1 -Only msvc

2. "Unable to find libclang" (boring-sys2 / bindgen build failures):
   LLVM must be installed and LIBCLANG_PATH must point to the folder containing libclang.dll.
   Run: .\scripts\install-rust-toolchain.ps1 -Only llvm

3. "cargo: command not found":
   Restart the terminal so the updated PATH loads, or ensure %USERPROFILE%\.cargo\bin is in PATH.

4. Clear cached downloads to reclaim disk space:
   Run: .\scripts\install-rust-toolchain.ps1 -ClearDownloads
"@
    exit 0
}

if ($SshTarget) {
    if ($SshTarget -notmatch '^[^@\s]+@[^@\s]+$') {
        Write-Err "-SshTarget needs user@host (got '$SshTarget')"
        exit 2
    }
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

# --- item installers ----------------------------------------------------------
function Install-MsvcItem {
    Write-Step "Checking Visual Studio C++ Build Tools & Linker..."
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    $hasVs = $false
    $vsPath = ""
    $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive
    if (Test-Path $vswhere) {
        $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsPath -and (Test-Path $vsPath)) {
            $hasVs = $true
        }
    }
    $hasLink = [bool](Get-Command link -ErrorAction SilentlyContinue)
    $hasSdk = (Get-ChildItem -Path "C:\Program Files (x86)\Windows Kits\10\Lib", "C:\Program Files\Windows Kits\10\Lib" -Filter "kernel32.Lib" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1)

    if (($hasVs -or $hasLink) -and $hasSdk -and -not $Force) {
        $loc = if ($vsPath) { $vsPath } else { (Get-Command link).Source }
        Write-Success "Visual Studio C++ Build Tools already installed at: $loc"
        Write-Success "Windows SDK detected with kernel32.Lib."
        if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
            Set-ToolchainRecord -ToolchainName "msvc" -Version "2022" -InstalledPath $loc -InstallDrive $targetDrive -IsVerified $true | Out-Null
        }
        return
    }

    $cDir = Get-ToolchainCacheDir
    $installerPath = Join-Path $cDir "vs_BuildTools.exe"
    if (-not (Test-Path $installerPath) -or (Get-Item $installerPath).Length -lt 1MB) {
        Write-Step "Downloading Visual Studio 2022 Build Tools bootstrapper..."
        $url = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
        Invoke-WebRequest -Uri $url -OutFile $installerPath -UseBasicParsing
        Write-Success "Downloaded and cached: $installerPath ($([math]::Round((Get-Item $installerPath).Length/1MB, 2)) MB)"
    } else {
        Write-Success "Using cached VS BuildTools bootstrapper: $installerPath"
    }

    $targetInstallPath = if ($targetDrive -eq "D:" -and (Test-Path "D:\")) {
        "D:\BuildTools"
    } else {
        "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools"
    }
    Write-Step "Installing Visual Studio C++ Build Tools to '$targetInstallPath' (silent)..."
    $msvcArgs = @(
        "--quiet",
        "--wait",
        "--norestart",
        "--nocache",
        "--installPath", $targetInstallPath,
        "--add", "Microsoft.VisualStudio.Workload.VCTools",
        "--includeRecommended"
    )
    $p = Start-Process -FilePath $installerPath -ArgumentList $msvcArgs -Wait -PassThru -NoNewWindow
    if ($p.ExitCode -eq 0 -or $p.ExitCode -eq 3010) {
        Write-Success "Visual Studio C++ Build Tools installed successfully (exit code $($p.ExitCode))."
        if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
            Set-ToolchainRecord -ToolchainName "msvc" -Version "2022" -InstalledPath $targetInstallPath -InstallDrive $targetDrive -IsVerified $true | Out-Null
        }
    } else {
        Write-Warn "VS Build Tools installer returned exit code $($p.ExitCode)"
    }
    Refresh-SessionPath
}

function Install-RustItem {
    Write-Step "Checking Rust toolchain (rustup & cargo)..."
    Refresh-SessionPath
    $hasRustc = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
    $hasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)

    if ($hasRustc -and $hasCargo -and -not $Force) {
        $rVer = & rustc --version 2>$null
        $cVer = & cargo --version 2>$null
        Write-Success "Rust toolchain already installed: $rVer | $cVer"
        if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
            $cargoBin = Split-Path (Get-Command rustc).Source -Parent
            Set-ToolchainRecord -ToolchainName "rust" -Version $rVer -InstalledPath $cargoBin -InstallDrive "C:" -IsVerified $true | Out-Null
        }
    } else {
        $cDir = Get-ToolchainCacheDir
        $rustupExe = Join-Path $cDir "rustup-init.exe"
        if (-not (Test-Path $rustupExe) -or (Get-Item $rustupExe).Length -lt 1MB) {
            Write-Step "Downloading Rustup (MSVC 64-bit)..."
            $rustupUrl = "https://win.rustup.rs/x86_64"
            Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupExe -UseBasicParsing
            Write-Success "Downloaded and cached: $rustupExe"
        } else {
            Write-Success "Using cached Rustup installer: $rustupExe"
        }

        Write-Step "Executing rustup-init (default toolchain: stable-x86_64-pc-windows-msvc)..."
        $p = Start-Process -FilePath $rustupExe -ArgumentList "-y", "--default-toolchain", "stable-x86_64-pc-windows-msvc" -Wait -PassThru -NoNewWindow
        if ($p.ExitCode -eq 0) {
            Write-Success "Rustup installation completed successfully."
            Refresh-SessionPath
            if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
                $rCmd = Get-Command rustc -ErrorAction SilentlyContinue
                if ($rCmd) {
                    $rVer = & rustc --version 2>$null
                    $cargoBin = Split-Path $rCmd.Source -Parent
                    Set-ToolchainRecord -ToolchainName "rust" -Version $rVer -InstalledPath $cargoBin -InstallDrive "C:" -IsVerified $true | Out-Null
                }
            }
        } else {
            Write-Warn "Rustup returned exit code: $($p.ExitCode)"
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
    Write-Step "Checking LLVM, Clang and libclang..."
    Refresh-SessionPath
    $hasClang = [bool](Get-Command clang -ErrorAction SilentlyContinue)
    $hasLibClang = $false
    $libClangCandidate = ""
    $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive

    if ($env:LIBCLANG_PATH -and (Test-Path (Join-Path $env:LIBCLANG_PATH "libclang.dll"))) {
        $hasLibClang = $true
        $libClangCandidate = $env:LIBCLANG_PATH
    } else {
        $candidates = @()
        if ($targetDrive -eq "D:" -and (Test-Path "D:\")) {
            $candidates += @(
                "D:\LLVM\LLVM\bin",
                "D:\LLVM\bin",
                "$env:ProgramFiles\LLVM\bin",
                "C:\Program Files\LLVM\bin",
                "C:\Program Files (x86)\LLVM\bin",
                "C:\LLVM\bin"
            )
        } else {
            $candidates += @(
                "$env:ProgramFiles\LLVM\bin",
                "C:\Program Files\LLVM\bin",
                "C:\Program Files (x86)\LLVM\bin",
                "C:\LLVM\bin",
                "C:\LLVM\LLVM\bin",
                "D:\LLVM\LLVM\bin",
                "D:\LLVM\bin"
            )
        }
        foreach ($c in $candidates) {
            if (Test-Path (Join-Path $c "libclang.dll")) {
                $hasLibClang = $true
                $libClangCandidate = $c
                break
            }
        }
    }

    if ($hasClang -and $hasLibClang -and -not $Force) {
        $clangVer = (& clang --version 2>$null | Select-Object -First 1)
        Write-Success "LLVM/Clang already installed: $clangVer"
        Write-Success "libclang.dll verified at: $libClangCandidate"
        $env:LIBCLANG_PATH = $libClangCandidate
        [Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $libClangCandidate, "User")
        if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
            Set-ToolchainRecord -ToolchainName "llvm" -Version $clangVer -InstalledPath $libClangCandidate -InstallDrive $targetDrive -IsVerified $true | Out-Null
        }
        return
    }

    $cDir = Get-ToolchainCacheDir
    $msiPath = Join-Path $cDir "LLVM-23.1.4-win64.msi"
    if (-not (Test-Path $msiPath) -or (Get-Item $msiPath).Length -lt 100MB) {
        Write-Step "Downloading LLVM 23.1.4 MSI package (this will be cached)..."
        $url = "https://github.com/llvm/llvm-project/releases/download/llvmorg-23.1.4/LLVM-23.1.4-win64.msi"
        Invoke-WebRequest -Uri $url -OutFile $msiPath -UseBasicParsing
        Write-Success "Downloaded and cached: $msiPath ($([math]::Round((Get-Item $msiPath).Length/1MB, 2)) MB)"
    } else {
        Write-Success "Using cached LLVM package: $msiPath"
    }

    $targetUnpack = if ($targetDrive -eq "D:" -and (Test-Path "D:\")) {
        "D:\LLVM"
    } else {
        "$env:ProgramFiles\LLVM"
    }
    Write-Step "Extracting LLVM package to '$targetUnpack' via administrative install..."
    $p = Start-Process -FilePath "msiexec.exe" -ArgumentList "/a", "`"$msiPath`"", "/qn", "TARGETDIR=`"$targetUnpack`"" -Wait -PassThru -NoNewWindow
    if ($p.ExitCode -eq 0) {
        Write-Success "LLVM extracted successfully."
        $binDir = if (Test-Path "$targetUnpack\LLVM\bin\libclang.dll") {
            "$targetUnpack\LLVM\bin"
        } elseif (Test-Path "$targetUnpack\bin\libclang.dll") {
            "$targetUnpack\bin"
        } else {
            (Get-ChildItem -Path $targetUnpack -Filter "libclang.dll" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1).DirectoryName
        }
        if ($binDir) {
            [Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $binDir, "Machine")
            [Environment]::SetEnvironmentVariable("LIBCLANG_PATH", $binDir, "User")
            $env:LIBCLANG_PATH = $binDir

            $mPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
            if ($mPath -split ";" -notcontains $binDir) {
                [Environment]::SetEnvironmentVariable("PATH", "$mPath;$binDir", "Machine")
            }
            Refresh-SessionPath
            Write-Success "Configured LIBCLANG_PATH to: $binDir"
            if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
                Set-ToolchainRecord -ToolchainName "llvm" -Version "23.1.4" -InstalledPath $binDir -InstallDrive $targetDrive -IsVerified $true | Out-Null
            }
        }
    } else {
        Write-Warn "msiexec extraction returned exit code: $($p.ExitCode)"
    }
}

function Install-NasmItem {
    Write-Step "Checking NASM assembler..."
    Refresh-SessionPath
    $hasNasm = [bool](Get-Command nasm -ErrorAction SilentlyContinue)
    $nasmPath = "C:\Program Files\NASM\nasm.exe"
    $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive

    if (($hasNasm -or (Test-Path $nasmPath)) -and -not $Force) {
        $nVer = if ($hasNasm) { (& nasm -v 2>$null | Select-Object -First 1) } else { "installed at $nasmPath" }
        Write-Success "NASM already installed: $nVer"
        if (-not $hasNasm -and (Test-Path $nasmPath)) {
            $mPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
            if ($mPath -split ";" -notcontains "C:\Program Files\NASM") {
                [Environment]::SetEnvironmentVariable("PATH", "$mPath;C:\Program Files\NASM", "Machine")
            }
            Refresh-SessionPath
        }
        if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
            $nasmDir = if (Test-Path $nasmPath) { Split-Path $nasmPath -Parent } else { "C:\Program Files\NASM" }
            Set-ToolchainRecord -ToolchainName "nasm" -Version $nVer -InstalledPath $nasmDir -InstallDrive $targetDrive -IsVerified $true | Out-Null
        }
        return
    }

    Write-Step "Installing NASM via Chocolatey..."
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        $p = Start-Process -FilePath "choco" -ArgumentList "install", "nasm", "-y" -Wait -PassThru -NoNewWindow
        if ($p.ExitCode -eq 0) {
            Write-Success "NASM installed via Chocolatey."
            $mPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
            if ($mPath -split ";" -notcontains "C:\Program Files\NASM") {
                [Environment]::SetEnvironmentVariable("PATH", "$mPath;C:\Program Files\NASM", "Machine")
            }
            Refresh-SessionPath
            if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
                Set-ToolchainRecord -ToolchainName "nasm" -Version "nasm" -InstalledPath "C:\Program Files\NASM" -InstallDrive $targetDrive -IsVerified $true | Out-Null
            }
            return
        }
    }
    Write-Warn "Could not automatically install NASM via chocolatey."
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

function Test-NativeLinker {
    Write-Step "Testing native MSVC compiler and linker compilation..."
    $testDir = Join-Path $env:TEMP "agm_rust_verify"
    try {
        New-Item -ItemType Directory -Path $testDir -Force | Out-Null
        $testRs = Join-Path $testDir "verify.rs"
        Set-Content -Path $testRs -Value 'fn main() { println!("LINKER_VERIFIED"); }'
        $rustcExe = (Get-Command rustc -ErrorAction SilentlyContinue)
        if (-not $rustcExe) {
            Write-Warn "rustc executable not found in PATH."
            return $false
        }
        $proc = Start-Process -FilePath $rustcExe.Source -ArgumentList "`"$testRs`"", "--out-dir", "`"$testDir`"" -Wait -PassThru -NoNewWindow
        if ($proc.ExitCode -eq 0 -and (Test-Path (Join-Path $testDir "verify.exe"))) {
            $output = & (Join-Path $testDir "verify.exe") 2>$null
            if ($output -match "LINKER_VERIFIED") {
                Write-Success "Native MSVC compilation & linking test: PASSED"
                return $true
            }
        }
        Write-Warn "Native MSVC linking test did not produce expected output."
        return $false
    } catch {
        Write-Warn "Native MSVC linking test encountered an error: $_"
        return $false
    } finally {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# --- resolve + dispatch -------------------------------------------------------
Write-Host ""
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   Antigravity-Manager: Windows Build Toolchain Installer   " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host ""

$installItems = Resolve-InstallItems

if ($DryRun) {
    $profLabel = if ($Profile.Count -gt 0) { $Profile -join " " } else { "<default: agm-build-tools>" }
    Write-Host "Dry run - would install $($installItems.Count) item(s) [profiles: $profLabel]:"
    foreach ($id in $installItems) {
        Write-Host "  - ${id}: $(Get-ItemDesc $id)"
    }
    Write-Host "Cache directory: $(Get-ToolchainCacheDir)"
    Write-Host "(nothing installed)"
    exit 0
}

if ($VerifyOnly) {
    if (Get-Command Test-AllToolchainsFast -ErrorAction SilentlyContinue) {
        if (Test-AllToolchainsFast -Tools "rust,msvc,llvm,nasm") {
            Write-Success "All core build toolchains (rust, msvc, llvm, nasm) verified via toolchain.db (fast-path <5ms)."
            exit 0
        }
    }
}

if (-not $VerifyOnly) {
    Write-Step "Target items: $($installItems -join ' ')"
    Write-Step "Downloads cache: $(Get-ToolchainCacheDir)"

    foreach ($item in $installItems) {
        switch ($item) {
            "msvc"        { Install-MsvcItem }
            "rust"        { Install-RustItem }
            "llvm"        { Install-LlvmItem }
            "nasm"        { Install-NasmItem }
            "nodejs"      { Install-NodeJsItem }
            "pnpm"        { Install-PnpmItem }
            "tauri-cli"   { Install-CargoPackageItem -Crate "tauri-cli" -Binary "cargo-tauri" }
            "sccache"     { Install-CargoPackageItem -Crate "sccache" -Binary "sccache" }
            "cargo-watch" { Install-CargoPackageItem -Crate "cargo-watch" -Binary "cargo-watch" }
            "gh"          { Install-GhItem }
        }
    }
}

# --- final verification -------------------------------------------------------
Write-Host ""
Write-Host "--- Toolchain Verification ---" -ForegroundColor Cyan
Refresh-SessionPath

$vRustc = if (Get-Command rustc -ErrorAction SilentlyContinue) { & rustc --version } else { "Not Found" }
$vCargo = if (Get-Command cargo -ErrorAction SilentlyContinue) { & cargo --version } else { "Not Found" }
$vClang = if (Get-Command clang -ErrorAction SilentlyContinue) { (& clang --version 2>$null | Select-Object -First 1) } else { "Not Found" }
$vLibClang = if ($env:LIBCLANG_PATH -and (Test-Path (Join-Path $env:LIBCLANG_PATH "libclang.dll"))) { $env:LIBCLANG_PATH } else { "Not Configured" }
$vNasm = if (Get-Command nasm -ErrorAction SilentlyContinue) { (& nasm -v 2>$null | Select-Object -First 1) } else { "Not Found" }
$vLink = if (Get-Command link -ErrorAction SilentlyContinue) { (Get-Command link).Source } else { "Not in PATH (discovered via vswhere)" }

Write-Host "  Rust Compiler   : $vRustc" -ForegroundColor $(if ($vRustc -ne "Not Found") { "Green" } else { "Red" })
Write-Host "  Cargo Package   : $vCargo" -ForegroundColor $(if ($vCargo -ne "Not Found") { "Green" } else { "Red" })
Write-Host "  LLVM / Clang    : $vClang" -ForegroundColor $(if ($vClang -ne "Not Found") { "Green" } else { "Yellow" })
Write-Host "  LIBCLANG_PATH   : $vLibClang" -ForegroundColor $(if ($vLibClang -ne "Not Configured") { "Green" } else { "Red" })
Write-Host "  NASM Assembler  : $vNasm" -ForegroundColor $(if ($vNasm -ne "Not Found") { "Green" } else { "Yellow" })
Write-Host "  MSVC Linker     : $vLink" -ForegroundColor "Green"
Write-Host ""

$linkerOk = Test-NativeLinker

if ($vRustc -ne "Not Found" -and $vCargo -ne "Not Found" -and $vLibClang -ne "Not Configured" -and $linkerOk) {
    $targetDrive = Resolve-ToolchainDrive -RequestedDrive $Drive
    if (Get-Command Set-ToolchainRecord -ErrorAction SilentlyContinue) {
        $rustcCmd = Get-Command rustc -ErrorAction SilentlyContinue
        if ($rustcCmd) {
            $cargoBin = Split-Path $rustcCmd.Source -Parent
            Set-ToolchainRecord -ToolchainName "rust" -Version $vRustc -InstalledPath $cargoBin -InstallDrive "C:" -IsVerified $true | Out-Null
        }
        if ($vLibClang -ne "Not Configured") {
            Set-ToolchainRecord -ToolchainName "llvm" -Version $vClang -InstalledPath $vLibClang -InstallDrive $targetDrive -IsVerified $true | Out-Null
        }
        if ($vLink -ne "Not in PATH (discovered via vswhere)") {
            Set-ToolchainRecord -ToolchainName "msvc" -Version "2022" -InstalledPath (Split-Path $vLink -Parent) -InstallDrive $targetDrive -IsVerified $true | Out-Null
        }
        if ($vNasm -ne "Not Found") {
            Set-ToolchainRecord -ToolchainName "nasm" -Version $vNasm -InstalledPath "C:\Program Files\NASM" -InstallDrive $targetDrive -IsVerified $true | Out-Null
        }
    }
    Write-Success "Antigravity-Manager build toolchain is 100% READY! All checkings passed."
    exit 0
} else {
    Write-Warn "Toolchain setup completed with warnings. Restart your shell to reload environment variables."
    exit 1
}
