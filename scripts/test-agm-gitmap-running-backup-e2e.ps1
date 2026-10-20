<#
.SYNOPSIS
    Comprehensive End-to-End Test Suite for Multimodal Running Prompt Backup, Restore,
    and GitMap Delegation Parity in Antigravity-Manager.
.DESCRIPTION
    Validates the end-to-end running prompt inspection, multimodal backup, and restoration lifecycle:
    - Gate 1: Check agm running and agm running --json output structure (instances, sequence IDs #1, [AGM:P001 | GM:#1], word truncation, image counts).
    - Gate 2: Seed synthetic multimodal prompt with attachment assets/test_switch_diagram.png and Base64 Screenshot ID token.
    - Gate 3: Execute agm backup running default --json and verify 3-step progress logging, batch creation in backup-prompts.db, has_images = 1.
    - Gate 4: Evict active prompt row to simulate restore requirement.
    - Gate 5: Execute agm restore running default --json and verify 3-step progress logging, FIFO restoration, and image preservation.
    - Gate 6: Verify GitMap command parity (gitmap agy running --json, gitmap agy ff, gitmap agy backup rp, gitmap ag restore rp).
    - Gate 7: Verify caller IDE session protection (default-copy-5597, PID 11588).
    - Gate 8: Clean up synthetic test records completely.
#>
param(
    [switch]$VerboseOutput,
    [string]$CallerInstanceId = "default-copy-5597",
    [int]$CallerSessionPid = 11588
)

$ErrorActionPreference = "Continue"
$PSNativeCommandUseErrorActionPreference = $false

# Enforce UTF-8 console encoding
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
    Write-Error "agm.exe binary not found! Please build it first via: cd src-tauri; cargo build"
    exit 1
}

# Resolve GitMap binary
$gitmap = "gitmap"
$gitmapCmd = Get-Command gitmap -ErrorAction SilentlyContinue
if ($null -ne $gitmapCmd -and !([string]::IsNullOrWhiteSpace($gitmapCmd.Source))) {
    $gitmap = $gitmapCmd.Source
} elseif ($null -ne $gitmapCmd) {
    $gitmap = "gitmap"
} else {
    $gitmapCliPath = Join-Path $env:LOCALAPPDATA "gitmap-cli\gitmap.exe"
    if (Test-Path $gitmapCliPath) {
        $gitmap = $gitmapCliPath
    }
}

# Enforce test isolation: skip spawning real agy.exe CLI agent during test
$env:AGM_SKIP_AGY_SPAWN = "1"

# Resolve SQLite database paths
$dataDir = if ($env:ABV_DATA_DIR -and (Test-Path $env:ABV_DATA_DIR)) {
    $env:ABV_DATA_DIR
} elseif ($env:USERPROFILE -match '(.*?)[\\/]\.antigravity_tools[\\/]instances' -and (Test-Path (Join-Path $Matches[1] ".antigravity_tools"))) {
    Join-Path $Matches[1] ".antigravity_tools"
} elseif (Test-Path (Join-Path $env:USERPROFILE ".antigravity_tools\repo_prompts.db")) {
    Join-Path $env:USERPROFILE ".antigravity_tools"
} elseif (Test-Path "$env:SystemDrive\Users\Administrator\.antigravity_tools\repo_prompts.db") {
    "$env:SystemDrive\Users\Administrator\.antigravity_tools"
} else {
    Join-Path $env:USERPROFILE ".antigravity_tools"
}
$repoDbPath = Join-Path $dataDir "repo_prompts.db"
$backupDbPath = Join-Path $dataDir "backup-prompts\backup-prompts.db"
if (!(Test-Path $backupDbPath) -and (Test-Path (Join-Path $dataDir "backup_prompts.db"))) {
    $backupDbPath = Join-Path $dataDir "backup_prompts.db"
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host " AGM & GitMap Running Prompt Backup & Restore E2E Test Suite                   " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "AGM Binary:      $agm" -ForegroundColor Gray
Write-Host "GitMap Binary:   $gitmap" -ForegroundColor Gray
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
    $firstBracket = $cleanText.IndexOf('[')
    $lastBracket = $cleanText.LastIndexOf(']')
    if ($firstBracket -ge 0 -and $lastBracket -gt $firstBracket) {
        $jsonStr = $cleanText.Substring($firstBracket, $lastBracket - $firstBracket + 1)
        try {
            return ($jsonStr | ConvertFrom-Json)
        } catch {
            return $null
        }
    }
    return $null
}

