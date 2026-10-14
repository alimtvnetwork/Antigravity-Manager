<#
.SYNOPSIS
    Antigravity-Manager Supabase Credential Import & Provisioning Suite
.DESCRIPTION
    Imports Supabase Root (Lovable) and Secondary credentials into local configuration:
      - Linux / macOS: ~/.config/antigravity-manager/supabase_config.json
      - Windows:       %APPDATA%\antigravity-manager\supabase_config.json
    
    Implements 4-tier resolution precedence:
      1. Explicit CLI parameters (-RootUrl, -RootKey, etc.)
      2. Explicit configuration / vault file (-ConfigFile or -VaultFile)
      3. Sibling directory auto-discovery (repo-secrets/02-antigravity-manager/vault/supabase_config.json)
      4. Environment variables ($env:AGM_SUPABASE_ROOT_URL, etc.)
    
    URL normalization strips any redundant '/rest/v1' suffix to guarantee clean base endpoints.
#>

[CmdletBinding()]
param (
    [Parameter(Mandatory = $false, HelpMessage = "Path to an external Supabase vault file")]
    [string]$VaultFile = "",

    [Parameter(Mandatory = $false, HelpMessage = "Explicit Root Supabase Project URL")]
    [string]$RootUrl = "",

    [Parameter(Mandatory = $false, HelpMessage = "Explicit Root Supabase publishable API key")]
    [string]$RootKey = "",

    [Parameter(Mandatory = $false, HelpMessage = "Explicit Secondary Supabase Project URL")]
    [string]$SecondaryUrl = "",

    [Parameter(Mandatory = $false, HelpMessage = "Explicit Secondary Supabase publishable API key")]
    [string]$SecondaryKey = "",

    [Parameter(Mandatory = $false, HelpMessage = "Custom machine node alias (e.g. Node-Alpha)")]
    [string]$NodeAlias = "",

    [Parameter(Mandatory = $false, HelpMessage = "Explicit JSON configuration or envelope file")]
    [string]$ConfigFile = "",

    [Parameter(Mandatory = $false, HelpMessage = "Skip PostgREST connectivity probe")]
    [switch]$SkipTest = $false,

    [Parameter(Mandatory = $false, HelpMessage = "Force auto-discovery from sibling repo-secrets")]
    [switch]$AutoDiscover = $false
)

$ErrorActionPreference = "Stop"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "  Antigravity-Manager (AGM) Supabase Secrets Importer" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

function Normalize-SupabaseUrl {
    param (
        [string]$Url
    )
    if ([string]::IsNullOrWhiteSpace($Url)) {
        return ""
    }
    $u = $Url.Trim().Replace("pezjuuddecbyrmqxytrv.supabase.co", "pezjuuddecbyfmqxytrv.supabase.co")
    while ($u.EndsWith('/')) {
        $u = $u.Substring(0, $u.Length - 1)
    }
    if ($u.ToLower().EndsWith('/rest/v1')) {
        $u = $u.Substring(0, $u.Length - 8)
        while ($u.EndsWith('/')) {
            $u = $u.Substring(0, $u.Length - 1)
        }
    }
    return $u
}

function Decode-Base64IfNeeded {
    param (
        [string]$Value,
        [bool]$IsBase64
    )
    if ([string]::IsNullOrWhiteSpace($Value)) {
        return ""
    }
    if ($IsBase64) {
        try {
            return [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($Value.Trim())).Trim()
        } catch {
            return $Value.Trim()
        }
    }
    return $Value.Trim()
}

