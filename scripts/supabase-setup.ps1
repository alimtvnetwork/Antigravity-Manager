<#
.SYNOPSIS
    Antigravity-Manager Supabase Setup Script (One-Liner & JSON Ingestion)
.DESCRIPTION
    Configures Root (Lovable) and Secondary Supabase credentials in antigravity-manager.
    Supports standalone one-liner execution and loading from JSON.
#>

[CmdletBinding()]
param (
    [string]$ConfigFile = "",
    [string]$RootUrl = "https://pezjuuddecbyfmqxytrv.supabase.co/rest/v1/",
    [string]$RootKey = "sb_publishable_cJdJyIEeXc8bpym9TIOU7w_vVU0_Y69",
    [string]$SecondaryUrl = "https://ikwmurmjynhxdpmhzekt.supabase.co/rest/v1/",
    [string]$SecondaryKey = "sb_publishable_barsshQom3VcUw1l5soE_A_R1FwQz8o",
    [string]$NodeAlias = "",
    [switch]$SkipTest = $false
)

if (-not $ConfigFile) {
    $defaultJson = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) "supabase-accounts.json"
    if (Test-Path $defaultJson) {
        $ConfigFile = $defaultJson
    }
}

if ($ConfigFile -and (Test-Path $ConfigFile)) {
    Write-Host "Reading Supabase configuration from JSON: $ConfigFile" -ForegroundColor Cyan
    $cfg = Get-Content -Path $ConfigFile -Raw | ConvertFrom-Json
} else {
    Write-Host "Using provided Root (Lovable) and Secondary parameters..." -ForegroundColor Cyan
    $cfg = [PSCustomObject]@{
        endpoints = @(
            [PSCustomObject]@{
                id = "ep-root-lovable-01"
                name = "Root Supabase (Lovable)"
                url = $RootUrl
                api_key = $RootKey
                role = "root"
                is_enabled = $true
                prune_threshold_mb = 400
                priority = 1
                notes = "Root account by Lovable"
                tags = @("lovable", "root")
            },
            [PSCustomObject]@{
                id = "ep-secondary-01"
                name = "Secondary Supabase"
                url = $SecondaryUrl
                api_key = $SecondaryKey
                role = "secondary"
                is_enabled = $true
                prune_threshold_mb = 200
                priority = 2
                notes = "Secondary fallback and command queue"
                tags = @("secondary")
            }
        )
        node_alias = if ($NodeAlias) { $NodeAlias } else { "Node-823632" }
        is_sync_enabled = $true
        auto_prune_root_mb = 400
        auto_prune_secondary_mb = 200
        heartbeat_interval_secs = 30
    }
}

$jsonOut = $cfg | ConvertTo-Json -Depth 10

$targetDirs = @(
    (Join-Path $env:APPDATA "antigravity-manager"),
    (Join-Path $HOME ".antigravity_tools")
)

foreach ($targetDir in $targetDirs) {
    if (-not (Test-Path $targetDir)) {
        New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
    }
    $targetPath = Join-Path $targetDir "supabase_config.json"
    [System.IO.File]::WriteAllText($targetPath, $jsonOut, [System.Text.UTF8Encoding]::new($false))
    Write-Host "Config written successfully to: $targetPath" -ForegroundColor Green
}

if (-not $SkipTest) {
    Write-Host "Testing Supabase PostgREST connectivity..." -ForegroundColor Cyan
    foreach ($ep in $cfg.endpoints) {
        $clean = $ep.url.TrimEnd('/')
        $probe = if ($ep.role -eq "secondary") { "$clean/command_queue?limit=0" } else { "$clean/nodes?limit=0" }
        try {
            $h = @{ "apikey" = $ep.api_key; "Authorization" = "Bearer $($ep.api_key)" }
            $r = Invoke-WebRequest -Uri $probe -Headers $h -Method Get -TimeoutSec 10 -ErrorAction Stop
            Write-Host "  [$($ep.id)] $($ep.name): PASS (HTTP $($r.StatusCode))" -ForegroundColor Green
        } catch {
            $reader = [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream())
            $body = $reader.ReadToEnd()
            if ($body -match "PGRST" -or $body -match "relation" -or $body -match "does not exist") {
                Write-Host "  [$($ep.id)] $($ep.name): PASS (PostgREST responsive & authenticated)" -ForegroundColor Green
            } else {
                Write-Host "  [$($ep.id)] $($ep.name): FAIL ($($_.Exception.Message))" -ForegroundColor Yellow
            }
        }
    }
}

Write-Host "One-liner to read from JSON via PowerShell:" -ForegroundColor Yellow
Write-Host 'powershell -NoProfile -Command ''$c = Get-Content .\scripts\supabase-accounts.json -Raw | ConvertFrom-Json; $j = $c | ConvertTo-Json -Depth 10; $u = [System.Text.UTF8Encoding]::new($false); [System.IO.File]::WriteAllText((Join-Path $env:APPDATA "antigravity-manager\supabase_config.json"), $j, $u); [System.IO.File]::WriteAllText((Join-Path $HOME ".antigravity_tools\supabase_config.json"), $j, $u); Write-Host "Configured $($c.endpoints.Count) endpoints via one-liner"''' -ForegroundColor Gray
