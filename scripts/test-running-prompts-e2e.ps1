<#
.SYNOPSIS
    End-to-End Test Suite for Running Prompts Liveness and CLI Parity in Antigravity-Manager.
.DESCRIPTION
    Validates that active prompts in running Antigravity IDE instances are accurately
    reflected across CLI commands (agm prompts tree and agm tree) and match frontend status.
#>
param(
    [switch]$VerboseOutput,
    [switch]$AllowZeroRunning
)

$ErrorActionPreference = "Continue"
$PSNativeCommandUseErrorActionPreference = $false

# Enforce UTF-8 encoding for unicode tree glyphs and status badges
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$agm = Join-Path $RootDir "src-tauri\target\debug\agm.exe"

if (!(Test-Path $agm)) {
    $agm = Join-Path $RootDir "src-tauri\target\release\agm.exe"
}

if (!(Test-Path $agm)) {
    $cmd = Get-Command agm -ErrorAction SilentlyContinue
    if ($null -ne $cmd) {
        $agm = $cmd.Source
    }
}

if (!(Test-Path $agm)) {
    $progPath = Join-Path $env:LOCALAPPDATA "Programs\agm-alim\agm.exe"
    if (Test-Path $progPath) {
        $agm = $progPath
    }
}

if (!(Test-Path $agm)) {
    Write-Error "agm.exe binary not found at $agm! Please build it first via: cd src-tauri; cargo build"
    exit 1
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " AGM Running Prompts Liveness & CLI Parity E2E Test Suite  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "Using binary: $agm" -ForegroundColor Gray

function Parse-CliJson {
    param([object]$rawOutput)
    if ($null -eq $rawOutput) { return $null }
    if ($rawOutput -is [array]) {
        $text = $rawOutput -join "`n"
    } else {
        $text = [string]$rawOutput
    }
    $cleanText = $text.Trim()
    $firstBrace = $cleanText.IndexOf('{')
    $lastBrace = $cleanText.LastIndexOf('}')
    if ($firstBrace -ge 0 -and $lastBrace -gt $firstBrace) {
        $jsonStr = $cleanText.Substring($firstBrace, $lastBrace - $firstBrace + 1)
        try {
            return ($jsonStr | ConvertFrom-Json)
        } catch {
            return $null
        }
    }
    return $null
}

function Extract-Projects {
    param([object]$envelopeJson)
    if ($null -eq $envelopeJson) { return @() }
    if ($null -ne $envelopeJson.data) {
        if ($envelopeJson.data -is [array]) {
            return @($envelopeJson.data)
        }
        if ($null -ne $envelopeJson.data.projects) {
            return @($envelopeJson.data.projects)
        }
    }
    return @()
}

$testResults = @()
function Assert-Condition {
    param(
        [string]$TestName,
        [bool]$Condition,
        [string]$FailureMessage
    )
    if ($Condition) {
        Write-Host "  [PASS] $TestName" -ForegroundColor Green
        $script:testResults += [PSCustomObject]@{ Test = $TestName; Status = "PASS"; Details = "" }
    } else {
        Write-Host "  [FAIL] $TestName - $FailureMessage" -ForegroundColor Red
        $script:testResults += [PSCustomObject]@{ Test = $TestName; Status = "FAIL"; Details = $FailureMessage }
    }
}

# -------------------------------------------------------------------------
# Stage 0: Pre-flight Process & Environment Safety Check
# -------------------------------------------------------------------------
Write-Host "`n[Stage 0/6] Pre-flight Environment Safety Check" -ForegroundColor Yellow
$userAgmProcesses = Get-Process | Where-Object { $_.Name -match "agm-alim" -or $_.Name -eq "agm" }
$mainIdeProcesses = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.Name -match 'Antigravity' }
$isIdeRunning = ($null -ne $mainIdeProcesses -and @($mainIdeProcesses).Count -gt 0)
$protectedPids = @($userAgmProcesses.Id) + @($mainIdeProcesses.ProcessId) | Where-Object { $_ } | Select-Object -Unique
Write-Host "  Protected Main IDE & AGM processes: $($protectedPids.Count) PIDs" -ForegroundColor Gray
Write-Host "  Live Antigravity IDE processes detected: $(@($mainIdeProcesses).Count)" -ForegroundColor Gray
Assert-Condition "Stage 0: Binary exists and is executable" (Test-Path $agm) "Binary missing"

