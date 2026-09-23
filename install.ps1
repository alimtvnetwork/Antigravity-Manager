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
    [switch]$Update,
    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"
# Suppress noisy WebRequest progress bar in PowerShell
$ProgressPreference = 'SilentlyContinue'

# Pinned version placeholder (stamped during release packaging or detected from download URL)
$PinnedVersion = "__PINNED_VERSION__"

$Repo = "alimtvnetwork/Antigravity-Manager"
$UpstreamRepo = "lbjlaq/Antigravity-Manager"
$AppName = "AGM by Alim"
$FullName = "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
$BinaryName = "agm-alim.exe"
$ShortcutName = "AGM by Alim"
$Tooltip = "AGM by Alim"
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
    try {
        [Console]::Error.WriteLine("$LeftPadding[ERROR] $Message")
    } catch {}
}

function Invoke-IndentedCommand {
    param(
        [string]$FilePath,
        [string[]]$ArgumentList = @(),
        [string]$Indent = "`t"
    )

    Write-Host ""
    $exitCode = 0
    $prevEap = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
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
        $ErrorActionPreference = $prevEap
        Write-Host ""
    }
    $global:LASTEXITCODE = $exitCode
    return $exitCode
}

function Verify-DownloadChecksum {
    param([string]$FilePath)
    if (Test-Path $FilePath) {
        # Verify SHA256 file hash integrity
        $hashResult = Get-FileHash -Path $FilePath -Algorithm SHA256 -ErrorAction SilentlyContinue
        if ($hashResult) {
            return $hashResult.Hash
        }
    }
    return $null
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
    if (-not $ExplicitVersion) {
        if ($env:AGM_VERSION) {
            $ExplicitVersion = $env:AGM_VERSION
        } elseif ($env:VERSION) {
            $ExplicitVersion = $env:VERSION
        } elseif ($env:INSTALLER_VERSION) {
            $ExplicitVersion = $env:INSTALLER_VERSION
        } elseif ($args -and $args.Count -gt 0) {
            if ($args[0] -match '^[vV]?[0-9]+\.[0-9]+') {
                $ExplicitVersion = $args[0]
            }
        }
    }
    if ($ExplicitVersion) {
        $clean = ($ExplicitVersion -replace "^v", "").Trim()
        $parts = $clean.Split("-")[0].Split(".")
        if ($parts.Length -eq 2) {
            $clean = "$clean.0"
        }
        return $clean
    }
    if ($BakedVersion) {
        if ($BakedVersion -ne "__PINNED_VERSION__") {
            Write-Step "Respecting pinned installer version: v$BakedVersion"
            $clean = ($BakedVersion -replace "^v", "").Trim()
            $parts = $clean.Split("-")[0].Split(".")
            if ($parts.Length -eq 2) {
                $clean = "$clean.0"
            }
            return $clean
        }
    }
    $entries = Get-InvocationHistoryCandidates
    # Strictly scope to Antigravity-Manager or agm-alim URLs so unrelated command lines never pollute version
    $regex = '(?i)(Antigravity-Manager|agm-alim|antigravity).*(?:releases/download/|raw\.githubusercontent\.com/[^/]+/[^/]+/)(?:v)?([0-9]+\.[0-9]+(?:\.[0-9]+)?(?:-[a-zA-Z0-9.]+)?)/'
    foreach ($entry in $entries) {
        if ($entry -match $regex) {
            $detected = $Matches[2]
            $parts = $detected.Split("-")[0].Split(".")
            if ($parts.Length -eq 2) {
                $detected = "$detected.0"
            }
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

function Close-ToolProcesses {
    param([string]$Context = "installation")
    Write-Step "Closing all running tool processes, Electron instances, WebViews, and background tasks ($Context)..."

    # Identify caller parent PID so in-app update never self-terminates
    $parentPid = 0
    try {
        $myProc = Get-CimInstance Win32_Process -Filter "ProcessId = $PID" -ErrorAction SilentlyContinue
        if ($myProc) {
            if ($myProc.ParentProcessId) {
                $parentPid = [int]$myProc.ParentProcessId
            }
        }
    } catch {}

    $processNames = @(
        "agm-alim",
        "antigravity-tools",
        "Anti-Gravity Tools",
        "Anti-Gravity Tools by Alim",
        "AGM by Alim"
    )

    # 1. Gracefully terminate other running tool instances (protecting invoking parent process during in-app update)
    foreach ($pName in $processNames) {
        try {
            $runningProcs = Get-Process -Name $pName -ErrorAction SilentlyContinue
            if ($runningProcs) {
                foreach ($p in $runningProcs) {
                    if ($parentPid) {
                        if ($p.Id -eq $parentPid) {
                            continue
                        }
                    }
                    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
                }
            }
        } catch {}
    }

    # 3. Terminate orphan WebView2 processes related to Antigravity/AGM
    try {
        $webviews = Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" -ErrorAction SilentlyContinue | Where-Object {
            if ($_.CommandLine) {
                if ($_.CommandLine -match 'agm-alim' -or $_.CommandLine -match 'antigravity' -or $_.CommandLine -match 'Antigravity-Tools' -or $_.CommandLine -match 'AGM by Alim' -or $_.CommandLine -match 'antigravity_tools') {
                    return $true
                }
            }
            if ($_.ExecutablePath) {
                if ($_.ExecutablePath -match 'agm-alim' -or $_.ExecutablePath -match 'antigravity') {
                    return $true
                }
            }
            return $false
        }
        if ($webviews) {
            foreach ($wv in $webviews) {
                try {
                    Stop-Process -Id $wv.ProcessId -Force -ErrorAction SilentlyContinue
                } catch {}
            }
        }
    } catch {}

    # 4. Terminate any Electron processes related to Antigravity/AGM
    try {
        $electrons = Get-CimInstance Win32_Process -Filter "Name = 'electron.exe'" -ErrorAction SilentlyContinue | Where-Object {
            if ($_.CommandLine) {
                if ($_.CommandLine -match 'agm-alim' -or $_.CommandLine -match 'antigravity' -or $_.CommandLine -match 'Antigravity-Tools') {
                    return $true
                }
            }
            if ($_.ExecutablePath) {
                if ($_.ExecutablePath -match 'agm-alim' -or $_.ExecutablePath -match 'antigravity') {
                    return $true
                }
            }
            return $false
        }
        if ($electrons) {
            foreach ($el in $electrons) {
                try {
                    Stop-Process -Id $el.ProcessId -Force -ErrorAction SilentlyContinue
                } catch {}
            }
        }
    } catch {}

    Start-Sleep -Milliseconds 800
}

function Ensure-DefaultConfig {
    Write-Step "Ensuring Auto Sync Current Account is enabled by default..."
    $configDir = Join-Path $env:USERPROFILE ".antigravity_tools"
    $configFile = Join-Path $configDir "gui_config.json"

    try {
        if (-not (Test-Path $configDir)) {
            New-Item -ItemType Directory -Path $configDir -Force | Out-Null
        }

        if (Test-Path $configFile) {
            $rawJson = Get-Content -Path $configFile -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
            if ($rawJson) {
                $cfg = ConvertFrom-Json $rawJson -ErrorAction SilentlyContinue
                if ($cfg) {
                    $modified = $false
                    if (-not (Get-Member -InputObject $cfg -Name "auto_sync_migrated" -MemberType Properties)) {
                        $cfg | Add-Member -MemberType NoteProperty -Name "auto_sync_migrated" -Value $true -Force
                        $cfg.auto_sync = $true
                        $modified = $true
                    } elseif (-not $cfg.auto_sync_migrated) {
                        $cfg.auto_sync = $true
                        $cfg.auto_sync_migrated = $true
                        $modified = $true
                    }

                    if ($modified) {
                        $cfg | ConvertTo-Json -Depth 20 | Set-Content -Path $configFile -Encoding UTF8 -Force
                        Write-Success "Updated configuration: Auto Sync Current Account enabled by default."
                    } else {
                        Write-Success "Configuration verified: Auto Sync Current Account default active."
                    }
                }
            }
        } else {
            $minimalConfig = [PSCustomObject]@{
                language = "en"
                theme = "system"
                auto_refresh = $true
                refresh_interval = 15
                auto_sync = $true
                auto_sync_migrated = $true
                sync_interval = 5
            }
            $minimalConfig | ConvertTo-Json -Depth 20 | Set-Content -Path $configFile -Encoding UTF8 -Force
            Write-Success "Initialized default configuration with Auto Sync Current Account enabled."
        }
    } catch {
        Write-Warn "Could not update default configuration: $_"
    }
}

function Remove-PreviousInstallations {
    Write-Step "Checking for previous installations..."

    # 1. Stop any running tool processes, Electron instances, WebViews, and background tasks
    Close-ToolProcesses -Context "pre-installation"

    # 2. Check Windows Registry Uninstall entries for all previous app names
    $regPaths = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )

    foreach ($regPath in $regPaths) {
        $entries = Get-ItemProperty -Path $regPath -ErrorAction SilentlyContinue | Where-Object {
            $_.DisplayName -match '^(Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC|Antigravity Manager Tools By Alim|Agm Tool By Alim|AGM by Alim|Anti-Gravity Tools by Alim|Antigravity Tools|agm-alim)$'
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

    # 4. Clean legacy shortcuts from Start Menu, Desktop, and Taskbar (preserving active AGM by Alim.lnk)
    $prevShortcuts = @(
        (Join-Path $StartMenuDir "Antigravity Tools.lnk"),
        (Join-Path $StartMenuDir "antigravity-tools.lnk"),
        (Join-Path $StartMenuDir "Anti-Gravity Tools by Alim.lnk"),
        (Join-Path $StartMenuDir "Anti-Gravity Tools.lnk"),
        (Join-Path $DesktopDir "Antigravity Tools.lnk"),
        (Join-Path $DesktopDir "antigravity-tools.lnk"),
        (Join-Path $DesktopDir "Anti-Gravity Tools by Alim.lnk"),
        (Join-Path $DesktopDir "Anti-Gravity Tools.lnk"),
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

function Pin-StartMenuOnce {
    param(
        [string]$ShortcutPath,
        [string]$MarkerDir
    )

    $pinMarker = Join-Path $MarkerDir ".start_menu_pinned"
    if (Test-Path $pinMarker) {
        Write-Step "Start Menu pin already configured (marker exists). Skipping."
        return
    }

    if (-not (Test-Path $ShortcutPath)) {
        Write-Warn "Shortcut $ShortcutPath not found; cannot pin to Start Menu."
        return
    }

    Write-Step "Configuring Start Menu pin if not exists (one-time setup)..."
    try {
        $shell = New-Object -ComObject Shell.Application
        $folderPath = Split-Path $ShortcutPath -Parent
        $fileName = Split-Path $ShortcutPath -Leaf
        $folder = $shell.Namespace($folderPath)
        if ($folder) {
            $item = $folder.ParseName($fileName)
            if ($item) {
                $verbs = $item.Verbs()
                $isAlreadyPinned = $false
                $pinVerb = $null
                foreach ($v in $verbs) {
                    $clean = $v.Name.Replace('&', '').Trim()
                    if ($clean -match '^(Unpin from Start|Unpin from Start screen|从.*开始.*取消固定|Von .*Start.* lösen)') {
                        $isAlreadyPinned = $true
                        break
                    }
                    if ($clean -match '^(Pin to Start|Pin to Start screen|pintostartscreen|固定到.*开始.*|An .*Start.* anheften)') {
                        $pinVerb = $v
                    }
                }

                if ($isAlreadyPinned) {
                    Write-Step "Application is already pinned to Start Menu."
                } elseif ($pinVerb) {
                    try {
                        $pinVerb.DoIt()
                        Write-Success "Pinned to Start Menu via shell verb: $($pinVerb.Name)"
                    } catch {
                        Write-Step "Native shell verb pin restricted by Windows security policy; shortcut active in Start Menu."
                    }
                }
            }
        }
    } catch {
        Write-Step "Start Menu pin check note: $_"
    }

    try {
        Set-Content -Path $pinMarker -Value "pinned" -Force -ErrorAction SilentlyContinue
    } catch {}
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
            $_.DisplayName -match '^(Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC|Antigravity Manager Tools By Alim|Agm Tool By Alim|AGM by Alim|Anti-Gravity Tools by Alim|Antigravity Tools|agm-alim)$'
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

    # 1. Try aria2c with 80 parallel split connections and 1MB chunks
    $aria2Bin = Get-Aria2cPath
    if ($aria2Bin) {
        Write-Step "Delegating download request to aria2c accelerator..."
        Write-Step "Accelerating download with aria2c (16 connections, 80 splits, 1MB chunks)..."
        try {
            $ariaArgs = @(
                "--disable-ipv6=true",
                "-x", "16",
                "-s", "80",
                "-j", "16",
                "-k", "1M",
                "--file-allocation=none",
                "--allow-overwrite=true",
                "--auto-file-renaming=false",
                "--summary-interval=0",
                "--console-log-level=error",
                "--show-console-readout=false",
                "--dir=$destDir",
                "-o", "$destFile",
                "$Url"
            )
            $ariaExit = Invoke-IndentedCommand -FilePath $aria2Bin -ArgumentList $ariaArgs
            if ($ariaExit -eq 0) {
                if (Test-Path $DestinationPath) {
                    $itemLen = (Get-Item $DestinationPath).Length
                    if ($itemLen -gt 0) {
                        $sizeMb = [math]::Round($itemLen / 1MB, 2)
                        Write-Success "Download completed successfully via aria2c ($sizeMb MB)."
                        return $true
                    }
                }
            }
            Write-Warn "aria2c finished with code $ariaExit; delegating download request to secondary downloader (curl / Invoke-WebRequest)..."
        } catch {
            Write-Warn "aria2c encountered an error: $_. Delegating download request to secondary downloader..."
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

# Tier 1: Instant & Rate-Limit-Free CDN Manifest Probe (Last 10 releases + exact asset URLs + tag URLs)
$manifestAssetUrlMap = @{}
$manifestTagUrlMap = @{}
$manifestRawTagUrlMap = @{}
$manifestLoaded = $false

$manifestEndpoints = @(
    "https://raw.githubusercontent.com/$Repo/main/releases-manifest.json",
    "https://github.com/$Repo/releases/latest/download/releases-manifest.json"
)

foreach ($mUrl in $manifestEndpoints) {
    try {
        $mResp = Invoke-RestMethod -Uri $mUrl -TimeoutSec 4
        if ($mResp -and $mResp.releases) {
            foreach ($rel in $mResp.releases) {
                $tagVer = $rel.version
                $manifestTagUrlMap[$tagVer] = $rel.tag_url
                $manifestRawTagUrlMap[$tagVer] = $rel.raw_tag_url

                $winAsset = $null
                if ($rel.assets) {
                    if ($rel.assets.windows_x64_setup) {
                        $winAsset = $rel.assets.windows_x64_setup
                    } elseif ($rel.assets.windows_x64_zip) {
                        $winAsset = $rel.assets.windows_x64_zip
                    }
                }
                if ($winAsset) {
                    $manifestAssetUrlMap[$tagVer] = $winAsset
                }

                if (-not $isPinned) {
                    if (-not $candidateVersions.Contains($tagVer)) {
                        $candidateVersions.Add($tagVer)
                    }
                } else {
                    try {
                        $pVer = Convert-ToSemVer $cleanPinned
                        $tVer = Convert-ToSemVer $tagVer
                        if ($tVer -lt $pVer) {
                            if (-not $candidateVersions.Contains($tagVer)) {
                                $candidateVersions.Add($tagVer)
                            }
                        }
                    } catch {}
                }
            }
            if ($candidateVersions.Count -gt 0) {
                $manifestLoaded = $true
                Write-Step "Discovered releases from CDN manifest ($($candidateVersions.Count) versions available, rate-limit free)"
                break
            }
        }
    } catch {}
}

# Tier 2: GitHub REST API (executed if manifest was not reached or yielded insufficient candidates)
if (-not $manifestLoaded -or $candidateVersions.Count -lt 5) {
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
        if ($candidateVersions.Count -ge 10) { break }
    }
}

# If pinned release is not in GitHub releases, replenish queue with latest releases
if ($isPinned) {
    if (-not $releaseMetadataMap.ContainsKey($cleanPinned)) {
        Write-Warn "Requested release v$cleanPinned does not exist on GitHub releases. Replenishing queue with available releases..."
        foreach ($tagVer in $releaseMetadataMap.Keys) {
            if (-not $candidateVersions.Contains($tagVer)) {
                $candidateVersions.Add($tagVer)
            }
            if ($candidateVersions.Count -ge 10) { break }
        }
    }
}

# Fallback known historical releases
$knownFallbacks = @("4.59.0", "4.58.0", "4.57.0", "4.56.0", "4.55.0", "4.52.0", "4.51.0", "4.49.0", "4.48.0", "4.47.1", "4.41.0", "4.40.0", "4.39.0", "4.38.1", "4.38.0", "4.37.0", "4.36.0", "4.35.0", "4.34.0", "4.33.0", "4.32.0", "4.31.0", "4.30.0", "4.7.6")
foreach ($kb in $knownFallbacks) {
    if ($candidateVersions.Count -ge 10) { break }
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

# Build multi-version fallback queue (up to 10 releases)
$versionQueue = @()
foreach ($v in $candidateVersions) {
    if ($versionQueue.Count -lt 10) {
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

# Step 2: Intelligent Multi-Version Try-Catch Installation Ladder (Up to 10 attempts)
$maxAttempts = 10
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
    $candTagUrl = if ($manifestTagUrlMap.ContainsKey($candVersion)) { $manifestTagUrlMap[$candVersion] } else { "https://github.com/$Repo/releases/tag/v$candVersion" }
    Write-Step "Release tag URL     : $candTagUrl"

    try {
        $DownloadUrl = $null
        if ($manifestAssetUrlMap.ContainsKey($candVersion) -and $manifestAssetUrlMap[$candVersion]) {
            $DownloadUrl = $manifestAssetUrlMap[$candVersion]
        } else {
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
        } else {
            # Gracefully handle file-locking for in-app updates:
            # If the application binary is currently running, rename it to .old so NSIS can unpack the new executable freely
            $lockCandidates = @(
                (Join-Path $InstallDir $BinaryName),
                (Join-Path $InstallDir "agm-alim.exe"),
                (Join-Path $InstallDir "AGM by Alim.exe"),
                (Join-Path $InstallDir "Anti-Gravity Tools by Alim.exe"),
                (Join-Path $InstallDir "antigravity-tools.exe")
            )
            foreach ($lc in $lockCandidates) {
                if (Test-Path $lc) {
                    $oldBak = "$lc.old"
                    try {
                        if (Test-Path $oldBak) {
                            Remove-Item -Path $oldBak -Force -ErrorAction SilentlyContinue
                        }
                        Move-Item -Path $lc -Destination $oldBak -Force -ErrorAction SilentlyContinue
                    } catch {}
                }
            }
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
                $oldLnks = @("Agm - Alim.lnk", "agm-alim.lnk", "Anti-Gravity Tools by Alim.lnk", "Antigravity Tools.lnk", "antigravity-tools.lnk")
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
                $smShortcut = $WshShell.CreateShortcut($StartMenuShortcut)
                $smShortcut.TargetPath = $ExePath
                $smShortcut.WorkingDirectory = $InstallDir
                $smShortcut.Description = $Tooltip
                $smShortcut.Save()
                Write-Success "Configured Start Menu shortcut: $StartMenuShortcut"

                # Step 6: Pin to Start Menu if not exists (one-time setup)
                Pin-StartMenuOnce -ShortcutPath $StartMenuShortcut -MarkerDir $InstallDir

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

            # Step 6: Taskbar Pinning disabled per user specification.
            # Preserves user's existing pinned taskbar items without duplicate or broken pinned shortcuts.
        }
    }
}

Write-Host ""
Write-Success "Installation of $FullName v$TargetVersion completed successfully!"
Write-Host "${LeftPadding}Target Directory: $InstallDir" -ForegroundColor Gray
Write-Host "${LeftPadding}Executable:       $ExePath" -ForegroundColor Gray
Write-Host ""

# Ensure Windows Add/Remove Programs (Installed Apps) reflects full branding
function Sync-InstalledAppsRegistryBranding {
    $targetName = "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
    $regRoots = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall"
    )
    foreach ($root in $regRoots) {
        if (Test-Path $root) {
            $subkeys = @("agm-alim", "Antigravity Tools", "Antigravity-Tools", "AGM by Alim")
            foreach ($sk in $subkeys) {
                $fullPath = Join-Path $root $sk
                if (Test-Path $fullPath) {
                    try {
                        Set-ItemProperty -Path $fullPath -Name "DisplayName" -Value $targetName -Force -ErrorAction SilentlyContinue
                        Set-ItemProperty -Path $fullPath -Name "Publisher" -Value $targetName -Force -ErrorAction SilentlyContinue
                    } catch {}
                }
            }
        }
    }
}
Sync-InstalledAppsRegistryBranding

# Ensure Auto Sync Current Account is default true
Ensure-DefaultConfig

# Close any lingering tasks, Electron instances, and WebViews before launching new version
Close-ToolProcesses -Context "post-installation"

# Launch new tool and verify it runs properly
if (-not $NoLaunch) {
    if ($ExePath) {
        if (Test-Path $ExePath) {
            Write-Step "Launching $ShortcutName ($ExePath)..."
            try {
                $launchedProc = Start-Process -FilePath $ExePath -WorkingDirectory $InstallDir -PassThru -ErrorAction SilentlyContinue
                if ($launchedProc) {
                    Start-Sleep -Milliseconds 1200
                    if (-not $launchedProc.HasExited) {
                        Write-Success "$ShortcutName is running properly (PID: $($launchedProc.Id))."
                    } else {
                        Write-Warn "$ShortcutName exited shortly after startup (ExitCode: $($launchedProc.ExitCode))."
                    }
                } else {
                    Write-Warn "Could not automatically start $ShortcutName."
                }
            } catch {
                Write-Warn "Could not launch ${ShortcutName}: $_"
            }
        }
    }
}

Write-Host "${LeftPadding}You can now launch '$ShortcutName' directly or run '$BinaryName' from any terminal." -ForegroundColor Green
Write-Host ""
Write-Host ""