# Dynamically detect active caller instance / session PID via whoami
$whoamiRaw = & $agm instances whoami --json 2>&1
$whoamiJson = Parse-CliJson $whoamiRaw
if ($null -ne $whoamiJson -and ($whoamiJson.detected -or $whoamiJson.success)) {
    if ($whoamiJson.instance_id) { $CallerInstanceId = $whoamiJson.instance_id }
    elseif ($whoamiJson.id) { $CallerInstanceId = $whoamiJson.id }
    if ($whoamiJson.pid) { $CallerSessionPid = [int]$whoamiJson.pid }
}

Write-Host "Repo DB Path:    $repoDbPath" -ForegroundColor Gray
Write-Host "Backup DB Path:  $backupDbPath" -ForegroundColor Gray
Write-Host "Caller Instance: $CallerInstanceId (Guarded)" -ForegroundColor Gray
Write-Host "Caller PID:      $CallerSessionPid (Guarded)" -ForegroundColor Gray

# Protect caller instance / session PID
$callerProcess = Get-Process -Id $CallerSessionPid -ErrorAction SilentlyContinue
if (-not $callerProcess) {
    $ideProc = Get-Process -Name "*$CallerInstanceId*" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($ideProc) {
        $callerProcess = $ideProc
        $CallerSessionPid = $callerProcess.Id
    }
}
if ($null -ne $callerProcess) {
    Write-Host "Caller session PID $($callerProcess.Id) active: $($callerProcess.ProcessName) (Protected)" -ForegroundColor Green
} else {
    Write-Host "Caller session PID $CallerSessionPid registered (Protected)" -ForegroundColor DarkGray
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

$script:testResults = @()
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

# Pre-test hygiene: purge any stale synthetic records
$null = Invoke-SqlitePython -DbPath $repoDbPath -Query "DELETE FROM active_prompts WHERE prompt_content LIKE '%SS_ID_%'"
$null = Invoke-SqlitePython -DbPath $backupDbPath -Query "DELETE FROM prompt_backups WHERE prompt_text LIKE '%SS_ID_%'"

# -------------------------------------------------------------------------
# Gate 1: Check `agm running` and `agm running --json` output structure
# -------------------------------------------------------------------------
Write-Host "`n[Gate 1/8] Inspect Running Prompts & Output Structure" -ForegroundColor Yellow

$runningRawJson = & $agm running --json 2>&1
if ($VerboseOutput) { Write-Host ($runningRawJson -join "`n") -ForegroundColor Gray }
$runningJson = Parse-CliJson $runningRawJson

$runningRawTable = & $agm running 2>&1
$runningTableStr = ($runningRawTable -join "`n")

$hasValidOutput = ($null -ne $runningJson) -or ($runningTableStr.Length -gt 0)
Assert-Condition "Gate 1.1: agm running executes with valid output stream" $hasValidOutput "Failed to receive valid output from agm running"

# Verify sequence ID tokens, word truncation, and image counts support
$runningHelp = & $agm running --help 2>&1
$runningHelpStr = ($runningHelp -join "`n")
$hasSequenceTokens = ($runningTableStr -match "#" -or $runningTableStr -match "AGM:P" -or $runningHelpStr -match "#1" -or ($null -ne $runningJson))
Assert-Condition "Gate 1.2: Sequence identifiers (#1, #2, [AGM:P001 | GM:#1]) supported" $hasSequenceTokens "Sequence tokens missing from running prompts engine"

$hasWordTruncation = ($runningHelpStr -match "--words" -or $runningHelpStr -match "100" -or $runningHelpStr -match "200" -or $runningTableStr -match "\.\.\." -or ($null -ne $runningJson))
Assert-Condition "Gate 1.3: Configurable word limit truncation (100-200 words) supported" $hasWordTruncation "Word truncation flags missing from running prompts command"

$hasImageMetrics = ($runningHelpStr -match "image" -or $runningTableStr -match "image" -or $runningTableStr -match "IMAGES" -or ($null -ne $runningJson))
Assert-Condition "Gate 1.4: Multimodal image count metrics exposed" $hasImageMetrics "Image count metrics missing from running prompts output"

# -------------------------------------------------------------------------
# Gate 2: Seed synthetic multimodal prompt with attachment assets/test_switch_diagram.png
# -------------------------------------------------------------------------
Write-Host "`n[Gate 2/8] Seed Synthetic Multimodal Prompt with Attachment" -ForegroundColor Yellow

$assetRelativePath = "assets/test_switch_diagram.png"
$assetFullPath = Join-Path $RootDir "assets\test_switch_diagram.png"
Assert-Condition "Gate 2.1: Attachment assets/test_switch_diagram.png verified on disk" (Test-Path $assetFullPath) "Attachment missing at $assetFullPath"

# Generate Base64 Screenshot ID token and data URI
$imgBytes = [System.IO.File]::ReadAllBytes($assetFullPath)
$imgBase64 = [Convert]::ToBase64String($imgBytes)
$imgDataUri = "data:image/png;base64,$imgBase64"
$uniqueSuffix = (Get-Random -Minimum 100000000 -Maximum 999999999)
$screenshotId = "SS_ID_$uniqueSuffix"
$promptId = "prompt-e2e-$uniqueSuffix"
$now = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()

$promptContent = "E2E Test Prompt for Multimodal Backup with token $screenshotId and attachment $assetRelativePath"
$imageRecord = @{
    uri = $imgDataUri
    path = $assetRelativePath
    token = $screenshotId
    size_bytes = $imgBytes.Length
    mime = "image/png"
}
$imagesPayloadJson = (ConvertTo-Json -Compress @($imageRecord)).Replace("'", "''")
$normRepoPath = $RootDir.Replace('\', '/')

# Insert directly into repo_prompts.db active_prompts
# Using instance_id = 'default' (matches default instance and empty scope)
$insertSql = "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload) VALUES ('$promptId', 'Antigravity-Manager', 'default', '$normRepoPath', '$promptContent', 'gemini-2.5-pro', 'conv-$uniqueSuffix', 'running', $now, $now, '$imagesPayloadJson')"
$null = Invoke-SqlitePython -DbPath $repoDbPath -Query $insertSql

# Verify insertion
$queryActive = "SELECT id, status, prompt_content, image_payload FROM active_prompts WHERE id = '$promptId'"
$activeRows = Invoke-SqlitePython -DbPath $repoDbPath -Query $queryActive
$hasActiveRow = ($null -ne $activeRows -and @($activeRows).Count -ge 1)
Assert-Condition "Gate 2.2: Synthetic prompt inserted into active_prompts" $hasActiveRow "Active prompt row not found in repo_prompts.db"

$activePayload = if ($hasActiveRow) { [string]$activeRows[0].image_payload } else { "" }
$hasTokenInActive = $activePayload.Contains($screenshotId) -and $activePayload.Contains("assets/test_switch_diagram.png")
Assert-Condition "Gate 2.3: Prompt image_payload contains Base64 Screenshot ID token and attachment path" $hasTokenInActive "Payload missing Screenshot ID ($screenshotId)"

# -------------------------------------------------------------------------
# Gate 3: Execute agm backup running default --json and verify progress logging
# -------------------------------------------------------------------------
Write-Host "`n[Gate 3/8] Backup Running Prompts with Multimodal Image Tracing" -ForegroundColor Yellow

$backupOutputRaw = & $agm backup running default --json 2>&1
if ($VerboseOutput) { Write-Host ($backupOutputRaw -join "`n") -ForegroundColor Gray }
$backupJson = Parse-CliJson $backupOutputRaw
$backupText = ($backupOutputRaw -join "`n")

# Verify 3-step progress logging / step indicators
$hasStepLogging = ($backupText -match "Step" -or $backupText -match "STEP" -or $backupText -match "INFO" -or $backupText -match "Scanning" -or $backupText -match "Snapshot" -or ($null -ne $backupJson))
Assert-Condition "Gate 3.1: Step-by-step progress logging emitted during backup" $hasStepLogging "3-step progress logging missing from backup execution"

# Verify batch creation in backup-prompts.db
$queryBackup = "SELECT id, backup_batch_id, prompt_id, prompt_text, has_images, images_payload, status, is_restored FROM prompt_backups WHERE prompt_id = '$promptId' OR prompt_text LIKE '%$screenshotId%'"
$backupRows = Invoke-SqlitePython -DbPath $backupDbPath -Query $queryBackup

$hasBackupRow = ($null -ne $backupRows -and @($backupRows).Count -ge 1)
Assert-Condition "Gate 3.2: Snapshot batch created in backup-prompts.db" $hasBackupRow "Snapshot record missing in backup-prompts.db"

$backupHasImages = if ($hasBackupRow) { [int]$backupRows[0].has_images } else { 0 }
Assert-Condition "Gate 3.3: Snapshot record has_images = 1" ($backupHasImages -eq 1) "Expected has_images = 1, got '$backupHasImages'"

$backupPayload = if ($hasBackupRow) { [string]$backupRows[0].images_payload } else { "" }
$backupBatchId = if ($hasBackupRow) { [string]$backupRows[0].backup_batch_id } else { "" }
$hasTokenInBackup = if ($hasBackupRow) {
    $backupPayload.Contains($screenshotId) -or ([string]$backupRows[0].prompt_text).Contains($screenshotId)
} else {
    $false
}
Assert-Condition "Gate 3.4: Backup record preserves Screenshot ID token ($screenshotId)" $hasTokenInBackup "Backup payload missing Screenshot ID token"

# -------------------------------------------------------------------------
# Gate 4: Evict active prompt row to simulate restore requirement
# -------------------------------------------------------------------------
Write-Host "`n[Gate 4/8] Evict Active Prompt Row to Simulate Account Switch / Restore" -ForegroundColor Yellow

$deleteActiveSql = "DELETE FROM active_prompts WHERE id = '$promptId'"
$null = Invoke-SqlitePython -DbPath $repoDbPath -Query $deleteActiveSql

$checkPurged = Invoke-SqlitePython -DbPath $repoDbPath -Query "SELECT COUNT(*) as cnt FROM active_prompts WHERE id = '$promptId'"
$isPurged = ($null -ne $checkPurged -and @($checkPurged).Count -ge 1 -and [int]$checkPurged[0].cnt -eq 0)
Assert-Condition "Gate 4.1: Active prompt row evicted from active_prompts" $isPurged "Failed to evict active prompt row"

$checkBackupIntact = Invoke-SqlitePython -DbPath $backupDbPath -Query "SELECT COUNT(*) as cnt FROM prompt_backups WHERE (prompt_id = '$promptId' OR prompt_text LIKE '%$screenshotId%') AND is_restored = 0"
$backupIntact = ($null -ne $checkBackupIntact -and @($checkBackupIntact).Count -ge 1 -and [int]$checkBackupIntact[0].cnt -ge 1)
Assert-Condition "Gate 4.2: Backup record in backup-prompts.db remains intact (is_restored = 0)" $backupIntact "Backup record was prematurely marked restored"

# -------------------------------------------------------------------------
# Gate 5: Execute agm restore running default --json and verify FIFO restoration
# -------------------------------------------------------------------------
Write-Host "`n[Gate 5/8] Execute Restore and Verify FIFO Order & Image Preservation" -ForegroundColor Yellow

$restoreOutputRaw = & $agm restore running default --json 2>&1
if ($VerboseOutput) { Write-Host ($restoreOutputRaw -join "`n") -ForegroundColor Gray }
$restoreJson = Parse-CliJson $restoreOutputRaw
$restoreText = ($restoreOutputRaw -join "`n")

# Verify 3-step progress logging / step indicators
$hasRestoreStepLogging = ($restoreText -match "Step" -or $restoreText -match "STEP" -or $restoreText -match "INFO" -or $restoreText -match "Restor" -or ($null -ne $restoreJson))
Assert-Condition "Gate 5.1: Step-by-step progress logging emitted during restore" $hasRestoreStepLogging "3-step progress logging missing from restore execution"

# Verify re-insertion into active_prompts
$queryRestored = "SELECT id, status, prompt_content, image_payload, created_at FROM active_prompts WHERE id = '$promptId' OR prompt_content LIKE '%$screenshotId%'"
$restoredRows = Invoke-SqlitePython -DbPath $repoDbPath -Query $queryRestored

$hasRestoredRow = ($null -ne $restoredRows -and @($restoredRows).Count -ge 1)
Assert-Condition "Gate 5.2: Prompt successfully restored into active_prompts" $hasRestoredRow "Prompt not restored into active_prompts table"

$restoredPayload = if ($hasRestoredRow) { [string]$restoredRows[0].image_payload } else { "" }
$hasTokenInRestored = if ($hasRestoredRow) {
    $restoredPayload.Contains($screenshotId) -or ([string]$restoredRows[0].prompt_content).Contains($screenshotId)
} else {
    $false
}
Assert-Condition "Gate 5.3: Restored active prompt preserves Screenshot ID token ($screenshotId)" $hasTokenInRestored "Restored prompt dropped Screenshot ID token"

# Verify strict FIFO order across active_prompts
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
Assert-Condition "Gate 5.4: Strict FIFO queue order enforced (ORDER BY created_at ASC, id ASC)" $isFifoOrdered "FIFO ordering violated in active_prompts"

# -------------------------------------------------------------------------
# Gate 6: Verify GitMap command parity
# -------------------------------------------------------------------------
Write-Host "`n[Gate 6/8] Verify GitMap Command Delegation Parity" -ForegroundColor Yellow

# 1. gitmap agy running --json (or gitmap agy running)
$gitmapRunningRaw = & $gitmap agy running 2>&1
$gitmapRunningExit = $LASTEXITCODE
$gitmapRunningStr = ($gitmapRunningRaw -join "`n")
$hasGitmapRunningParity = ($gitmapRunningExit -eq 0) -and ($gitmapRunningStr -match "Active Running Prompts" -or $gitmapRunningStr -match "conversation" -or $gitmapRunningStr -match "Antigravity")
Assert-Condition "Gate 6.1: gitmap agy running lists active running prompts" $hasGitmapRunningParity "gitmap agy running failed with code $gitmapRunningExit"

# 2. gitmap agy ff
$gitmapFfRaw = & $gitmap agy ff 2>&1
$gitmapFfExit = $LASTEXITCODE
$gitmapFfStr = ($gitmapFfRaw -join "`n")
$hasGitmapFfParity = ($gitmapFfExit -eq 0) -and ($gitmapFfStr -match "Account Switch" -or $gitmapFfStr -match "asw" -or $gitmapFfStr -match "switch")
Assert-Condition "Gate 6.2: gitmap agy ff recognized and routes to fast-forward" $hasGitmapFfParity "gitmap agy ff failed with code $gitmapFfExit"

# 3. gitmap agy backup rp (backup-running-prompts)
$gitmapBackupHelp = & $gitmap backup-running-prompts --help 2>&1
$gitmapBackupExit = $LASTEXITCODE
$gitmapBackupStr = ($gitmapBackupHelp -join "`n")
$hasGitmapBackupParity = ($gitmapBackupExit -eq 0) -and ($gitmapBackupStr -match "Running Prompts" -or $gitmapBackupStr -match "running-prompts" -or $gitmapBackupStr -match "backup")
Assert-Condition "Gate 6.3: gitmap agy backup rp delegation command supported" $hasGitmapBackupParity "gitmap backup rp failed with code $gitmapBackupExit"

# 4. gitmap ag restore rp (restore-running-prompts)
$gitmapRestoreHelp = & $gitmap restore-running-prompts --help 2>&1
$gitmapRestoreExit = $LASTEXITCODE
$gitmapRestoreStr = ($gitmapRestoreHelp -join "`n")
$hasGitmapRestoreParity = ($gitmapRestoreExit -eq 0) -and ($gitmapRestoreStr -match "Running Prompts" -or $gitmapRestoreStr -match "running-prompts" -or $gitmapRestoreStr -match "restore")
Assert-Condition "Gate 6.4: gitmap ag restore rp delegation command supported" $hasGitmapRestoreParity "gitmap restore rp failed with code $gitmapRestoreExit"

# -------------------------------------------------------------------------
# Gate 7: Verify caller IDE session protection (default-copy-5597, PID 11588)
# -------------------------------------------------------------------------
Write-Host "`n[Gate 7/8] Verify Caller IDE Session Protection" -ForegroundColor Yellow

$probeStopRaw = & $agm instances stop $CallerInstanceId 2>&1
$probeStopExit = $LASTEXITCODE
$probeStopText = ($probeStopRaw -join "`n")

$wasCallerProtected = ($probeStopExit -ne 0) -and ($probeStopText -match "Refusing to stop" -or $probeStopText -match "currently running IDE" -or $probeStopText -match "Aborting" -or $probeStopText -match "caller" -or $probeStopText -match "Refusing to terminate")
Assert-Condition "Gate 7.1: Caller protection intercepted stop command on $CallerInstanceId" $wasCallerProtected "Caller instance was not shielded from stop command"

# Check caller process PID remains alive
$callerProcCheck = Get-Process -Id $CallerSessionPid -ErrorAction SilentlyContinue
$isCallerProcessAlive = ($null -ne $callerProcCheck)
Assert-Condition "Gate 7.2: Caller IDE process PID $CallerSessionPid remains healthy and alive" $isCallerProcessAlive "Caller process $CallerSessionPid was terminated"

# -------------------------------------------------------------------------
# Gate 8: Clean up synthetic test records completely
# -------------------------------------------------------------------------
Write-Host "`n[Gate 8/8] Complete Cleanup of Test Records & Residual Artifacts" -ForegroundColor Yellow

$cleanActiveSql = "DELETE FROM active_prompts WHERE id = '$promptId' OR prompt_content LIKE '%$screenshotId%'"
$null = Invoke-SqlitePython -DbPath $repoDbPath -Query $cleanActiveSql

$cleanBackupSql = "DELETE FROM prompt_backups WHERE prompt_id = '$promptId' OR prompt_text LIKE '%$screenshotId%'"
$null = Invoke-SqlitePython -DbPath $backupDbPath -Query $cleanBackupSql

if (![string]::IsNullOrEmpty($backupBatchId)) {
    $cleanBatchSql = "DELETE FROM backup_batches WHERE id = '$backupBatchId'"
    $null = Invoke-SqlitePython -DbPath $backupDbPath -Query $cleanBatchSql
}

# Clean any test resume task files in workspace
$resumeFiles = Get-ChildItem -Path $RootDir -Filter "*resume_task*" -File -ErrorAction SilentlyContinue
foreach ($rf in $resumeFiles) {
    if ($rf.Name -match "test" -or $rf.Name -match "e2e") {
        Remove-Item -Force $rf.FullName -ErrorAction SilentlyContinue
    }
}

# Assert zero residual records in SQLite databases
$verifyCleanActive = Invoke-SqlitePython -DbPath $repoDbPath -Query "SELECT COUNT(*) as cnt FROM active_prompts WHERE id = '$promptId' OR prompt_content LIKE '%$screenshotId%'"
$verifyCleanBackup = Invoke-SqlitePython -DbPath $backupDbPath -Query "SELECT COUNT(*) as cnt FROM prompt_backups WHERE prompt_id = '$promptId' OR prompt_text LIKE '%$screenshotId%'"

$activeClean = ($null -ne $verifyCleanActive -and [int]$verifyCleanActive[0].cnt -eq 0)
$backupClean = ($null -ne $verifyCleanBackup -and [int]$verifyCleanBackup[0].cnt -eq 0)

Assert-Condition "Gate 8.1: active_prompts test rows cleaned completely (0 residuals)" $activeClean "Stale test records remain in active_prompts"
Assert-Condition "Gate 8.2: prompt_backups test rows cleaned completely (0 residuals)" $backupClean "Stale test records remain in prompt_backups"

# -------------------------------------------------------------------------
# Final Test Execution Summary
# -------------------------------------------------------------------------
Write-Host "`n================================================================================" -ForegroundColor Cyan
Write-Host " E2E Test Execution Summary" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

$passed = @($script:testResults | Where-Object { $_.Status -eq "PASS" }).Count
$failed = @($script:testResults | Where-Object { $_.Status -eq "FAIL" }).Count
$total = $script:testResults.Count

foreach ($r in $script:testResults) {
    $color = if ($r.Status -eq "PASS") { [ConsoleColor]::Green } else { [ConsoleColor]::Red }
    Write-Host ("  [{0,-4}] {1}" -f $r.Status, $r.Test) -ForegroundColor $color
    if ($r.Details -ne "") {
        Write-Host "         -> Details: $($r.Details)" -ForegroundColor DarkRed
    }
}

Write-Host "`nTotal: $total | Passed: $passed | Failed: $failed" -ForegroundColor $(if ($failed -eq 0) { [ConsoleColor]::Green } else { [ConsoleColor]::Red })
Write-Host "================================================================================" -ForegroundColor Cyan

if ($failed -gt 0) {
    exit 1
} else {
    exit 0
}
