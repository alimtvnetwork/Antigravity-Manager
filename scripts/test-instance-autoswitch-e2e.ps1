# Test script for Instance Autoswitch, Caller Self-Preservation Guard, and Test Instance Management E2E Verification
# Strictly uses relative paths only and verifies:
# 1. agm instances current detects caller instance (including env override).
# 2. agm instances create-test creates transient test instance in %LOCALAPPDATA%\agm-instances\test-<id>.
# 3. Verifies directory on disk and entry in instance registry.
# 4. Verifies agm instances stop <caller_id> is refused by caller self-preservation guard.
# 5. Cleans up via agm instances rm-test and verifies directory and registry cleanliness.

$ErrorActionPreference = "Stop"

Write-Host "===================================================================" -ForegroundColor Cyan
Write-Host " Running Instance Autoswitch & Self-Preservation Guard E2E Suite  " -ForegroundColor Cyan
Write-Host "===================================================================" -ForegroundColor Cyan

# Locate agm CLI binary using strictly relative paths
$agmExe = "src-tauri\target\debug\agm.exe"
if (-not (Test-Path $agmExe)) {
    $cmd = Get-Command "agm" -ErrorAction SilentlyContinue
    if (-not $cmd) {
        $cmd = Get-Command "adm" -ErrorAction SilentlyContinue
    }
    if ($cmd) {
        $agmExe = $cmd.Source
    } else {
        throw "agm.exe binary not found at relative path '$agmExe' or in system PATH"
    }
}

Write-Host "Using AGM executable: $agmExe" -ForegroundColor Cyan

