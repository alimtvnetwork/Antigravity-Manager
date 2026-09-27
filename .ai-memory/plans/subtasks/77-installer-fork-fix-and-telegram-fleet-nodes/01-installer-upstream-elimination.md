# Subtask 01: Installer Upstream Fork Elimination

## Metadata
- **Parent Plan:** `77-installer-fork-fix-and-telegram-fleet-nodes.md`
- **Status:** Completed
- **Target Files:**
  - `install.ps1`
  - `install.sh`
  - `deploy/arch/install.sh`
  - `README_EN.md`

---

## 1. Description
Eliminate all instances of `lbjlaq/Antigravity-Manager` from installation routines:
1. `install.ps1`: Delete `$UpstreamRepo = "lbjlaq/Antigravity-Manager"`, delete API endpoints querying `$UpstreamRepo`, and delete line 1306 `$UpstreamDownloadUrl = $DownloadUrl -replace ...`.
2. `install.sh`: Delete `UPSTREAM_REPO="lbjlaq/Antigravity-Manager"`, delete `UPSTREAM_API`, and delete all `${DOWNLOAD_URL//$REPO/$UPSTREAM_REPO}` fallback candidate arrays.
3. `deploy/arch/install.sh`: Delete unused `UPSTREAM_REPO` variable.
4. `README_EN.md`: Update all curl, brew, and download URLs to `alimtvnetwork/Antigravity-Manager`.

---

## 2. Verification Criteria
- [x] `git grep -i "lbjlaq"` shows 0 results in `install.ps1`, `install.sh`, `deploy/arch/install.sh`.
- [x] `install.ps1 -DryRun` confirms download targets resolve to `alimtvnetwork`.
