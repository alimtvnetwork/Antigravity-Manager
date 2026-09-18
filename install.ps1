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
$AppName = "Antigravity Tools"
$BinaryName = "Antigravity Tools.exe"

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
        "https://api.github.com/repos/$Repo/releases/latest",
        "https://api.github.com/repos/$UpstreamRepo/releases/latest"
    )

    foreach ($endpoint in $apiEndpoints) {
        try {
            $releaseData = Invoke-RestMethod -Uri $endpoint -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 8
            if ($releaseData -and $releaseData.tag_name) {
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
        $TargetVersion = "4.21.0"
        Write-Warn "Could not resolve latest tag from API, falling back to default v$TargetVersion"
    }
}

Write-Success "Target release version: v$TargetVersion (Architecture: $Arch)"

# Step 2: Determine Asset URL
$matchedAsset = $null
$ZipPattern = "*windows_${Arch}.zip"
$NsisPattern = "*_${Arch}-setup.exe"

if ($releaseData -and $releaseData.assets) {
    $matchedAsset = $releaseData.assets | Where-Object { $_.name -like $ZipPattern -or $_.name -like "*.zip" } | Select-Object -First 1
    if (-not $matchedAsset) {
        $matchedAsset = $releaseData.assets | Where-Object { $_.name -like $NsisPattern -or $_.name -like "*-setup.exe" } | Select-Object -First 1
    }
}

if ($matchedAsset) {
    $DownloadUrl = $matchedAsset.browser_download_url
    $IsZipPackage = $matchedAsset.name -like "*.zip"
} else {
    $ZipAsset = "Antigravity.Tools_${TargetVersion}_windows_${Arch}.zip"
    $DownloadUrl = "https://github.com/$Repo/releases/download/v${TargetVersion}/$ZipAsset"
    $IsZipPackage = $true
}

Write-Step "Download source: $DownloadUrl"

# Step 3: Execute Installation
if ($DryRun) {
    Write-Warn "[DRY RUN] Would download $DownloadUrl"
    Write-Warn "[DRY RUN] Would extract/install into $InstallDir"
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

if ($IsZipPackage) {
    Write-Step "Extracting portable package to $InstallDir..."
    Expand-Archive -Path $DownloadedFile -DestinationPath $InstallDir -Force
    Remove-Item $DownloadedFile -Force -ErrorAction SilentlyContinue
} else {
    Write-Step "Running standalone installer..."
    Start-Process -FilePath $DownloadedFile -ArgumentList "/S" -Wait
    Remove-Item $DownloadedFile -Force -ErrorAction SilentlyContinue
}

# Locate Main Executable
$ExePath = Join-Path $InstallDir $BinaryName
if (-not (Test-Path $ExePath)) {
    # Check for child executables
    $found = Get-ChildItem -Path $InstallDir -Filter "*.exe" -Recurse | Select-Object -First 1
    if ($found) {
        $ExePath = $found.FullName
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
}

Write-Host ""
Write-Success "Installation of $AppName v$TargetVersion completed successfully!"
Write-Host "Target Directory: $InstallDir" -ForegroundColor Gray
Write-Host "Executable:       $ExePath" -ForegroundColor Gray
Write-Host ""
Write-Host "You can now launch '$AppName' directly or run 'Antigravity Tools.exe' from any terminal." -ForegroundColor Green
Write-Host ""
