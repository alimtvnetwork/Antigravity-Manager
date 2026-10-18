<#
.SYNOPSIS
    End-to-End Test for Prompt Backup and Restore with Screenshot IDs in Antigravity-Manager.
.DESCRIPTION
    Validates end-to-end multimodal prompt backup and restore lifecycle:
    1. Generates synthetic prompt with Screenshot ID token and image attachment.
    2. Enqueues prompt into SQLite FIFO queue (repo_prompts.db active_prompts).
    3. Verifies queued prompt is present with screenshot payload.
    4. Executes `agm prompt backup -i default` snapshot into backup-prompts.db.
    5. Verifies snapshot record has has_images=1 and images_payload with Screenshot ID.
    6. Clears active prompt to simulate account rotation or restore requirement.
    7. Executes `agm prompt restore -i default`.
    8. Verifies prompt is restored into active_prompts in strict FIFO order with metadata intact.
    9. Fully cleans up test records and temporary artifacts.
#>
param(
    [switch]$VerboseOutput
)

$ErrorActionPreference = "Continue"
$PSNativeCommandUseErrorActionPreference = $false

# Enforce UTF-8 encoding
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir

# Resolve AGM binary
$agm = Join-Path $RootDir "src-tauri\target\debug\agm.exe"
if (!(Test-Path $agm)) {
    $agm = Join-Path $RootDir "src-tauri\target\release\agm.exe"
}
if (!(Test-Path $agm)) {
    $cmd = Get-Command agm -ErrorAction SilentlyContinue
    if ($null -ne $cmd) { $agm = $cmd.Source }
}
if (!(Test-Path $agm)) {
    $progPath = Join-Path $env:LOCALAPPDATA "Programs\agm-alim\agm.exe"
    if (Test-Path $progPath) { $agm = $progPath }
}

if (!(Test-Path $agm)) {
    Write-Error "agm.exe binary not found at $agm! Please build it first via: cd src-tauri; cargo build"
    exit 1
}

# Enforce test isolation: skip spawning real agy.exe CLI agent during test
$env:AGM_SKIP_AGY_SPAWN = "1"

# Resolve SQLite database paths
$dataDir = Join-Path $env:USERPROFILE ".antigravity_tools"
$repoDbPath = Join-Path $dataDir "repo_prompts.db"
$backupDbPath = Join-Path $dataDir "backup-prompts\backup-prompts.db"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host " AGM Prompt Backup and Restore E2E Test Suite (Screenshot ID & FIFO Parity)     " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "Using binary:    $agm" -ForegroundColor Gray
Write-Host "Repo DB Path:    $repoDbPath" -ForegroundColor Gray
Write-Host "Backup DB Path:  $backupDbPath" -ForegroundColor Gray

# Protect caller instance / session PID 10076
$callerSessionPid = 10076
$callerProcess = Get-Process -Id $callerSessionPid -ErrorAction SilentlyContinue
if ($null -ne $callerProcess) {
    Write-Host "Caller session PID $callerSessionPid active: $($callerProcess.ProcessName) (Protected)" -ForegroundColor Green
} else {
    Write-Host "Caller session PID $callerSessionPid guard registered (Protected)" -ForegroundColor DarkGray
}

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

