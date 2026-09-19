<#
.SYNOPSIS
    Standalone Portable Installer for Antigravity Tools on Windows.

.DESCRIPTION
    Downloads and installs portable Antigravity Tools release assets without requiring an installer wizard.
    Configures user PATH, desktop, and start menu shortcuts automatically.

.PARAMETER Version
    Specific release version to install (e.g. "4.9.0"). Defaults to latest.

.PARAMETER InstallDir
    Target directory for installation.
    Default: "$env:LOCALAPPDATA\Programs\Antigravity-Tools"

.PARAMETER Arch
    Target system architecture ("x64" or "arm64"). Defaults to system architecture.

.PARAMETER NoPath
    Skip adding the install directory to user PATH.

.PARAMETER NoShortcut
    Skip creating Start Menu and Desktop shortcuts.

.PARAMETER DryRun
    Simulate actions without downloading or modifying the system.

.PARAMETER Uninstall
    Remove Antigravity Tools installation, shortcuts, and PATH entries.

.EXAMPLE
    irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
    .\install.ps1 -Version "4.9.0"
    .\install.ps1 -Uninstall
#>

[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$InstallDir = "",
    [string]$Arch = "",
    [switch]$NoPath,
    [switch]$NoShortcut,
    [switch]$DryRun,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$Repo = "alimtvnetwork/Antigravity-Manager"
$UpstreamRepo = "lbjlaq/Antigravity-Manager"
$AppName = "Anti-Gravity Tools by Alim"
$BinaryName = "Anti-Gravity Tools by Alim.exe"

function Write-Step {
    param([string]$Message)
    Write-Host "[*] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[!] $Message" -ForegroundColor Yellow
}

function Write-Err {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Resolve Architecture
if (-not $Arch) {
    if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64" -or $env:PROCESSOR_ARCHITEW6432 -eq "ARM64") {
        $Arch = "arm64"
    } else {
        $Arch = "x64"
    }
}

# Resolve Default Install Directory
if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\Antigravity-Tools"
}

# Shortcuts paths
$StartMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$StartMenuShortcut = Join-Path $StartMenuDir "$AppName.lnk"
$DesktopDir = [Environment]::GetFolderPath("Desktop")
$DesktopShortcut = Join-Path $DesktopDir "$AppName.lnk"
$TaskbarDir = Join-Path $env:APPDATA "Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar"

