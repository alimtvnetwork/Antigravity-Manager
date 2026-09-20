# Learned Protocol: Ruby Homebrew Cask Architecture & Ecosystem Role

## 1. Why Ruby Files Exist in the Repository

The presence of `Casks/antigravity-tools.rb` is strictly due to the package distribution format required by **Homebrew**, the primary package manager on macOS (and Linuxbrew):

1. **Homebrew's Core Architecture:**
   - Homebrew is implemented in Ruby. All Homebrew package manifests—Formulae (for command-line tools) and Casks (for graphical desktop applications, DMG installers, and AppImages)—are written in Ruby using Homebrew's declarative Domain-Specific Language (DSL).
   - The `cask "..." do ... end` syntax is an executable Ruby block interpreted by Homebrew when a user runs:
     ```bash
     brew install --cask antigravity-tools
     ```
   - There is no alternative YAML or JSON format supported by native Homebrew Cask repositories; Ruby is the mandatory format.

2. **Zero Application Runtime Involvement:**
   - The Ruby cask file has **0% involvement** in the running application.
   - The desktop client is completely powered by:
     - **Rust:** Tauri v2 backend compiled to native machine binaries (`agm-alim.exe` on Windows, `agm-alim` ELF on Linux, Mach-O on macOS).
     - **TypeScript / React 19:** Frontend UI bundle running within the OS Webview (WebView2 on Windows, WebKitGTK on Linux, WKWebView on macOS).
   - Neither Ruby nor any Ruby gem is bundled, invoked, or required at runtime or build time.

## 2. Standardized Configuration for `alimtvnetwork`

The cask manifest has been standardized to pull release binaries directly from our fork repository:
- **Repository URL:** `https://github.com/alimtvnetwork/Antigravity-Manager`
- **Release Assets:** `agm-alim_#{version}_#{arch}.dmg` (macOS) and `agm-alim_#{version}_#{arch}.AppImage` (Linux)
- **Application Bundle:** `agm-alim.app`
- **Binary Command Target:** `agm-alim`
- **Quarantine Bypass:** Postflight `/usr/bin/xattr -rd com.apple.quarantine "#{appdir}/agm-alim.app"` to ensure smooth launches without unsigned Gatekeeper quarantine blocks.

## 3. Recommended Codebase & Error Management Improvements

When inspecting upstream PR #3 changes against our coding guidelines (`02-spec/02-coding-guidelines/` and `02-spec/03-error-manage/`):

1. **Unified Error Manager Ingestion:**
   - Upstream proxy routines frequently use raw `.map_err(|e| e.to_string())` or silent fallback defaults.
   - All critical failure paths (HTTP 403 authorization failures, token expiry, upstream rate limits, and network dropouts) must funnel into `crate::error::AppError` and be registered with our frontend `ErrorManagerDrawer` (`/components/errors/`) via structured event emissions.
2. **Boolean Hygiene:**
   - Upstream occasionally uses mixed polarity expressions (e.g. `isA && !isB`). Our codebase standard enforces unmixed condition evaluations and positive variable prefixes (`has_`, `is_`).
3. **English Translation Discipline:**
   - Upstream contributions introduce Chinese comments, docstrings, and inline error messages. Our standard enforces pure English for code comments, logs, and default UI values, maintaining separate localized JSON files (`src/locales/`) for internationalization.
