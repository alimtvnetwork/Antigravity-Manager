<#
.SYNOPSIS
    Automated PowerShell Verification Test for Two-Machine SSH Cluster Connectivity
    and Instance-Account Mapping in Antigravity-Manager.
#>

param(
    [int]$TargetCallerPid = 10076
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host " Automated SSH Cluster & Instance-Account Mapping Verification " -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# Helper: Parse JSON from CLI output safely
# ---------------------------------------------------------------------------
function Parse-CliJson {
    param([object]$rawOutput)
    if ($null -eq $rawOutput) { return $null }
    $text = ($rawOutput | Out-String).Trim()

    try {
        return ($text | ConvertFrom-Json)
    } catch {}

    $firstBracket = $text.IndexOf('[')
    $firstBrace = $text.IndexOf('{')

    if ($firstBracket -ge 0 -and ($firstBrace -lt 0 -or $firstBracket -lt $firstBrace)) {
        $lastBracket = $text.LastIndexOf(']')
        if ($lastBracket -gt $firstBracket) {
            $slice = $text.Substring($firstBracket, $lastBracket - $firstBracket + 1)
            try { return ($slice | ConvertFrom-Json) } catch {}
        }
    }

    if ($firstBrace -ge 0) {
        $lastBrace = $text.LastIndexOf('}')
        if ($lastBrace -gt $firstBrace) {
            $slice = $text.Substring($firstBrace, $lastBrace - $firstBrace + 1)
            try { return ($slice | ConvertFrom-Json) } catch {}
        }
    }

    return ($text | ConvertFrom-Json)
}

# ---------------------------------------------------------------------------
# 0. Safety Invariant Check: Verify & Protect Caller Session PID 10076
# ---------------------------------------------------------------------------
Write-Host "`n[STEP 0] Verifying caller session PID $TargetCallerPid protection..." -ForegroundColor Yellow
$callerProc = Get-Process -Id $TargetCallerPid -ErrorAction SilentlyContinue
if (-not $callerProc) {
    Write-Warning "Target caller PID $TargetCallerPid is not currently running. Checking for any active Antigravity IDE process..."
    $callerProc = Get-Process -Name "Antigravity" -ErrorAction SilentlyContinue | Select-Object -First 1
}
if ($callerProc) {
    Write-Host "  [OK] Caller IDE session verified: PID $($callerProc.Id) ($($callerProc.ProcessName))" -ForegroundColor Green
    $protectedPid = $callerProc.Id
} else {
    Write-Host "  [INFO] No active Antigravity IDE process found" -ForegroundColor Yellow
    $protectedPid = $null
}

# Locate agm and gitmap binaries
$agmExe = Join-Path $RootDir "src-tauri\target\debug\agm.exe"
if (-not (Test-Path $agmExe)) {
    $cmd = Get-Command "agm" -ErrorAction SilentlyContinue
    if (-not $cmd) { $cmd = Get-Command "adm" -ErrorAction SilentlyContinue }
    if ($cmd) {
        $agmExe = $cmd.Source
    } else {
        throw "agm binary not found at $agmExe or in PATH"
    }
}
Write-Host "  [OK] Using AGM binary: $agmExe" -ForegroundColor Gray

$gitmapCmd = Get-Command "gitmap" -ErrorAction SilentlyContinue
if (-not $gitmapCmd) {
    throw "gitmap command not found in PATH"
}
Write-Host "  [OK] Using GitMap command: $($gitmapCmd.Name)" -ForegroundColor Gray

# ---------------------------------------------------------------------------
# 1. Query GitMap cluster nodes (gitmap cluster nodes --json)
# ---------------------------------------------------------------------------
Write-Host "`n[STEP 1] Querying GitMap cluster nodes ('gitmap cluster nodes --json')..." -ForegroundColor Yellow
$gitmapOut = & $gitmapCmd cluster nodes --json 2>&1
$gitmapNodes = Parse-CliJson $gitmapOut

if (-not $gitmapNodes) {
    throw "ASSERTION FAILED: Failed to parse output from 'gitmap cluster nodes --json'"
}

$gitmapW1 = $gitmapNodes | Where-Object { $_.alias -eq "w1" }
$gitmapW2 = $gitmapNodes | Where-Object { $_.alias -eq "w2" }

if (-not $gitmapW1) {
    throw "ASSERTION FAILED: Node 'w1' not found in GitMap cluster nodes"
}
if ($gitmapW1.ip -ne "192.168.1.3") {
    throw "ASSERTION FAILED: Node 'w1' IP in GitMap is '$($gitmapW1.ip)', expected '192.168.1.3'"
}
Write-Host "  [OK] GitMap node 'w1' verified: alias=$($gitmapW1.alias), ip=$($gitmapW1.ip)" -ForegroundColor Green

if (-not $gitmapW2) {
    throw "ASSERTION FAILED: Node 'w2' not found in GitMap cluster nodes"
}
if ($gitmapW2.ip -ne "192.168.1.7") {
    throw "ASSERTION FAILED: Node 'w2' IP in GitMap is '$($gitmapW2.ip)', expected '192.168.1.7'"
}
Write-Host "  [OK] GitMap node 'w2' verified: alias=$($gitmapW2.alias), ip=$($gitmapW2.ip)" -ForegroundColor Green

# Check for duplicate nodes in GitMap
$gitmapAliases = @($gitmapNodes | ForEach-Object { $_.alias.ToLowerInvariant().Trim() })
$dupAliases = $gitmapAliases | Group-Object | Where-Object { $_.Count -gt 1 }
if ($dupAliases) {
    throw "ASSERTION FAILED: Duplicate aliases detected in GitMap cluster nodes: $($dupAliases.Name -join ', ')"
}
Write-Host "  [OK] No duplicate aliases in GitMap cluster nodes (Total nodes: $($gitmapNodes.Count))" -ForegroundColor Green

# ---------------------------------------------------------------------------
# 2. Query SSH nodes (agm ssh nodes --json and ~/.antigravity_tools/ssh_nodes.json)
# ---------------------------------------------------------------------------
Write-Host "`n[STEP 2] Querying SSH nodes ('agm ssh nodes --json')..." -ForegroundColor Yellow
$sshOut = & $agmExe ssh nodes --json 2>&1
$sshNodes = Parse-CliJson $sshOut

if (-not $sshNodes) {
    throw "ASSERTION FAILED: Failed to parse output from 'agm ssh nodes --json'"
}

$sshW1 = $sshNodes | Where-Object { $_.alias -eq "w1" }
$sshW2 = $sshNodes | Where-Object { $_.alias -eq "w2" }

if (-not $sshW1) {
    throw "ASSERTION FAILED: SSH node 'w1' not found in 'agm ssh nodes --json'"
}
if ($sshW1.ip_address -ne "192.168.1.3") {
    throw "ASSERTION FAILED: SSH node 'w1' IP is '$($sshW1.ip_address)', expected '192.168.1.3'"
}
Write-Host "  [OK] SSH node 'w1' verified: alias=$($sshW1.alias), ip=$($sshW1.ip_address)" -ForegroundColor Green

if (-not $sshW2) {
    throw "ASSERTION FAILED: SSH node 'w2' not found in 'agm ssh nodes --json'"
}
if ($sshW2.ip_address -ne "192.168.1.7") {
    throw "ASSERTION FAILED: SSH node 'w2' IP is '$($sshW2.ip_address)', expected '192.168.1.7'"
}
Write-Host "  [OK] SSH node 'w2' verified: alias=$($sshW2.alias), ip=$($sshW2.ip_address)" -ForegroundColor Green

# Verify no duplicate entries in SSH nodes
$sshAliases = @($sshNodes | ForEach-Object { $_.alias.ToLowerInvariant().Trim() })
$dupSshAliases = $sshAliases | Group-Object | Where-Object { $_.Count -gt 1 }
if ($dupSshAliases) {
    throw "ASSERTION FAILED: Duplicate aliases detected in SSH nodes: $($dupSshAliases.Name -join ', ')"
}
$sshIps = @($sshNodes | ForEach-Object { $_.ip_address.Trim() })
$dupSshIps = $sshIps | Group-Object | Where-Object { $_.Count -gt 1 }
if ($dupSshIps) {
    throw "ASSERTION FAILED: Duplicate IP addresses detected in SSH nodes: $($dupSshIps.Name -join ', ')"
}
Write-Host "  [OK] No duplicate node aliases or IPs in SSH nodes (Total nodes: $($sshNodes.Count))" -ForegroundColor Green

# Also check ~/.antigravity_tools/ssh_nodes.json file if present
$sshNodesJsonPath = Join-Path $env:USERPROFILE ".antigravity_tools\ssh_nodes.json"
if (Test-Path $sshNodesJsonPath) {
    Write-Host "  [INFO] Verifying $sshNodesJsonPath..." -ForegroundColor Gray
    $storeContent = Get-Content $sshNodesJsonPath -Raw | ConvertFrom-Json
    $storeNodes = if ($storeContent.data -and $storeContent.data.nodes) {
        $storeContent.data.nodes
    } elseif ($storeContent.nodes) {
        $storeContent.nodes
    } else {
        @()
    }
    $fileAliases = @($storeNodes | ForEach-Object { $_.alias.ToLowerInvariant().Trim() })
    $dupFileAliases = $fileAliases | Group-Object | Where-Object { $_.Count -gt 1 }
    if ($dupFileAliases) {
        throw "ASSERTION FAILED: Duplicate aliases detected in ${sshNodesJsonPath}: $($dupFileAliases.Name -join ', ')"
    }
    Write-Host "  [OK] No duplicate aliases in $sshNodesJsonPath" -ForegroundColor Green
}

# ---------------------------------------------------------------------------
# 3. Verify that two machines (w1 and w2) are registered and configured in ~/.ssh/config
# ---------------------------------------------------------------------------
Write-Host "`n[STEP 3] Verifying ~/.ssh/config registration for w1 and w2..." -ForegroundColor Yellow
$sshConfigPath = Join-Path $env:USERPROFILE ".ssh\config"
if (-not (Test-Path $sshConfigPath)) {
    throw "ASSERTION FAILED: SSH config file not found at $sshConfigPath"
}

$sshConfigText = Get-Content $sshConfigPath -Raw

# Check Host w1 configuration
$hasHostW1 = [regex]::IsMatch($sshConfigText, '(?m)^\s*Host\s+w1\b')
if (-not $hasHostW1) {
    throw "ASSERTION FAILED: 'Host w1' not found in $sshConfigPath"
}
$matchW1 = [regex]::Match($sshConfigText, '(?ms)Host\s+w1\b.*?HostName\s+([0-9.]+)')
if (-not $matchW1.Success -or $matchW1.Groups[1].Value -ne "192.168.1.3") {
    throw "ASSERTION FAILED: 'Host w1' HostName is not 192.168.1.3 in $sshConfigPath"
}
Write-Host "  [OK] Host w1 configured with HostName 192.168.1.3 in ~/.ssh/config" -ForegroundColor Green

# Check Host w2 configuration
$hasHostW2 = [regex]::IsMatch($sshConfigText, '(?m)^\s*Host\s+w2\b')
if (-not $hasHostW2) {
    throw "ASSERTION FAILED: 'Host w2' not found in $sshConfigPath"
}
$matchW2 = [regex]::Match($sshConfigText, '(?ms)Host\s+w2\b.*?HostName\s+([0-9.]+)')
if (-not $matchW2.Success -or $matchW2.Groups[1].Value -ne "192.168.1.7") {
    throw "ASSERTION FAILED: 'Host w2' HostName is not 192.168.1.7 in $sshConfigPath"
}
Write-Host "  [OK] Host w2 configured with HostName 192.168.1.7 in ~/.ssh/config" -ForegroundColor Green

# Verify managed block duplicate check
$managedMatch = [regex]::Match($sshConfigText, '(?s)# --- agm ssh managed \(do not edit\) ---\s*(.*?)\s*# --- end agm ssh managed ---')
if ($managedMatch.Success) {
    $managedBlock = $managedMatch.Groups[1].Value
    $managedHosts = [regex]::Matches($managedBlock, '(?m)^\s*Host\s+([^\s#]+)') | ForEach-Object { $_.Groups[1].Value.ToLowerInvariant() }
    $dupManagedHosts = $managedHosts | Group-Object | Where-Object { $_.Count -gt 1 }
    if ($dupManagedHosts) {
        throw "ASSERTION FAILED: Duplicate Host entries found in ~/.ssh/config managed block: $($dupManagedHosts.Name -join ', ')"
    }
    Write-Host "  [OK] Zero duplicate Host entries in ~/.ssh/config managed section" -ForegroundColor Green
}

# ---------------------------------------------------------------------------
# 4. Query agm quota show --json (runs get_cluster_instances_and_balances)
# ---------------------------------------------------------------------------
Write-Host "`n[STEP 4] Querying 'agm quota show --json' (get_cluster_instances_and_balances)..." -ForegroundColor Yellow
$quotaOut = & $agmExe quota show --json 2>&1
$quotaData = Parse-CliJson $quotaOut

if (-not $quotaData -or -not $quotaData.data -or -not $quotaData.data.instances_and_balances) {
    throw "ASSERTION FAILED: 'agm quota show --json' did not return valid 'data.instances_and_balances'"
}

$balances = $quotaData.data.instances_and_balances
Write-Host "  [INFO] Received $($balances.Count) cluster instance balance items." -ForegroundColor Gray

# Sub-assertion 4a: w1 (local) reports active running instances with bound Google account emails
Write-Host "`n[STEP 4a] Asserting w1 (local) running instances and account mapping..." -ForegroundColor Yellow
$w1Items = @($balances | Where-Object { $_.machine_alias -eq "w1" })
if ($w1Items.Count -eq 0) {
    throw "ASSERTION FAILED: No cluster items returned for machine_alias 'w1'"
}

# Check for Default instance bound to riseup.team.now@gmail.com
$w1Default = $w1Items | Where-Object { $_.instance_name -eq "Default" -or $_.instance_id -eq "default" }
if (-not $w1Default) {
    throw "ASSERTION FAILED: 'Default' instance not found on w1"
}
if ($w1Default.bound_email -ne "riseup.team.now@gmail.com") {
    throw "ASSERTION FAILED: w1 'Default' bound_email is '$($w1Default.bound_email)', expected 'riseup.team.now@gmail.com'"
}
if ($w1Default.running -ne $true) {
    throw "ASSERTION FAILED: w1 'Default' running status is '$($w1Default.running)', expected true"
}
Write-Host "  [OK] w1 'Default' instance: bound_email=$($w1Default.bound_email), running=$($w1Default.running), status=$($w1Default.status_label)" -ForegroundColor Green

# Check for Default Copy instance bound to zihadxshohag2@gmail.com
$w1Copy = $w1Items | Where-Object { $_.instance_name -eq "Default Copy" -or $_.instance_id -like "*copy*" }
if (-not $w1Copy) {
    throw "ASSERTION FAILED: 'Default Copy' instance not found on w1"
}
if ($w1Copy.bound_email -ne "zihadxshohag2@gmail.com") {
    throw "ASSERTION FAILED: w1 'Default Copy' bound_email is '$($w1Copy.bound_email)', expected 'zihadxshohag2@gmail.com'"
}
if ($w1Copy.status_label -notin @("running", "idle")) {
    throw "ASSERTION FAILED: w1 'Default Copy' status is '$($w1Copy.status_label)', expected running or idle"
}
Write-Host "  [OK] w1 'Default Copy' instance: bound_email=$($w1Copy.bound_email), running=$($w1Copy.running), status=$($w1Copy.status_label)" -ForegroundColor Green

# Sub-assertion 4b: w2 (remote) reports genuine connectivity status and 0.0% quota (NOT false 100%)
Write-Host "`n[STEP 4b] Asserting w2 (remote) genuine connectivity status and quota protection..." -ForegroundColor Yellow
$w2Items = @($balances | Where-Object { $_.machine_alias -eq "w2" })
if ($w2Items.Count -eq 0) {
    throw "ASSERTION FAILED: No cluster items returned for machine_alias 'w2'"
}

foreach ($w2Item in $w2Items) {
    Write-Host "  [INFO] w2 item: status=$($w2Item.status_label), running=$($w2Item.running), 4h_quota=$($w2Item.quota_4h_pct)%, weekly_quota=$($w2Item.quota_weekly_pct)%" -ForegroundColor Gray
    
    # Verify genuine connectivity status
    if ($w2Item.status_label -eq "offline" -or $w2Item.running -eq $false) {
        # Offline node must report 0.0% quota and MUST NOT report false 100% quota
        if ([double]$w2Item.quota_4h_pct -gt 0.0) {
            throw "ASSERTION FAILED: Offline node w2 reported false positive quota_4h_pct: $($w2Item.quota_4h_pct)%, expected 0.0%"
        }
        if ([double]$w2Item.quota_weekly_pct -gt 0.0) {
            throw "ASSERTION FAILED: Offline node w2 reported false positive quota_weekly_pct: $($w2Item.quota_weekly_pct)%, expected 0.0%"
        }
        if ($w2Item.error_message) {
            Write-Host "  [OK] w2 genuine offline status verified: $($w2Item.error_message)" -ForegroundColor Green
        } else {
            Write-Host "  [OK] w2 offline status verified with 0.0% quota" -ForegroundColor Green
        }
    } else {
        Write-Host "  [OK] w2 is online: status=$($w2Item.status_label)" -ForegroundColor Green
    }
    
    # Must NOT report false 100.0% quota when offline/unreachable
    if ($w2Item.status_label -eq "offline" -and ([double]$w2Item.quota_4h_pct -eq 100.0 -or [double]$w2Item.quota_weekly_pct -eq 100.0)) {
        throw "ASSERTION FAILED: w2 shows false 100% quota while offline!"
    }
}
Write-Host "  [OK] w2 reports genuine status with zero false-100% quota leakage" -ForegroundColor Green

# Sub-assertion 4c: No duplicate node entries exist
Write-Host "`n[STEP 4c] Asserting no duplicate node entries in cluster balances..." -ForegroundColor Yellow
$nodeInstances = $balances | ForEach-Object { "$($_.machine_alias):$($_.instance_id)" }
$dupInstances = $nodeInstances | Group-Object | Where-Object { $_.Count -gt 1 }
if ($dupInstances) {
    throw "ASSERTION FAILED: Duplicate node/instance entries found in balances: $($dupInstances.Name -join ', ')"
}
Write-Host "  [OK] Zero duplicate instances across all cluster items" -ForegroundColor Green

# ---------------------------------------------------------------------------
# 5. Session PID Protection Invariant Verification
# ---------------------------------------------------------------------------
Write-Host "`n[STEP 5] Verifying caller session PID $TargetCallerPid was never disrupted..." -ForegroundColor Yellow
if ($protectedPid) {
    $checkProc = Get-Process -Id $protectedPid -ErrorAction SilentlyContinue
    if (-not $checkProc) {
        throw "CRITICAL FAILURE: Protected caller session PID $protectedPid was killed during test execution!"
    }
    Write-Host "  [OK] Protected caller session PID $protectedPid is healthy and running!" -ForegroundColor Green
}

# ---------------------------------------------------------------------------
# Test Summary
# ---------------------------------------------------------------------------
Write-Host "`n=================================================================" -ForegroundColor Green
Write-Host " ALL TEST SUITE ASSERTIONS PASSED SUCCESSFULLY!                 " -ForegroundColor Green
Write-Host " - GitMap Cluster Nodes: w1 (192.168.1.3), w2 (192.168.1.7)      " -ForegroundColor Green
Write-Host " - SSH Nodes & ~/.ssh/config: Fully synced and deduplicated     " -ForegroundColor Green
Write-Host " - w1 Local Active Instances: Default + Default Copy mapped      " -ForegroundColor Green
Write-Host " - w2 Remote Status: Genuine offline report, zero false-100%     " -ForegroundColor Green
Write-Host " - Caller Session PID $($protectedPid): Protected and undisturbed    " -ForegroundColor Green
Write-Host "=================================================================" -ForegroundColor Green

exit 0