function Remove-LegacyUpstreamInstallation {
    Write-Step "Checking for legacy or upstream (lbjlaq/Antigravity-Manager) installations..."

    # 1. Stop any running Antigravity Tools processes
    $legacyProcesses = Get-Process | Where-Object {
        $_.ProcessName -eq "antigravity-tools" -or
        $_.ProcessName -eq "Anti-Gravity Tools" -or
        $_.ProcessName -eq "Anti-Gravity Tools by Alim"
    }
    if ($legacyProcesses) {
        Write-Step "Stopping running Antigravity Tools processes..."
        foreach ($proc in $legacyProcesses) {
            try {
                Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            } catch {}
        }
        Start-Sleep -Milliseconds 800
    }

    # 2. Check Windows Registry Uninstall entries
    $regPaths = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )

    foreach ($regPath in $regPaths) {
        $entries = Get-ItemProperty -Path $regPath -ErrorAction SilentlyContinue | Where-Object {
            ($_.Publisher -match "lbjlaq") -or
            ($_.DisplayName -eq "Antigravity Tools") -or
            ($_.InstallLocation -match "Antigravity Tools")
        }

        if ($entries) {
            foreach ($entry in $entries) {
                # Skip if already our brand in the exact target install dir
                if ($entry.DisplayName -eq $AppName) {
                    continue
                }

                Write-Step "Found legacy installation: $($entry.DisplayName) by $($entry.Publisher)"
                if ($entry.UninstallString) {
                    Write-Step "Executing legacy uninstaller: $($entry.UninstallString)"
                    try {
                        $uninstClean = $entry.UninstallString.Trim('"')
                        if (Test-Path $uninstClean) {
                            $p = Start-Process -FilePath $uninstClean -ArgumentList "/S", "/currentuser" -Wait -PassThru
                            Write-Success "Legacy uninstaller completed (exit code: $($p.ExitCode))"
                        }
                    } catch {
                        Write-Warn "Could not execute uninstaller: $_"
                    }
                }
            }
        }
    }

    # 3. Clean legacy directory paths if still existing
    $legacyDirs = @(
        (Join-Path $env:LOCALAPPDATA "Antigravity Tools"),
        (Join-Path $env:LOCALAPPDATA "Programs\antigravity-tools"),
        (Join-Path $env:ProgramFiles "Antigravity Tools"),
        (Join-Path ${env:ProgramFiles(x86)} "Antigravity Tools")
    )

    foreach ($ldir in $legacyDirs) {
        if (Test-Path $ldir) {
            if ($ldir -ne $InstallDir) {
                $uninstExe = Join-Path $ldir "uninstall.exe"
                if (Test-Path $uninstExe) {
                    Write-Step "Running uninstaller in $ldir..."
                    try {
                        Start-Process -FilePath $uninstExe -ArgumentList "/S" -Wait -ErrorAction SilentlyContinue
                    } catch {}
                }
                Write-Step "Purging legacy directory: $ldir"
                Remove-Item -Path $ldir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
    }

    # 4. Clean legacy shortcuts from Start Menu, Desktop, and Taskbar
    $legacyShortcuts = @(
        (Join-Path $StartMenuDir "Antigravity Tools.lnk"),
        (Join-Path $StartMenuDir "antigravity-tools.lnk"),
        (Join-Path $DesktopDir "Antigravity Tools.lnk"),
        (Join-Path $DesktopDir "antigravity-tools.lnk"),
        (Join-Path $TaskbarDir "Antigravity Tools.lnk"),
        (Join-Path $TaskbarDir "antigravity-tools.lnk")
    )
    foreach ($sc in $legacyShortcuts) {
        if (Test-Path $sc) {
            Remove-Item -Path $sc -Force -ErrorAction SilentlyContinue
        }
    }

    Write-Success "Legacy cleanup complete."
}

function Pin-TaskbarShortcut {
    param(
        [string]$TargetExe,
        [string]$TargetWorkDir,
        [string]$ShortcutSource
    )

    Write-Step "Configuring Taskbar pinning for Windows 10, 11, and Windows Server..."

    if (-not (Test-Path $TaskbarDir)) {
        try {
            New-Item -ItemType Directory -Path $TaskbarDir -Force | Out-Null
        } catch {}
    }

    # Method 1: Create or update shortcut directly in User Pinned Taskbar directory
    if (Test-Path $TaskbarDir) {
        $pinnedShortcut = Join-Path $TaskbarDir "$AppName.lnk"
        try {
            $WshShell = New-Object -ComObject WScript.Shell
            $sc = $WshShell.CreateShortcut($pinnedShortcut)
            $sc.TargetPath = $TargetExe
            $sc.WorkingDirectory = $TargetWorkDir
            $sc.Description = $AppName
            $sc.Save()
            Write-Success "Pinned shortcut created in Taskbar directory: $pinnedShortcut"
        } catch {
            Write-Warn "Direct Taskbar shortcut creation note: $_"
        }
    }

    # Method 2: Shell.Application InvokeVerb (Native Pin to taskbar verb for Windows 10, 11, and Windows Server)
    if ($ShortcutSource) {
        if (Test-Path $ShortcutSource) {
            try {
                $shell = New-Object -ComObject Shell.Application
                $folderPath = Split-Path $ShortcutSource
                $fileName = Split-Path $ShortcutSource -Leaf
                $folder = $shell.Namespace($folderPath)
                if ($folder) {
                    $item = $folder.ParseName($fileName)
                    if ($item) {
                        $verbs = $item.Verbs()
                        $pinVerb = $verbs | Where-Object {
                            $_.Name.Replace('&', '') -match '^(Pin to taskbar|TaskbarPin|固定到任务栏|An Taskleiste anheften|Épingler à la barre des tâches)'
                        }
                        if ($pinVerb) {
                            $pinVerb.DoIt()
                            Write-Success "Invoked shell verb: $($pinVerb.Name)"
                        }
                    }
                }
            } catch {
                Write-Warn "Shell verb taskbar pinning note: $_"
            }
        }
    }
}

# --- UNINSTALL FLOW ---
if ($Uninstall) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Magenta
    Write-Host "    $AppName Uninstaller" -ForegroundColor Magenta
    Write-Host "========================================" -ForegroundColor Magenta
    Write-Host ""

    if ($DryRun) {
        Write-Warn "[DRY RUN] Would remove install directory: $InstallDir"
        Write-Warn "[DRY RUN] Would remove shortcuts: $StartMenuShortcut, $DesktopShortcut"
        Write-Warn "[DRY RUN] Would clean User PATH"
        return
    }

    # 1. Remove files
    if (Test-Path $InstallDir) {
        Write-Step "Removing installation directory: $InstallDir"
        Remove-Item -Path $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    # 2. Remove shortcuts
    if (Test-Path $StartMenuShortcut) {
        Write-Step "Removing Start Menu shortcut"
        Remove-Item -Path $StartMenuShortcut -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $DesktopShortcut) {
        Write-Step "Removing Desktop shortcut"
        Remove-Item -Path $DesktopShortcut -Force -ErrorAction SilentlyContinue
    }

    # 3. Clean User PATH
    Write-Step "Cleaning User PATH environment variable..."
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath) {
        $paths = $userPath -split ";" | Where-Object { $_ -and $_ -ne $InstallDir }
        $newPath = $paths -join ";"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    }

    Write-Success "$AppName has been cleanly uninstalled."
    return
}

