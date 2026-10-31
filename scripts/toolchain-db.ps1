<#
.SYNOPSIS
    PowerShell SQLite Toolchain Tracker & Fast Verification Module.

.DESCRIPTION
    Interacts with the toolchain SQLite database (toolchain.db) using the companion
    Python engine (scripts/toolchain_db.py) to record installation metadata and
    perform sub-5ms preflight verification.
#>

[CmdletBinding()]
param()

$Script:ToolchainDbPy = Join-Path $PSScriptRoot "toolchain_db.py"

function Get-ToolchainDbPath {
    <#
    .SYNOPSIS
        Resolves the physical file path of toolchain.db.
    #>
    if ($env:AGM_TOOLCHAIN_DB) {
        return $env:AGM_TOOLCHAIN_DB
    }
    $localApp = [Environment]::GetFolderPath("LocalApplicationData")
    if ($localApp) {
        $targetDir = Join-Path $localApp "agm-toolchain"
        if (-not (Test-Path $targetDir)) {
            New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
        }
        return (Join-Path $targetDir "toolchain.db")
    }
    return (Join-Path $PSScriptRoot "..\toolchain.db")
}

function Initialize-ToolchainDb {
    <#
    .SYNOPSIS
        Initializes the SQLite schema in toolchain.db.
    #>
    param([string]$DbPath = "")
    $db = if ($DbPath) { $DbPath } else { Get-ToolchainDbPath }
    if (Test-Path $Script:ToolchainDbPy) {
        $py = (Get-Command python -ErrorAction SilentlyContinue)
        if ($py) {
            & python $Script:ToolchainDbPy init --db $db 2>$null | Out-Null
            return $true
        }
    }
    return $false
}

function Set-ToolchainRecord {
    <#
    .SYNOPSIS
        Records or updates a toolchain installation in toolchain.db.
    #>
    param(
        [Parameter(Mandatory=$true)][string]$ToolchainName,
        [Parameter(Mandatory=$true)][string]$Version,
        [Parameter(Mandatory=$true)][string]$InstalledPath,
        [string]$InstallDrive = "",
        [bool]$IsVerified = $true,
        [string]$DbPath = ""
    )
    $db = if ($DbPath) { $DbPath } else { Get-ToolchainDbPath }
    $normDrive = if ($InstallDrive) { $InstallDrive } else {
        if ($InstalledPath -match '^[a-zA-Z]:') { $Matches[0].ToUpper() } else { "C:" }
    }

    if (Test-Path $Script:ToolchainDbPy) {
        $py = (Get-Command python -ErrorAction SilentlyContinue)
        if ($py) {
            $cmdArgs = @(
                $Script:ToolchainDbPy,
                "upsert",
                "--name", $ToolchainName,
                "--version", $Version,
                "--path", $InstalledPath,
                "--drive", $normDrive,
                "--db", $db
            )
            if ($IsVerified) {
                $cmdArgs += "--verified"
            } else {
                $cmdArgs += "--unverified"
            }
            try {
                & python @cmdArgs | Out-Null
                return $true
            } catch {
                Write-Verbose "Toolchain DB upsert warning: $_"
                return $false
            }
        }
    }
    return $false
}

function Get-ToolchainRecord {
    <#
    .SYNOPSIS
        Retrieves status of a toolchain component from toolchain.db.
    #>
    param(
        [Parameter(Mandatory=$true)][string]$ToolchainName,
        [string]$DbPath = ""
    )
    $db = if ($DbPath) { $DbPath } else { Get-ToolchainDbPath }
    if (-not (Test-Path $db)) { return $null }

    if (Test-Path $Script:ToolchainDbPy) {
        $py = (Get-Command python -ErrorAction SilentlyContinue)
        if ($py) {
            try {
                $raw = & python $Script:ToolchainDbPy verify --name $ToolchainName --db $db 2>$null
                if ($raw) {
                    return ($raw | ConvertFrom-Json -ErrorAction SilentlyContinue)
                }
            } catch {
                return $null
            }
        }
    }
    return $null
}

function Test-ToolchainDbFast {
    <#
    .SYNOPSIS
        Performs sub-5ms verification against toolchain.db and tests file existence on disk.
    #>
    param(
        [Parameter(Mandatory=$true)][string]$ToolchainName,
        [string]$ExpectedFile = "",
        [string]$DbPath = ""
    )
    $db = if ($DbPath) { $DbPath } else { Get-ToolchainDbPath }
    if (-not (Test-Path $db)) { return $false }

    if (Test-Path $Script:ToolchainDbPy) {
        $py = (Get-Command python -ErrorAction SilentlyContinue)
        if ($py) {
            $cmdArgs = @($Script:ToolchainDbPy, "verify", "--name", $ToolchainName, "--db", $db)
            if ($ExpectedFile) {
                $cmdArgs += @("--file", $ExpectedFile)
            }
            try {
                $raw = & python @cmdArgs 2>$null
                if ($LASTEXITCODE -eq 0 -and $raw) {
                    $obj = ($raw | ConvertFrom-Json -ErrorAction SilentlyContinue)
                    return [bool]($obj -and $obj.is_verified)
                }
            } catch {
                return $false
            }
        }
    }
    return $false
}

function Test-AllToolchainsFast {
    <#
    .SYNOPSIS
        Verifies whether all core build toolchains (rust, msvc, llvm, nasm) are verified and present.
    #>
    param(
        [string]$Tools = "rust,msvc,llvm,nasm",
        [string]$DbPath = ""
    )
    $db = if ($DbPath) { $DbPath } else { Get-ToolchainDbPath }
    if (-not (Test-Path $db)) { return $false }

    if (Test-Path $Script:ToolchainDbPy) {
        $py = (Get-Command python -ErrorAction SilentlyContinue)
        if ($py) {
            & python $Script:ToolchainDbPy check-all --tools $Tools --db $db 2>$null | Out-Null
            return ($LASTEXITCODE -eq 0)
        }
    }
    return $false
}