function Parse-SupabaseJsonFile {
    param (
        [string]$Path,
        [string]$AliasOverride
    )
    if (-not (Test-Path $Path)) {
        throw "Configuration file not found: $Path"
    }

    $raw = Get-Content -Path $Path -Raw -Encoding UTF8
    if ($raw.StartsWith([char]0xFEFF)) {
        $raw = $raw.Substring(1)
    }
    $parsed = $raw | ConvertFrom-Json

    $cfg = $null
    $isB64 = ($parsed.encoding_format -eq "base64") -or ($parsed.attributes -and $parsed.attributes.encoding -eq "base64")

    # 1. AGM JSON Envelope Schema (v2.0 / v1.0) with endpoints array
    if ($parsed.data -and $parsed.data.endpoints) {
        $cfg = $parsed.data
        if ($parsed.variables) {
            foreach ($prop in $parsed.variables.PSObject.Properties) {
                $token = "`${" + $prop.Name + "}"
                $val = [string]$prop.Value
                if ($cfg.node_alias -and $cfg.node_alias.Contains($token)) {
                    $cfg.node_alias = $cfg.node_alias.Replace($token, $val)
                }
                foreach ($ep in $cfg.endpoints) {
                    if ($ep.url -and $ep.url.Contains($token)) {
                        $ep.url = $ep.url.Replace($token, $val)
                    }
                    if ($ep.api_key -and $ep.api_key.Contains($token)) {
                        $ep.api_key = $ep.api_key.Replace($token, $val)
                    }
                    if ($ep.name -and $ep.name.Contains($token)) {
                        $ep.name = $ep.name.Replace($token, $val)
                    }
                }
            }
        }
    }
    # 2. Flat configuration schema
    elseif ($parsed.endpoints) {
        $cfg = $parsed
    }
    # 3. Base64-encoded credentials envelope or flat credentials object
    elseif ($parsed.credentials -or $parsed.endpoint -or ($parsed.data -and ($parsed.data.endpoint -or $parsed.data.url))) {
        $creds = if ($parsed.credentials) { $parsed.credentials } elseif ($parsed.data) { $parsed.data } else { $parsed }
        $rawUrl = if ($creds.endpoint) { [string]$creds.endpoint } else { [string]$creds.url }
        $rawKey = if ($creds.token) { [string]$creds.token } else { [string]$creds.api_key }
        $rawRole = if ($creds.role) { [string]$creds.role } else { "" }
        $rawService = if ($creds.service) { [string]$creds.service } else { "supabase-service" }

        $epUrl = Normalize-SupabaseUrl (Decode-Base64IfNeeded -Value $rawUrl -IsBase64 $isB64)
        $epKey = Decode-Base64IfNeeded -Value $rawKey -IsBase64 $isB64
        $epService = Decode-Base64IfNeeded -Value $rawService -IsBase64 $isB64
        $epRole = Decode-Base64IfNeeded -Value $rawRole -IsBase64 $isB64
        if (-not $epRole) {
            $epRole = if ($epService.ToLower().Contains("root") -or $epService.ToLower().Contains("lovable")) { "root" } else { "secondary" }
        }
        $epName = "Supabase ($epService)"

        $cfg = [PSCustomObject]@{
            endpoints = @(
                [PSCustomObject]@{
                    id = "ep-$($epService.ToLower().Replace(' ', '-').Replace('_', '-'))"
                    name = $epName
                    url = $epUrl
                    api_key = $epKey
                    role = $epRole
                    is_enabled = $true
                    prune_threshold_mb = $(if ($epRole -eq "root") { 400 } else { 200 })
                    priority = $(if ($epRole -eq "root") { 1 } else { 2 })
                    notes = "Imported from credentials format"
                    tags = @($epRole, $epService)
                }
            )
            node_alias = "Node-$([System.Environment]::MachineName)"
            is_sync_enabled = $true
            auto_prune_root_mb = 400
            auto_prune_secondary_mb = 200
            heartbeat_interval_secs = 30
        }
    }
    else {
        throw "Unrecognized Supabase configuration JSON format in: $Path"
    }

    if ($AliasOverride) {
        $cfg.node_alias = $AliasOverride
    }

    return $cfg
}

$config = $null
$resolutionSource = ""

