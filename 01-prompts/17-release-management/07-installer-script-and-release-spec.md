# Installer Script & Release Page Engineering Prompt

> [!IMPORTANT]
> **Prompt Role:** Standalone Installer Architect & Release Engineer  
> **Target Domains:** Cross-Platform Installation (`install.ps1`, `install.sh`), GitHub Release Generation, Resilient Fallback Ladders, Aria2c Acceleration  
> **Strict Mandate:** Never package installer scripts into release assets. Always format release page commands in separate code blocks.

---

## 1. System Topology & Architectural Mandates

When preparing, auditing, or refactoring installer scripts and release publishing workflows:

1. **Decouple Installer Scripts from Binary Assets**:
   - Standalone installer scripts (`install.ps1`, `install.sh`) must be maintained at the repository root and fetched via raw content (`raw.githubusercontent.com/<owner>/<repo>/main/...` or git tags).
   - Never push or upload installer scripts to the GitHub release page assets. Binary assets must only contain compiled binaries, packages, and checksums.
2. **Multi-Version Fallback Ladder (5 to 10 Releases)**:
   - When an installer runs, it must construct a candidate queue of up to 10 release versions (retrieved via GitHub API `/releases?per_page=30` and `/tags`, supplemented by embedded historical releases).
   - If the first/latest candidate fails (404, checksum failure, corrupted archive, broken download stream), the installer must not crash; it must automatically retreat to the previous release version and re-attempt installation.
   - The installer only fails if all 10 candidate versions fail.
3. **Explicit & Pinned Version Support**:
   - Pinned installation must work seamlessly via named parameters (`-Version`, `--version`), positional arguments, environment variables (`$env:AGM_VERSION`, `VERSION`), scriptblock execution, and raw git tag URL pattern detection.
   - Pinned installations attempt the specified tag first; if unavailable, they fall back to immediately preceding releases in the queue.
4. **Clean Aria2c Delegation & Logging**:
   - When aria2c is detected, emit an explicit delegation message:
     `Delegating download request to aria2c accelerator...`
     `Accelerating download with aria2c (16 connections, 80 splits, 1MB chunks)...`
   - Run aria2c in quiet mode (`--summary-interval=0`, `--console-log-level=error`, `--show-console-readout=false`) to eliminate noisy multi-line terminal dumps.
   - On completion, report a single-line summary with transfer size. On failure, emit a clean warning before delegating to the fallback stream (curl / Invoke-WebRequest).
5. **Separate Release Code Blocks (Direct vs Pinned)**:
   - Every published release page must format installation commands into distinct, dedicated Markdown code blocks.
   - Block 1: PowerShell Direct Latest
   - Block 2: PowerShell Pinned Version
   - Block 3: Bash Direct Latest
   - Block 4: Bash Pinned Version
   - Never combine commands into a single code block with comments.

---

## 2. Windows PowerShell Implementation Standard (`install.ps1`)

```powershell
# Precedence-based Pinned Version Resolution
function Resolve-PinnedVersion {
    param([string]$ExplicitVersion, [string]$BakedVersion)

    if (-not $ExplicitVersion) {
        if ($env:AGM_VERSION) { $ExplicitVersion = $env:AGM_VERSION }
        elseif ($env:VERSION) { $ExplicitVersion = $env:VERSION }
        elseif ($args -and $args.Count -gt 0) {
            if ($args[0] -match '^[vV]?[0-9]+\.[0-9]+') { $ExplicitVersion = $args[0] }
        }
    }
    if ($ExplicitVersion) {
        return ($ExplicitVersion -replace "^v", "").Trim()
    }
    # Detect from raw git tag URL in invocation line
    $regex = '(?i)(?:releases/download/|raw\.githubusercontent\.com/[^/]+/[^/]+/)(?:v)?([0-9]+\.[0-9]+(?:\.[0-9]+)?(?:-[a-zA-Z0-9.]+)?)/'
    foreach ($entry in (Get-InvocationHistoryCandidates)) {
        if ($entry -match $regex) {
            return $Matches[1]
        }
    }
    return $null
}

# Clean Aria2c Accelerator Delegation
function Invoke-FastDownload {
    param([string]$Url, [string]$DestinationPath)

    $aria2 = Get-Command aria2c.exe -ErrorAction SilentlyContinue
    if ($aria2) {
        Write-Step "Delegating download request to aria2c accelerator..."
        Write-Step "Accelerating download with aria2c (16 connections, 80 splits, 1MB chunks)..."
        $ariaArgs = @(
            "--disable-ipv6=true", "-x", "16", "-s", "80", "-j", "16", "-k", "1M",
            "--file-allocation=none", "--allow-overwrite=true", "--auto-file-renaming=false",
            "--summary-interval=0", "--console-log-level=error", "--show-console-readout=false",
            "-d", (Split-Path $DestinationPath), "-o", (Split-Path -Leaf $DestinationPath), "$Url"
        )
        $exitCode = (Start-Process -FilePath $aria2.Source -ArgumentList $ariaArgs -Wait -PassThru).ExitCode
        if ($exitCode -eq 0 -and (Test-Path $DestinationPath) -and (Get-Item $DestinationPath).Length -gt 0) {
            Write-Success "Download completed successfully via aria2c."
            return $true
        }
        Write-Warn "aria2c failed (exit code $exitCode); delegating download request to secondary downloader..."
    }
    # Secondary downloader: curl / Invoke-WebRequest
    return (Invoke-CurlDownload -Url $Url -DestinationPath $DestinationPath)
}

# Multi-Version Fallback Ladder (Up to 10 Releases)
$maxAttempts = 10
$attempt = 0
foreach ($candVer in $versionQueue) {
    $attempt++
    if ($attempt -gt $maxAttempts) { break }
    Write-Step "=== Installation Attempt $attempt of $maxAttempts: Release v$candVer ==="
    try {
        Install-Candidate -Version $candVer
        Write-Success "Installation of v$candVer verified successfully!"
        break
    } catch {
        Write-Warn "Attempt $attempt failed for release v$candVer: $($_.Exception.Message)"
        if ($attempt -lt $maxAttempts) {
            Write-Step "Falling back to previous release version in sequence..."
        }
    }
}
```

