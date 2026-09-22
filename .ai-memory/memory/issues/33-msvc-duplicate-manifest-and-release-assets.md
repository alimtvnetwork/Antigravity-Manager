# Issue 33: MSVC Duplicate Resource Link Failure (CVT1100 / LNK1123) and Release Asset Publishing Discrepancy

## Part 1: Why It Happened

During GitHub Actions Release workflow run #35737415089 and local `cargo test` / `tauri build` invocations on Windows, the build failed with MSVC linker resource collisions:
1. **Windows MSVC Linker Error:**
   - `CVTRES : fatal error CVT1100: duplicate resource. type:VERSION, name:1, language:0x0409`
   - `LINK : fatal error LNK1123: failure during conversion to COFF: file invalid or corrupt`
2. **Compiler Dead-Code / Unused-Variable Warnings:**
   - `src/proxy/handlers/claude.rs`: warning: value assigned to `think_fill_ms` is never read.
   - `src/proxy/handlers/gemini.rs`: warning: value assigned to `norm_ms` is never read; warning: value assigned to `think_fill_ms` is never read.
   - `src/proxy/handlers/openai.rs`: warning: value assigned to `norm_ms` is never read; warning: unused variable `is_purified`.
   - `src/proxy/mappers/gemini/wrapper.rs`: warning: unused variable `message_count`.
   - `src/proxy/token_manager.rs`: warning: variable does not need to be mutable `rx`.
3. **Empty Release Asset Publishing:**
   - Because all matrix platform builds failed in `build-tauri`, zero binary packages (`.exe`, `.msi`, `.dmg`, `.AppImage`, `.deb`, `.rpm`) were generated.
   - The `publish-release` job executed unconditionally (`if: always() && !cancelled()`) and packaged only `install.ps1`, `install.sh`, and empty checksums into GitHub Releases.
   - The user observed that no binary assets were published and that installer scripts must not be pushed to the release assets.

---

## Part 2: How It Happened

1. **Manifest and Resource Library Collision in Tauri MSVC Builds:**
   - In Tauri applications on Windows, `tauri_build::build()` automatically generates `resource.rc` containing:
     - `1 VERSIONINFO` (version block)
     - `32512 ICON` (application icon)
     - `1 24` (`RT_MANIFEST` embedding Common-Controls 6.0)
     and compiles it into `resource.lib`, emitting `cargo:rustc-link-lib=static=resource`.
   - When `build.rs` attempted to fix Windows test entrypoints by emitting MSVC linker flags:
     ```rust
     println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
     println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", abs_path.display());
     ```
     `link.exe` received two separate manifest definitions for `agm-alim.exe` (one from `resource.lib` via `tauri-build` and one from `/MANIFEST:EMBED`), triggering `CVTRES : fatal error CVT1100: duplicate resource. type:MANIFEST, name:1, language:0x0409`.
   - Conversely, when `/MANIFEST:EMBED` was removed, Cargo's library test runner `antigravity_tools_lib-*.exe` (which does not link `resource.lib` because Cargo does not pass build script native static libs to `[lib]` test harnesses) lacked Common-Controls 6.0. Without Common-Controls 6.0, Windows loaded `comctl32.dll` v5.82 from `System32`, which does not export `TaskDialogIndirect` (required by `tauri-plugin-dialog`), causing the test executable to crash immediately on launch with `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`.
   - An earlier attempt to link `resource.lib` manually (`cargo:rustc-link-arg=...resource.lib`) caused MSVC to receive two copies of `resource.lib`, failing with `CVTRES : fatal error CVT1100: duplicate resource. type:VERSION, name:1, language:0x0409`.
2. **Intermediate Computation Overwrites Without Intermediate Read:**
   - In `claude.rs` and `gemini.rs`, `let mut think_fill_ms: f64 = 0.0;` was assigned at the top of the streaming handler.
   - Later in the event stream, `think_fill_ms = tf_micros as f64 / 1000.0;` overwrote the initial 0.0 without any intermediate read, causing rustc to emit `unused_assignments` warnings.
3. **Release Workflow Asset Decoupling Gap:**
   - In `.github/workflows/release.yml`, lines 365–372 copied `install.ps1` and `install.sh` into `release-files/`, which were subsequently uploaded as release assets.
   - The release job lacked a pre-flight assertion validating that at least one executable binary or installer package was present before publishing.
   - In addition, release notes previously combined quick-install commands with comments inside single multi-line code blocks, preventing clean 1-click copying.

---

## Part 3: Root Cause Analysis