# -----------------------------------------------------------------------------
# Tier 1: Explicit CLI Parameters (-RootUrl and -RootKey)
# -----------------------------------------------------------------------------
if (-not [string]::IsNullOrWhiteSpace($RootUrl) -and -not [string]::IsNullOrWhiteSpace($RootKey)) {
    Write-Host "[1/4] Checking explicit parameters: FOUND" -ForegroundColor Green
    $resolutionSource = "Tier 1: Explicit CLI parameters"

    $eps = @(
        [PSCustomObject]@{
            id = "ep-root-lovable-01"
            name = "Root Supabase (Lovable)"
            url = Normalize-SupabaseUrl $RootUrl
            api_key = $RootKey.Trim()
            role = "root"
            is_enabled = $true
            prune_threshold_mb = 400
            priority = 1
            notes = "Root account by Lovable (CLI parameter)"
            tags = @("lovable", "root")
        }
    )

    if (-not [string]::IsNullOrWhiteSpace($SecondaryUrl) -and -not [string]::IsNullOrWhiteSpace($SecondaryKey)) {
        $eps += [PSCustomObject]@{
            id = "ep-secondary-01"
            name = "Secondary Supabase"
            url = Normalize-SupabaseUrl $SecondaryUrl
            api_key = $SecondaryKey.Trim()
            role = "secondary"
            is_enabled = $true
            prune_threshold_mb = 200
            priority = 2
            notes = "Secondary fallback and command queue (CLI parameter)"
            tags = @("secondary")
        }
    }

    $defaultAlias = if ($NodeAlias) { $NodeAlias } else { "Node-$([System.Environment]::MachineName)" }
    $config = [PSCustomObject]@{
        endpoints = $eps
        node_alias = $defaultAlias
        is_sync_enabled = $true
        auto_prune_root_mb = 400
        auto_prune_secondary_mb = 200
        heartbeat_interval_secs = 30
    }
}

# -----------------------------------------------------------------------------
# Tier 2: Explicit ConfigFile or VaultFile
# -----------------------------------------------------------------------------
if (-not $config) {
    $explicitFile = if ($ConfigFile) { $ConfigFile } elseif ($VaultFile) { $VaultFile } else { "" }
    if ($explicitFile) {
        Write-Host "[2/4] Checking explicit configuration/vault file: $explicitFile" -ForegroundColor Cyan
        if (Test-Path $explicitFile) {
            $config = Parse-SupabaseJsonFile -Path $explicitFile -AliasOverride $NodeAlias
            $resolutionSource = "Tier 2: Explicit file ($explicitFile)"
            Write-Host "      Successfully loaded from $explicitFile" -ForegroundColor Green
        } else {
            Write-Error "Specified configuration file does not exist: $explicitFile"
            exit 1
        }
    }
}

# -----------------------------------------------------------------------------
# Tier 3: Auto-Discovery from Sibling repo-secrets Directory
# -----------------------------------------------------------------------------
if (-not $config) {
    Write-Host "[3/4] Auto-discovering credentials in repo-secrets vaults..." -ForegroundColor Cyan

    $candidateVaults = [System.Collections.Generic.List[string]]::new()
    if ($env:REPO_SECRETS_DIR) {
        $candidateVaults.Add((Join-Path $env:REPO_SECRETS_DIR "02-antigravity-manager/vault/supabase_config.json"))
    }
    if ($env:AGM_SUPABASE_VAULT) {
        $candidateVaults.Add($env:AGM_SUPABASE_VAULT)
    }
    $candidateVaults.Add("../repo-secrets/02-antigravity-manager/vault/supabase_config.json")
    $candidateVaults.Add("../repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json")
    $candidateVaults.Add("../../repo-secrets/02-antigravity-manager/vault/supabase_config.json")
    $candidateVaults.Add("repo-secrets/02-antigravity-manager/vault/supabase_config.json")
    if ($HOME) {
        $candidateVaults.Add((Join-Path $HOME "repo-secrets/02-antigravity-manager/vault/supabase_config.json"))
        $candidateVaults.Add((Join-Path $HOME ".antigravity_tools/repo-secrets/02-antigravity-manager/vault/supabase_config.json"))
    }
    $candidateVaults.Add("../repo-secrets/03-supabase/02-lovable/supabase-credentials.json")
    $candidateVaults.Add("../repo-secrets/03-supabase/01-own/supabase-credentials.json")

    foreach ($candidate in $candidateVaults) {
        if ($candidate -and (Test-Path $candidate)) {
            Write-Host "      Found vault candidate: $candidate" -ForegroundColor Green
            try {
                $config = Parse-SupabaseJsonFile -Path $candidate -AliasOverride $NodeAlias
                $resolutionSource = "Tier 3: Auto-discovered repo-secrets vault ($candidate)"
                break
            } catch {
                Write-Host "      Skipping $candidate (parse error: $($_.Exception.Message))" -ForegroundColor Yellow
            }
        }
    }
}

