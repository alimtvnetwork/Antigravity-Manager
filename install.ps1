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
    [switch]$Uninstall,
    [switch]$CheckUpdate,
    [switch]$Update
)

$ErrorActionPreference = "Stop"
# Suppress noisy WebRequest progress bar in PowerShell
$ProgressPreference = 'SilentlyContinue'

# Pinned version placeholder (stamped during release packaging or detected from download URL)
$PinnedVersion = "__PINNED_VERSION__"

$Repo = "alimtvnetwork/Antigravity-Manager"
$UpstreamRepo = "lbjlaq/Antigravity-Manager"
$AppName = "Agm Tool By Alim"
$FullName = "Antigravity Manager Tools By Alim"
$BinaryName = "agm-alim.exe"
$ShortcutName = "Agm - Alim"
$Tooltip = "Antigravity Manager Tool By Alim"
$LeftPadding = "    "

function Write-Step {
    param([string]$Message)
    Write-Host "$LeftPadding[*] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "$LeftPadding[OK] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "$LeftPadding[!] $Message" -ForegroundColor Yellow
}

function Write-Err {
    param([string]$Message)
    Write-Host "$LeftPadding[ERROR] $Message" -ForegroundColor Red
}

function Invoke-IndentedCommand {
    param(
        [string]$FilePath,
        [string[]]$ArgumentList = @(),
        [string]$Indent = "`t"
    )

    Write-Host ""
    $exitCode = 0
    try {
        & $FilePath @ArgumentList 2>&1 | ForEach-Object {
            $line = "$_"
            if ($line.Trim().Length -gt 0) {
                Write-Host "$Indent$line"
            }
        }
        $exitCode = $LASTEXITCODE
    } catch {
        Write-Host "$Indent[ERROR] $($_.Exception.Message)" -ForegroundColor Red
        if ($LASTEXITCODE) {
            if ($LASTEXITCODE -ne 0) {
                $exitCode = $LASTEXITCODE
            } else {
                $exitCode = 1
            }
        } else {
            $exitCode = 1
        }
    } finally {
        Write-Host ""
    }
    $global:LASTEXITCODE = $exitCode
    return $exitCode
}

function Get-InvocationHistoryCandidates {
    $candidates = [System.Collections.Generic.List[string]]::new()
    try {
        if ($MyInvocation) {
            if ($MyInvocation.Line) { $candidates.Add($MyInvocation.Line) }
            if ($MyInvocation.Statement) { $candidates.Add($MyInvocation.Statement) }
        }
    } catch {}
    try {
        $hist = Get-History -Count 10 -ErrorAction SilentlyContinue
        if ($hist) {
            foreach ($h in $hist) {
                if ($h.CommandLine) { $candidates.Add($h.CommandLine) }
            }
        }
    } catch {}
    try {
        $rlPath = (Get-PSReadLineOption -ErrorAction SilentlyContinue).HistorySavePath
        if ($rlPath) {
            if (Test-Path $rlPath) {
                $lastLines = Get-Content -Path $rlPath -Tail 20 -ErrorAction SilentlyContinue
                if ($lastLines) {
                    foreach ($line in $lastLines) {
                        if ($line) { $candidates.Add($line) }
                    }
                }
            }
        }
    } catch {}
    try {
        $rlItems = [Microsoft.PowerShell.PSConsoleReadLine]::GetHistoryItems()
        if ($rlItems) {
            $lastItems = $rlItems | Select-Object -Last 10
            foreach ($item in $lastItems) {
                if ($item) { $candidates.Add("$item") }
            }
        }
    } catch {}
    try {
        $envCmd = [System.Environment]::CommandLine
        if ($envCmd) { $candidates.Add($envCmd) }
    } catch {}
    try {
        $curProc = Get-CimInstance Win32_Process -Filter "ProcessId = $PID" -ErrorAction SilentlyContinue
        if ($curProc) {
            if ($curProc.CommandLine) { $candidates.Add($curProc.CommandLine) }
            if ($curProc.ParentProcessId) {
                $parentProc = Get-CimInstance Win32_Process -Filter "ProcessId = $($curProc.ParentProcessId)" -ErrorAction SilentlyContinue
                if ($parentProc) {
                    if ($parentProc.CommandLine) { $candidates.Add($parentProc.CommandLine) }
                }
            }
        }
    } catch {}
    return $candidates
}