# --- INSTALL FLOW ---
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "    $AppName Portable Installer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Remove any pre-existing lbjlaq/Antigravity-Manager or legacy installations first
Remove-LegacyUpstreamInstallation

# Step 1: Resolve Release Version
$TargetVersion = $Version
if ($TargetVersion) {
    $TargetVersion = $TargetVersion -replace "^v", ""
    # Attempt to resolve asset metadata for explicitly pinned version
    try {
        $tagEndpoint = "https://api.github.com/repos/$Repo/releases/tags/v$TargetVersion"
        $releaseData = Invoke-RestMethod -Uri $tagEndpoint -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 6
    } catch {
        try {
            $tagEndpoint = "https://api.github.com/repos/$UpstreamRepo/releases/tags/v$TargetVersion"
            $releaseData = Invoke-RestMethod -Uri $tagEndpoint -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 6
        } catch {}
    }
} else {
    Write-Step "Discovering latest release version from GitHub..."
    $apiEndpoints = @(
        "https://api.github.com/repos/$Repo/releases",
        "https://api.github.com/repos/$Repo/releases/latest",
        "https://api.github.com/repos/$UpstreamRepo/releases",
        "https://api.github.com/repos/$UpstreamRepo/releases/latest"
    )

    foreach ($endpoint in $apiEndpoints) {
        try {
            $resp = Invoke-RestMethod -Uri $endpoint -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 8
            if ($resp -is [System.Array] -and $resp.Count -gt 0) {
                # Pick newest release that already has uploaded assets
                $candidate = $resp | Where-Object { $_.assets -and $_.assets.Count -gt 0 } | Select-Object -First 1
                if (-not $candidate) { $candidate = $resp[0] }
                if ($candidate -and $candidate.tag_name) {
                    $releaseData = $candidate
                    $TargetVersion = $releaseData.tag_name -replace "^v", ""
                    break
                }
            } elseif ($resp -and $resp.tag_name) {
                $releaseData = $resp
                $TargetVersion = $releaseData.tag_name -replace "^v", ""
                break
            }
        } catch {
            # Try next endpoint
        }
    }

    # Fallback to updater.json
    if (-not $TargetVersion) {
        try {
            $updater = Invoke-RestMethod -Uri "https://github.com/$Repo/releases/latest/download/updater.json" -TimeoutSec 6
            if ($updater -and $updater.version) {
                $TargetVersion = $updater.version -replace "^v", ""
            }
        } catch {}
    }

    if (-not $TargetVersion) {
        $TargetVersion = "4.28.0"
        Write-Warn "Could not resolve latest tag from API, falling back to default v$TargetVersion"
    }
}

