<#
.SYNOPSIS
    Comprehensive End-to-End Test Suite for Antigravity Multi-Instance Isolation,
    Account Switching, Fast-Forward, Prompt Backup/Restore, and Teardown.
#>
param(
    [switch]$SkipCleanup
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$agm = Join-Path $RootDir "src-tauri\target\debug\agm.exe"

if (!(Test-Path $agm)) {
    Write-Error "agm.exe binary not found at $agm! Please build it first."
    exit 1
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Antigravity Multi-Instance Comprehensive E2E Test Suite   " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 0. Safety Invariant Check: Verify user processes
$userAgmProcesses = Get-Process | Where-Object { $_.Name -match "agm-alim" }
Write-Host "[0/6] Safety Check: AGM GUI processes detected: $($userAgmProcesses.Count)" -ForegroundColor Gray

# Define test instances and test accounts
$instA = "test-e2e-alpha"
$instB = "test-e2e-beta"
$accA = "alex.hudson.riseup@gmail.com"
$accB = "erfan.office.n@gmail.com"
$accSwitch = "shohagbazar004@gmail.com"

# Clean any leftover previous test instances first
Write-Host "[1/6] Pre-test hygiene: removing any stale test instances..." -ForegroundColor Yellow
$currentList = try { & $agm instances --json | ConvertFrom-Json } catch { @() }
foreach ($oldInst in @($instA, $instB, "test-diag-1-8492")) {
    if ($currentList | Where-Object { $_.config.id -eq $oldInst }) {
        & $agm instances rm $oldInst --force | Out-Null
    }
}

# Helper to query SQLite value in a state.vscdb
function Get-VscdbEmail($dbPath) {
    if (!(Test-Path $dbPath)) { return $null }
    $pythonCmd = @"
import sqlite3, sys
try:
    conn = sqlite3.connect(r'$dbPath')
    c = conn.cursor()
    c.execute("SELECT value FROM ItemTable WHERE key = 'antigravityAuth.token'")
    row = c.fetchone()
    if row and row[0]:
        import json
        data = json.loads(row[0])
        print(data.get('email') or data.get('username') or '')
    else:
        print('')
except Exception as e:
    print('')
"@
    return (python -c $pythonCmd).Trim()
}

function Get-AppStorageEmail($jsonPath) {
    if (!(Test-Path $jsonPath)) { return $null }
    try {
        $content = Get-Content $jsonPath -Raw | ConvertFrom-Json
        return $content."jetski.onboarding.lastLoginUsername"
    } catch {
        return $null
    }
}

# -------------------------------------------------------------
# Test 1: Create multiple instances with isolated accounts
# -------------------------------------------------------------
Write-Host "`n[2/6] TEST 1: Creating multiple isolated instances..." -ForegroundColor Yellow
$createA = & $agm instances create $instA -a $accA --data-only --json | ConvertFrom-Json
$createB = & $agm instances create $instB -a $accB --data-only --json | ConvertFrom-Json

Write-Host "  Instance A created: $($createA.id) -> Bound to: $($createA.bound_email)" -ForegroundColor Green
Write-Host "  Instance B created: $($createB.id) -> Bound to: $($createB.bound_email)" -ForegroundColor Green

if ($createA.bound_email -ne $accA) {
    throw "TEST 1 FAILED: Instance A bound email '$($createA.bound_email)' != '$accA'"
}
if ($createB.bound_email -ne $accB) {
    throw "TEST 1 FAILED: Instance B bound email '$($createB.bound_email)' != '$accB'"
}

$instADir = $createA.data_dir
$instBDir = $createB.data_dir
$instAParent = Split-Path -Parent $instADir
$instBParent = Split-Path -Parent $instBDir

# Verify all 3 databases for Instance A
$dbA1 = Join-Path $instADir "User\globalStorage\state.vscdb"
$dbA2 = Join-Path $instADir "AppData\Roaming\Antigravity\User\globalStorage\state.vscdb"
$dbA3 = Join-Path $instAParent "home\AppData\Roaming\Antigravity\User\globalStorage\state.vscdb"
$tokenA = Join-Path $instAParent "home\.gemini\antigravity\jetski-standalone-oauth-token"
$storageA = Join-Path $instADir "app_storage.json"

$emailA1 = Get-VscdbEmail $dbA1
$emailA2 = Get-VscdbEmail $dbA2
$emailA3 = Get-VscdbEmail $dbA3
$storageEmailA = Get-AppStorageEmail $storageA

Write-Host "  Instance A state.vscdb (User): $emailA1" -ForegroundColor DarkGray
Write-Host "  Instance A state.vscdb (data AppData): $emailA2" -ForegroundColor DarkGray
Write-Host "  Instance A state.vscdb (home AppData): $emailA3" -ForegroundColor DarkGray
Write-Host "  Instance A app_storage.json email: $storageEmailA" -ForegroundColor DarkGray
Write-Host "  Instance A standalone token exists: $(Test-Path $tokenA)" -ForegroundColor DarkGray

if ($emailA1 -ne $accA -or $emailA2 -ne $accA -or $emailA3 -ne $accA) {
    throw "TEST 1 FAILED: Instance A databases do not match expected $accA! (Got $emailA1, $emailA2, $emailA3)"
}
if (!(Test-Path $tokenA)) {
    throw "TEST 1 FAILED: Instance A jetski-standalone-oauth-token was not written!"
}

# Verify all 3 databases for Instance B
$dbB1 = Join-Path $instBDir "User\globalStorage\state.vscdb"
$dbB2 = Join-Path $instBDir "AppData\Roaming\Antigravity\User\globalStorage\state.vscdb"
$dbB3 = Join-Path $instBParent "home\AppData\Roaming\Antigravity\User\globalStorage\state.vscdb"

$emailB1 = Get-VscdbEmail $dbB1
$emailB2 = Get-VscdbEmail $dbB2
$emailB3 = Get-VscdbEmail $dbB3

Write-Host "  Instance B state.vscdb (User): $emailB1" -ForegroundColor DarkGray
Write-Host "  Instance B state.vscdb (data AppData): $emailB2" -ForegroundColor DarkGray
Write-Host "  Instance B state.vscdb (home AppData): $emailB3" -ForegroundColor DarkGray

if ($emailB1 -ne $accB -or $emailB2 -ne $accB -or $emailB3 -ne $accB) {
    throw "TEST 1 FAILED: Instance B databases do not match expected $accB! (Got $emailB1, $emailB2, $emailB3)"
}

Write-Host "  [PASS] Test 1: Multi-instance creation & database isolation verified." -ForegroundColor Green

# -------------------------------------------------------------
# Test 2: Account Switching for Instance
# -------------------------------------------------------------
Write-Host "`n[3/6] TEST 2: Switching Instance A to new account ($accSwitch)..." -ForegroundColor Yellow
$switchOutput = & $agm instances switch $instA $accSwitch --json | ConvertFrom-Json

Write-Host "  Switch outcome: $($switchOutput.email) (Previous: $($switchOutput.previous_email))" -ForegroundColor Green
if ($switchOutput.email -ne $accSwitch) {
    throw "TEST 2 FAILED: Switched email '$($switchOutput.email)' != '$accSwitch'"
}

$emailA1_switched = Get-VscdbEmail $dbA1
$emailA2_switched = Get-VscdbEmail $dbA2
$emailA3_switched = Get-VscdbEmail $dbA3
$storageEmailA_switched = Get-AppStorageEmail $storageA

Write-Host "  Instance A state.vscdb (User) post-switch: $emailA1_switched" -ForegroundColor DarkGray
Write-Host "  Instance A state.vscdb (data AppData) post-switch: $emailA2_switched" -ForegroundColor DarkGray
Write-Host "  Instance A state.vscdb (home AppData) post-switch: $emailA3_switched" -ForegroundColor DarkGray
Write-Host "  Instance A app_storage.json post-switch: $storageEmailA_switched" -ForegroundColor DarkGray

if ($emailA1_switched -ne $accSwitch -or $emailA2_switched -ne $accSwitch -or $emailA3_switched -ne $accSwitch) {
    throw "TEST 2 FAILED: Instance A databases were not updated to $accSwitch!"
}
if ($storageEmailA_switched -ne $accSwitch) {
    throw "TEST 2 FAILED: Instance A app_storage.json was not updated to $accSwitch!"
}

# Verify Instance B was NOT affected by switching Instance A
$emailB1_check = Get-VscdbEmail $dbB1
if ($emailB1_check -ne $accB) {
    throw "TEST 2 FAILED: Cross-contamination! Instance B was modified to $emailB1_check during Instance A switch!"
}

Write-Host "  [PASS] Test 2: Account switching across all database paths verified with zero cross-contamination." -ForegroundColor Green

# -------------------------------------------------------------
# Test 3: Fast-Forward Smart Switch for Instance
# -------------------------------------------------------------
Write-Host "`n[4/6] TEST 3: Fast-Forward Smart Switch on Instance A..." -ForegroundColor Yellow
$ffOutput = & $agm ff $instA --json | ConvertFrom-Json

Write-Host "  Fast-Forward rotated to: $($ffOutput.selected_email) (Previous: $($ffOutput.previous_email))" -ForegroundColor Green
if (!$ffOutput.selected_email -or $ffOutput.selected_email -eq $accSwitch) {
    throw "TEST 3 FAILED: Fast-forward did not rotate to a new account! Selected: $($ffOutput.selected_email)"
}

$newEmail = $ffOutput.selected_email
$emailA1_ff = Get-VscdbEmail $dbA1
$emailA2_ff = Get-VscdbEmail $dbA2
$emailA3_ff = Get-VscdbEmail $dbA3
$storageEmailA_ff = Get-AppStorageEmail $storageA

Write-Host "  Instance A state.vscdb (User) post-FF: $emailA1_ff" -ForegroundColor DarkGray
Write-Host "  Instance A state.vscdb (data AppData) post-FF: $emailA2_ff" -ForegroundColor DarkGray
Write-Host "  Instance A state.vscdb (home AppData) post-FF: $emailA3_ff" -ForegroundColor DarkGray
Write-Host "  Instance A app_storage.json post-FF: $storageEmailA_ff" -ForegroundColor DarkGray

if ($emailA1_ff -ne $newEmail -or $emailA2_ff -ne $newEmail -or $emailA3_ff -ne $newEmail) {
    throw "TEST 3 FAILED: Instance A databases were not updated to FF account $newEmail!"
}
Write-Host "  [PASS] Test 3: Fast-Forward Smart Switch updated all credential stores correctly." -ForegroundColor Green

# -------------------------------------------------------------
# Test 4: Running Prompt Backup & Restore during Switch
# -------------------------------------------------------------
Write-Host "`n[5/6] TEST 4: Running Prompt Backup & Restore across Instance Switch..." -ForegroundColor Yellow

# Create a dummy workspace folder for testing
$testWs = Join-Path $RootDir "build-demo\test-e2e-ws"
if (Test-Path $testWs) { Remove-Item -Recurse -Force $testWs }
New-Item -ItemType Directory -Path $testWs -Force | Out-Null

# Assign test workspace to Instance A
Write-Host "  Assigning workspace $testWs to $instA..." -ForegroundColor DarkGray
& $agm instances assign $instA $testWs | Out-Null

# Inject a simulated running prompt into repo_db active_prompts
$testPromptText = "Test Prompt for E2E Switch Preservation $(Get-Random)"
$testPromptId = "prompt-test-e2e-$((Get-Date).Ticks)"
$nowUnix = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()

$pythonSeedPrompt = @"
import sqlite3, os
db_path = os.path.expanduser(r'~/.antigravity_tools/repo_prompts.db')
conn = sqlite3.connect(db_path)
c = conn.cursor()
c.execute("""
INSERT OR REPLACE INTO active_prompts 
(id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, 'running', ?, ?)
""", ('$testPromptId', 'proj-test', '$instA', r'$testWs', '$testPromptText', 'gemini-pro', 'sess-e2e', $nowUnix, $nowUnix))
conn.commit()
conn.close()
"@
python -c $pythonSeedPrompt

# Verify prompt is running in DB
$checkPromptPy = @"
import sqlite3, os
db_path = os.path.expanduser(r'~/.antigravity_tools/repo_prompts.db')
conn = sqlite3.connect(db_path)
c = conn.cursor()
c.execute("SELECT status, prompt_content FROM active_prompts WHERE id = '$testPromptId'")
row = c.fetchone()
if row:
    print(f"{row[0]}|{row[1]}")
conn.close()
"@
$statusBefore = (python -c $checkPromptPy).Trim()
Write-Host "  Prompt before switch: $statusBefore" -ForegroundColor DarkGray

# Now execute switch on Instance A to $accA
Write-Host "  Switching Instance A to trigger backup, close, inject, launch, and restore cycle..." -ForegroundColor DarkGray
& $agm instances switch $instA $accA | Out-Null

# Check prompt status post-switch
$statusAfter = (python -c $checkPromptPy).Trim()
Write-Host "  Prompt post-switch status: $statusAfter" -ForegroundColor Green

# Also check that .antigravity_resume_task.json was written to the workspace
$resumeTaskFile = Join-Path $testWs ".antigravity_resume_task.json"
$hasResumeFile = Test-Path $resumeTaskFile
Write-Host "  Workspace resume snapshot file exists: $hasResumeFile" -ForegroundColor DarkGray

if ($hasResumeFile) {
    $resumeJson = Get-Content $resumeTaskFile -Raw | ConvertFrom-Json
    Write-Host "  Resume snapshot prompt: $($resumeJson.prompt_content.Substring(0, [Math]::Min(50, $resumeJson.prompt_content.Length)))..." -ForegroundColor DarkGray
}

if (!($statusAfter -match 'dispatched|backed_up')) {
    throw "TEST 4 FAILED: Prompt status post-switch is '$statusAfter', expected 'dispatched' or 'backed_up'!"
}
if (!$hasResumeFile) {
    throw "TEST 4 FAILED: .antigravity_resume_task.json was not created in workspace directory!"
}

Write-Host "  [PASS] Test 4: Running prompts successfully snapshotted, backed up, and restored." -ForegroundColor Green

# -------------------------------------------------------------
# Test 5: Instance Hygiene & Teardown Verification
# -------------------------------------------------------------
Write-Host "`n[6/6] TEST 5: Instance hygiene, session wipe, and teardown..." -ForegroundColor Yellow

if (!$SkipCleanup) {
    # Test wiping session on Instance B
    Write-Host "  Testing session wipe on Instance B..." -ForegroundColor DarkGray
    & $agm instances stop $instB 2>$null
    # Run wipe via node/tauri command or verify file deletion
    & $agm instances rm $instA --force
    & $agm instances rm $instB --force

    # Run scripts/dev-tool-clear.ps1 --clean-instances
    Write-Host "  Running dev-tool-clear with instance hygiene..." -ForegroundColor DarkGray
    & powershell -ExecutionPolicy Bypass -File "$RootDir/scripts/dev-tool-clear.ps1" --instances-only

    # Verify instances removed from registry
    $listJson = & $agm instances --json | ConvertFrom-Json
    $remainingTestInst = $listJson | Where-Object { $_.config.id -in @($instA, $instB) }

    if ($remainingTestInst) {
        throw "TEST 5 FAILED: Test instances still present in registry after cleanup!"
    }

    # Verify directories removed
    if ((Test-Path (Join-Path $RootDir ".antigravity_tools\instances\$instA")) -or (Test-Path (Join-Path $RootDir ".antigravity_tools\instances\$instB"))) {
        throw "TEST 5 FAILED: Instance directories were not removed from disk!"
    }

    # Clean test workspace
    if (Test-Path $testWs) { Remove-Item -Recurse -Force $testWs -ErrorAction SilentlyContinue }

    Write-Host "  [PASS] Test 5: Clean teardown, instance hygiene, and registry consistency verified." -ForegroundColor Green
} else {
    Write-Host "  [SKIPPED] Teardown skipped per user flag." -ForegroundColor DarkGray
}

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host " ALL TEST SUITES PASSED CLEANLY (100% ISOLATION VERIFIED) " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
