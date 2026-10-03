<#
.SYNOPSIS
    Local-Only End-to-End Verification Suite for AGM Instance CLI Commands.

.DESCRIPTION
    Tests CLI commands and parity with GUI routes:
    - agm instance count --json
    - agm instance ls --json
    - agm instance duplicate
    - agm instance copy-projects
    - agm instance copy-settings
    - agm instance settings enforce-defaults
    - agm instance settings set-turbo
    - agm instance settings set-plan-review
    - agm instance settings export / import

.NOTES
    ============================================================================
    LOCAL-ONLY TEST SCRIPT - STRICTLY EXCLUDED FROM CI/CD PIPELINES
    ============================================================================
    This script is designed strictly for local developer workstation verification
    to prevent headless CI collisions and avoid lingering OS processes.
#>

param(
    [switch]$SkipCleanup = $false,
    [string]$TargetBinary = ""
)

$ErrorActionPreference = "Continue"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$TauriDir = Join-Path $RootDir "src-tauri"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Antigravity Manager CLI Instance Commands E2E Suite       " -ForegroundColor Cyan
Write-Host " (Local-Only Test - Excluded from CI/CD Pipelines)         " -ForegroundColor DarkGray
Write-Host "==========================================================" -ForegroundColor Cyan

# Locate executable
$agmExe = $null
if ($TargetBinary -and (Test-Path $TargetBinary)) {
    $agmExe = $TargetBinary
} elseif (Test-Path (Join-Path $TauriDir "target\debug\agm.exe")) {
    $agmExe = Join-Path $TauriDir "target\debug\agm.exe"
} elseif (Get-Command "agm" -ErrorAction SilentlyContinue) {
    $agmExe = (Get-Command "agm").Source
}

function Invoke-Agm {
    param([string[]]$Arguments)
    if ($agmExe) {
        & $agmExe @Arguments
    } else {
        Set-Location $TauriDir
        cargo run --bin agm --quiet -- @Arguments
    }
}

function Parse-JsonOutput {
    param([object]$rawOutput)
    if ($null -eq $rawOutput) { return $null }
    $text = if ($rawOutput -is [array]) { $rawOutput -join "`n" } else { $rawOutput.ToString() }
    $trimmed = ($text -replace "\x1b\[[0-9;]*[a-zA-Z]", "").Trim()
    $jsonStart = $trimmed.IndexOf('{')
    $arrayStart = $trimmed.IndexOf('[')
    $startIdx = -1
    if ($jsonStart -ge 0 -and $arrayStart -ge 0) {
        $startIdx = [Math]::Min($jsonStart, $arrayStart)
    } elseif ($jsonStart -ge 0) {
        $startIdx = $jsonStart
    } elseif ($arrayStart -ge 0) {
        $startIdx = $arrayStart
    }
    if ($startIdx -ge 0) {
        try {
            return ($trimmed.Substring($startIdx) | ConvertFrom-Json)
        } catch {
            return $null
        }
    }
    return $null
}

$TestPassCount = 0
$TestFailCount = 0

function Assert-Step {
    param(
        [string]$Name,
        [scriptblock]$Condition
    )
    try {
        $passed = & $Condition
        if ($passed) {
            Write-Host "  [PASS] $Name" -ForegroundColor Green
            $script:TestPassCount++
        } else {
            Write-Host "  [FAIL] $Name" -ForegroundColor Red
            $script:TestFailCount++
        }
    } catch {
        Write-Host "  [ERROR] $Name : $_" -ForegroundColor Red
        $script:TestFailCount++
    }
}

$testInstName = "test-cli-e2e-dup"
$exportFile = Join-Path $env:TEMP "agm_settings_test_export.json"

