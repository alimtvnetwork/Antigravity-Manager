# Plan 150 — Toolchain Installer: UI + profiles + SSH + more items

## Goal
A UI version of the toolchain installer (like gitmap's settings UI) in
Antigravity-Manager: users pick items/profiles in the browser UI, the
frontend calls backend endpoints (Tauri commands), progress streams back
live. Plus: more installable items, install profiles (single + stacked)
with preview for sh+ps1, troubleshooting content, SSH remote install.

## Architecture

```
[React UI: src/pages/Toolchain.tsx]
        │  invoke('toolchain_list_items')
        │  invoke('toolchain_check')
        │  invoke('toolchain_install', {items, profile, ssh_target})
        ▼
[Rust backend: src-tauri/src/commands/toolchain.rs]
  - toolchain_list_items() -> Vec<ToolchainItem {id, name, desc, installed}>
  - toolchain_check() -> CheckReport (runs script's `check`)
  - toolchain_install(...) -> spawns script, emits 'toolchain-progress' events
        │
        ▼
[scripts/install-rust-toolchain.sh v3 / .ps1]
  --profile (repeatable) --ssh --troubleshoot --list-items --check
```

## Profiles (stackable via repeat --profile)
| Profile   | Includes |
|-----------|----------|
| minimal   | rust toolchain + clippy/rustfmt only |
| rust-dev  | minimal + sccache + cargo-watch |
| frontend  | nodejs (fnm) + pnpm + tauri-cli |
| full      | rust-dev + frontend + gh |

Profile preview: `--profile full --dry-run` (or UI) prints the resolved
item list without installing. Stacking: `--profile rust-dev --profile
frontend` merges item sets.

## New installable items (v3)
| Item        | Method |
|-------------|--------|
| rust        | rustup (existing) |
| nodejs      | fnm (fast node manager) |
| sccache     | cargo install sccache |
| cargo-watch | cargo install cargo-watch |
| tauri-cli   | cargo install tauri-cli |
| gh          | apt (linux) / brew (macOS) / winget (windows) |
| pnpm        | npm i -g pnpm (after nodejs) |

## SSH remote install
`--ssh user@host`: script re-invokes itself on the remote:
`ssh -o BatchMode=yes user@host "sh -s -- <args>" < "$0"`.
Requires pre-configured SSH keys (never passwords). UI: optional
"remote host" field, backend validates `user@host` format.

## Troubleshooting (`--troubleshoot` + UI section)
- "cargo: command not found" → PATH / restart shell / `source ~/.cargo/env`
- "gdk-sys build failed" → install GTK deps (rerun without --skip-native-deps)
- "rustup update failed (offline)" → check network / proxy
- "permission denied (apt)" → rerun with sudo / root
- "toolchain mismatch" → `rustup default <ver>` or --toolchain flag

## Backend details (toolchain.rs)
- `toolchain_list_items()`: static item catalog + live `installed` detection
  (command -v probes + rustup component list).
- `toolchain_check()`: runs `sh install-rust-toolchain.sh check`, parses
  OK/MISSING lines into structured report.
- `toolchain_install(req)`: builds argv from items/profile/flags,
  spawns via `std::process::Command`, streams stdout lines as
  `toolchain-progress` events; supports optional ssh_target.
- Register in `commands/mod.rs`; add to Tauri invoke handler list.

## Frontend details (Toolchain.tsx)
- Route/page under Settings nav (follow existing Settings.tsx patterns).
- Sections: (1) Profile picker (cards with item preview),
  (2) Item checklist (individual override), (3) Options
  (force/quiet/skip-native-deps/ssh target), (4) Install button +
  live log terminal, (5) Troubleshooting accordion.
- Uses `invoke` from '@tauri-apps/api/core', listens for progress events.

## .ps1 parity
Same profiles/items/flags (`-Profile` string array, `-SshTarget`,
`-Troubleshoot`, `-ListItems`) for Windows.

## Testing
1. `bash -n` / `sh -n` on v3 script.
2. `--list-items`, `--troubleshoot`, `--profile full --dry-run` output.
3. `check` still honest on this machine.
4. `tsc --noEmit` clean; `npm run build` clean.
5. `cargo fmt -- --check` on toolchain.rs (clippy blocked: no GTK libs here).

## Release
Minor bump → 4.169.0, changelog + READMEs, tag, push, manual GitHub
Release (Actions disabled on this repo — CI cannot run there; verify
locally instead and report honestly).

## Out of scope
- scripts-fixer-v20 fixture port (separate repo, separate plan).
- Actual SSH key provisioning (user pre-configures keys).