# -------------------------------------------------------------------------
# Stage 1: Ingest Live Instances via CLI
# -------------------------------------------------------------------------
Write-Host "`n[Stage 1/6] Ingest Live Instances" -ForegroundColor Yellow
$instListRaw = & $agm instances list --json 2>&1
$instListJson = Parse-CliJson $instListRaw
$hasInstData = ($null -ne $instListJson -and ($instListJson.status -eq "ok" -or $instListJson.status -eq "success" -or $instListJson.success -eq $true))
Assert-Condition "Stage 1: agm instances list returns ok" $hasInstData "Failed to query instances list"

# -------------------------------------------------------------------------
# Stage 2: Query agm prompts tree --json
# -------------------------------------------------------------------------
Write-Host "`n[Stage 2/6] Query agm prompts tree --json" -ForegroundColor Yellow
$promptsTreeRaw = & $agm prompts tree --json 2>&1
$promptsTreeJson = Parse-CliJson $promptsTreeRaw
$hasPromptsTree = ($null -ne $promptsTreeJson -and ($promptsTreeJson.status -eq "ok" -or $promptsTreeJson.status -eq "success" -or $promptsTreeJson.success -eq $true))
Assert-Condition "Stage 2: agm prompts tree --json returns valid envelope" $hasPromptsTree "Invalid envelope"

$projectsA = Extract-Projects $promptsTreeJson
Write-Host "  Discovered $($projectsA.Count) project nodes in prompts tree" -ForegroundColor Gray

# -------------------------------------------------------------------------
# Stage 3: Query agm tree --json and Assert Dual Parity
# -------------------------------------------------------------------------
Write-Host "`n[Stage 3/6] Query agm tree --json & Verify Parity" -ForegroundColor Yellow
$treeRaw = & $agm tree --json 2>&1
$treeJson = Parse-CliJson $treeRaw
$hasTree = ($null -ne $treeJson -and ($treeJson.status -eq "ok" -or $treeJson.status -eq "success" -or $treeJson.success -eq $true))
Assert-Condition "Stage 3: agm tree --json returns valid envelope" $hasTree "Invalid envelope"

$projectsB = Extract-Projects $treeJson
Write-Host "  Discovered $($projectsB.Count) project nodes in tree" -ForegroundColor Gray

$isCountParity = ($projectsA.Count -eq $projectsB.Count)
Assert-Condition "Stage 3: Dual CLI command parity (total projects match)" $isCountParity "Count mismatch: $($projectsA.Count) vs $($projectsB.Count)"

$convsCountA = 0
foreach ($p in $projectsA) {
    if ($null -ne $p.conversations) { $convsCountA += @($p.conversations).Count }
}
$convsCountB = 0
foreach ($p in $projectsB) {
    if ($null -ne $p.conversations) { $convsCountB += @($p.conversations).Count }
}
$isConvParity = [Math]::Abs($convsCountA - $convsCountB) -le 2
Assert-Condition "Stage 3: Dual CLI command parity (total conversations match within live variance)" $isConvParity "Conversation count mismatch: $convsCountA vs $convsCountB"

# -------------------------------------------------------------------------
# Stage 4: Running Project & Prompt Assertions
# -------------------------------------------------------------------------
Write-Host "`n[Stage 4/6] Running Project & Prompt Assertions" -ForegroundColor Yellow
$runningProjects = $projectsA | Where-Object { $_.is_running -eq $true -or $_.running_count -gt 0 }
$runningConvs = @()
foreach ($p in $projectsA) {
    if ($null -ne $p.conversations) {
        foreach ($c in $p.conversations) {
            if ($c.is_running -eq $true -or $c.status -eq "RUNNING") {
                $runningConvs += $c
            }
        }
    }
}

Write-Host "  Actively running projects: $($runningProjects.Count)" -ForegroundColor Gray
Write-Host "  Actively running conversations: $($runningConvs.Count)" -ForegroundColor Gray

