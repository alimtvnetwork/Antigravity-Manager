## Quick Install v4.65.0

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

#### Pinned Version Install (v4.65.0)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/v4.65.0/install.ps1 | iex
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

#### Pinned Version Install (v4.65.0)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/v4.65.0/install.sh | bash
```

---

- **[Release v4.65.0 & Installed Apps Branding Alignment] Display Title & Publisher Finalization with Full macOS Universal Bundle Support**:
    - **Windows Installed Apps Branding Alignment**: Formatted the Windows "Installed apps" entry to display Title: **Antigravity Manager Tools** with subtitle / Publisher line: **Maintained by Alim, Sponsored by RISEUP ASIA LLC**, stamped directly through NSIS post-install hooks (`src-tauri/hooks.nsh`), `tauri.conf.json`, and `install.ps1` registry sync logic.
    - **macOS Universal Packaging Hook**: Added `scripts/before-bundle.js` and wired `"beforeBundleCommand"` in `src-tauri/tauri.conf.json`. Automatically detects `universal-apple-darwin` targets, verifies architecture compilation, and uses `lipo -create` to merge the standalone `agm` CLI binary before Tauri macOS bundler packaging, resolving packaging failures on auxiliary binaries.
    - **Explicit Default Primary Binary**: Added `default-run = "agm-alim"` in `src-tauri/Cargo.toml` to strictly identify the GUI application binary.
    - **Rust Trait Warning Suppression**: Added `#[allow(async_fn_in_trait)]` to `SystemIntegration` trait in `src-tauri/src/modules/integration.rs`, removing compiler suggestions on public trait async methods.
    - **CI/CD RCA 35 Grounded**: Authored `.ai-memory/cicd-issues/35-macos-universal-bundle-agm-missing-binary-rca.md` and registered in index.
    - **CI/CD RCA 36 Grounded**: Authored `.ai-memory/cicd-issues/36-ci-windows-setup-node-transient-hang-and-runner-concurrency-rca.md` resolving runner concurrency cache lockup.
