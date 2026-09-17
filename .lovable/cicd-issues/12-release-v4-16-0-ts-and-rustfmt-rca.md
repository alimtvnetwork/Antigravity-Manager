# CI/CD Issue RCA: TypeScript TS6196/TS6133/TS2304 and Rustfmt in Release v4.16.0

> **Version:** 1.0.0
> **Date:** 2026-09-17
> **Failed Runs:** [#35244093417](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35244093417) (Release), [#35244074052](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35244074052) (CI)
> **Trigger Commit:** `90a35c2c`
> **Fixed In:** `HEAD`

---

## 4-Part Root Cause Analysis (RCA)

### 1. Symptom
In GitHub Actions Release pipeline [#35244093417](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35244093417), all six matrix build jobs failed during Tauri's `beforeBuildCommand` (`npm run build` running `tsc && vite build`):
- `build-tauri (macos-latest, --target aarch64-apple-darwin)`
- `build-tauri (macos-latest, --target universal-apple-darwin)`
- `build-tauri (macos-latest, --target x86_64-apple-darwin)`
- `build-tauri (ubuntu-latest, --target aarch64-unknown-linux-gnu)`
- `build-tauri (ubuntu-latest, --target x86_64-unknown-linux-gnu)`
- `build-tauri (windows-latest, --target x86_64-pc-windows-msvc)`

Because the build phase failed, the release upload step never executed, leaving GitHub release `v4.16.0` with only generic source archive tarballs and zero compiled installers (DMG, AppImage, EXE, deb).

Concurrently, CI run [#35244074052](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35244074052) reported:
```text
src/components/accounts/AccountTable.tsx(54,24): error TS6196: 'ModelQuota' is declared but never used.
src/components/accounts/AccountTable.tsx(61,36): error TS6133: 'resolveQuotaModels' is declared but its value is never read.
src/components/accounts/AccountTable.tsx(61,56): error TS6133: 'ensurePinnedImageSelector' is declared but its value is never read.
src/pages/Settings.tsx(2,30): error TS6133: 'MessageCircle' is declared but its value is never read.
src/pages/Settings.tsx(1534,46): error TS2304: Cannot find name 'Sparkles'.
```
Additionally, `cargo fmt -- --check` failed across all platforms on 6 files:
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/modules/account.rs`
- `src-tauri/src/proxy/mappers/gemini/wrapper.rs`
- `src-tauri/src/proxy/server.rs`
- `src-tauri/src/proxy/session_manager.rs`
- `src-tauri/src/proxy/thinking_store.rs`

### 2. Root Cause
1. **Frontend TypeScript Type & Unused Variable Checks:**
   - In `src/components/accounts/AccountTable.tsx`, unused imports `ModelQuota`, `resolveQuotaModels`, and `ensurePinnedImageSelector` were retained after simplifying the quota view into unified Gemini and Claude metrics.
   - In `src/pages/Settings.tsx`, `MessageCircle` was imported from `lucide-react` but unused, while `Sparkles` was referenced in the Upstream Genesis attribution card without being imported.
2. **Rustfmt Style Divergence:**
   - Error messages in `commands/mod.rs` and `modules/account.rs` were on a single long line exceeding rustfmt line length limits.
   - Multi-condition expressions and iterator closures in `wrapper.rs`, `server.rs`, `session_manager.rs`, and `thinking_store.rs` had formatting differences compared to standard `rustfmt` formatting.

### 3. Resolution
1. **Frontend Fixes:**
   - Removed unused imports `ModelQuota`, `resolveQuotaModels`, `ensurePinnedImageSelector` from `src/components/accounts/AccountTable.tsx`.
   - Replaced unused `MessageCircle` with required `Sparkles` in `src/pages/Settings.tsx`.
2. **Backend Rustfmt Fixes:**
   - Formatted all 6 files to match standard `rustfmt` rules exactly:
     - `src-tauri/src/commands/mod.rs`: split error returns and `create_dir_all` across lines.
     - `src-tauri/src/modules/account.rs`: split `copy_dir_recursive` and `dir_is_empty` across lines.
     - `src-tauri/src/proxy/mappers/gemini/wrapper.rs`: unified boolean expression formatting.
     - `src-tauri/src/proxy/server.rs`: formatted `TcpListener::from_std` map_err block.
     - `src-tauri/src/proxy/session_manager.rs`: single-line string repeat.
     - `src-tauri/src/proxy/thinking_store.rs`: formatted iterator closure block.
3. **Pipeline Re-trigger:**
   - Recreate and push git tag `v4.16.0` to trigger the Release workflow and upload all binary assets to GitHub.

### 4. Prevention & Learnings
- **Strict Lint Verification Before Tagging:** Always ensure TypeScript unused variable checks and Rustfmt formatting pass prior to tagging release commits.
- **Unified Build Verification:** `npm run build` runs `tsc && vite build`, which fails if unused variables or missing imports exist.