---

## 3. Unix & macOS Bash Implementation Standard (`install.sh`)

```bash
# Robust Argument Parsing for Pinned Versions
while [[ $# -gt 0 ]]; do
    case "$1" in
        --help|-h) show_help ;;
        --version|-v)
            if [[ $# -gt 1 && ! "$2" =~ ^-- ]]; then
                VERSION="$2"
                shift 2
            else
                echo "install.sh v2.5.0"; exit 0
            fi
            ;;
        --version=*|-v=*) VERSION="${1#*=}"; shift ;;
        v[0-9]*|[0-9]*)   VERSION="$1"; shift ;;
        *) shift ;;
    esac
done

# Aria2c Accelerator Delegation with Quiet Output
if command -v aria2c &>/dev/null; then
    info "Delegating download request to aria2c accelerator..."
    info "Accelerating download with aria2c (16 connections, 80 splits, 1MB chunks)..."
    local aria_exit=0
    aria2c --disable-ipv6=true -x 16 -s 80 -j 16 -k 1M \
        --allow-overwrite=true \
        --auto-file-renaming=false \
        --summary-interval=0 \
        --console-log-level=error \
        --show-console-readout=false \
        -d "$dest_dir" -o "$dest_file" "$url" || aria_exit=$?

    if [[ "$aria_exit" -eq 0 && -f "$full_path" && -s "$full_path" ]]; then
        success "Download completed successfully via aria2c."
        return 0
    fi
    warn "aria2c failed; delegating download request to secondary downloader (curl)..."
fi

# Multi-Version Fallback Ladder (Up to 10 Releases)
local max_attempts=10
local attempt=0
for cand_ver in "${CANDIDATE_VERSIONS[@]}"; do
    attempt=$((attempt + 1))
    if [[ $attempt -gt $max_attempts ]]; then break; fi

    step "Installation attempt $attempt of $max_attempts: Release v$cand_ver"
    if download_and_install "$cand_ver"; then
        success "Installation of v$cand_ver verified successfully!"
        break
    else
        warn "Attempt $attempt failed for release v$cand_ver."
        if [[ $attempt -lt $max_attempts ]]; then
            info "Falling back to previous release version in sequence..."
        fi
    fi
done
```

---

## 4. Release Page Formatting Specification

Every release orchestrator or GitHub Actions pipeline must generate release notes with isolated code blocks:

```markdown
## Quick Install vX.Y.Z

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/<owner>/<repo>/main/install.ps1 | iex
```

#### Pinned Version Install (vX.Y.Z)
```powershell
irm https://raw.githubusercontent.com/<owner>/<repo>/vX.Y.Z/install.ps1 | iex
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/main/install.sh | bash
```

#### Pinned Version Install (vX.Y.Z)
```bash
curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/vX.Y.Z/install.sh | bash
```

---
```

---

## 5. Verification Checklist

- [ ] Installer scripts are not present in `release-files/` and not uploaded as release assets.
- [ ] Pinned versions install accurately via arguments, env vars, and raw tag URLs.
- [ ] Aria2c delegation prints clear notices without progress bar terminal spam.
- [ ] Fallback ladder retreats through up to 10 release candidates upon failure.
- [ ] Release body renders separate code blocks for direct vs pinned commands.