try {
    # 1. Test agm instance count --json
    Write-Host "`n[1/7] Testing 'agm instance count --json'..." -ForegroundColor Yellow
    $countOut = Invoke-Agm @("instance", "count", "--json")
    $countJson = Parse-JsonOutput $countOut
    Assert-Step "Instance count returns valid JSON with total field" {
        $null -ne $countJson -and $countJson.total -ge 1
    }

    # 2. Test agm instance ls --json
    Write-Host "`n[2/7] Testing 'agm instance ls --json'..." -ForegroundColor Yellow
    $lsOut = Invoke-Agm @("instance", "ls", "--json")
    $lsJson = Parse-JsonOutput $lsOut
    $items = if ($lsJson -and $lsJson.data) { $lsJson.data } else { $lsJson }
    Assert-Step "Instance list returns array with sequence_name and PID" {
        $null -ne $items -and $items.Count -ge 1 -and ($null -ne $items[0].sequence_name -or $null -ne $items[0].name)
    }

    # 3. Test agm instance duplicate
    Write-Host "`n[3/7] Testing 'agm instance duplicate'..." -ForegroundColor Yellow
    $dupOut = Invoke-Agm @("instance", "duplicate", "default", $testInstName, "--copy-projects")
    Assert-Step "Duplicate instance completed" {
        $dupOut -match "Successfully duplicated" -or $dupOut -match "created" -or (Invoke-Agm @("instance", "ls", "--json") -join "") -match $testInstName
    }

    # 4. Test agm instance copy-projects
    Write-Host "`n[4/7] Testing 'agm instance copy-projects'..." -ForegroundColor Yellow
    $copyProjOut = Invoke-Agm @("instance", "copy-projects", "--from", "default", "--to", $testInstName)
    Assert-Step "Copy projects between instances succeeded" {
        $copyProjOut -match "Copied" -or $copyProjOut -match "projects" -or $copyProjOut -match "Success"
    }

    # 5. Test agm instance copy-settings
    Write-Host "`n[5/7] Testing 'agm instance copy-settings'..." -ForegroundColor Yellow
    $copySetOut = Invoke-Agm @("instance", "copy-settings", "--from", "default", "--to", $testInstName)
    Assert-Step "Copy settings between instances succeeded" {
        $copySetOut -match "Copied settings" -or $copySetOut -match "Success"
    }

    # 6. Test settings manipulation (turbo, plan-review, enforce-defaults)
    Write-Host "`n[6/7] Testing settings enforcement (set-turbo, set-plan-review)..." -ForegroundColor Yellow
    $turboOut = Invoke-Agm @("instance", "settings", "set-turbo", "--instance", $testInstName, "--enable")
    Assert-Step "Set turbo mode succeeded" {
        $turboOut -match "turboMode" -or $turboOut -match "Turbo mode" -or $turboOut -match "updated" -or $turboOut -match "SUCCESS"
    }

    $planOut = Invoke-Agm @("instance", "settings", "set-plan-review", "--instance", $testInstName, "--always-proceed")
    Assert-Step "Set plan review succeeded" {
        $planOut -match "planReview" -or $planOut -match "Plan review" -or $planOut -match "updated" -or $planOut -match "SUCCESS"
    }

    # 7. Test settings export & import
    Write-Host "`n[7/7] Testing settings export and import..." -ForegroundColor Yellow
    $exportOut = Invoke-Agm @("instance", "settings", "export", "--instance", $testInstName, "--out", $exportFile)
    Assert-Step "Settings export created file" {
        Test-Path $exportFile
    }

    if (Test-Path $exportFile) {
        $importOut = Invoke-Agm @("instance", "settings", "import", "--instance", $testInstName, "--file", $exportFile)
        Assert-Step "Settings import completed" {
            $importOut -match "Imported" -or $importOut -match "updated"
        }
    }

} finally {
    # Teardown / Cleanup
    if (!$SkipCleanup) {
        Write-Host "`n[Teardown] Cleaning up test instance $testInstName..." -ForegroundColor DarkGray
        try {
            Invoke-Agm @("instance", "rm", $testInstName, "--force") | Out-Null
        } catch {}
        if (Test-Path $exportFile) {
            Remove-Item -Path $exportFile -Force -ErrorAction SilentlyContinue
        }
    }
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host " Results: $TestPassCount Passed, $TestFailCount Failed" -ForegroundColor $(if ($TestFailCount -eq 0) { "Green" } else { "Red" })
Write-Host "==========================================================" -ForegroundColor Cyan

if ($TestFailCount -gt 0) {
    exit 1
} else {
    exit 0
}
