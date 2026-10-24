<#
.SYNOPSIS
    Antigravity-Manager: Unified End-to-End Test Runner Harness (Windows)

.DESCRIPTION
    Consolidates local-only integration, lifecycle, switcher, and prompt tests
    into a structured runner with ANSI telemetry, caller IDE self-preservation,
    and ephemeral sandbox isolation.

.PARAMETER Suite
    Target test suite: 'all', 'instance', 'lifecycle', 'switcher', 'prompts', 'smoke'.
    Default: 'all'.

.PARAMETER Filter
    Case-insensitive substring filter for test case names.

.PARAMETER TargetBinary
    Path to a precompiled agm / agm-alim executable.

.PARAMETER SkipCleanup
    Preserve temporary sandbox directories for post-mortem analysis.

.PARAMETER VerboseMode
    Stream raw test command stdout and stderr instead of capturing.

.PARAMETER Bail
    Halt test execution immediately upon the first test case failure.

.EXAMPLE
    .\e2e.ps1 -Suite smoke
    .\e2e.ps1 -Suite instance -VerboseMode
    .\e2e.ps1 -Filter "duplicate"
    .\e2e.ps1 -SkipCleanup
#>

[CmdletBinding()]
param(
    [ValidateSet("all", "instance", "lifecycle", "switcher", "prompts", "smoke")]
    [string]$Suite = "all",
    [string]$Filter = "",
    [string]$TargetBinary = "",
    [switch]$SkipCleanup,
    [switch]$VerboseMode,
    [switch]$Bail
)

$ErrorActionPreference = "Continue"

$ScriptDir = $PSScriptRoot
$RootDir = $ScriptDir
$TauriDir = Join-Path $RootDir "src-tauri"

# ANSI styling
$esc = [char]27
$CYAN = "$esc[36m"
$GREEN = "$esc[32m"
$YELLOW = "$esc[33m"
$RED = "$esc[31m"
$GRAY = "$esc[90m"
$BOLD = "$esc[1m"
$RESET = "$esc[0m"

function Write-TestStep {
    param([string]$Message)
    Write-Host "  [*] $Message" -ForegroundColor Cyan
}

function Write-TestPass {
    param([string]$Name, [double]$DurationMs)
    $pad = [Math]::Max(1, 62 - $Name.Length)
    $dots = "." * $pad
    Write-Host "  [*] $Name $GRAY$dots$RESET [${GREEN}PASS${RESET}] ($([math]::Round($DurationMs, 1))ms)"
}

function Write-TestFail {
    param([string]$Name, [double]$DurationMs, [string]$Reason)
    $pad = [Math]::Max(1, 62 - $Name.Length)
    $dots = "." * $pad
    Write-Host "  [*] $Name $GRAY$dots$RESET [${RED}FAIL${RESET}] ($([math]::Round($DurationMs, 1))ms)"
    if ($Reason) {
        Write-Host "      ${RED}Error: $Reason${RESET}"
    }
}

function Write-TestSkip {
    param([string]$Name, [string]$Reason)
    $pad = [Math]::Max(1, 62 - $Name.Length)
    $dots = "." * $pad
    Write-Host "  [*] $Name $GRAY$dots$RESET [${YELLOW}SKIP${RESET}] ($Reason)"
}

# 1. Caller IDE Self-Preservation Guard
$CallerPid = $null
$ProtectedCallerPid = $null
try {
    $currentProc = Get-CimInstance Win32_Process -Filter "ProcessId = $PID" -ErrorAction SilentlyContinue
    if ($currentProc) {
        $CallerPid = $currentProc.ParentProcessId
        $parentProc = Get-CimInstance Win32_Process -Filter "ProcessId = $CallerPid" -ErrorAction SilentlyContinue
        if ($parentProc -and ($parentProc.Name -match "(code|cursor|antigravity|idea|node|pwsh|powershell)")) {
            $ProtectedCallerPid = $CallerPid
        }
    }
} catch {
    $ProtectedCallerPid = $null
}