Write-Success "Target release version: v$TargetVersion (Architecture: $Arch)"

# Step 2: Determine Asset URL
$matchedAsset = $null

if ($releaseData -and $releaseData.assets) {
    # 1. Prioritize architecture-specific NSIS setup EXE
    $matchedAsset = $releaseData.assets | Where-Object { $_.name -like "*${Arch}*setup.exe" -or $_.name -like "*setup.exe" } | Select-Object -First 1
    # 2. Other Windows executables
    if (-not $matchedAsset) {
        $matchedAsset = $releaseData.assets | Where-Object { $_.name -like "*.exe" -and $_.name -notlike "*build*" } | Select-Object -First 1
    }
}

if ($matchedAsset) {
    $DownloadUrl = $matchedAsset.browser_download_url
} else {
    # Direct NSIS setup asset URL fallback
    $ExeAsset = "Anti-Gravity.Tools.by.Alim_${TargetVersion}_${Arch}-setup.exe"
    $DownloadUrl = "https://github.com/$Repo/releases/download/v${TargetVersion}/$ExeAsset"
}

Write-Step "Download source: $DownloadUrl"

# Step 3: Execute Installation
if ($DryRun) {
    Write-Warn "[DRY RUN] Would download $DownloadUrl"
    Write-Warn "[DRY RUN] Would execute/install into $InstallDir"
    if (-not $NoPath) { Write-Warn "[DRY RUN] Would append $InstallDir to User PATH" }
    if (-not $NoShortcut) { Write-Warn "[DRY RUN] Would create Desktop & Start Menu shortcuts" }
    Write-Success "[DRY RUN] Dry run completed successfully."
    return
}

$TempDir = [System.IO.Path]::GetTempPath()
$DownloadedFile = Join-Path $TempDir ($DownloadUrl -split "/" | Select-Object -Last 1)

Write-Step "Downloading release package..."
try {
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12 -bor [System.Net.SecurityProtocolType]::Tls13
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $DownloadedFile -UseBasicParsing
} catch {
    Write-Warn "Primary download failed: $_. Attempting upstream fallback..."
    $UpstreamDownloadUrl = $DownloadUrl -replace [regex]::Escape($Repo), $UpstreamRepo
    try {
        Invoke-WebRequest -Uri $UpstreamDownloadUrl -OutFile $DownloadedFile -UseBasicParsing
        Write-Success "Downloaded successfully from upstream: $UpstreamDownloadUrl"
    } catch {
        Write-Err "Upstream fallback download also failed: $_"
        exit 1
    }
}

if (-not (Test-Path $DownloadedFile)) {
    Write-Err "Downloaded file not found at: $DownloadedFile"
    exit 1
}

# Create Target Directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Step "Running installer ($DownloadedFile)..."
$installProc = Start-Process -FilePath $DownloadedFile -ArgumentList "/S", "/D=$InstallDir" -Wait -PassThru
Write-Success "Installer finished with exit code $($installProc.ExitCode)"
Remove-Item $DownloadedFile -Force -ErrorAction SilentlyContinue

