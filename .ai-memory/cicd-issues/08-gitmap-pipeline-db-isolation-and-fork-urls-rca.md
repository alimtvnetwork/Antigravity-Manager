# 4-Part Root Cause Analysis: Gitmap Pipeline Database Isolation, Deduplication & Fork URL Realignment

## Metadata
- **Repository**: alimtvnetwork/Antigravity-Manager
- **Related Repository**: alimtvnetwork/gitmap-v28
- **Workflow**: `.github/workflows/release.yml` & `.github/workflows/ci.yml`
- **Date**: 2026-09-16
- **Status**: Resolved

---

## 1. Symptoms

1. **Repo Workspace Pollution**:
   When running `gitmap` commands (such as pipeline status checks or error reports), Gitmap CLI generated `.gitmap/data/pipeline.db`, `.gitmap/pipeline/`, and `.gitmap/last_error.log` directly inside the target repository working tree (`.gitmap`).
2. **Persistent Phantom Failure Report (#35045785921)**:
   Even after release `#35067964730` (`v4.11.0`) succeeded 100% green, running `gitmap pipeline errorlogs` continued to report old failed run `#35045785921` from 5 hours prior, displaying `Saved Log: .gitmap/pipeline/35045785921.log`.
3. **CI Formatting Failure**:
   CI pipeline `#35070380603` failed on the `Check Rust formatting` step with exit code 1 due to an unformatted line in `src-tauri/src/modules/update_checker.rs:309`.
4. **Upstream URL Coupling & Fallback Drift**:
   Update checkers, Tauri updater endpoints, installation scripts, Homebrew Cask recipes, and web portal links referenced upstream `lbjlaq/Antigravity-Manager` with fallback mechanisms that bypassed the current fork.

---

## 2. Root Cause

1. **Local Repo Fallback in Pipeline DB & Log Resolvers**:
   - In `gitmap-cli` (`cli/pipelinedb/pipeline_split_db.go`), `ResolvePipelineDbPath()` previously placed `pipeline.db` in `filepath.Join(repoRoot, ".gitmap", "data", "pipeline.db")`.
   - In `gitmap-cli` (`cli/cmdpipeline/pipeline_persist.go`), `resolvePipelineDir()` returned `filepath.Join(resolveRepoRootDir(), ".gitmap", "pipeline")` and `writeLastErrorLog()` wrote directly into `repoRoot/.gitmap`.
2. **Exact-Name Workflow Mismatch on Syntax Recovery**:
   When a workflow file has a YAML syntax error (as in `#35045785921`), GitHub Actions sets `r.Name` to the workflow path (`.github/workflows/release.yml`). Once fixed, subsequent successful runs use the parsed YAML name (`Release`). Gitmap's `checkAndCollectRun` keyed `succeeded` solely on `r.Name` (`succeeded["Release"] = true`), leaving `succeeded[".github/workflows/release.yml"]` false. Consequently, older syntax failures were treated as active, unrecovered errors.
3. **Line Length Overflow in Rust Formatter**:
   Substituting `alimtvnetwork` (13 chars) for `lbjlaq` (6 chars) extended `download_url` in `update_checker.rs` to 104 characters, exceeding rustfmt's 100-column maximum and causing `cargo fmt -- --check` to reject the file.
4. **Unmigrated Fork URLs**:
   Following repository forking, core application metadata, installer scripts, and updater endpoints retained upstream hardcoded strings and fallbacks to `lbjlaq/Antigravity-Manager`.

---

## 3. Resolution

1. **Centralized Pipeline Database & Log Storage**:
   In `gitmap`:
   - `cli/pipelinedb/pipeline_split_db.go`: `PipelineDbDir()` points strictly to `filepath.Join(store.BinaryDataDir(), "pipeline")` (`%LOCALAPPDATA%\gitmap-cli\data\pipeline\`), storing `pipeline_<slug>.db`.
   - `cli/cmdpipeline/pipeline_persist.go`: `resolvePipelineDir()` returns `pipelinedb.PipelineDbDir()`, eliminating `.gitmap/pipeline`. `writeLastErrorLog` writes to the CLI data folder.
   - `cli/cmdpipeline/pipeline_sync_cache.go`: `resolveDbStatTarget()` points to `pipelinedb.ResolvePipelineDbPath(...)`.
   - `cli/cmd/rootusagefooter.go`: Added `● Pipeline DB: <path>` under the `current repo` section.
2. **Normalized Workflow Key Deduplication**:
   In `cli/cmdpipeline/pipeline_logs.go`, introduced `normalizeWorkflowKey(name)` which strips `.github/workflows/`, `workflows/`, and `.yml`/`.yaml` extensions. Both `Release` and `.github/workflows/release.yml` map to `"release"`, allowing successful runs to properly supersede earlier syntax failures.
3. **Rustfmt Multi-Line Split**:
   In `src-tauri/src/modules/update_checker.rs`, split `download_url` across two lines adhering to rustfmt column bounds.
4. **Purged Rogue Workspace Directory**:
   Deleted `.gitmap` directory.
5. **Complete Decoupling of Fork URLs & Docker Gating**:
   Re-routed all update checkers, download links, installer scripts, and cask formulas to `alimtvnetwork/Antigravity-Manager`. Gated Docker Hub build steps behind `vars.ENABLE_DOCKER_PUSH == 'true'` so missing Docker Hub credentials in forks gracefully skip instead of failing.

---

## 4. Verification & Prevention

1. **Verification**:
   - Running `gitmap pipeline errorlogs` inside repo root confirms:
     - Old failure `#35045785921` is recognized as superseded and no longer reported.
     - All logs, error reports, and database files reside in `%LOCALAPPDATA%\gitmap-cli\data\pipeline\`.
     - `Test-Path .gitmap` returns `False` (zero repo pollution).
   - Local `cargo fmt -- --check` passed 100% clean with exit code 0.
   - Remote CI run `#35071131724` passed `Check Rust formatting`, `Run Clippy`, and `Check Rust compilation` steps.
   - Release run `#35071192432` for `v4.12.0` actively building across all 6 platforms.
2. **Prevention Rule**:
   - **CLI Storage Rule**: CLI tools must never write state, telemetry, or cache databases into target project directories. All dynamic state must reside within the designated user configuration/data directory (`AppData/Local/<cli>/data/`).
   - **Workflow Normalization Rule**: Pipeline trackers must normalize workflow filenames and names (`.github/workflows/<file>.yml` -> `<file>`) to ensure syntax-failure recovery is accurately detected across run lifecycles.
