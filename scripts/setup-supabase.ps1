<#
.SYNOPSIS
    One-liner Supabase setup: imports vault keys through the AGM CLI.
.DESCRIPTION
    Thin wrapper around `agm supabase push-settings`. All key handling lives
    in the AGM CLI; this script embeds no keys and no default credentials.
    Runnable one-liner:
      pwsh -NoProfile -File ./scripts/setup-supabase.ps1 -VaultPath <vault-json-path>
.PARAMETER VaultPath
    Path to a Supabase vault JSON file, or a folder containing Supabase JSON
    files. Defaults to the AGM_SUPABASE_VAULT environment variable.
#>
[CmdletBinding()]
param (
    [Parameter(Mandatory = $false)]
    [string]$VaultPath = $env:AGM_SUPABASE_VAULT
)

if (-not $VaultPath) {
    Write-Error "VaultPath is required: pass -VaultPath <vault-json-path> or set AGM_SUPABASE_VAULT."
    exit 2
}

agm supabase push-settings --vault $VaultPath
exit $LASTEXITCODE