function Resolve-PinnedVersion {
    param(
        [string]$ExplicitVersion,
        [string]$BakedVersion
    )
    if ($ExplicitVersion) {
        return ($ExplicitVersion -replace "^v", "")
    }
    if ($BakedVersion) {
        if ($BakedVersion -ne "__PINNED_VERSION__") {
            Write-Step "Respecting pinned installer version: v$BakedVersion"
            return ($BakedVersion -replace "^v", "")
        }
    }
    $entries = Get-InvocationHistoryCandidates
    $regex = 'releases/download/v?([0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?)(/|$|\s|"|' + "')"
    foreach ($entry in $entries) {
        if ($entry -match $regex) {
            $detected = $Matches[1]
            Write-Step "Detected pinned version from download URL: v$detected"
            return $detected
        }
    }
    return $null
}

# Resolve Pinned Version or URL invocation
$resolvedPin = Resolve-PinnedVersion -ExplicitVersion $Version -BakedVersion $PinnedVersion
if ($resolvedPin) {
    $Version = $resolvedPin
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
    $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\agm-alim"
}

# Shortcuts paths
$StartMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$StartMenuShortcut = Join-Path $StartMenuDir "$ShortcutName.lnk"
$DesktopDir = [Environment]::GetFolderPath("Desktop")
$DesktopShortcut = Join-Path $DesktopDir "$ShortcutName.lnk"
$TaskbarDir = Join-Path $env:APPDATA "Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar"