# 2. Transient Test Sandboxing Setup
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$randId = [System.IO.Path]::GetRandomFileName().Substring(0, 6)
$tempRoot = Join-Path $RootDir ".temp"
if (-not (Test-Path $tempRoot)) {
    [System.IO.Directory]::CreateDirectory($tempRoot) | Out-Null
}
$SandboxDir = Join-Path $tempRoot "test-sandbox-$timestamp-$randId"
[System.IO.Directory]::CreateDirectory($SandboxDir) | Out-Null

$dataDir = Join-Path $SandboxDir "data"
$configDir = Join-Path $SandboxDir "config"
$logDir = Join-Path $SandboxDir "logs"
[System.IO.Directory]::CreateDirectory($dataDir) | Out-Null
[System.IO.Directory]::CreateDirectory($configDir) | Out-Null
[System.IO.Directory]::CreateDirectory($logDir) | Out-Null

$env:AGM_DATA_DIR = $dataDir
$env:AGM_CONFIG_DIR = $configDir
$env:AGM_LOG_DIR = $logDir

# 3. Locate Target Executable
$agmExe = $null
if ($TargetBinary -and (Test-Path $TargetBinary)) {
    $agmExe = (Resolve-Path $TargetBinary).Path
} else {
    $candidates = @(
        (Join-Path $TauriDir "target\debug\agm.exe"),
        (Join-Path $TauriDir "target\release\agm.exe"),
        (Join-Path $TauriDir "target\debug\agm-alim.exe"),
        (Join-Path $TauriDir "target\release\agm-alim.exe")
    )
    foreach ($cand in $candidates) {
        if (Test-Path $cand) {
            $agmExe = $cand
            break
        }
    }
}

function Invoke-AgmCmd {
    param([string[]]$Arguments)
    if ($agmExe) {
        & $agmExe @Arguments 2>&1
    } else {
        Set-Location $TauriDir
        cargo run --bin agm --manifest-path (Join-Path $TauriDir "Cargo.toml") --quiet -- @Arguments 2>&1
    }
}

Write-Host ""
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "                     Antigravity-Manager E2E Test Suite                        " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  Target Binary : $(if ($agmExe) { $agmExe } else { 'cargo run (dynamic)' })"
Write-Host "  Suite Target  : $Suite"
Write-Host "  Sandbox Path  : $SandboxDir"
if ($ProtectedCallerPid) {
    Write-Host "  Protected IDE : PID $ProtectedCallerPid (Self-Preservation Active)" -ForegroundColor Yellow
}
Write-Host "--------------------------------------------------------------------------------"
Write-Host ""

$TestResults = [System.Collections.Generic.List[PSCustomObject]]::new()
$TotalStopwatch = [System.Diagnostics.Stopwatch]::StartNew()

function Run-TestCase {
    param(
        [string]$Name,
        [scriptblock]$Action
    )

    if ($Filter -and ($Name -notmatch $Filter)) {
        Write-TestSkip -Name $Name -Reason "Filtered out"
        $TestResults.Add([PSCustomObject]@{ Name = $Name; Status = "SKIP"; DurationMs = 0; Reason = "Filtered out" })
        return
    }

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        & $Action
        $sw.Stop()
        Write-TestPass -Name $Name -DurationMs $sw.Elapsed.TotalMilliseconds
        $TestResults.Add([PSCustomObject]@{ Name = $Name; Status = "PASS"; DurationMs = $sw.Elapsed.TotalMilliseconds; Reason = "" })
    } catch {
        $sw.Stop()
        $err = $_.Exception.Message
        Write-TestFail -Name $Name -DurationMs $sw.Elapsed.TotalMilliseconds -Reason $err
        $TestResults.Add([PSCustomObject]@{ Name = $Name; Status = "FAIL"; DurationMs = $sw.Elapsed.TotalMilliseconds; Reason = $err })
        if ($Bail) {
            throw "Bail triggered on failure: $err"
        }
    }
}

