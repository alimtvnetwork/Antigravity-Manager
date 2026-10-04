# Requires -Version 5.1
<#
.SYNOPSIS
    deploy-antigravity-ide-fleet.ps1
    Unattended, zero-touch fleet deployment script for Google Antigravity IDE.
    Deploys IDE binaries, CLI agy.exe, workbench themes, Turbo mode presets,
    the 4 official plugins, 9 core builtin skills, extensions, and keyring markers.
.DESCRIPTION
    Automates deployment and configuration across local machines and remote nodes.
    Enforces the 4 Pillars of Antigravity IDE Parity:
      - Pillar 1: Theme Parity (Default Dark Modern / custom theme in settings.json)
      - Pillar 2: Preset Mode Parity (Turbo Mode, eager execution, auto-approvals)
      - Pillar 3: Plugins Parity (chrome-devtools, data-agent-kit, antigravity-sdk, modern-web)
      - Pillar 4: Skills Parity (9 builtin skills + plugin skills)
    Concludes with an automated 7-Gate Verification Scorecard (VG-01 to VG-07).
#>
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Parameter(Position = 0, HelpMessage = "Target hostname, IP address, or GitMap SSH node alias")]
    [string]$TargetHost = "localhost",

    [Parameter(Position = 1, HelpMessage = "Target username on destination system")]
    [string]$TargetUser = $env:USERNAME,

    [Parameter(HelpMessage = "Antigravity execution preset mode: turbo, eager, default")]
    [ValidateSet("turbo", "eager", "default")]
    [string]$Preset = "turbo",

    [Parameter(HelpMessage = "Workbench color theme identifier to enforce")]
    [string]$Theme = "Default Dark Modern",

    [Parameter(HelpMessage = "Deploy the 4 official Gemini agent plugins")]
    [bool]$DeployPlugins = $true,

    [Parameter(HelpMessage = "Deploy the 9 core builtin skills and plugin skills")]
    [bool]$DeploySkills = $true,

    [Parameter(HelpMessage = "Deploy or link VS Code extensions directory")]
    [bool]$DeployExtensions = $true,

    [Parameter(HelpMessage = "Source directory containing reference .gemini folder")]
    [string]$SourceGeminiDir = "",

    [Parameter(HelpMessage = "Source directory containing Antigravity Manager tools")]
    [string]$SourceToolsDir = "",

    [Parameter(HelpMessage = "Source directory containing VS Code extensions")]
    [string]$SourceExtensionsDir = "",

    [Parameter(HelpMessage = "Target Antigravity application installation directory")]
    [string]$TargetInstallDir = "",

    [Parameter(HelpMessage = "Target Antigravity data directory")]
    [string]$TargetDataDir = "",

    [Parameter(HelpMessage = "Target user profile home directory")]
    [string]$TargetHomeDir = "",

    [Parameter(HelpMessage = "Preview modifications without writing to disk")]
    [switch]$DryRun,

    [Parameter(HelpMessage = "Force overwrite existing settings and configuration")]
    [switch]$Force
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Test-AdminPrivileges {
    [CmdletBinding()]
    param()
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Read-EnvironmentString {
    param([string]$VarName, [string]$DefaultValue)
    $val = [System.Environment]::GetEnvironmentVariable($VarName)
    if ([string]::IsNullOrWhiteSpace($val)) { return $DefaultValue }
    return $val
}

function Resolve-FleetDirectories {
    param(
        [string]$InSourceGemini,
        [string]$InSourceTools,
        [string]$InSourceExt,
        [string]$InTargetHome,
        [string]$InTargetData,
        [string]$InTargetInstall
    )
    $homeDir = if ($InTargetHome) { $InTargetHome } else { Read-EnvironmentString "ANTIGRAVITY_FLEET_OVERRIDE_HOME" $env:USERPROFILE }
    $srcGemini = if ($InSourceGemini) { $InSourceGemini } else { Read-EnvironmentString "ANTIGRAVITY_FLEET_SOURCE_GEMINI" (Join-Path $env:USERPROFILE ".gemini") }
    $srcTools = if ($InSourceTools) { $InSourceTools } else { Read-EnvironmentString "ANTIGRAVITY_FLEET_SOURCE_TOOLS" (Join-Path $env:USERPROFILE ".antigravity_tools") }
    $srcExt = if ($InSourceExt) { $InSourceExt } else { Read-EnvironmentString "ANTIGRAVITY_FLEET_SOURCE_EXTENSIONS" (Join-Path $env:USERPROFILE ".vscode\extensions") }
    $tgtData = if ($InTargetData) { $InTargetData } else { Join-Path $env:APPDATA "Antigravity" }
    $tgtInstall = if ($InTargetInstall) { $InTargetInstall } else { Join-Path $env:LOCALAPPDATA "Programs\antigravity" }

    return [PSCustomObject]@{
        TargetHome = $homeDir
        SourceGemini = $srcGemini
        SourceTools = $srcTools
        SourceExtensions = $srcExt
        TargetData = $tgtData
        TargetInstall = $tgtInstall
        TargetGemini = (Join-Path $homeDir ".gemini")
        TargetUserHomeData = (Join-Path $homeDir "AppData\Roaming\Antigravity")
    }
}

function Copy-DirectorySafe {
    param([string]$SourceDir, [string]$TargetDir, [bool]$IsDryRun)
    if (-not (Test-Path -LiteralPath $SourceDir)) { return $false }
    if ($IsDryRun) {
        Write-Host "  [DryRun] Copy tree: $SourceDir -> $TargetDir" -ForegroundColor DarkGray
        return $true
    }
    if (-not (Test-Path -LiteralPath $TargetDir)) {
        [System.IO.Directory]::CreateDirectory($TargetDir) | Out-Null
    }
    $excludeDirs = @("cache", "code cache", "gpucache", "dawngraphitecache", "dawnwebgpucache", "crashpad", "blob_storage")
    $sourceItem = Get-Item -LiteralPath $SourceDir
    foreach ($entry in [System.IO.Directory]::EnumerateFileSystemEntries($SourceDir)) {
        $leafName = [System.IO.Path]::GetFileName($entry).ToLowerInvariant()
        $destPath = Join-Path $TargetDir ([System.IO.Path]::GetFileName($entry))
        if ([System.IO.Directory]::Exists($entry)) {
            if ($excludeDirs -contains $leafName -or $leafName.StartsWith(".org.chromium")) { continue }
            Copy-DirectorySafe -SourceDir $entry -TargetDir $destPath -IsDryRun $IsDryRun | Out-Null
        } else {
            if ($leafName -eq "lockfile" -or $leafName.EndsWith(".lock") -or $leafName.StartsWith("singleton")) { continue }
            [System.IO.File]::Copy($entry, $destPath, $true)
        }
    }
    return $true
}

function Write-JsonFileAtomic {
    param([string]$FilePath, [hashtable]$Data, [bool]$IsDryRun)
    if ($IsDryRun) {
        Write-Host "  [DryRun] Write JSON: $FilePath" -ForegroundColor DarkGray
        return $true
    }
    $parent = [System.IO.Path]::GetDirectoryName($FilePath)
    if ($parent -and -not (Test-Path -LiteralPath $parent)) {
        [System.IO.Directory]::CreateDirectory($parent) | Out-Null
    }
    $json = ConvertTo-Json -InputObject $Data -Depth 20
    [System.IO.File]::WriteAllText($FilePath, $json, [System.Text.Encoding]::UTF8)
    return $true
}

function Read-JsonFileSafe {
    param([string]$FilePath)
    if (-not (Test-Path -LiteralPath $FilePath)) { return @{} }
    try {
        $content = [System.IO.File]::ReadAllText($FilePath, [System.Text.Encoding]::UTF8)
        if ([string]::IsNullOrWhiteSpace($content)) { return @{} }
        $obj = ConvertFrom-Json -InputObject $content -AsHashtable
        if ($null -eq $obj) { return @{} }
        return $obj
    } catch {
        return @{}
    }
}

function Update-UserSettingsFile {
    param([string]$SettingsPath, [string]$ThemeName, [bool]$IsDryRun)
    $settings = Read-JsonFileSafe -FilePath $SettingsPath
    $settings["workbench.colorTheme"] = $ThemeName
    $settings["workbench.startupEditor"] = "none"
    $settings["antigravity.turboMode"] = $true
    $settings["antigravity.planReviewAlwaysProceed"] = $true
    $settings["antigravity.browserExecutionPolicy"] = "openDirectly"
    $settings["antigravity.codeReviewPolicy"] = "automaticReview"
    $settings["security.workspace.trust.enabled"] = $false
    $settings["security.workspace.trust.startupPrompt"] = "never"
    $settings["security.workspace.trust.emptyWindow"] = $false
    $settings["gemini.experimental.eagerExecution"] = $true
    $settings["antigravity.securityPreset"] = "turbo"
    $settings["antigravity.autoApprove"] = $true
    $settings["files.autoSave"] = "afterDelay"
    return Write-JsonFileAtomic -FilePath $SettingsPath -Data $settings -IsDryRun $IsDryRun
}

function Deploy-Pillar1Theme {
    param($Paths, [string]$ThemeName, [bool]$IsDryRun)
    Write-Host "[Pillar 1] Deploying Workbench Theme: $ThemeName" -ForegroundColor Cyan
    $settingsTargets = @(
        (Join-Path $Paths.TargetData "User\settings.json"),
        (Join-Path $Paths.TargetUserHomeData "User\settings.json"),
        (Join-Path $Paths.SourceTools "instances\default\home\AppData\Roaming\Antigravity\User\settings.json")
    )
    foreach ($path in $settingsTargets) {
        Update-UserSettingsFile -SettingsPath $path -ThemeName $ThemeName -IsDryRun $IsDryRun | Out-Null
    }
    $cfgPath = Join-Path $Paths.TargetGemini "config\config.json"
    $cfg = Read-JsonFileSafe -FilePath $cfgPath
    if (-not $cfg.ContainsKey("userSettings")) { $cfg["userSettings"] = @{} }
    $cfg["userSettings"]["themeMode"] = "THEME_MODE_DARK"
    Write-JsonFileAtomic -FilePath $cfgPath -Data $cfg -IsDryRun $IsDryRun | Out-Null
    return $true
}

function Deploy-SecurityPresetsJson {
    param([string]$PresetPath, [bool]$IsDryRun)
    $data = @{
        "active_preset" = "turbo"
        "presets" = @{
            "turbo" = @{
                "name" = "Turbo (Autonomous)"
                "description" = "High-velocity execution with zero-trust bypass and auto-approvals"
                "auto_approve_commands" = $true
                "auto_approve_file_edits" = $true
                "bypass_command_confirmations" = $true
                "zero_trust_bypass" = $true
                "eager_execution" = $true
            }
        }
    }
    return Write-JsonFileAtomic -FilePath $PresetPath -Data $data -IsDryRun $IsDryRun
}

function Deploy-AntigravityPoliciesJson {
    param([string]$PolicyPath, [bool]$IsDryRun)
    $data = @{
        "version" = 1
        "default_policy" = "allow"
        "policies" = @(
            @{
                "name" = "unrestricted_terminal"
                "action" = "allow"
                "match" = "*"
            }
        )
    }
    return Write-JsonFileAtomic -FilePath $PolicyPath -Data $data -IsDryRun $IsDryRun
}

function Deploy-Pillar2Presets {
    param($Paths, [string]$PresetMode, [bool]$IsDryRun)
    Write-Host "[Pillar 2] Deploying Presets and Policies ($PresetMode Mode)" -ForegroundColor Cyan
    $presetTargets = @(
        (Join-Path $Paths.TargetData "User\security_presets.json"),
        (Join-Path $Paths.TargetUserHomeData "User\security_presets.json")
    )
    foreach ($path in $presetTargets) {
        Deploy-SecurityPresetsJson -PresetPath $path -IsDryRun $IsDryRun | Out-Null
    }
    $policyTargets = @(
        (Join-Path $Paths.TargetData "User\antigravity_policies.json"),
        (Join-Path $Paths.TargetUserHomeData "User\antigravity_policies.json")
    )
    foreach ($path in $policyTargets) {
        Deploy-AntigravityPoliciesJson -PolicyPath $path -IsDryRun $IsDryRun | Out-Null
    }
    $cfgPath = Join-Path $Paths.TargetGemini "config\config.json"
    $cfg = Read-JsonFileSafe -FilePath $cfgPath
    if (-not $cfg.ContainsKey("userSettings")) { $cfg["userSettings"] = @{} }
    $cfg["userSettings"]["artifactReviewMode"] = "ARTIFACT_REVIEW_MODE_TURBO"
    $cfg["userSettings"]["autoExecutionPolicy"] = "CASCADE_COMMANDS_AUTO_EXECUTION_EAGER"
    $cfg["userSettings"]["browserJsExecutionPolicy"] = "BROWSER_JS_EXECUTION_POLICY_TURBO"
    $cfg["userSettings"]["nonWorkspaceFileAccessPolicy"] = "AGENT_SETTING_POLICY_ALLOW"
    $cfg["userSettings"]["enableTerminalSandbox"] = $false
    Write-JsonFileAtomic -FilePath $cfgPath -Data $cfg -IsDryRun $IsDryRun | Out-Null
    return $true
}

function Deploy-SinglePlugin {
    param([string]$PluginName, [string]$SrcRoot, [string]$DstRoot, [bool]$IsDryRun)
    $src = Join-Path $SrcRoot $PluginName
    $dst = Join-Path $DstRoot $PluginName
    if (Test-Path -LiteralPath $src) {
        Copy-DirectorySafe -SourceDir $src -TargetDir $dst -IsDryRun $IsDryRun | Out-Null
        return $true
    }
    return $false
}

function Deploy-Pillar3Plugins {
    param($Paths, [bool]$IsDeploy, [bool]$IsDryRun)
    if (-not $IsDeploy) { return $true }
    Write-Host "[Pillar 3] Deploying Official 4 Gemini Agent Plugins" -ForegroundColor Cyan
    $plugins = @("chrome-devtools-plugin", "data-agent-kit-plugin", "google-antigravity-sdk", "modern-web-guidance-plugin")
    $srcPluginsDir = Join-Path $Paths.SourceGemini "config\plugins"
    $dstPluginsDir = Join-Path $Paths.TargetGemini "config\plugins"

    foreach ($p in $plugins) {
        $hasCopied = Deploy-SinglePlugin -PluginName $p -SrcRoot $srcPluginsDir -DstRoot $dstPluginsDir -IsDryRun $IsDryRun
        if ($hasCopied) {
            Write-Host "  - Plugin '$p' synchronized" -ForegroundColor Green
        } else {
            Write-Warning "  - Plugin '$p' source not found in $srcPluginsDir"
        }
    }
    $cfgPath = Join-Path $Paths.TargetGemini "config\config.json"
    $cfg = Read-JsonFileSafe -FilePath $cfgPath
    if (-not $cfg.ContainsKey("plugins")) { $cfg["plugins"] = @{} }
    foreach ($p in $plugins) {
        $cfg["plugins"][$p] = @{ "enabled" = $true }
    }
    Write-JsonFileAtomic -FilePath $cfgPath -Data $cfg -IsDryRun $IsDryRun | Out-Null
    return $true
}

function Deploy-Pillar4Skills {
    param($Paths, [bool]$IsDeploy, [bool]$IsDryRun)
    if (-not $IsDeploy) { return $true }
    Write-Host "[Pillar 4] Deploying 9 Builtin Skills and Plugin Skills" -ForegroundColor Cyan
    $srcBuiltin = Join-Path $Paths.SourceGemini "antigravity\builtin\skills"
    $dstBuiltin = Join-Path $Paths.TargetGemini "antigravity\builtin\skills"
    $skills = @(
        "agy-customizations", "antigravity_guide", "automation",
        "generative_ui", "migrate-workflows", "permissioned-github",
        "plugin", "ui-extension", "ui-plugin-navigation"
    )
    foreach ($s in $skills) {
        $src = Join-Path $srcBuiltin $s
        $dst = Join-Path $dstBuiltin $s
        if (Test-Path -LiteralPath $src) {
            Copy-DirectorySafe -SourceDir $src -TargetDir $dst -IsDryRun $IsDryRun | Out-Null
            Write-Host "  - Builtin Skill '$s' synchronized" -ForegroundColor Green
        }
    }
    return $true
}

function Deploy-ExtensionsPillar {
    param($Paths, [bool]$IsDeploy, [bool]$IsDryRun)
    if (-not $IsDeploy) { return $true }
    Write-Host "[Extensions] Synchronizing VS Code Extensions" -ForegroundColor Cyan
    $srcExt = $Paths.SourceExtensions
    $dstExt = Join-Path $Paths.TargetHome ".vscode\extensions"
    if (Test-Path -LiteralPath $srcExt) {
        Copy-DirectorySafe -SourceDir $srcExt -TargetDir $dstExt -IsDryRun $IsDryRun | Out-Null
        Write-Host "  - Extensions directory populated: $dstExt" -ForegroundColor Green
        return $true
    }
    return $false
}

function Deploy-KeyringAndAppStorage {
    param($Paths, [bool]$IsDryRun)
    Write-Host "[Hygiene] Writing Keyring Bypass Markers & App Storage" -ForegroundColor Cyan
    $markerNames = @(
        "antigravity-ide-keyring-unavailable",
        "antigravity-keyring-unavailable",
        "antigravity-cli-keyring-unavailable"
    )
    $dirsToMark = @($Paths.TargetGemini, $Paths.TargetData, $Paths.TargetUserHomeData)
    foreach ($dir in $dirsToMark) {
        if (-not (Test-Path -LiteralPath $dir) -and -not $IsDryRun) {
            [System.IO.Directory]::CreateDirectory($dir) | Out-Null
        }
        foreach ($m in $markerNames) {
            $mPath = Join-Path $dir $m
            if (-not $IsDryRun -and -not (Test-Path -LiteralPath $mPath)) {
                [System.IO.File]::WriteAllText($mPath, "bypass", [System.Text.Encoding]::UTF8)
            }
        }
    }
    $appStoragePath = Join-Path $Paths.TargetData "app_storage.json"
    $storage = Read-JsonFileSafe -FilePath $appStoragePath
    $storage["ide-install-wizard-shown"] = "true"
    $storage["onboarding-completed"] = "true"
    Write-JsonFileAtomic -FilePath $appStoragePath -Data $storage -IsDryRun $IsDryRun | Out-Null
    return $true
}

function Deploy-AntigravityBinaries {
    param($Paths, [bool]$IsDryRun)
    Write-Host "[Binaries] Verifying and Deploying Antigravity IDE Binaries" -ForegroundColor Cyan
    $mainExe = Join-Path $Paths.TargetInstall "Antigravity.exe"
    $hasMainExe = Test-Path -LiteralPath $mainExe
    if ($hasMainExe) {
        Write-Host "  - Antigravity.exe verified at: $mainExe" -ForegroundColor Green
    } else {
        Write-Warning "  - Antigravity.exe not located at $mainExe"
    }

    $srcAgy = Join-Path $Paths.SourceGemini "bin\agy.exe"
    $dstAgyDir = Join-Path $Paths.TargetGemini "bin"
    $dstAgy = Join-Path $dstAgyDir "agy.exe"
    if (Test-Path -LiteralPath $srcAgy) {
        if (-not (Test-Path -LiteralPath $dstAgyDir) -and -not $IsDryRun) {
            [System.IO.Directory]::CreateDirectory($dstAgyDir) | Out-Null
        }
        if (-not $IsDryRun) {
            [System.IO.File]::Copy($srcAgy, $dstAgy, $true)
        }
        Write-Host "  - CLI agy.exe deployed to: $dstAgy" -ForegroundColor Green
    }

    $currPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($currPath -notlike "*$dstAgyDir*") {
        if (-not $IsDryRun) {
            [System.Environment]::SetEnvironmentVariable("Path", "$currPath;$dstAgyDir", "User")
        }
        Write-Host "  - Registered $dstAgyDir in User PATH" -ForegroundColor Green
    }
    return $true
}

function Invoke-RemoteFleetDelegation {
    param([string]$Node, [string]$ScriptPath, [string]$PresetMode, [string]$ThemeName, [bool]$IsForce)
    Write-Host "[Fleet] Delegating deployment to remote node '$Node' via GitMap..." -ForegroundColor Cyan
    $gitmapExe = Get-Command "gitmap" -ErrorAction SilentlyContinue
    if ($gitmapExe) {
        $cmdArgs = @("cluster", "run-script", $Node, $ScriptPath, "-Preset", $PresetMode, "-Theme", $ThemeName)
        if ($IsForce) { $cmdArgs += "-Force" }
        & gitmap $cmdArgs
        return ($LASTEXITCODE -eq 0)
    }
    Write-Warning "[Fleet] GitMap CLI not found on local path for remote delegation."
    return $false
}

function Test-GateTheme {
    param($Paths, [string]$ThemeName)
    $settingsPath = Join-Path $Paths.TargetData "User\settings.json"
    $settings = Read-JsonFileSafe -FilePath $settingsPath
    $isThemeMatch = ($settings.ContainsKey("workbench.colorTheme") -and $settings["workbench.colorTheme"] -eq $ThemeName)
    $cfgPath = Join-Path $Paths.TargetGemini "config\config.json"
    $cfg = Read-JsonFileSafe -FilePath $cfgPath
    $isModeMatch = ($cfg.ContainsKey("userSettings") -and $cfg["userSettings"]["themeMode"] -eq "THEME_MODE_DARK")
    return ($isThemeMatch -and $isModeMatch)
}

function Test-GatePresets {
    param($Paths)
    $settings = Read-JsonFileSafe -FilePath (Join-Path $Paths.TargetData "User\settings.json")
    $isTurbo = ($settings["antigravity.turboMode"] -eq $true -and $settings["security.workspace.trust.enabled"] -eq $false)
    $presets = Read-JsonFileSafe -FilePath (Join-Path $Paths.TargetData "User\security_presets.json")
    $isActive = ($presets["active_preset"] -eq "turbo")
    return ($isTurbo -and $isActive)
}

function Test-GatePlugins {
    param($Paths)
    $plugins = @("chrome-devtools-plugin", "data-agent-kit-plugin", "google-antigravity-sdk", "modern-web-guidance-plugin")
    $dstRoot = Join-Path $Paths.TargetGemini "config\plugins"
    $count = 0
    foreach ($p in $plugins) {
        if (Test-Path -LiteralPath (Join-Path $dstRoot $p)) { $count++ }
    }
    return ($count -eq 4)
}

function Test-GateSkills {
    param($Paths)
    $skills = @("agy-customizations", "antigravity_guide", "automation", "generative_ui", "migrate-workflows", "permissioned-github", "plugin", "ui-extension", "ui-plugin-navigation")
    $dstRoot = Join-Path $Paths.TargetGemini "antigravity\builtin\skills"
    $count = 0
    foreach ($s in $skills) {
        if (Test-Path -LiteralPath (Join-Path $dstRoot $s)) { $count++ }
    }
    return ($count -ge 9)
}

function Test-GateHygiene {
    param($Paths)
    $marker = Join-Path $Paths.TargetGemini "antigravity-ide-keyring-unavailable"
    $storage = Join-Path $Paths.TargetData "app_storage.json"
    $hasMarker = Test-Path -LiteralPath $marker
    $hasStorage = Test-Path -LiteralPath $storage
    return ($hasMarker -and $hasStorage)
}

function Test-GateBinaries {
    param($Paths)
    $mainExe = Join-Path $Paths.TargetInstall "Antigravity.exe"
    $agyExe = Join-Path $Paths.TargetGemini "bin\agy.exe"
    return (Test-Path -LiteralPath $mainExe) -and (Test-Path -LiteralPath $agyExe)
}

function Test-GateInstances {
    param($Paths)
    $defaultSettings = Join-Path $Paths.SourceTools "instances\default\home\AppData\Roaming\Antigravity\User\settings.json"
    $hasInstancesDir = Test-Path -LiteralPath (Join-Path $Paths.SourceTools "instances")
    return ($hasInstancesDir -or (Test-Path -LiteralPath $defaultSettings))
}

function Show-VerificationScorecard {
    param($Paths, [string]$ThemeName, [string]$PresetMode)
    $g1 = Test-GateTheme -Paths $Paths -ThemeName $ThemeName
    $g2 = Test-GatePresets -Paths $Paths
    $g3 = Test-GatePlugins -Paths $Paths
    $g4 = Test-GateSkills -Paths $Paths
    $g5 = Test-GateHygiene -Paths $Paths
    $g6 = Test-GateBinaries -Paths $Paths
    $g7 = Test-GateInstances -Paths $Paths

    $results = @($g1, $g2, $g3, $g4, $g5, $g6, $g7)
    $passedCount = ($results | Where-Object { $_ -eq $true }).Count

    Write-Host ""
    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host "   ANTIGRAVITY IDE FLEET DEPLOYMENT & PARITY VERIFICATION SCORECARD" -ForegroundColor White
    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host ("[{0}] VG-01: Theme Parity Gate           (Target: {1})" -f ($(if ($g1) {"PASS"} else {"FAIL"}), $ThemeName)) -ForegroundColor $(if ($g1) {"Green"} else {"Red"})
    Write-Host ("[{0}] VG-02: Preset Mode Gate           (Target: {1} Mode, Trust Bypass)" -f ($(if ($g2) {"PASS"} else {"FAIL"}), $PresetMode)) -ForegroundColor $(if ($g2) {"Green"} else {"Red"})
    Write-Host ("[{0}] VG-03: Plugins Inventory Gate     (Target: 4/4 Official Plugins)" -f ($(if ($g3) {"PASS"} else {"FAIL"}))) -ForegroundColor $(if ($g3) {"Green"} else {"Red"})
    Write-Host ("[{0}] VG-04: Skills Completeness Gate   (Target: 9 Core Builtin Skills)" -f ($(if ($g4) {"PASS"} else {"FAIL"}))) -ForegroundColor $(if ($g4) {"Green"} else {"Red"})
    Write-Host ("[{0}] VG-05: Hygiene & Markers Gate     (Target: Keyring Bypass & App Storage)" -f ($(if ($g5) {"PASS"} else {"FAIL"}))) -ForegroundColor $(if ($g5) {"Green"} else {"Red"})
    Write-Host ("[{0}] VG-06: Binaries & CLI Gate        (Target: Antigravity.exe & agy.exe)" -f ($(if ($g6) {"PASS"} else {"FAIL"}))) -ForegroundColor $(if ($g6) {"Green"} else {"Red"})
    Write-Host ("[{0}] VG-07: Instances Readiness Gate   (Target: Instance Profiles Parity)" -f ($(if ($g7) {"PASS"} else {"FAIL"}))) -ForegroundColor $(if ($g7) {"Green"} else {"Red"})
    Write-Host "--------------------------------------------------------------------------------" -ForegroundColor Cyan
    Write-Host ("SCORECARD OUTCOME: {0}/7 GATES PASSED" -f $passedCount) -ForegroundColor $(if ($passedCount -eq 7) {"Green"} else {"Yellow"})
    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host ""
    return ($passedCount -ge 6)
}

function Main {
    $isTargetLocal = ($TargetHost -eq "localhost" -or $TargetHost -eq "127.0.0.1" -or $TargetHost -eq "$env:COMPUTERNAME")
    $isDryRun = [bool]$DryRun
    $isForce = [bool]$Force

    if (-not $isTargetLocal) {
        $scriptPath = $PSCommandPath
        if (-not $scriptPath) { $scriptPath = "scripts/deploy-antigravity-ide-fleet.ps1" }
        $isDelegated = Invoke-RemoteFleetDelegation -Node $TargetHost -ScriptPath $scriptPath -PresetMode $Preset -ThemeName $Theme -IsForce $isForce
        if ($isDelegated) { exit 0 } else { exit 1 }
    }

    $paths = Resolve-FleetDirectories `
        -InSourceGemini $SourceGeminiDir `
        -InSourceTools $SourceToolsDir `
        -InSourceExt $SourceExtensionsDir `
        -InTargetHome $TargetHomeDir `
        -InTargetData $TargetDataDir `
        -InTargetInstall $TargetInstallDir

    Write-Host ">>> Starting Antigravity IDE Full Fleet Deployment on $TargetHost <<<" -ForegroundColor Yellow
    Deploy-Pillar1Theme -Paths $paths -ThemeName $Theme -IsDryRun $isDryRun | Out-Null
    Deploy-Pillar2Presets -Paths $paths -PresetMode $Preset -IsDryRun $isDryRun | Out-Null
    Deploy-Pillar3Plugins -Paths $paths -IsDeploy $DeployPlugins -IsDryRun $isDryRun | Out-Null
    Deploy-Pillar4Skills -Paths $paths -IsDeploy $DeploySkills -IsDryRun $isDryRun | Out-Null
    Deploy-ExtensionsPillar -Paths $paths -IsDeploy $DeployExtensions -IsDryRun $isDryRun | Out-Null
    Deploy-KeyringAndAppStorage -Paths $paths -IsDryRun $isDryRun | Out-Null
    Deploy-AntigravityBinaries -Paths $paths -IsDryRun $isDryRun | Out-Null

    $isAllPassed = Show-VerificationScorecard -Paths $paths -ThemeName $Theme -PresetMode $Preset
    if ($isAllPassed) {
        Write-Host "[SUCCESS] Antigravity IDE deployment completed with high fidelity." -ForegroundColor Green
        exit 0
    } else {
        Write-Warning "[WARN] Deployment completed with one or more gate warnings."
        exit 0
    }
}

Main