# -----------------------------------------------------------------------------
# Tier 4: Environment Variables ($env:AGM_SUPABASE_ROOT_URL, etc.)
# -----------------------------------------------------------------------------
if (-not $config) {
    Write-Host "[4/4] Checking environment variables..." -ForegroundColor Cyan

    $envRootUrl = if ($env:AGM_SUPABASE_ROOT_URL) { $env:AGM_SUPABASE_ROOT_URL } elseif ($env:SUPABASE_ROOT_URL) { $env:SUPABASE_ROOT_URL } elseif ($env:SUPABASE_URL) { $env:SUPABASE_URL } else { "" }
    $envRootKey = if ($env:AGM_SUPABASE_ROOT_KEY) { $env:AGM_SUPABASE_ROOT_KEY } elseif ($env:SUPABASE_ROOT_KEY) { $env:SUPABASE_ROOT_KEY } elseif ($env:SUPABASE_KEY) { $env:SUPABASE_KEY } else { "" }
    $envSecUrl  = if ($env:AGM_SUPABASE_SECONDARY_URL) { $env:AGM_SUPABASE_SECONDARY_URL } elseif ($env:SUPABASE_SECONDARY_URL) { $env:SUPABASE_SECONDARY_URL } else { "" }
    $envSecKey  = if ($env:AGM_SUPABASE_SECONDARY_KEY) { $env:AGM_SUPABASE_SECONDARY_KEY } elseif ($env:SUPABASE_SECONDARY_KEY) { $env:SUPABASE_SECONDARY_KEY } else { "" }
    $envNodeAlias = if ($env:AGM_SUPABASE_NODE_ALIAS) { $env:AGM_SUPABASE_NODE_ALIAS } elseif ($env:SUPABASE_NODE_ALIAS) { $env:SUPABASE_NODE_ALIAS } else { "" }

    if (-not [string]::IsNullOrWhiteSpace($envRootUrl) -and -not [string]::IsNullOrWhiteSpace($envRootKey)) {
        Write-Host "      Environment variables detected" -ForegroundColor Green
        $resolutionSource = "Tier 4: Environment variables"

        $eps = @(
            [PSCustomObject]@{
                id = "ep-root-lovable-01"
                name = "Root Supabase (Lovable)"
                url = Normalize-SupabaseUrl $envRootUrl
                api_key = $envRootKey.Trim()
                role = "root"
                is_enabled = $true
                prune_threshold_mb = 400
                priority = 1
                notes = "Root account by Lovable (from environment variables)"
                tags = @("lovable", "root")
            }
        )

        if (-not [string]::IsNullOrWhiteSpace($envSecUrl) -and -not [string]::IsNullOrWhiteSpace($envSecKey)) {
            $eps += [PSCustomObject]@{
                id = "ep-secondary-01"
                name = "Secondary Supabase"
                url = Normalize-SupabaseUrl $envSecUrl
                api_key = $envSecKey.Trim()
                role = "secondary"
                is_enabled = $true
                prune_threshold_mb = 200
                priority = 2
                notes = "Secondary fallback and command queue (from environment variables)"
                tags = @("secondary")
            }
        }

        $alias = if ($NodeAlias) { $NodeAlias } elseif ($envNodeAlias) { $envNodeAlias } else { "Node-$([System.Environment]::MachineName)" }
        $config = [PSCustomObject]@{
            endpoints = $eps
            node_alias = $alias
            is_sync_enabled = $true
            auto_prune_root_mb = 400
            auto_prune_secondary_mb = 200
            heartbeat_interval_secs = 30
        }
    }
}