# Locate Main Executable
$ExePath = Join-Path $InstallDir $BinaryName
if (-not (Test-Path $ExePath)) {
    $altNames = @(
        "Anti-Gravity Tools by Alim.exe",
        "Anti-Gravity Tools.exe",
        "antigravity-tools.exe",
        "Antigravity Tools.exe"
    )
    foreach ($name in $altNames) {
        $candidate = Join-Path $InstallDir $name
        if (Test-Path $candidate) {
            $ExePath = $candidate
            $BinaryName = $name
            break
        }
    }
}
if (-not (Test-Path $ExePath)) {
    $found = Get-ChildItem -Path $InstallDir -Filter "*.exe" -Recurse | Where-Object { $_.Name -notlike "*uninstall*" -and $_.Name -notlike "*setup*" } | Select-Object -First 1
    if ($found) {
        $ExePath = $found.FullName
        $BinaryName = $found.Name
    }
}
if (-not (Test-Path $ExePath)) {
    $commonDirs = @(
        (Join-Path $env:LOCALAPPDATA "Programs\Anti-Gravity Tools by Alim"),
        (Join-Path $env:LOCALAPPDATA "Programs\antigravity-tools"),
        (Join-Path $env:ProgramFiles "Anti-Gravity Tools by Alim")
    )
    foreach ($dir in $commonDirs) {
        if (Test-Path $dir) {
            $found = Get-ChildItem -Path $dir -Filter "*.exe" -Recurse | Where-Object { $_.Name -notlike "*uninstall*" } | Select-Object -First 1
            if ($found) {
                $InstallDir = $dir
                $ExePath = $found.FullName
                $BinaryName = $found.Name
                break
            }
        }
    }
}

# Step 4: Configure User PATH
if (-not $NoPath) {
    Write-Step "Configuring User PATH environment variable..."
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        $newPath = if ($currentPath) { "$currentPath;$InstallDir" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Success "Added to User PATH: $InstallDir"
    } else {
        Write-Step "User PATH already includes install directory."
    }
}

# Step 5: Create Shortcuts
if (-not $NoShortcut -and (Test-Path $ExePath)) {
    Write-Step "Creating Application Shortcuts..."
    try {
        $WshShell = New-Object -ComObject WScript.Shell

        # Start Menu
        if (-not (Test-Path $StartMenuDir)) {
            New-Item -ItemType Directory -Path $StartMenuDir -Force | Out-Null
        }
        $smShortcut = $WshShell.CreateShortcut($StartMenuShortcut)
        $smShortcut.TargetPath = $ExePath
        $smShortcut.WorkingDirectory = $InstallDir
        $smShortcut.Description = "Antigravity Tools - AI Account & Instance Management"
        $smShortcut.Save()
        Write-Success "Created Start Menu shortcut: $StartMenuShortcut"

        # Desktop
        if (Test-Path $DesktopDir) {
            $dtShortcut = $WshShell.CreateShortcut($DesktopShortcut)
            $dtShortcut.TargetPath = $ExePath
            $dtShortcut.WorkingDirectory = $InstallDir
            $dtShortcut.Description = "Antigravity Tools"
            $dtShortcut.Save()
            Write-Success "Created Desktop shortcut: $DesktopShortcut"
        }
    } catch {
        Write-Warn "Could not create shortcuts: $_"
    }

    # Step 6: Taskbar Pinning (Windows 10, Windows 11, Windows Server)
    Pin-TaskbarShortcut -TargetExe $ExePath -TargetWorkDir $InstallDir -ShortcutSource $StartMenuShortcut
}

Write-Host ""
Write-Success "Installation of $AppName v$TargetVersion completed successfully!"
Write-Host "Target Directory: $InstallDir" -ForegroundColor Gray
Write-Host "Executable:       $ExePath" -ForegroundColor Gray
Write-Host ""
Write-Host "You can now launch '$AppName' directly or run 'Antigravity Tools.exe' from any terminal." -ForegroundColor Green
Write-Host ""