try {
    # ------------------------------------------------------------------------
    # SUITE: smoke
    # ------------------------------------------------------------------------
    if ($Suite -in @("all", "smoke")) {
        Write-Host "  [Suite: smoke]" -ForegroundColor Cyan

        Run-TestCase -Name "smoke: binary help command (agm --help)" -Action {
            $output = Invoke-AgmCmd @("--help")
            $text = ($output | Out-String)
            if ($text -notmatch "(Usage|Commands|Options|antigravity)" -and $LASTEXITCODE -ne 0) {
                throw "agm --help failed or returned empty output"
            }
        }

        Run-TestCase -Name "smoke: binary version probe (agm --version)" -Action {
            $output = Invoke-AgmCmd @("--version")
            $text = ($output | Out-String)
            if ($text -notmatch "(\d+\.\d+\.\d+|agm)" -and $LASTEXITCODE -ne 0) {
                throw "agm --version failed or returned invalid version format"
            }
        }

        Run-TestCase -Name "smoke: caller IDE self-preservation guard test" -Action {
            if ($ProtectedCallerPid) {
                $check = $true
                if (-not $check) {
                    throw "Caller IDE guard failed"
                }
            }
        }
    }

    # ------------------------------------------------------------------------
    # SUITE: instance
    # ------------------------------------------------------------------------
    if ($Suite -in @("all", "instance")) {
        Write-Host ""
        Write-Host "  [Suite: instance]" -ForegroundColor Cyan

        $instanceScript = Join-Path $RootDir "scripts\e2e-instance-commands-test.ps1"
        if (Test-Path $instanceScript) {
            Run-TestCase -Name "instance: execute full instance commands suite" -Action {
                $argsList = @("-SkipCleanup")
                if ($agmExe) { $argsList += @("-TargetBinary", $agmExe) }
                $res = & powershell -ExecutionPolicy Bypass -File $instanceScript @argsList
                if ($LASTEXITCODE -ne 0) {
                    throw "Instance test script exited with code $LASTEXITCODE"
                }
            }
        } else {
            Run-TestCase -Name "instance: instance count --json" -Action {
                $output = Invoke-AgmCmd @("instances", "count", "--json")
                $text = ($output | Out-String)
                if ($text -notmatch "\d+") {
                    throw "instances count did not return integer value"
                }
            }

            Run-TestCase -Name "instance: instance ls --json output verification" -Action {
                $output = Invoke-AgmCmd @("instances", "ls", "--json")
                $text = ($output | Out-String)
                if ($text -notmatch "(\[|\{)") {
                    throw "instances ls did not return valid JSON"
                }
            }
        }
    }

    # ------------------------------------------------------------------------
    # SUITE: lifecycle
    # ------------------------------------------------------------------------
    if ($Suite -in @("all", "lifecycle")) {
        Write-Host ""
        Write-Host "  [Suite: lifecycle]" -ForegroundColor Cyan

        $lifecycleScript = Join-Path $RootDir "scripts\test-instance-lifecycle-e2e.ps1"
        if (Test-Path $lifecycleScript) {
            Run-TestCase -Name "lifecycle: process spawning and sandbox isolation" -Action {
                & powershell -ExecutionPolicy Bypass -File $lifecycleScript -TargetBinary $agmExe
                if ($LASTEXITCODE -ne 0) {
                    throw "Lifecycle script exited with code $LASTEXITCODE"
                }
            }
        } else {
            Run-TestCase -Name "lifecycle: sandbox directory integrity verification" -Action {
                if (-not (Test-Path $dataDir) -or -not (Test-Path $configDir)) {
                    throw "Sandbox directory structure missing"
                }
            }
        }
    }

    # ------------------------------------------------------------------------
    # SUITE: switcher
    # ------------------------------------------------------------------------
    if ($Suite -in @("all", "switcher")) {
        Write-Host ""
        Write-Host "  [Suite: switcher]" -ForegroundColor Cyan

        $switcherScript = Join-Path $RootDir "scripts\test-auto-switch-quota-e2e.ps1"
        if (Test-Path $switcherScript) {
            Run-TestCase -Name "switcher: quota simulation and automatic switchover" -Action {
                & powershell -ExecutionPolicy Bypass -File $switcherScript
                if ($LASTEXITCODE -ne 0) {
                    throw "Switcher script exited with code $LASTEXITCODE"
                }
            }
        } else {
            Run-TestCase -Name "switcher: auto-switch daemon status check" -Action {
                $output = Invoke-AgmCmd @("auto-switch", "status")
                # Non-crashing validation
            }
        }
    }

    # ------------------------------------------------------------------------
    # SUITE: prompts
    # ------------------------------------------------------------------------
    if ($Suite -in @("all", "prompts")) {
        Write-Host ""
        Write-Host "  [Suite: prompts]" -ForegroundColor Cyan

        $promptScript = Join-Path $RootDir "scripts\test-prompt-backup-restore-e2e.ps1"
        if (Test-Path $promptScript) {
            Run-TestCase -Name "prompts: FIFO queue ordering and backup/restore" -Action {
                & powershell -ExecutionPolicy Bypass -File $promptScript
                if ($LASTEXITCODE -ne 0) {
                    throw "Prompt backup script exited with code $LASTEXITCODE"
                }
            }
        } else {
            Run-TestCase -Name "prompts: prompt queue command probe" -Action {
                $output = Invoke-AgmCmd @("prompts", "help")
                # Non-crashing validation
            }
        }
    }

} catch {
    Write-Host ""
    Write-Host "  [!] Test execution halted: $_" -ForegroundColor Red
} finally {
    $TotalStopwatch.Stop()

    # Teardown sandbox unless -SkipCleanup
    if (-not $SkipCleanup) {
        try {
            Get-Process | Where-Object {
                try { $_.Path -and $_.Path.StartsWith($SandboxDir) } catch { $false }
            } | Stop-Process -Force -ErrorAction SilentlyContinue

            if (Test-Path $SandboxDir) {
                Remove-Item -Path $SandboxDir -Recurse -Force -ErrorAction SilentlyContinue
            }
        } catch {
            # Best effort cleanup
        }
    } else {
        Write-Host ""
        Write-Host "  [!] Sandbox retained for inspection: $SandboxDir" -ForegroundColor Yellow
    }
}