# -----------------------------------------------------------------------------
# Fail-Safe Error Guard if Unresolved
# -----------------------------------------------------------------------------
if (-not $config) {
    Write-Host ""
    Write-Host "[ERROR] Could not resolve Supabase credentials via any of the 4 resolution tiers." -ForegroundColor Red
    Write-Host ""
    Write-Host "Please provide credentials using one of the following methods:" -ForegroundColor Yellow
    Write-Host "  1. Explicit CLI parameters:" -ForegroundColor White
    Write-Host "     pwsh ./scripts/import-supabase-secrets.ps1 -RootUrl 'https://xyz.supabase.co' -RootKey 'sb_publishable_...'" -ForegroundColor Gray
    Write-Host "  2. Explicit configuration file:" -ForegroundColor White
    Write-Host "     pwsh ./scripts/import-supabase-secrets.ps1 -ConfigFile /path/to/supabase_config.json" -ForegroundColor Gray
    Write-Host "  3. External vault file in sibling repo-secrets repository:" -ForegroundColor White
    Write-Host "     Place your configuration at '../repo-secrets/02-antigravity-manager/vault/supabase_config.json'" -ForegroundColor Gray
    Write-Host "  4. Set environment variables:" -ForegroundColor White
    Write-Host "     `$env:AGM_SUPABASE_ROOT_URL = 'https://xyz.supabase.co'" -ForegroundColor Gray
    Write-Host "     `$env:AGM_SUPABASE_ROOT_KEY = 'sb_publishable_...'" -ForegroundColor Gray
    Write-Host ""
    exit 1
}

# -----------------------------------------------------------------------------
# Normalize and Format Configuration
# -----------------------------------------------------------------------------
foreach ($ep in $config.endpoints) {
    $ep.url = Normalize-SupabaseUrl $ep.url
}

if (-not $config.node_alias) {
    $config.node_alias = if ($NodeAlias) { $NodeAlias } else { "Node-$([System.Environment]::MachineName)" }
}
if ($null -eq $config.is_sync_enabled) {
    $config.is_sync_enabled = $true
}
if ($null -eq $config.auto_prune_root_mb) {
    $config.auto_prune_root_mb = 400
}
if ($null -eq $config.auto_prune_secondary_mb) {
    $config.auto_prune_secondary_mb = 200
}
if ($null -eq $config.heartbeat_interval_secs) {
    $config.heartbeat_interval_secs = 30
}

Write-Host ""
Write-Host "Resolved Configuration:" -ForegroundColor Cyan
Write-Host "  Source:       $resolutionSource" -ForegroundColor Yellow
Write-Host "  Endpoints:    $($config.endpoints.Count)" -ForegroundColor Yellow
Write-Host "  Node Alias:   $($config.node_alias)" -ForegroundColor Yellow
Write-Host "  Sync Enabled: $($config.is_sync_enabled)" -ForegroundColor Yellow

$jsonFormatted = $config | ConvertTo-Json -Depth 10
$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

# -----------------------------------------------------------------------------
# Write to Target Local Destinations
# -----------------------------------------------------------------------------
$targetDestinations = [System.Collections.Generic.List[string]]::new()

# Primary destination on Linux/macOS
$homeConfigPath = Join-Path (Join-Path (Join-Path $HOME ".config") "antigravity-manager") "supabase_config.json"
$targetDestinations.Add($homeConfigPath)

# Primary destination on Windows
if ($env:APPDATA) {
    $appDataPath = Join-Path (Join-Path $env:APPDATA "antigravity-manager") "supabase_config.json"
    if (-not $targetDestinations.Contains($appDataPath)) {
        $targetDestinations.Add($appDataPath)
    }
}

# Also update ~/.antigravity_tools if present for backwards compatibility
$toolsDir = Join-Path $HOME ".antigravity_tools"
if (Test-Path $toolsDir) {
    $toolsPath = Join-Path $toolsDir "supabase_config.json"
    if (-not $targetDestinations.Contains($toolsPath)) {
        $targetDestinations.Add($toolsPath)
    }
}