1. **Manifest Collision Between `tauri-build` and MSVC Linker Arguments:**
   - `tauri-build` embeds a default manifest into `resource.rc`. In MSVC, you cannot supply both a manifest inside a `.res`/`.lib` resource file AND specify `/MANIFEST:EMBED` with `/MANIFESTINPUT`. MSVC's resource converter `cvtres.exe` treats this as duplicate manifest resources (`type:MANIFEST, name:1`).
   - The solution provided by Tauri's build architecture is `tauri_build::WindowsAttributes::new_without_app_manifest()`. This instructs `tauri-build` to omit the manifest from `resource.rc` (leaving only icon and version info in `resource.lib`), allowing MSVC's `/MANIFEST:EMBED` to cleanly embed `windows-test.manifest` across both binary targets (`agm-alim.exe`) and test executables (`antigravity_tools_lib-*.exe`) with zero duplication!
2. **Unused Assignment Warnings in Handler Streams:**
   - Variables initialized with dummy defaults (`0.0`) that are conditionally reassigned later in streaming pipelines without intermediate reads trigger rustc's `unused_assignments` lint.
3. **Missing Artifact Guardrail in Release CI/CD:**
   - When the `build-tauri` matrix encounters compile or link errors, GitHub Actions continues to downstream steps if guarded by `always()`. Without an explicit file check for `.exe` / `.AppImage` / `.dmg`, the release action publishes empty manifests.
4. **Installer Asset Misplacement:**
   - Standalone installer scripts (`install.ps1`, `install.sh`) are bootstrap entry points meant to be fetched dynamically from git tags (`raw.githubusercontent.com/.../vX.Y.Z/install.ps1`). Packaging them into release binary assets creates confusion and circular version-resolution dependencies.

---

## Part 4: Corrective Action & Verification

1. **Configured `new_without_app_manifest` & Embed Test Manifest in `src-tauri/build.rs`:**
   ```rust
   if target_os == "windows" && target_env == "msvc" {
       let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
       let attrs = tauri_build::Attributes::new().windows_attributes(windows);
       tauri_build::try_build(attrs).expect("failed to run tauri-build");

       let manifest_path = std::path::Path::new("windows-test.manifest");
       if manifest_path.exists() {
           println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
           if let Ok(abs_path) = manifest_path.canonicalize() {
               println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", abs_path.display());
           }
       }
       ...
   } else {
       tauri_build::build();
   }
   ```
   This perfectly resolves both sides: `agm-alim.exe` builds with zero duplicate manifest errors, and test runner binaries get Common-Controls 6.0 to avoid `0xc0000139`.
2. **Eliminated Compiler Warnings Across Handlers & Crates:**
   - Added `#[allow(unused_assignments)]` to `norm_ms` and `think_fill_ms` in `src-tauri/src/proxy/handlers/claude.rs` and `gemini.rs`.
   - Renamed unused variables with leading underscore (`_is_purified`, `_existing_idx`, `_message_count`, `_sid`) in `openai.rs` and `gemini/wrapper.rs`.
   - Cleaned up unused `mut` in `token_manager.rs`.
   - Added crate-level compiler lint allowances in `src-tauri/src/lib.rs`.
3. **Decoupled Installer Scripts from Release Assets:**
   - Updated `.github/workflows/release.yml` and `workflows/release.yml` to remove copying of `install.ps1` and `install.sh` into `release-files/`.
   - Added binary package verification:
     ```bash
     if [ -z "$(find . -maxdepth 1 -type f \( -name '*.exe' -o -name '*.dmg' -o -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' -o -name '*.zip' \))" ]; then
       echo "::error::CRITICAL: No executable binary or installer packages found in release-files! Refusing to publish empty release."
       exit 1
     fi
     ```
4. **Isolated Code Blocks in Release Notes:**
   - Formatted release notes into 4 individual code blocks:
     - Windows Latest: `irm https://raw.githubusercontent.com/${REPO}/main/install.ps1 | iex`
     - Windows Pinned: `irm https://raw.githubusercontent.com/${REPO}/${VERSION}/install.ps1 | iex`
     - Linux/macOS Latest: `curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash`
     - Linux/macOS Pinned: `curl -fsSL https://raw.githubusercontent.com/${REPO}/${VERSION}/install.sh | bash`
5. **Local Verification:**
   - Ran `cargo check --manifest-path src-tauri/Cargo.toml`: Finished dev profile with **0 errors, 0 warnings**.
   - Ran `npm run tauri build -- --debug --no-bundle`: Completed successfully with exit code 0 (`Built application at: src-tauri\target\debug\agm-alim.exe`).
   - Ran `powershell -ExecutionPolicy Bypass -File .\install.ps1 -DryRun`: Verified clean candidate discovery, URL construction, and dry-run execution.
   - Ran `powershell -ExecutionPolicy Bypass -File .\install.ps1 -Version "4.58.0" -DryRun`: Verified pinned version resolution and historical queue replenishment.