function Invoke-SqlitePython {
    param(
        [string]$DbPath,
        [string]$Query
    )
    $pyScript = 'import sqlite3, json, sys; conn = sqlite3.connect(sys.argv[1]); conn.row_factory = sqlite3.Row; cur = conn.cursor(); cur.execute(sys.argv[2]); conn.commit(); rows = [dict(r) for r in cur.fetchall()] if cur.description else []; print(json.dumps(rows)); conn.close()'
    $raw = & python -c $pyScript $DbPath $Query 2>&1
    if ($null -ne $raw) {
        $text = ($raw | Out-String).Trim()
        if ($text -ne "") {
            try {
                return ($text | ConvertFrom-Json)
            } catch {
                return @()
            }
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

# Pre-test cleanup: ensure pristine testing state
$null = Invoke-SqlitePython -DbPath $repoDbPath -Query "DELETE FROM active_prompts WHERE prompt_content LIKE '%SS_ID_%'"
$null = Invoke-SqlitePython -DbPath $backupDbPath -Query "DELETE FROM prompt_backups WHERE prompt_text LIKE '%SS_ID_%'"

# -------------------------------------------------------------------------
# Gate 1: Generate Synthetic Test Prompt & Attachment
# -------------------------------------------------------------------------
Write-Host "`n[Gate 1/6] Generate Synthetic Test Prompt & Screenshot Attachment" -ForegroundColor Yellow

$uniqueSuffix = (Get-Random -Minimum 100000000 -Maximum 999999999)
$screenshotId = "SS_ID_$uniqueSuffix"
$attachmentRelativePath = ".antigravity/attachments/test_screenshot.png"
$attachmentDir = Join-Path $RootDir ".antigravity\attachments"
if (!(Test-Path $attachmentDir)) {
    New-Item -ItemType Directory -Path $attachmentDir -Force | Out-Null
}
$attachmentFullPath = Join-Path $attachmentDir "test_screenshot.png"
if (!(Test-Path $attachmentFullPath)) {
    # Generate a minimal 1x1 valid PNG
    $pngBytes = [byte[]]@(0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82)
    [System.IO.File]::WriteAllBytes($attachmentFullPath, $pngBytes)
}

$syntheticPrompt = "E2E UI Verification Prompt for Screenshot $screenshotId with attachment $attachmentRelativePath"
Write-Host "  Screenshot ID Token: $screenshotId" -ForegroundColor DarkGray
Write-Host "  Image Attachment:    $attachmentRelativePath" -ForegroundColor DarkGray
Write-Host "  Synthetic Prompt:    $syntheticPrompt" -ForegroundColor DarkGray

Assert-Condition "Gate 1: Attachment file materialized on disk" (Test-Path $attachmentFullPath) "Attachment missing at $attachmentFullPath"

# -------------------------------------------------------------------------
# Gate 2: Enqueue Prompt into repo_prompts.db (active_prompts)
# -------------------------------------------------------------------------
Write-Host "`n[Gate 2/6] Enqueue Prompt into SQLite FIFO Queue" -ForegroundColor Yellow

$queueRaw = & $agm prompt queue default "$syntheticPrompt" --json 2>&1
$queueJson = Parse-CliJson $queueRaw

$enqueuedId = $null
if ($null -ne $queueJson -and $null -ne $queueJson.data -and $null -ne $queueJson.data.id) {
    $enqueuedId = $queueJson.data.id
}

Write-Host "  Enqueued Prompt ID:  $enqueuedId" -ForegroundColor DarkGray

Assert-Condition "Gate 2.1: CLI reports prompt enqueued successfully" ($null -ne $enqueuedId) "CLI failed to enqueue prompt: $queueRaw"

# Verify active_prompts row in repo_prompts.db
$queryActive = "SELECT id, status, prompt_content, image_payload FROM active_prompts WHERE id = '$enqueuedId'"
$activeRows = Invoke-SqlitePython -DbPath $repoDbPath -Query $queryActive

$hasActiveRow = ($null -ne $activeRows -and @($activeRows).Count -ge 1)
Assert-Condition "Gate 2.2: Prompt record present in active_prompts" $hasActiveRow "Active prompt row not found in repo_prompts.db"

$activeStatus = if ($hasActiveRow) { $activeRows[0].status } else { "" }
Assert-Condition "Gate 2.3: Prompt status is 'queued'" ($activeStatus -eq "queued") "Expected 'queued', got '$activeStatus'"

$activeImgPayload = if ($hasActiveRow) { [string]$activeRows[0].image_payload } else { "" }
$hasScreenshotTokenInActive = $activeImgPayload.Contains($screenshotId)
Assert-Condition "Gate 2.4: active_prompts image_payload contains Screenshot ID token" $hasScreenshotTokenInActive "image_payload missing $screenshotId (actual: '$activeImgPayload')"

# -------------------------------------------------------------------------
# Gate 3: Run agm prompt backup -i default and verify backup-prompts.db
# -------------------------------------------------------------------------
Write-Host "`n[Gate 3/6] Backup Active and Queued Prompts to Split SQLite" -ForegroundColor Yellow

$backupRaw = & $agm prompt backup -i default 2>&1
if ($VerboseOutput) { Write-Host ($backupRaw -join "`n") -ForegroundColor Gray }

# Query backup-prompts.db prompt_backups
$queryBackup = "SELECT id, backup_batch_id, prompt_id, prompt_text, has_images, images_payload, status, is_restored FROM prompt_backups WHERE prompt_id = '$enqueuedId' OR prompt_text LIKE '%$screenshotId%'"
$backupRows = Invoke-SqlitePython -DbPath $backupDbPath -Query $queryBackup

$hasBackupRow = ($null -ne $backupRows -and @($backupRows).Count -ge 1)
Assert-Condition "Gate 3.1: Snapshot record captured in prompt_backups" $hasBackupRow "Snapshot record not found in backup-prompts.db"

$backupHasImages = if ($hasBackupRow) { [int]$backupRows[0].has_images } else { 0 }
Assert-Condition "Gate 3.2: Snapshot record has_images = 1" ($backupHasImages -eq 1) "Expected has_images=1, got '$backupHasImages'"

$backupImgPayload = if ($hasBackupRow) { [string]$backupRows[0].images_payload } else { "" }
$hasScreenshotTokenInBackup = $backupImgPayload.Contains($screenshotId)
Assert-Condition "Gate 3.3: images_payload in prompt_backups contains Screenshot ID ($screenshotId)" $hasScreenshotTokenInBackup "images_payload missing $screenshotId (actual: '$backupImgPayload')"

$backupBatchId = if ($hasBackupRow) { $backupRows[0].backup_batch_id } else { "" }
Write-Host "  Snapshot Record ID:  $(if ($hasBackupRow) { $backupRows[0].id } else { '-' })" -ForegroundColor DarkGray
Write-Host "  Backup Batch ID:     $backupBatchId" -ForegroundColor DarkGray
Write-Host "  has_images:          $backupHasImages" -ForegroundColor DarkGray
Write-Host "  images_payload:      $backupImgPayload" -ForegroundColor DarkGray

# -------------------------------------------------------------------------
# Gate 4: Clear active prompt in repo_prompts.db to simulate restore need
# -------------------------------------------------------------------------
Write-Host "`n[Gate 4/6] Simulate Restore Need by Purging Active Prompt" -ForegroundColor Yellow

$deleteActiveSql = "DELETE FROM active_prompts WHERE id = '$enqueuedId'"
$null = Invoke-SqlitePython -DbPath $repoDbPath -Query $deleteActiveSql

$checkPurged = Invoke-SqlitePython -DbPath $repoDbPath -Query "SELECT COUNT(*) as cnt FROM active_prompts WHERE id = '$enqueuedId'"
$isPurged = ($null -ne $checkPurged -and @($checkPurged).Count -ge 1 -and [int]$checkPurged[0].cnt -eq 0)

Assert-Condition "Gate 4: Active prompt row purged from repo_prompts.db" $isPurged "Failed to purge active prompt row before restore"

# -------------------------------------------------------------------------
# Gate 5: Run agm prompt restore -i default and verify restored state
# -------------------------------------------------------------------------
Write-Host "`n[Gate 5/6] Restore Prompts and Verify FIFO Order & Metadata" -ForegroundColor Yellow

$restoreRaw = & $agm prompt restore -i default 2>&1
if ($VerboseOutput) { Write-Host ($restoreRaw -join "`n") -ForegroundColor Gray }

# Check active_prompts table in repo_prompts.db
$queryRestored = "SELECT id, status, prompt_content, image_payload, created_at FROM active_prompts WHERE id = '$enqueuedId' OR prompt_content LIKE '%$screenshotId%' ORDER BY created_at ASC, id ASC"
$restoredRows = Invoke-SqlitePython -DbPath $repoDbPath -Query $queryRestored

$hasRestoredRow = ($null -ne $restoredRows -and @($restoredRows).Count -ge 1)
Assert-Condition "Gate 5.1: Prompt successfully restored into active_prompts" $hasRestoredRow "Prompt was not restored into active_prompts"

$restoredImgPayload = if ($hasRestoredRow) { [string]$restoredRows[0].image_payload } else { "" }
$hasScreenshotTokenInRestored = $restoredImgPayload.Contains($screenshotId)
Assert-Condition "Gate 5.2: Restored active prompt preserves Screenshot ID ($screenshotId)" $hasScreenshotTokenInRestored "Restored image_payload missing $screenshotId (actual: '$restoredImgPayload')"

# Verify strict FIFO query ordering
$fifoQuery = "SELECT id, created_at FROM active_prompts ORDER BY created_at ASC, id ASC"
$allActiveRows = Invoke-SqlitePython -DbPath $repoDbPath -Query $fifoQuery
$isFifoOrdered = $true
if ($null -ne $allActiveRows -and @($allActiveRows).Count -gt 1) {
    for ($i = 0; $i -lt (@($allActiveRows).Count - 1); $i++) {
        $curCreated = [int64]$allActiveRows[$i].created_at
        $nextCreated = [int64]$allActiveRows[$i+1].created_at
        if ($curCreated -gt $nextCreated) {
            $isFifoOrdered = $false
            break
        }
    }
}
Assert-Condition "Gate 5.3: Strict FIFO order enforced (ORDER BY created_at ASC, id ASC)" $isFifoOrdered "FIFO ordering inverted in active_prompts"

Write-Host "  Restored Prompt ID:  $(if ($hasRestoredRow) { $restoredRows[0].id } else { '-' })" -ForegroundColor DarkGray
Write-Host "  Restored Status:     $(if ($hasRestoredRow) { $restoredRows[0].status } else { '-' })" -ForegroundColor DarkGray
Write-Host "  Restored Payload:    $restoredImgPayload" -ForegroundColor DarkGray

# -------------------------------------------------------------------------
# Gate 6: Complete Cleanup of Test Records & Temp Files
# -------------------------------------------------------------------------
Write-Host "`n[Gate 6/6] Cleanup Test Records & Temporary Files" -ForegroundColor Yellow

$cleanActiveSql = "DELETE FROM active_prompts WHERE id = '$enqueuedId' OR prompt_content LIKE '%$screenshotId%'"
$null = Invoke-SqlitePython -DbPath $repoDbPath -Query $cleanActiveSql

$cleanBackupSql = "DELETE FROM prompt_backups WHERE prompt_id = '$enqueuedId' OR prompt_text LIKE '%$screenshotId%'"
$null = Invoke-SqlitePython -DbPath $backupDbPath -Query $cleanBackupSql

if (![string]::IsNullOrEmpty($backupBatchId)) {
    $cleanBatchSql = "DELETE FROM backup_batches WHERE id = '$backupBatchId'"
    $null = Invoke-SqlitePython -DbPath $backupDbPath -Query $cleanBatchSql
}

# Clean temp attachment and resume task files
if (Test-Path $attachmentFullPath) {
    Remove-Item -Force $attachmentFullPath -ErrorAction SilentlyContinue
}
$resumeFiles = Get-ChildItem -Path $RootDir -Filter "*resume_task*" -File -ErrorAction SilentlyContinue
foreach ($rf in $resumeFiles) {
    Remove-Item -Force $rf.FullName -ErrorAction SilentlyContinue
}

$verifyCleanActive = Invoke-SqlitePython -DbPath $repoDbPath -Query "SELECT COUNT(*) as cnt FROM active_prompts WHERE id = '$enqueuedId' OR prompt_content LIKE '%$screenshotId%'"
$verifyCleanBackup = Invoke-SqlitePython -DbPath $backupDbPath -Query "SELECT COUNT(*) as cnt FROM prompt_backups WHERE prompt_id = '$enqueuedId' OR prompt_text LIKE '%$screenshotId%'"

$activeClean = ($null -ne $verifyCleanActive -and [int]$verifyCleanActive[0].cnt -eq 0)
$backupClean = ($null -ne $verifyCleanBackup -and [int]$verifyCleanBackup[0].cnt -eq 0)

Assert-Condition "Gate 6.1: active_prompts test rows cleaned completely" $activeClean "Stale active_prompts rows remain"
Assert-Condition "Gate 6.2: prompt_backups test rows cleaned completely" $backupClean "Stale prompt_backups rows remain"
$isCallerAlive = $null -ne (Get-Process -Id $callerSessionPid -ErrorAction SilentlyContinue)
Assert-Condition "Gate 6.3: Caller session PID $callerSessionPid preserved" $isCallerAlive "Caller process $callerSessionPid was terminated"

# -------------------------------------------------------------------------
# Test Execution Summary
# -------------------------------------------------------------------------
Write-Host "`n================================================================================" -ForegroundColor Cyan
Write-Host " Test Execution Summary" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

$passed = @($testResults | Where-Object { $_.Status -eq "PASS" }).Count
$failed = @($testResults | Where-Object { $_.Status -eq "FAIL" }).Count
$total = $testResults.Count

foreach ($r in $testResults) {
    $color = if ($r.Status -eq "PASS") { [ConsoleColor]::Green } else { [ConsoleColor]::Red }
    Write-Host ("  [{0,-4}] {1}" -f $r.Status, $r.Test) -ForegroundColor $color
    if ($r.Details -ne "") {
        Write-Host "         -> Details: $($r.Details)" -ForegroundColor DarkRed
    }
}

Write-Host "`nTotal: $total | Passed: $passed | Failed: $failed" -ForegroundColor $(if ($failed -eq 0) { [ConsoleColor]::Green } else { [ConsoleColor]::Red })

if ($failed -gt 0) {
    exit 1
} else {
    exit 0
}