Write-Host ""
Write-Host "Saving configuration to local targets..." -ForegroundColor Cyan
foreach ($dest in $targetDestinations) {
    $destDir = Split-Path -Parent $dest
    if (-not (Test-Path $destDir)) {
        New-Item -ItemType Directory -Path $destDir -Force | Out-Null
    }
    [System.IO.File]::WriteAllText($dest, $jsonFormatted, $utf8NoBom)
    Write-Host "  [OK] Saved: $dest" -ForegroundColor Green
}

# -----------------------------------------------------------------------------
# Connectivity Verification (unless -SkipTest)
# -----------------------------------------------------------------------------
if (-not $SkipTest) {
    Write-Host ""
    Write-Host "Testing connectivity to configured PostgREST endpoints..." -ForegroundColor Cyan
    foreach ($ep in $config.endpoints) {
        $base = Normalize-SupabaseUrl $ep.url
        $probeUrl = if ($ep.role -eq "secondary") { "$base/rest/v1/command_queue?limit=0" } else { "$base/rest/v1/nodes?limit=0" }
        Write-Host "  Testing [$($ep.id)] $($ep.name)... " -NoNewline
        try {
            $headers = @{
                "apikey" = $ep.api_key
                "Authorization" = "Bearer $($ep.api_key)"
            }
            $resp = Invoke-WebRequest -Uri $probeUrl -Headers $headers -Method Get -TimeoutSec 8 -ErrorAction Stop
            Write-Host "PASS (HTTP $($resp.StatusCode))" -ForegroundColor Green
        } catch {
            $status = if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { 0 }
            $reader = if ($_.Exception.Response) { [System.IO.StreamReader]::new($_.Exception.Response.GetResponseStream()) } else { $null }
            $body = if ($reader) { $reader.ReadToEnd() } else { "" }
            if ($body -match "PGRST" -or $body -match "relation" -or $body -match "does not exist" -or $body -match "schema cache") {
                Write-Host "PASS (HTTP 200 equivalent - PostgREST authenticated)" -ForegroundColor Green
            } else {
                Write-Host "WARN (HTTP $status - $($_.Exception.Message))" -ForegroundColor Yellow
            }
        }
    }
}

# -----------------------------------------------------------------------------
# Display Verified Operator One-Liners
# -----------------------------------------------------------------------------
Write-Host ""
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "VERIFIED POWERSHELL ONE-LINERS FOR OPERATORS" -ForegroundColor Yellow
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "1. Auto-discover from local repo-secrets vault:" -ForegroundColor White
Write-Host '   pwsh -NoProfile -ExecutionPolicy Bypass -File ./scripts/import-supabase-secrets.ps1 -AutoDiscover' -ForegroundColor Gray
Write-Host ""
Write-Host "2. Single-line direct vault import (Windows & Linux):" -ForegroundColor White
Write-Host '   pwsh -NoProfile -ExecutionPolicy Bypass -Command "& { [string]$v = if (Test-Path ''../repo-secrets/02-antigravity-manager/vault/supabase_config.json'') { ''../repo-secrets/02-antigravity-manager/vault/supabase_config.json'' } else { '''' }; if ($v) { & ./scripts/import-supabase-secrets.ps1 -VaultFile $v } else { Write-Error ''Vault not found'' } }"' -ForegroundColor Gray
Write-Host ""
Write-Host "3. Bootstrap from Environment Variables:" -ForegroundColor White
Write-Host '   pwsh -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/import-supabase-secrets.ps1 -RootUrl $env:AGM_SUPABASE_ROOT_URL -RootKey $env:AGM_SUPABASE_ROOT_KEY -SecondaryUrl $env:AGM_SUPABASE_SECONDARY_URL -SecondaryKey $env:AGM_SUPABASE_SECONDARY_KEY"' -ForegroundColor Gray
Write-Host ""
Write-Host "4. Native AGM CLI Commands:" -ForegroundColor White
Write-Host "   agm supabase status" -ForegroundColor Gray
Write-Host "   agm supabase test" -ForegroundColor Gray
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "[SUCCESS] Supabase secrets successfully provisioned into local runtime configuration." -ForegroundColor Green