function Remove-PreviousInstallations {
    Write-Step "Checking for previous installations..."

    # 1. Stop any running tool processes across all previous names
    $runningProcesses = Get-Process | Where-Object {
        $_.ProcessName -eq "agm-alim" -or
        $_.ProcessName -eq "antigravity-tools" -or
        $_.ProcessName -eq "Anti-Gravity Tools" -or
        $_.ProcessName -eq "Anti-Gravity Tools by Alim" -or
        $_.ProcessName -eq "AGM by Alim"
    }
    if ($runningProcesses) {
        Write-Step "Closing active application processes..."
        foreach ($proc in $runningProcesses) {
            try {
                Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
            } catch {}
        }
        Start-Sleep -Milliseconds 800
    }

    # 2. Check Windows Registry Uninstall entries for all previous app names
    $regPaths = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )

    foreach ($regPath in $regPaths) {
        $entries = Get-ItemProperty -Path $regPath -ErrorAction SilentlyContinue | Where-Object {
            $_.DisplayName -match '^(Antigravity Manager Tools By Alim|Agm Tool By Alim|AGM by Alim|Anti-Gravity Tools by Alim|Antigravity Tools|agm-alim)$'
        }

        if ($entries) {
            foreach ($entry in $entries) {
                # Skip if already our brand in the exact target install dir
                if ($entry.InstallLocation -and (Resolve-Path $entry.InstallLocation -ErrorAction SilentlyContinue).Path -eq (Resolve-Path $InstallDir -ErrorAction SilentlyContinue).Path) {
                    continue
                }

                Write-Step "Uninstalling previous version ($($entry.DisplayName))..."
                if ($entry.UninstallString) {
                    try {
                        $uninstClean = $entry.UninstallString.Trim('"')
                        if (Test-Path $uninstClean) {
                            $uninstExit = Invoke-IndentedCommand -FilePath $uninstClean -ArgumentList @("/S", "/currentuser")
                            Write-Success "Previous uninstaller completed (exit code: $uninstExit)"
                        }
                    } catch {
                        Write-Warn "Could not execute uninstaller: $_"
                    }
                }
            }
        }
    }

    # 3. Clean previous directory paths if still existing
    $prevDirs = @(
        (Join-Path $env:LOCALAPPDATA "Antigravity Tools"),
        (Join-Path $env:LOCALAPPDATA "Programs\antigravity-tools"),
        (Join-Path $env:LOCALAPPDATA "Programs\Antigravity-Tools"),
        (Join-Path $env:LOCALAPPDATA "Programs\AGM by Alim"),
        (Join-Path $env:LOCALAPPDATA "Programs\Anti-Gravity Tools by Alim"),
        (Join-Path $env:ProgramFiles "Antigravity Tools"),
        (Join-Path $env:ProgramFiles "AGM by Alim"),
        (Join-Path $env:ProgramFiles "Anti-Gravity Tools by Alim"),
        (Join-Path ${env:ProgramFiles(x86)} "Antigravity Tools"),
        (Join-Path ${env:ProgramFiles(x86)} "AGM by Alim"),
        (Join-Path ${env:ProgramFiles(x86)} "Anti-Gravity Tools by Alim")
    )

    foreach ($pdir in $prevDirs) {
        if (Test-Path $pdir) {
            if ($pdir -ne $InstallDir) {
                $uninstExe = Join-Path $pdir "uninstall.exe"
                if (Test-Path $uninstExe) {
                    Write-Step "Running uninstaller in $pdir..."
                    try {
                        Invoke-IndentedCommand -FilePath $uninstExe -ArgumentList @("/S")
                    } catch {}
                }
                Write-Step "Cleaning previous installation folder: $pdir"
                Remove-Item -Path $pdir -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
    }

    # 4. Clean previous shortcuts from Start Menu, Desktop, and Taskbar
    $prevShortcuts = @(
        (Join-Path $StartMenuDir "AGM by Alim.lnk"),
        (Join-Path $StartMenuDir "Antigravity Tools.lnk"),
        (Join-Path $StartMenuDir "antigravity-tools.lnk"),
        (Join-Path $StartMenuDir "Anti-Gravity Tools by Alim.lnk"),
        (Join-Path $StartMenuDir "Anti-Gravity Tools.lnk"),
        (Join-Path $DesktopDir "AGM by Alim.lnk"),
        (Join-Path $DesktopDir "Antigravity Tools.lnk"),
        (Join-Path $DesktopDir "antigravity-tools.lnk"),
        (Join-Path $DesktopDir "Anti-Gravity Tools by Alim.lnk"),
        (Join-Path $DesktopDir "Anti-Gravity Tools.lnk"),
        (Join-Path $TaskbarDir "AGM by Alim.lnk"),
        (Join-Path $TaskbarDir "Antigravity Tools.lnk"),
        (Join-Path $TaskbarDir "antigravity-tools.lnk"),
        (Join-Path $TaskbarDir "Anti-Gravity Tools by Alim.lnk"),
        (Join-Path $TaskbarDir "Anti-Gravity Tools.lnk")
    )
    foreach ($sc in $prevShortcuts) {
        if (Test-Path $sc) {
            Remove-Item -Path $sc -Force -ErrorAction SilentlyContinue
        }
    }

    Write-Success "Previous installation cleanup complete."
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

    # Method 1: Create shortcut in User Pinned Taskbar directory if not present
    if (Test-Path $TaskbarDir) {
        $pinnedShortcut = Join-Path $TaskbarDir "$ShortcutName.lnk"
        if (Test-Path $pinnedShortcut) {
            Write-Step "Taskbar shortcut already exists: $pinnedShortcut"
            return
        }
        try {
            $WshShell = New-Object -ComObject WScript.Shell
            $sc = $WshShell.CreateShortcut($pinnedShortcut)
            $sc.TargetPath = $TargetExe
            $sc.WorkingDirectory = $TargetWorkDir
            $sc.Description = $Tooltip
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
    Write-Host ""
    Write-Host "$LeftPadding========================================" -ForegroundColor Magenta
    Write-Host "$LeftPadding    $FullName Uninstaller" -ForegroundColor Magenta
    Write-Host "$LeftPadding========================================" -ForegroundColor Magenta
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

function Get-InstalledVersion {
    # 1. Check current configured install directory
    $candidateExes = @(
        (Join-Path $InstallDir $BinaryName),
        (Join-Path $InstallDir "agm-alim.exe"),
        (Join-Path $InstallDir "AGM by Alim.exe"),
        (Join-Path $InstallDir "Anti-Gravity Tools by Alim.exe"),
        (Join-Path $InstallDir "antigravity-tools.exe"),
        (Join-Path $InstallDir "Antigravity Tools.exe")
    )
    foreach ($exe in $candidateExes) {
        if (Test-Path $exe) {
            try {
                $ver = (Get-Item $exe).VersionInfo.ProductVersion
                if ($ver) { return ($ver -replace '^v', '').Trim() }
            } catch {}
        }
    }

    # 2. Check previous installation directories
    $prevDirs = @(
        (Join-Path $env:LOCALAPPDATA "Programs\agm-alim"),
        (Join-Path $env:LOCALAPPDATA "Programs\AGM by Alim"),
        (Join-Path $env:LOCALAPPDATA "Programs\Antigravity-Tools"),
        (Join-Path $env:LOCALAPPDATA "Programs\antigravity-tools"),
        (Join-Path $env:LOCALAPPDATA "Programs\Anti-Gravity Tools by Alim"),
        (Join-Path $env:LOCALAPPDATA "Antigravity Tools"),
        (Join-Path $env:ProgramFiles "agm-alim"),
        (Join-Path $env:ProgramFiles "AGM by Alim"),
        (Join-Path $env:ProgramFiles "Antigravity Tools"),
        (Join-Path $env:ProgramFiles "Anti-Gravity Tools by Alim"),
        (Join-Path ${env:ProgramFiles(x86)} "Antigravity Tools")
    )
    foreach ($dir in $prevDirs) {
        if (Test-Path $dir) {
            $foundExe = Get-ChildItem -Path $dir -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "*uninstall*" -and $_.Name -notlike "*setup*" } | Select-Object -First 1
            if ($foundExe) {
                try {
                    $ver = $foundExe.VersionInfo.ProductVersion
                    if ($ver) { return ($ver -replace '^v', '').Trim() }
                } catch {}
            }
        }
    }

    # 3. Check Windows Registry Uninstall entries
    $regPaths = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )
    foreach ($regPath in $regPaths) {
        $entries = Get-ItemProperty -Path $regPath -ErrorAction SilentlyContinue | Where-Object {
            $_.DisplayName -match '^(Antigravity Manager Tools By Alim|Agm Tool By Alim|AGM by Alim|Anti-Gravity Tools by Alim|Antigravity Tools|agm-alim)$'
        }
        foreach ($entry in $entries) {
            if ($entry.DisplayVersion) {
                return ($entry.DisplayVersion -replace '^v', '').Trim()
            }
        }
    }

    return ""
}

# --- CHECK-UPDATE FLOW ---
if ($CheckUpdate) {
    # Step 1: Resolve Release Version quietly
    $TargetVersion = $Version
    if ($TargetVersion) {
        $TargetVersion = $TargetVersion -replace "^v", ""
    } else {
        $apiEndpoints = @(
            "https://api.github.com/repos/$Repo/releases",
            "https://api.github.com/repos/$Repo/releases/latest",
            "https://api.github.com/repos/$UpstreamRepo/releases",
            "https://api.github.com/repos/$UpstreamRepo/releases/latest"
        )
        foreach ($endpoint in $apiEndpoints) {
            try {
                $resp = Invoke-RestMethod -Uri $endpoint -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 6
                if ($resp -is [System.Array] -and $resp.Count -gt 0) {
                    $candidate = $resp | Where-Object { $_.assets -and $_.assets.Count -gt 0 } | Select-Object -First 1
                    if (-not $candidate) { $candidate = $resp[0] }
                    if ($candidate -and $candidate.tag_name) {
                        $TargetVersion = $candidate.tag_name -replace "^v", ""
                        break
                    }
                } elseif ($resp -and $resp.tag_name) {
                    $TargetVersion = $resp.tag_name -replace "^v", ""
                    break
                }
            } catch {}
        }
        if (-not $TargetVersion) {
            try {
                $updater = Invoke-RestMethod -Uri "https://github.com/$Repo/releases/latest/download/updater.json" -TimeoutSec 6
                if ($updater -and $updater.version) {
                    $TargetVersion = $updater.version -replace "^v", ""
                }
            } catch {}
        }
        if (-not $TargetVersion) {
            $TargetVersion = "4.30.0"
        }
    }

    $curr = Get-InstalledVersion
    $hasUpdate = $false
    if (-not $curr) {
        if ($TargetVersion) { $hasUpdate = $true }
    } elseif ($TargetVersion) {
        if ($curr -ne $TargetVersion) {
            $hasUpdate = $true
        }
    }

    $jsonObj = [PSCustomObject]@{
        has_update = $hasUpdate
        current_version = if ($curr) { $curr } else { "unknown" }
        latest_version = if ($TargetVersion) { $TargetVersion } else { "unknown" }
        download_url = "https://github.com/$Repo/releases/tag/v$TargetVersion"
    }
    $jsonObj | ConvertTo-Json -Compress
    return
}

function Convert-ToSemVer {
    param([string]$v)
    $clean = $v -replace "^v", ""
    $parts = $clean.Split("-")[0].Split(".")
    $major = if ($parts.Length -ge 1) { [int]$parts[0] } else { 0 }
    $minor = if ($parts.Length -ge 2) { [int]$parts[1] } else { 0 }
    $patch = if ($parts.Length -ge 3) { [int]$parts[2] } else { 0 }
    return [version]::new($major, $minor, $patch)
}

function Get-Aria2cPath {
    $cmd = Get-Command aria2c.exe -ErrorAction SilentlyContinue
    if (-not $cmd) {
        $cmd = Get-Command aria2c -ErrorAction SilentlyContinue
    }
    if ($cmd) {
        return $cmd.Source
    }

    $commonPaths = @(
        "C:\ProgramData\chocolatey\bin\aria2c.exe",
        "$env:LOCALAPPDATA\Programs\aria2\aria2c.exe",
        "$env:ProgramFiles\aria2\aria2c.exe",
        "${env:ProgramFiles(x86)}\aria2\aria2c.exe",
        "$env:USERPROFILE\scoop\shims\aria2c.exe",
        (Join-Path $env:TEMP "aria2c.exe")
    )
    foreach ($p in $commonPaths) {
        if (Test-Path $p) {
            return $p
        }
    }

    return $null
}

function Invoke-FastDownload {
    param(
        [string]$Url,
        [string]$DestinationPath
    )

    $destDir = Split-Path -Parent $DestinationPath
    $destFile = Split-Path -Leaf $DestinationPath

    if (-not (Test-Path $destDir)) {
        New-Item -ItemType Directory -Path $destDir -Force | Out-Null
    }

    if (Test-Path $DestinationPath) {
        Remove-Item -Path $DestinationPath -Force -ErrorAction SilentlyContinue
    }
    $aria2Control = "$DestinationPath.aria2"
    if (Test-Path $aria2Control) {
        Remove-Item -Path $aria2Control -Force -ErrorAction SilentlyContinue
    }

    # 1. Try aria2c with 80 parallel split connections and 500KB chunks
    $aria2Bin = Get-Aria2cPath
    if ($aria2Bin) {
        Write-Step "Accelerating download with aria2c (80 splits, 500KB chunks)..."
        try {
            $ariaArgs = @(
                "--disable-ipv6=true",
                "-x", "16",
                "-s", "80",
                "-j", "16",
                "-k", "500K",
                "--file-allocation=none",
                "--allow-overwrite=true",
                "--auto-file-renaming=false",
                "--summary-interval=1",
                "--console-log-level=warn",
                "--dir=$destDir",
                "-o", "$destFile",
                "$Url"
            )
            $ariaExit = Invoke-IndentedCommand -FilePath $aria2Bin -ArgumentList $ariaArgs
            if ($ariaExit -eq 28) {
                Write-Step "Adapting aria2c segment size to 1MB minimum threshold (80 splits)..."
                $ariaArgs1M = @(
                    "--disable-ipv6=true",
                    "-x", "16",
                    "-s", "80",
                    "-j", "16",
                    "-k", "1M",
                    "--file-allocation=none",
                    "--allow-overwrite=true",
                    "--auto-file-renaming=false",
                    "--summary-interval=1",
                    "--console-log-level=warn",
                    "--dir=$destDir",
                    "-o", "$destFile",
                    "$Url"
                )
                $ariaExit = Invoke-IndentedCommand -FilePath $aria2Bin -ArgumentList $ariaArgs1M
            }
            if ($ariaExit -eq 0) {
                if (Test-Path $DestinationPath) {
                    if ((Get-Item $DestinationPath).Length -gt 0) {
                        Write-Success "Download completed via aria2c."
                        return $true
                    }
                }
            }
            Write-Warn "aria2c finished with code $ariaExit; falling back to secondary downloader..."
        } catch {
            Write-Warn "aria2c encountered an error: $_. Falling back..."
        }
    } else {
        Write-Step "aria2c not found; proceeding with standard download stream..."
    }

    # 2. Try curl.exe (built into modern Windows)
    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if ($curl) {
        Write-Step "Downloading with curl..."
        try {
            $curlArgs = @("-fSL", "--progress-bar", "--connect-timeout", "10", "--retry", "3", "-o", $DestinationPath, $Url)
            $curlExit = Invoke-IndentedCommand -FilePath $curl.Source -ArgumentList $curlArgs
            if ($curlExit -eq 0) {
                if (Test-Path $DestinationPath) {
                    if ((Get-Item $DestinationPath).Length -gt 0) {
                        Write-Success "Download completed via curl."
                        return $true
                    }
                }
            }
        } catch {}
    }

    # 3. Fallback to Invoke-WebRequest
    Write-Step "Downloading with Invoke-WebRequest..."
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12 -bor [System.Net.SecurityProtocolType]::Tls13
    $prevProgress = $ProgressPreference
    $ProgressPreference = 'SilentlyContinue'
    try {
        Invoke-WebRequest -Uri $Url -OutFile $DestinationPath -UseBasicParsing
        $ProgressPreference = $prevProgress
        if (Test-Path $DestinationPath) {
            if ((Get-Item $DestinationPath).Length -gt 0) {
                Write-Success "Download completed via Invoke-WebRequest."
                return $true
            }
        }
    } catch {
        $ProgressPreference = $prevProgress
        Write-Warn "Invoke-WebRequest failed: $_"
    }

    return $false
}

# --- INSTALL FLOW ---
Write-Host ""
Write-Host ""
Write-Host "$LeftPadding========================================" -ForegroundColor Cyan
Write-Host "$LeftPadding    $FullName Installer" -ForegroundColor Cyan
Write-Host "$LeftPadding========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Discover Available Release Versions & Build 4-Candidate Queue
$candidateVersions = [System.Collections.Generic.List[string]]::new()
$releaseMetadataMap = @{}

# Ensure pinned version resolution is finalized
if (-not $Version) {
    $resolvedPin = Resolve-PinnedVersion -ExplicitVersion $Version -BakedVersion $PinnedVersion
    if ($resolvedPin) {
        $Version = $resolvedPin
    }
}

$isPinned = $false
$cleanPinned = $null
if ($Version) {
    $cleanPinned = $Version -replace "^v", ""
    $candidateVersions.Add($cleanPinned)
    $isPinned = $true
    Write-Step "Respecting pinned release version: v$cleanPinned"
}

Write-Step "Discovering available release versions from GitHub..."
$apiEndpoints = @()
if ($isPinned) {
    $apiEndpoints += "https://api.github.com/repos/$Repo/releases/tags/v$cleanPinned"
    $apiEndpoints += "https://api.github.com/repos/$UpstreamRepo/releases/tags/v$cleanPinned"
}
$apiEndpoints += @(
    "https://api.github.com/repos/$Repo/releases?per_page=30",
    "https://api.github.com/repos/$Repo/releases/latest",
    "https://api.github.com/repos/$UpstreamRepo/releases?per_page=30",
    "https://api.github.com/repos/$UpstreamRepo/releases/latest"
)

foreach ($endpoint in $apiEndpoints) {
    try {
        $resp = Invoke-RestMethod -Uri $endpoint -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 6
        if ($resp -is [System.Array]) {
            foreach ($rel in $resp) {
                if ($rel.tag_name) {
                    $tagVer = $rel.tag_name -replace "^v", ""
                    if (-not $releaseMetadataMap.ContainsKey($tagVer)) {
                        $releaseMetadataMap[$tagVer] = $rel
                    }
                    if (-not $isPinned) {
                        if (-not $candidateVersions.Contains($tagVer)) {
                            $candidateVersions.Add($tagVer)
                        }
                    } else {
                        try {
                            $pVer = Convert-ToSemVer $cleanPinned
                            $cVer = Convert-ToSemVer $tagVer
                            if ($cVer -lt $pVer) {
                                if (-not $candidateVersions.Contains($tagVer)) {
                                    $candidateVersions.Add($tagVer)
                                }
                            }
                        } catch {}
                    }
                }
            }
        } elseif ($resp) {
            if ($resp.tag_name) {
                $tagVer = $resp.tag_name -replace "^v", ""
                if (-not $releaseMetadataMap.ContainsKey($tagVer)) {
                    $releaseMetadataMap[$tagVer] = $resp
                }
                if (-not $isPinned) {
                    if (-not $candidateVersions.Contains($tagVer)) {
                        $candidateVersions.Add($tagVer)
                    }
                }
            }
        }
    } catch {}
    if ($candidateVersions.Count -ge 4) { break }
}

# Fallback known historical releases
$knownFallbacks = @("4.41.0", "4.40.0", "4.39.0", "4.38.1", "4.38.0", "4.37.0", "4.36.0", "4.35.0", "4.34.0", "4.33.0", "4.32.0", "4.31.0", "4.30.0", "4.7.6")
foreach ($kb in $knownFallbacks) {
    if ($candidateVersions.Count -ge 4) { break }
    if (-not $isPinned) {
        if (-not $candidateVersions.Contains($kb)) {
            $candidateVersions.Add($kb)
        }
    } else {
        try {
            $pVer = Convert-ToSemVer $cleanPinned
            $kVer = Convert-ToSemVer $kb
            if ($kVer -lt $pVer) {
                if (-not $candidateVersions.Contains($kb)) {
                    $candidateVersions.Add($kb)
                }
            }
        } catch {}
    }
}

# Build strict 4-version queue
$versionQueue = @()
foreach ($v in $candidateVersions) {
    if ($versionQueue.Count -lt 4) {
        $versionQueue += $v
    }
}

$TargetVersion = $versionQueue[0]

if ($Update) {
    $curr = Get-InstalledVersion
    if ($curr) {
        if ($curr -eq $TargetVersion) {
            Write-Success "Already on the latest version ($curr)."
            return
        }
    }
}

$CurrentVersion = Get-InstalledVersion
if ($CurrentVersion) {
    Write-Step "Current installed version: v$CurrentVersion"
    Write-Step "Target release version   : v$TargetVersion (Architecture: $Arch)"
    if ($CurrentVersion -eq $TargetVersion) {
        Write-Step "Migration mode           : Reinstalling / Updating v$TargetVersion"
    } else {
        Write-Step "Migration path           : v$CurrentVersion -> v$TargetVersion"
    }
} else {
    Write-Step "Target release version   : v$TargetVersion (Architecture: $Arch)"
    Write-Step "Installation mode        : Fresh installation (v$TargetVersion)"
}

# Step 2: Intelligent Multi-Version Try-Catch Installation Ladder (Up to 4 attempts)
$maxAttempts = 4
$attempt = 0
$installedOk = $false
$installedVersion = $null
$ExePath = $null

foreach ($candVersion in $versionQueue) {
    $attempt++
    if ($attempt -gt $maxAttempts) {
        break
    }

    Write-Host ""
    Write-Step "=== Installation Attempt $attempt of ${maxAttempts}: Release v$candVersion ==="

    try {
        $relData = $releaseMetadataMap[$candVersion]
        if (-not $relData) {
            try {
                $relData = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/v$candVersion" -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 6
            } catch {
                try {
                    $relData = Invoke-RestMethod -Uri "https://api.github.com/repos/$UpstreamRepo/releases/tags/v$candVersion" -Headers @{ "User-Agent" = "Antigravity-Installer" } -TimeoutSec 6
                } catch {}
            }
        }

        $matchedAsset = $null
        if ($relData) {
            if ($relData.assets) {
                $matchedAsset = $relData.assets | Where-Object { $_.name -like "*${Arch}*setup.exe" -or $_.name -like "*setup.exe" } | Select-Object -First 1
                if (-not $matchedAsset) {
                    $matchedAsset = $relData.assets | Where-Object { $_.name -like "*.exe" -and $_.name -notlike "*build*" } | Select-Object -First 1
                }
            }
        }

        if ($matchedAsset) {
            $DownloadUrl = $matchedAsset.browser_download_url
        } else {
            $DownloadUrl = "https://github.com/$Repo/releases/download/v$candVersion/agm-alim_${candVersion}_${Arch}-setup.exe"
        }

        Write-Step "Package download URL: $DownloadUrl"

        if ($DryRun) {
            Write-Warn "[DRY RUN] Would download $DownloadUrl"
            Write-Warn "[DRY RUN] Would execute/install into $InstallDir"
            if (-not $NoPath) { Write-Warn "[DRY RUN] Would append $InstallDir to User PATH" }
            if (-not $NoShortcut) { Write-Warn "[DRY RUN] Would configure Desktop & Start Menu shortcuts" }
            Write-Success "[DRY RUN] Dry run completed successfully."
            return
        }

        $TempDir = [System.IO.Path]::GetTempPath()
        $DownloadedFile = Join-Path $TempDir ($DownloadUrl -split "/" | Select-Object -Last 1)

        Write-Step "Downloading release package..."
        $downloaded = Invoke-FastDownload -Url $DownloadUrl -DestinationPath $DownloadedFile
        if (-not $downloaded) {
            Write-Warn "Primary download failed. Attempting upstream fallback..."
            $UpstreamDownloadUrl = $DownloadUrl -replace [regex]::Escape($Repo), $UpstreamRepo
            $downloaded = Invoke-FastDownload -Url $UpstreamDownloadUrl -DestinationPath $DownloadedFile
        }

        if (-not $downloaded) {
            throw "Failed to download release package for v$candVersion"
        }

        if (-not (Test-Path $DownloadedFile)) {
            throw "Downloaded package not found at: $DownloadedFile"
        }

        if ((Get-Item $DownloadedFile).Length -lt 1024) {
            throw "Downloaded package is empty or corrupt (<1KB)"
        }

        # Safe removal of previous installation only after new file verified
        Remove-PreviousInstallations

        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        Write-Step "Executing installer package ($DownloadedFile)..."
        $installExit = Invoke-IndentedCommand -FilePath $DownloadedFile -ArgumentList @("/S", "/D=$InstallDir")
        Remove-Item $DownloadedFile -Force -ErrorAction SilentlyContinue

        # Locate and verify main executable
        $detectedExe = Join-Path $InstallDir $BinaryName
        if (-not (Test-Path $detectedExe)) {
            $altNames = @(
                "agm-alim.exe",
                "AGM by Alim.exe",
                "Anti-Gravity Tools by Alim.exe",
                "Anti-Gravity Tools.exe",
                "antigravity-tools.exe",
                "Antigravity Tools.exe"
            )
            foreach ($an in $altNames) {
                $chk = Join-Path $InstallDir $an
                if (Test-Path $chk) {
                    $detectedExe = $chk
                    $BinaryName = $an
                    break
                }
            }
        }
        if (-not (Test-Path $detectedExe)) {
            $found = Get-ChildItem -Path $InstallDir -Filter "*.exe" -Recurse | Where-Object { $_.Name -notlike "*uninstall*" -and $_.Name -notlike "*setup*" } | Select-Object -First 1
            if ($found) {
                $detectedExe = $found.FullName
                $BinaryName = $found.Name
            }
        }
        if (-not (Test-Path $detectedExe)) {
            $commonDirs = @(
                (Join-Path $env:LOCALAPPDATA "Programs\agm-alim"),
                (Join-Path $env:LOCALAPPDATA "Programs\AGM by Alim"),
                (Join-Path $env:LOCALAPPDATA "Programs\Antigravity-Tools"),
                (Join-Path $env:LOCALAPPDATA "Programs\Anti-Gravity Tools by Alim"),
                (Join-Path $env:LOCALAPPDATA "Programs\antigravity-tools"),
                (Join-Path $env:ProgramFiles "agm-alim"),
                (Join-Path $env:ProgramFiles "AGM by Alim"),
                (Join-Path $env:ProgramFiles "Anti-Gravity Tools by Alim")
            )
            foreach ($dir in $commonDirs) {
                if (Test-Path $dir) {
                    $found = Get-ChildItem -Path $dir -Filter "*.exe" -Recurse | Where-Object { $_.Name -notlike "*uninstall*" } | Select-Object -First 1
                    if ($found) {
                        $InstallDir = $dir
                        $detectedExe = $found.FullName
                        $BinaryName = $found.Name
                        break
                    }
                }
            }
        }

        if (-not (Test-Path $detectedExe)) {
            throw "Main executable not found in $InstallDir after running installer package"
        }

        # Success!
        $ExePath = $detectedExe
        $TargetVersion = $candVersion
        $installedOk = $true
        Write-Success "Installation of v$candVersion verified successfully!"
        break
    } catch {
        Write-Warn "Attempt $attempt failed for release v${candVersion}: $($_.Exception.Message)"
        if ($attempt -lt $maxAttempts) {
            Write-Step "Falling back to previous release version in sequence..."
        }
    }
}

if (-not $installedOk) {
    Write-Err "All $maxAttempts attempts failed. I fail, so I cannot do anything."
    exit 1
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
if (-not $NoShortcut) {
    if ($ExePath) {
        if (Test-Path $ExePath) {
            Write-Step "Configuring Application Shortcuts..."
            try {
                $WshShell = New-Object -ComObject WScript.Shell

                # Clean any remaining legacy shortcut on Desktop or Start Menu
                $oldLnks = @("AGM by Alim.lnk", "Anti-Gravity Tools by Alim.lnk", "Antigravity Tools.lnk", "antigravity-tools.lnk")
                foreach ($old in $oldLnks) {
                    $f1 = Join-Path $DesktopDir $old
                    if (Test-Path $f1) { Remove-Item -Path $f1 -Force -ErrorAction SilentlyContinue }
                    $f2 = Join-Path $StartMenuDir $old
                    if (Test-Path $f2) { Remove-Item -Path $f2 -Force -ErrorAction SilentlyContinue }
                    $f3 = Join-Path $TaskbarDir $old
                    if (Test-Path $f3) { Remove-Item -Path $f3 -Force -ErrorAction SilentlyContinue }
                }

                # Start Menu
                if (-not (Test-Path $StartMenuDir)) {
                    New-Item -ItemType Directory -Path $StartMenuDir -Force | Out-Null
                }
                if (Test-Path $StartMenuShortcut) {
                    Write-Step "Start Menu shortcut already exists: $StartMenuShortcut"
                } else {
                    $smShortcut = $WshShell.CreateShortcut($StartMenuShortcut)
                    $smShortcut.TargetPath = $ExePath
                    $smShortcut.WorkingDirectory = $InstallDir
                    $smShortcut.Description = $Tooltip
                    $smShortcut.Save()
                    Write-Success "Created Start Menu shortcut: $StartMenuShortcut"
                }

                # Desktop
                if (Test-Path $DesktopDir) {
                    if (Test-Path $DesktopShortcut) {
                        Write-Step "Desktop shortcut already exists: $DesktopShortcut"
                    } else {
                        $dtShortcut = $WshShell.CreateShortcut($DesktopShortcut)
                        $dtShortcut.TargetPath = $ExePath
                        $dtShortcut.WorkingDirectory = $InstallDir
                        $dtShortcut.Description = $Tooltip
                        $dtShortcut.Save()
                        Write-Success "Created Desktop shortcut: $DesktopShortcut"
                    }
                }
            } catch {
                Write-Warn "Could not create shortcuts: $_"
            }

            # Step 6: Taskbar Pinning (Windows 10, Windows 11, Windows Server)
            Pin-TaskbarShortcut -TargetExe $ExePath -TargetWorkDir $InstallDir -ShortcutSource $StartMenuShortcut
        }
    }
}

Write-Host ""
Write-Success "Installation of $FullName v$TargetVersion completed successfully!"
Write-Host "${LeftPadding}Target Directory: $InstallDir" -ForegroundColor Gray
Write-Host "${LeftPadding}Executable:       $ExePath" -ForegroundColor Gray
Write-Host ""
Write-Host "${LeftPadding}You can now launch '$ShortcutName' directly or run '$BinaryName' from any terminal." -ForegroundColor Green
Write-Host ""
Write-Host ""