$hasActiveTurns = ($runningProjects.Count -gt 0 -or $runningConvs.Count -gt 0)
if ($isIdeRunning -and !$AllowZeroRunning) {
    # If active turns are present, assert correlation; if completely idle, inform gracefully
    if ($hasActiveTurns) {
        Assert-Condition "Stage 4: Running projects or conversations detected" $true ""
    } else {
        Write-Host "  [INFO] No active in-flight prompt turns detected in running IDE instances." -ForegroundColor Yellow
        Write-Host "  [INFO] Skipping strict non-zero running assertion (IDE idle; use -AllowZeroRunning to explicitly bypass)" -ForegroundColor Gray
    }
} else {
    Write-Host "  [INFO] Skipping strict non-zero running assertion (IDE not running or AllowZeroRunning specified)" -ForegroundColor Gray
}

# When running projects exist, assert that running conversation nodes have proper previews
if ($runningProjects.Count -gt 0) {
    $hasValidRunningConv = ($runningConvs.Count -gt 0)
    Assert-Condition "Stage 4: Running conversations correlate with running projects" $hasValidRunningConv "Running project detected but no running conversations found"
}

# Ghost conversation suppression check: Ensure no 0-word Untitled conversation exists
$ghostConvs = @()
foreach ($p in $projectsA) {
    if ($null -ne $p.conversations) {
        foreach ($c in $p.conversations) {
            $isUntitled = ($c.title -match "^Untitled")
            $hasEmptyPrompt = [string]::IsNullOrWhiteSpace($c.prompt) -and [string]::IsNullOrWhiteSpace($c.prompt_preview_200w)
            $isZeroWords = ($null -eq $c.prompt_word_count -or $c.prompt_word_count -eq 0)
            if ($isUntitled -and $hasEmptyPrompt -and $isZeroWords) {
                $ghostConvs += $c
            }
        }
    }
}
Assert-Condition "Stage 4: Ghost conversations suppressed (zero empty untitled nodes)" ($ghostConvs.Count -eq 0) "Ghost conversations found: $($ghostConvs.Count)"

# -------------------------------------------------------------------------
# Stage 5: Text Output Formatting & Green Pulse Status
# -------------------------------------------------------------------------
Write-Host "`n[Stage 5/6] Text Formatting & Status Badges" -ForegroundColor Yellow
$promptsTextOutput = (& $agm prompts tree 2>&1) -join "`n"
$hasPromptsBanner = ($promptsTextOutput -match "TREE" -and $promptsTextOutput -match "PROJECT")
Assert-Condition "Stage 5: Header formatting matches expected AGM tree banner" $hasPromptsBanner "Missing tree banner in agm prompts tree"

$treeTextOutput = (& $agm tree 2>&1) -join "`n"
$hasTreeBanner = ($treeTextOutput -match "TREE" -and $treeTextOutput -match "PROJECT")
Assert-Condition "Stage 5: Header formatting matches expected tree banner for agm tree" $hasTreeBanner "Missing tree banner in agm tree"

if ($runningProjects.Count -gt 0) {
    Assert-Condition "Stage 5: Text tree contains [N RUNNING] badge" ($promptsTextOutput -match '\[\d+ RUNNING\]') "Missing [RUNNING] badge in text output"
}
if ($runningConvs.Count -gt 0) {
    Assert-Condition "Stage 5: Text tree contains RUNNING conversation status" ($promptsTextOutput -match 'RUNNING') "Missing RUNNING badge in conversation rows"
}

# -------------------------------------------------------------------------
# Stage 6: Structured Report Output and Teardown Summary
# -------------------------------------------------------------------------
Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host " [Stage 6/6] E2E Verification Summary                      " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
$failedCount = ($script:testResults | Where-Object { $_.Status -eq "FAIL" }).Count
$passedCount = ($script:testResults | Where-Object { $_.Status -eq "PASS" }).Count
Write-Host "Total Passed: $passedCount" -ForegroundColor Green
Write-Host "Total Failed: $failedCount" -ForegroundColor $(if ($failedCount -eq 0) { "Green" } else { "Red" })

$script:testResults | Format-Table -AutoSize -Property Test, Status, Details

if ($failedCount -eq 0) {
    Write-Host "`nALL E2E RUNNING PROMPT TESTS PASSED SUCCESSFULLY." -ForegroundColor Green
    exit 0
} else {
    Write-Host "`nONE OR MORE TESTS FAILED." -ForegroundColor Red
    exit 1
}