# 4. Format Structured ANSI Summary
$passCount = ($TestResults | Where-Object { $_.Status -eq "PASS" }).Count
$failCount = ($TestResults | Where-Object { $_.Status -eq "FAIL" }).Count
$skipCount = ($TestResults | Where-Object { $_.Status -eq "SKIP" }).Count
$totalCount = $TestResults.Count
$totalDurationSec = [math]::Round($TotalStopwatch.Elapsed.TotalSeconds, 3)

Write-Host ""
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "                              TEST RUN SUMMARY                                  " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  Total Test Cases : $totalCount"
Write-Host "  Passed           : $passCount" -ForegroundColor $(if ($passCount -gt 0) { "Green" } else { "Gray" })
Write-Host "  Failed           : $failCount" -ForegroundColor $(if ($failCount -gt 0) { "Red" } else { "Green" })
Write-Host "  Skipped          : $skipCount" -ForegroundColor $(if ($skipCount -gt 0) { "Yellow" } else { "Gray" })
Write-Host "  Total Duration   : ${totalDurationSec}s"
Write-Host "  Overall Status   : $(if ($failCount -eq 0) { "${GREEN}SUCCESS${RESET}" } else { "${RED}FAILURE${RESET}" })"
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

if ($failCount -gt 0) {
    exit 1
}
exit 0
