# Issue 20: Installer Upstream Fork Inversion & Fallback Binary Injection RCA

## Executive Summary
During installer execution across Windows (`install.ps1`), Linux/macOS (`install.sh`), and Arch Linux (`deploy/arch/install.sh`), installation scripts unexpectedly downloaded and deployed release assets from the original upstream repository `lbjlaq/Antigravity-Manager` rather than the enhanced production repository `alimtvnetwork/Antigravity-Manager`. As a consequence, machines provisioned with these installers received stale upstream binaries lacking all enterprise additions (Multi-Instance Manager, Split SQLite Databases, Email Inbound Watcher, Telegram Bot Control Center, Auto-Switcher, and GitMap Cluster integration).

---

## 1. Symptoms & Incident Walkthrough
1. **User Ran Clean Installation:** An operator executed the standard install one-liner on a new machine.
2. **Installation Completed Successfully:** The installer script completed without errors, reporting successful deployment.
3. **Stale Binary Deployed:** Upon launching the application, the UI and CLI displayed upstream Chinese fork components without multi-instance profiles, auto-switch, telegram bot, email watcher, or gitmap integration.
4. **User Investigation:** Code inspection of `install.ps1` and `install.sh` revealed that fallback logic actively redirected asset requests to `https://github.com/lbjlaq/Antigravity-Manager/releases`.

---

## 2. Root Cause Analysis (4-Part RCA)

### 2.1 Why Did Installers Contain `lbjlaq/Antigravity-Manager` References?
- When the fork was initially created from upstream `lbjlaq/Antigravity-Manager`, developers left an `$UpstreamRepo = "lbjlaq/Antigravity-Manager"` variable in `install.ps1`, `install.sh`, and `deploy/arch/install.sh` under the mistaken assumption that if `alimtvnetwork` lacked a particular pre-built binary, falling back to upstream would provide a working base.
- However, `alimtvnetwork/Antigravity-Manager` had diverged completely from upstream, introducing breaking database schemas, IPC endpoints, CLI tools (`agm`), and multi-account managers. Upstream binaries are completely incompatible with our features.

### 2.2 Why Did the Windows Installer (`install.ps1`) Execute the Upstream Fallback?
- In `install.ps1`:
  - Lines 840–841 and 1125–1126 included `$UpstreamRepo` in `$apiEndpoints`:
    ```powershell
    $apiEndpoints = @(
        "https://api.github.com/repos/$Repo/releases",
        "https://api.github.com/repos/$Repo/releases/latest",
        "https://api.github.com/repos/$UpstreamRepo/releases",
        "https://api.github.com/repos/$UpstreamRepo/releases/latest"
    )
    ```
    If rate-limited or if an asset tag lookup failed on `alimtvnetwork`, the script queried `lbjlaq` releases.
  - Crucially, on line 1306:
    ```powershell
    if (-not $downloaded) {
        Write-Warn "Primary download failed. Attempting upstream fallback..."
        $UpstreamDownloadUrl = $DownloadUrl -replace [regex]::Escape($Repo), $UpstreamRepo
        $downloaded = Invoke-FastDownload -Url $UpstreamDownloadUrl -DestinationPath $DownloadedFile
    }
    ```
    If any network hiccup or asset mismatch occurred during the primary download, the script actively string-replaced `alimtvnetwork` with `lbjlaq` and downloaded the upstream fork binary!

### 2.3 Why Did the Linux/macOS Installer (`install.sh`) Invert the Package Source?
- In `install.sh`:
  - Line 22: `UPSTREAM_REPO="lbjlaq/Antigravity-Manager"`
  - Line 517: `api_urls+=("${GITHUB_API}?per_page=30" "${UPSTREAM_API}?per_page=30")`
  - Lines 738–740:
    ```bash
    local candidate_urls=(
        "$DOWNLOAD_URL"
        "${DOWNLOAD_URL//$REPO/$UPSTREAM_REPO}"
    )
    ```
  - Line 749:
    ```bash
    candidate_urls+=("https://github.com/${UPSTREAM_REPO}/releases/download/v${FALLBACK_STABLE_VERSION}/${fallback_file}")
    ```
  - If a package wasn't cached or encountered a transient 404, `download_installer` looped through `candidate_urls` and downloaded the upstream package from `lbjlaq`.

### 2.4 Why Did Documentation Point to Upstream Repositories?
- In `README_EN.md`, several installation commands had not been updated after the repo migration:
  - Line 177: `curl -sSL https://raw.githubusercontent.com/lbjlaq/Antigravity-Manager/main/deploy/arch/install.sh | bash`
  - Line 182: `brew tap lbjlaq/antigravity-manager https://github.com/lbjlaq/Antigravity-Manager`
  - Line 190: `Download from GitHub Releases (https://github.com/lbjlaq/Antigravity-Manager/releases)`

---

## 3. Corrective Measures & Architectural Invariants

### 3.1 Complete Elimination of Upstream Fork Variables
- Delete `$UpstreamRepo` in `install.ps1`.
- Delete `UPSTREAM_REPO` and `UPSTREAM_API` in `install.sh`.
- Delete `UPSTREAM_REPO` in `deploy/arch/install.sh`.
- Only `alimtvnetwork/Antigravity-Manager` may ever be contacted for manifests, release metadata, tag lists, or package binaries.

### 3.2 Elimination of Upstream Download Fallbacks
- In `install.ps1`: Delete the `$UpstreamDownloadUrl` replacement block. If primary download fails, fail gracefully with clear diagnostic output pointing only to `alimtvnetwork/Antigravity-Manager`.
- In `install.sh`: Delete `${DOWNLOAD_URL//$REPO/$UPSTREAM_REPO}` and all candidate URLs referencing `UPSTREAM_REPO`.
- In `deploy/arch/install.sh`: Ensure all asset generation points strictly to `$REPO`.

### 3.3 Documentation URL Sanitization
- Update all curl, brew, and download URLs in `README_EN.md` to reference `alimtvnetwork/Antigravity-Manager`. Preserve author attribution in dedicated credits sections without linking active install flows to upstream forks.

---

## 4. Verification Gate
- [x] Run `Select-String -Path install.ps1, install.sh, deploy/arch/install.sh -Pattern "lbjlaq"` and verify 0 active download/API references remain.
- [x] Test `install.ps1 -DryRun` to ensure resolved download URLs point exclusively to `https://github.com/alimtvnetwork/Antigravity-Manager`.
- [x] Verify markdown documentation links across `README_EN.md`.
