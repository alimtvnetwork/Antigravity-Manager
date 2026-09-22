# Consolidated Plan: Rust/Cargo/LLVM Toolchain Installers, Cross-Platform Runners, Scripts-Fixer Dev-Rust Profile & Account Switch Email/Telegram Notifications

> **Origin:** User request to provide cargo/LLVM toolchain installers (PowerShell & Unix/macOS), cross-platform application runners (`run.ps1`, `run.sh`), a distinct `Dev Rust` profile in `scripts-fixer` (after `git pull`), and unified Email & Telegram notifications for both manual and auto account switching.
> **Total Steps / Loops:** 6 discrete subtasks executed across 1 continuous orchestration loop.
> **Status:** COMPLETED

---

## 1. Verbatim Request & Actionable Task Traceability

```text
Can you please create a cargo installer PowerShell file into the repository? Cargo, LLVM, and other stuff, the toolchain that is required to build this. Can you please do that so that you can test stuff here? And also create the Ubuntu version for the Ubuntu or the Unix version for the installation or Mac version for the installation as well for this repository. Also create a run.ps1 file or a run.cs and run.sh shell file that would actually run the code directly here by using every tool that we have. That's another thing. Also, I want you to add the installation stuff to the scripts fixture as well, okay, as a new item. Do not add this cargo to the dev profile because it seems like it's very big. It should be separate, like dev Rust. Okay, so this is where this LLVM and the cargo will stand. So user can install it separately, these two items. Yeah. So before you make changes to scripts fixture, I want you to do a Git pull. That's must, and then you do that. Is it understood? Can you please help me with this? And also remember, the account switching, let's say the one that needs to switch. Once it does that, it should also send an email if the email recipient is added. Same goes for the Telegram and other stuff when the Telegram is added. Do you understand? How confident are you on the Telegram stuff? Can you check your code?
```

- **Task-01:** Synchronize repositories via `git pull origin main` (Antigravity-Manager & scripts-fixer).
- **Task-02:** Windows Rust, Cargo & LLVM toolchain installer (`scripts/install-rust-toolchain.ps1`).
- **Task-03:** Unix/macOS Rust, Cargo & LLVM toolchain installer (`scripts/install-rust-toolchain.sh`).
- **Task-04:** Cross-platform application runners (`run.ps1` and `run.sh`).
- **Task-05:** Scripts-fixer toolchain item `110` (LLVM & Clang) and dedicated `dev Rust` profile.
- **Task-06:** Multi-channel Email & Telegram account switch notifications (`notification_hub.rs`).
- **Task-07:** Verification, consolidation, and atomic git commit & push.

---

## 2. Implemented Components Summary

### A. Windows Toolchain Installer (`scripts/install-rust-toolchain.ps1`)
- Detects existing `rustc`, `cargo`, and `clang`.
- Downloads and non-interactively installs `rustup-init.exe` with default toolchain `stable-x86_64-pc-windows-msvc`.
- Installs LLVM & Clang via Winget (`LLVM.LLVM`) with Chocolatey (`llvm`) fallback.
- Detects Visual Studio C++ Build Tools (`vswhere.exe` / `link.exe`).
- Refreshes session `$env:Path` and provides clear colored verification outputs.

### B. Unix/Linux/macOS Toolchain Installer (`scripts/install-rust-toolchain.sh`)
- Identifies Linux (Ubuntu/Debian) vs macOS (Darwin).
- On Ubuntu/Debian: Installs `build-essential`, `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`, `pkg-config`, `clang`, `llvm`, `mold`.
- On macOS: Checks and installs Xcode Command Line Tools and Homebrew LLVM.
- Installs `rustup` via official curl pipeline if missing, sources `~/.cargo/env`, and validates compilers.

### C. Cross-Platform Application Runners (`run.ps1`, `run.sh`)
- Inspects Node.js, npm, and Rust toolchain prerequisites.
- Automatically prompts or invokes the toolchain installer if rustc/cargo are absent.
- Automatically installs `node_modules` via `npm install --legacy-peer-deps` if missing.
- Dispatches execution modes:
  - Default: `npm run tauri dev`
  - `-Build`: `npm run tauri build`
  - `-Debug` / `-DebugMode`: `RUST_LOG=debug npm run tauri dev`
  - `-FrontendOnly`: `npm run dev`
  - `-InstallToolchain`: Runs toolchain installer script directly.

### D. Scripts-Fixer Toolchain Item & Dev Rust Profile
- `registry.yaml`: Registered script `110` (`110-install-llvm`, phase `05`, title: `LLVM & Clang compiler toolchain`).
- `scripts/110-install-llvm/`: Created `manifest.json`, `config.json`, `log-messages.json`, `helpers/llvm.ps1`, and `run.ps1`.
- `scripts/12-install-all-dev-tools/config.json`:
  - Isolated Cargo (44) from general `Dev Runtimes` profile.
  - Added dedicated `Dev Rust` profile (`{ "letter": "rs", "label": "Dev Rust (44,110)", "ids": ["44", "110"] }`).
  - Synchronized via `node tools/registry-sync.cjs`.
  - Pushed to `github.com:alimtvnetwork/scripts-fixer-v20` on `main`.

### E. Unified Account Switch Notifications (`notification_hub.rs`)
- Coordinates unified notification dispatch upon account switching:
  - **Email**: Dispatches HTML notification with host telemetry, instance name, target email, switch trigger, and reason to all active recipients if email notifications are enabled.
  - **Telegram**: Dispatches formatted HTML message via Telegram Bot API (`https://api.telegram.org/bot<token>/sendMessage`) to `allowed_chat_id` if bot token is configured and enabled.
- Integrated into:
  - `auto_switcher::execute_profile_rotation` (auto-switching on quota threshold or period boundary)
  - `instance::switch_account_to_instance` (manual instance binding & launch)
  - `account::switch_account` (core account switch logic)
- Added unit tests for label formatting and non-blocking asynchronous execution.
