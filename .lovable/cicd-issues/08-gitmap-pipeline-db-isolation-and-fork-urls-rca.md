# 4-Part Root Cause Analysis: Gitmap Pipeline Database Isolation & Fork URL Realignment

## Metadata
- **Repository**: alimtvnetwork/Antigravity-Manager
- **Related Repository**: alimtvnetwork/gitmap-v28
- **Workflow**: `.github/workflows/release.yml`
- **Date**: 2026-09-16
- **Status**: Resolved

---

## 1. Symptoms

1. **Repo Workspace Pollution**:
   When running `gitmap` commands (such as pipeline status checks or error reports), Gitmap CLI generated a `.gitmap/data/pipeline.db` directory structure directly inside the target repository working tree (`d:\work\Antigravity-Manager\.gitmap\data\pipeline.db`).
2. **Missing Pipeline DB in Gitmap CLI Footer**:
   The `gitmap` command's root usage footer output under `current repo` did not report the location of the active pipeline database.
3. **Upstream URL Coupling & Fallback Drift**:
   Update checkers (`src-tauri/src/modules/update_checker.rs`), Tauri updater endpoints (`src-tauri/tauri.conf.json`), installation scripts (`install.ps1`, `install.sh`), Homebrew Cask recipes (`Casks/antigravity-tools.rb`), Arch Linux templates (`deploy/arch/PKGBUILD.template`), and web portal links referenced upstream `lbjlaq/Antigravity-Manager` with fallback mechanisms that bypassed the current fork.

---

## 2. Root Cause

1. **Local Repo Fallback in Pipeline DB Resolver**:
   In `gitmap-cli` (`cli/pipelinedb/pipeline_split_db.go`), `ResolvePipelineDbPath()` previously checked `store.FindRepoRoot()` and placed `pipeline.db` in `filepath.Join(repoRoot, ".gitmap", "data", "pipeline.db")` if a git repository was detected. This violated isolation principles by modifying monitored user workspaces.
2. **Missing Display Property in CLI Root Usage Footer**:
   In `gitmap-cli` (`cli/cmd/rootusagefooter.go`), `emitIdentityRows()` rendered repo path, slug, branch, and global store DB, but lacked an entry for the pipeline database path.
3. **Unmigrated Fork URLs**:
   Following repository forking to `alimtvnetwork/Antigravity-Manager`, core application metadata, installer scripts, and updater endpoints retained upstream hardcoded strings and fallbacks to `lbjlaq/Antigravity-Manager`.

---

## 3. Resolution

1. **Centralized Pipeline Database Storage**:
   In `d:\work\gitmap` (`cli/pipelinedb/pipeline_split_db.go`):
   - Defined `PipelineDbDir()` to strictly point to `filepath.Join(store.BinaryDataDir(), "pipeline")` (`%LOCALAPPDATA%\gitmap-cli\data\pipeline\`).
   - Defined `ResolvePipelineDbPath()` to return `filepath.Join(PipelineDbDir(), "pipeline_" + slug + ".db")` based on sanitized repo slug.
   - Removed all repository-local `.gitmap` directory creation.
2. **Surfaced Pipeline DB in Gitmap Output**:
   In `d:\work\gitmap` (`cli/cmd/rootusagefooter.go`), imported `pipelinedb` and added `● Pipeline DB: <path>` to `emitIdentityRows` under the `current repo` section.
   Compiled and deployed the new `gitmap.exe` binary to `C:\Users\Administrator\AppData\Local\gitmap-cli\gitmap.exe`.
3. **Purged Rogue Workspace Directory**:
   Deleted `d:\work\Antigravity-Manager\.gitmap`.
4. **Complete Decoupling of Fork URLs**:
   In `d:\work\Antigravity-Manager`:
   - Updated `update_checker.rs` to query `alimtvnetwork/Antigravity-Manager` across all update strategies.
   - Updated `tauri.conf.json` updater endpoint to `https://github.com/alimtvnetwork/Antigravity-Manager/releases/latest/download/updater.json`.
   - Updated `install.ps1` and `install.sh` to target `alimtvnetwork/Antigravity-Manager` without upstream fallback.
   - Updated `Casks/antigravity-tools.rb`, `PKGBUILD.template`, `install.sh`, and web portal pages.
   - Updated `version.json` `RepoSlug` and `RepoUrl`.

---

## 4. Verification & Prevention

1. **Verification**:
   - Running `gitmap` inside `d:\work\Antigravity-Manager` confirms:
     ```text
     ● Pipeline DB: C:\Users\Administrator\AppData\Local\gitmap-cli\data\pipeline\pipeline_alimtvnetwork-antigravity-manager.db
     ```
   - No `.gitmap` directory is created in `d:\work\Antigravity-Manager`.
   - All tests in `gitmap/cli/pipelinedb` pass (`go test -v ./pipelinedb/...`).
   - Clean git status in both repositories.
2. **Prevention Rule**:
   - **CLI Storage Rule**: CLI tools must never write state, telemetry, or cache databases into target project directories. All dynamic state must reside within the designated user configuration/data directory (`AppData/Local/<cli>/data/`).
   - **Fork Independence Rule**: For all forked projects, verify that updater endpoints, installation scripts, and continuous integration triggers point exclusively to the active organization repository without upstream fallbacks.