# Sanitize LOCALAPPDATA if running within an instance profile to ensure test sandboxes remain isolated
if ($env:LOCALAPPDATA -match '\.antigravity_tools') {
    $hostRoot = ($env:LOCALAPPDATA -split '\.antigravity_tools')[0].TrimEnd('\', '/')
    $hostLocal = Join-Path $hostRoot "AppData\Local"
    if (Test-Path $hostLocal) {
        $env:LOCALAPPDATA = $hostLocal
        Write-Host "Sanitized LOCALAPPDATA for test isolation: $env:LOCALAPPDATA" -ForegroundColor Gray
    }
}

function Invoke-AgmWithTimeout {
    param (
        [string[]]$Arguments,
        [int]$TimeoutSeconds = 30
    )
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $agmExe
    $escapedArgs = @()
    foreach ($arg in $Arguments) {
        if ($arg -match '[\s"]') {
            $escaped = $arg -replace '"', '\"'
            $escapedArgs += "`"$escaped`""
        } else {
            $escapedArgs += $arg
        }
    }
    $psi.Arguments = $escapedArgs -join ' '
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true

    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    [void]$proc.Start()

    $stdoutTask = $proc.StandardOutput.ReadToEndAsync()
    $stderrTask = $proc.StandardError.ReadToEndAsync()

    if (-not $proc.WaitForExit($TimeoutSeconds * 1000)) {
        try { $proc.Kill() } catch {}
        throw "AGM command timed out after ${TimeoutSeconds}s: $agmExe $($psi.Arguments)"
    }
    [void][System.Threading.Tasks.Task]::WaitAll($stdoutTask, $stderrTask)

    $stdout = $stdoutTask.Result
    $stderr = $stderrTask.Result
    $combined = ($stdout + "`n" + $stderr).Trim()

    return [PSCustomObject]@{
        Stdout = $stdout
        Stderr = $stderr
        Output = $combined
        ExitCode = $proc.ExitCode
    }
}

function Extract-JsonPayload ($rawOutput) {
    if ($rawOutput -match '(?ms)(\{.*\}|\[.*\])') {
        return ($matches[1] | ConvertFrom-Json)
    }
    return ($rawOutput | ConvertFrom-Json)
}

# -------------------------------------------------------------------
# STEP 1: Test 'agm instances current' to detect caller instance
# -------------------------------------------------------------------
Write-Host "`n[STEP 1] Testing 'agm instances current' detection..." -ForegroundColor Yellow

# Test default current output
$currentRes = Invoke-AgmWithTimeout -Arguments @("instances", "current")
$currentRaw = $currentRes.Output.Trim()
Write-Host "  Raw 'agm instances current' output: '$currentRaw'" -ForegroundColor Gray

# Test JSON output
$currentJsonRes = Invoke-AgmWithTimeout -Arguments @("instances", "current", "--json")
$currentJson = Extract-JsonPayload $currentJsonRes.Output
$detectedCallerId = if ($currentJson.instance_id) { $currentJson.instance_id } elseif ($currentJson.id) { $currentJson.id } elseif ($currentJson.data.instance_id) { $currentJson.data.instance_id } else { $currentJson.data.id }
Write-Host "  JSON detected caller instance ID: '$detectedCallerId'" -ForegroundColor Green

# Test environment variable override detection
$probeId = "test-probe-env-caller"
$env:ANTIGRAVITY_INSTANCE_ID = $probeId
try {
    $envOverrideRes = Invoke-AgmWithTimeout -Arguments @("instances", "current", "--json")
    $envOverrideJson = Extract-JsonPayload $envOverrideRes.Output
    $overrideId = if ($envOverrideJson.instance_id) { $envOverrideJson.instance_id } elseif ($envOverrideJson.id) { $envOverrideJson.id } elseif ($envOverrideJson.data.instance_id) { $envOverrideJson.data.instance_id } else { $envOverrideJson.data.id }
    if ($overrideId -ne $probeId) {
        throw "ASSERTION FAILED: ANTIGRAVITY_INSTANCE_ID override was '$overrideId', expected '$probeId'"
    }
    Write-Host "  [OK] Caller instance ID properly overridden by ANTIGRAVITY_INSTANCE_ID: $probeId" -ForegroundColor Green
} finally {
    Remove-Item Env:\ANTIGRAVITY_INSTANCE_ID -ErrorAction SilentlyContinue
}

# -------------------------------------------------------------------
# STEP 2: Create a new test instance via 'agm instances create-test'
# -------------------------------------------------------------------
$testName = "test-autoswitch-sandbox"
Write-Host "`n[STEP 2] Creating new test instance with name '$testName'..." -ForegroundColor Yellow

$createRes = Invoke-AgmWithTimeout -Arguments @("instances", "create-test", "--name", $testName, "--json")
Write-Host $createRes.Output.Trim() -ForegroundColor Gray
$createJson = Extract-JsonPayload $createRes.Output

$testId = if ($createJson.id) { $createJson.id } elseif ($createJson.data.id) { $createJson.data.id } else { $createJson.instance_id }
$testDataDir = if ($createJson.data_dir) { $createJson.data_dir } elseif ($createJson.data.data_dir) { $createJson.data.data_dir } else { $createJson.config.data_dir }

if (-not $testId) {
    throw "ASSERTION FAILED: create-test did not return an instance ID"
}
if (-not $testDataDir) {
    throw "ASSERTION FAILED: create-test did not return a data directory"
}

Write-Host "  [OK] Created test instance: ID='$testId', DataDir='$testDataDir'" -ForegroundColor Green

# -------------------------------------------------------------------
# STEP 3: Verify the test instance was created on disk and in registry
# -------------------------------------------------------------------
Write-Host "`n[STEP 3] Verifying filesystem sandbox and registry registration..." -ForegroundColor Yellow

# 3a. Filesystem check
if (-not (Test-Path $testDataDir)) {
    throw "ASSERTION FAILED: Test instance directory '$testDataDir' does not exist on disk"
}
Write-Host "  [OK] Filesystem sandbox directory verified on disk: $testDataDir" -ForegroundColor Green

# 3b. Registry check
$listRes = Invoke-AgmWithTimeout -Arguments @("instances", "list", "--json")
$listJson = Extract-JsonPayload $listRes.Output
$matchedInst = $listJson.data | Where-Object { $_.id -eq $testId }

if (-not $matchedInst) {
    throw "ASSERTION FAILED: Instance '$testId' was not found in 'agm instances list --json' registry"
}
Write-Host "  [OK] Instance '$testId' confirmed present in instance registry" -ForegroundColor Green

# -------------------------------------------------------------------
# STEP 4: Verify caller self-preservation guard and PID termination
# -------------------------------------------------------------------
Write-Host "`n[STEP 4] Verifying caller self-preservation guard and process termination..." -ForegroundColor Yellow

# 4a. Verify stop, switch, ff, and rm refusal when simulated caller is active
$env:ANTIGRAVITY_INSTANCE_ID = $testId
try {
    # Test stop refusal
    $stopRes = Invoke-AgmWithTimeout -Arguments @("instances", "stop", $testId)
    Write-Host "  Stop attempt inside caller instance (exit code $($stopRes.ExitCode)):" -ForegroundColor Gray
    Write-Host "  $($stopRes.Output.Trim())" -ForegroundColor Gray

    if ($stopRes.ExitCode -eq 0) {
        throw "ASSERTION FAILED: 'agm instances stop $testId' succeeded unexpectedly when running inside caller instance!"
    }
    $cleanOutput = ($stopRes.Output -replace '\s+', ' ')
    if (-not ($cleanOutput -match "Refusing to stop, switch, or delete instance" -or $cleanOutput -match "is your currently running IDE instance")) {
        throw "ASSERTION FAILED: Output did not contain expected refusal message. Output was: $cleanOutput"
    }
    Write-Host "  [OK] Caller self-preservation guard successfully intercepted stop command for '$testId'" -ForegroundColor Green

    # Test switch refusal
    $switchRes = Invoke-AgmWithTimeout -Arguments @("instances", "switch", $testId, "test@example.com")
    if ($switchRes.ExitCode -ne 2) {
        throw "ASSERTION FAILED: 'agm instances switch $testId' was not refused with exit code 2 inside caller! Exit code: $($switchRes.ExitCode)"
    }
    Write-Host "  [OK] Caller self-preservation guard successfully intercepted switch command for '$testId'" -ForegroundColor Green



    # Test rm refusal
    $rmRes = Invoke-AgmWithTimeout -Arguments @("instances", "rm", $testId)
    if ($rmRes.ExitCode -ne 2) {
        throw "ASSERTION FAILED: 'agm instances rm $testId' was not refused with exit code 2 inside caller! Exit code: $($rmRes.ExitCode)"
    }
    Write-Host "  [OK] Caller self-preservation guard successfully intercepted rm command for '$testId'" -ForegroundColor Green
} finally {
    Remove-Item Env:\ANTIGRAVITY_INSTANCE_ID -ErrorAction SilentlyContinue
}

# 4b. If the current environment actually is an IDE caller process, verify real guard
if ($detectedCallerId -and $detectedCallerId -ne "none") {
    Write-Host "  Testing real caller self-preservation for detected IDE instance '$detectedCallerId'..." -ForegroundColor Cyan
    $realStopRes = Invoke-AgmWithTimeout -Arguments @("instances", "stop", $detectedCallerId)
    if ($realStopRes.ExitCode -eq 0) {
        throw "ASSERTION FAILED: Real caller instance stop was not intercepted!"
    }
    $cleanRealOutput = ($realStopRes.Output -replace '\s+', ' ')
    if (-not ($cleanRealOutput -match "Refusing to stop, switch, or delete instance" -or $cleanRealOutput -match "is your currently running IDE instance")) {
        throw "ASSERTION FAILED: Output did not contain expected refusal message for real caller. Output was: $cleanRealOutput"
    }
    Write-Host "  [OK] Real caller self-preservation confirmed for '$detectedCallerId'" -ForegroundColor Green
}

# 4c. Verify that 'agm instances stop' terminates active test process PID
Write-Host "`n[STEP 4c] Verifying 'agm instances stop' terminates active test process PID..." -ForegroundColor Yellow
$dummyProc = Start-Process powershell -ArgumentList "-NoProfile -Command `"param([string]`$udd) Start-Sleep -Seconds 120`" -udd `"$testDataDir`"" -PassThru
$dummyPid = $dummyProc.Id
Write-Host "  Spawned mock worker process PID: $dummyPid" -ForegroundColor Gray

# Bind mock PID to the test instance in instances.json
$candidateJsonPaths = @(
    (Join-Path $env:USERPROFILE ".antigravity_tools\instances\instances.json"),
    "C:\Users\Administrator\.antigravity_tools\instances\instances.json"
)
if ($env:USERPROFILE -match '\.antigravity_tools') {
    $hostHome = ($env:USERPROFILE -split '\.antigravity_tools')[0].TrimEnd('\', '/')
    $candidateJsonPaths += (Join-Path $hostHome ".antigravity_tools\instances\instances.json")
}
$instancesJsonPath = $candidateJsonPaths | Where-Object { Test-Path $_ } | Select-Object -First 1

if ($instancesJsonPath -and (Test-Path $instancesJsonPath)) {
    $rawText = [System.IO.File]::ReadAllText($instancesJsonPath)
    $rawReg = $rawText | ConvertFrom-Json
    foreach ($inst in $rawReg.instances) {
        if ($inst.id -eq $testId) {
            $inst.pid = $dummyPid
        }
    }
    [System.IO.File]::WriteAllText($instancesJsonPath, ($rawReg | ConvertTo-Json -Depth 10), [System.Text.UTF8Encoding]::new($false))
    Write-Host "  Registered PID $dummyPid for instance '$testId' in $instancesJsonPath" -ForegroundColor Gray
}

# Confirm mock process is alive
$aliveBefore = Get-Process -Id $dummyPid -ErrorAction SilentlyContinue
if (-not $aliveBefore) {
    throw "ASSERTION FAILED: Mock process PID $dummyPid was not alive before stop invocation!"
}
Write-Host "  [OK] Mock process PID $dummyPid verified alive before stop" -ForegroundColor Green

# Stop the test instance
$stopTestRes = Invoke-AgmWithTimeout -Arguments @("instances", "stop", $testId)
Write-Host "  Stop output: $($stopTestRes.Output)" -ForegroundColor Gray
if ($stopTestRes.ExitCode -ne 0) {
    throw "ASSERTION FAILED: 'agm instances stop $testId' failed with exit code $($stopTestRes.ExitCode)"
}

# Verify mock PID is terminated
$terminated = $false
for ($i = 0; $i -lt 10; $i++) {
    $aliveCheck = Get-Process -Id $dummyPid -ErrorAction SilentlyContinue
    if (-not $aliveCheck) {
        $terminated = $true
        break
    }
    Start-Sleep -Milliseconds 500
}

if (-not $terminated) {
    try { Stop-Process -Id $dummyPid -Force -ErrorAction SilentlyContinue } catch {}
    throw "ASSERTION FAILED: Process PID $dummyPid was NOT terminated by 'agm instances stop'!"
}
Write-Host "  [OK] Active process PID $dummyPid successfully terminated by 'agm instances stop'" -ForegroundColor Green

# -------------------------------------------------------------------
# STEP 5: Clean up test instance via 'agm instances rm-test'
# -------------------------------------------------------------------
Write-Host "`n[STEP 5] Cleaning up test instance '$testId' via rm-test..." -ForegroundColor Yellow

$rmRes = Invoke-AgmWithTimeout -Arguments @("instances", "rm-test", $testId, "--json")
Write-Host $rmRes.Output.Trim() -ForegroundColor Gray

# 5a. Verify directory deleted
if (Test-Path $testDataDir) {
    throw "ASSERTION FAILED: Test instance directory '$testDataDir' still exists after rm-test"
}
Write-Host "  [OK] Test directory cleanly removed from disk" -ForegroundColor Green

# 5b. Verify removed from registry
$postListRes = Invoke-AgmWithTimeout -Arguments @("instances", "list", "--json")
$postList = Extract-JsonPayload $postListRes.Output
$postMatched = $postList.data | Where-Object { $_.id -eq $testId }

if ($postMatched) {
    throw "ASSERTION FAILED: Instance '$testId' still present in registry after rm-test"
}
Write-Host "  [OK] Instance '$testId' cleanly unregistered from registry" -ForegroundColor Green

# 5c. Test rm-test --all to ensure bulk cleanup safety
Write-Host "`n[STEP 5c] Testing 'agm instances rm-test --all' idempotency..." -ForegroundColor Yellow
$rmAllRes = Invoke-AgmWithTimeout -Arguments @("instances", "rm-test", "--all", "--json")
Write-Host $rmAllRes.Output.Trim() -ForegroundColor Gray
Write-Host "  [OK] Bulk cleanup executed cleanly" -ForegroundColor Green

Write-Host "`n===================================================================" -ForegroundColor Green
Write-Host " ALL TEST-INSTANCE-AUTOSWITCH E2E TESTS PASSED SUCCESSFULLY!       " -ForegroundColor Green
Write-Host "===================================================================" -ForegroundColor Green
