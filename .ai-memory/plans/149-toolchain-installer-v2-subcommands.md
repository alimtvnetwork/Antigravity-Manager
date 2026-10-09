# Plan 149 — install-rust-toolchain.sh v2: gitmap-style subcommands

## Goal
Rework `scripts/install-rust-toolchain.sh` into a v2 CLI with gitmap-style
subcommands, full `--help` docs, and the review suggestions implemented.
Bring the `.sh` to feature parity with the Windows `.ps1`
(`-Force`/`-Quiet`/`-SkipLlvm`).

## CLI design (gitmap-style)

```
install-rust-toolchain.sh [global flags] <command> [command flags]

Commands:
  install     Install the Rust toolchain + native deps (default if omitted)
  check       Verify this machine can build — no changes made
  help        Show help (also: <command> --help)

Global flags:
  --force               Reinstall even if already present
  --quiet               Minimal output, non-interactive (CI-friendly)
  --skip-native-deps    Skip OS packages (Rust toolchain only)
  --toolchain <ver>     Toolchain to install (default: stable, or
                        src-tauri/rust-toolchain.toml channel if present)
  -h, --help            Show help and exit
```

### `install` behavior
1. Parse flags; resolve toolchain: explicit `--toolchain` > `rust-toolchain.toml`
   channel (if file exists) > `stable`.
2. OS deps (unless `--skip-native-deps`): apt set (Linux) / xcode+brew (macOS).
   Split into ESSENTIAL vs OPTIONAL lists so one bad mirror doesn't kill all.
3. rustup: download installer to temp file first (no `curl | sh`), then run
   with `-y --no-modify-path --profile minimal`.
4. `rustup toolchain install <ver> --profile minimal`; set default.
5. Components: `clippy`, `rustfmt` (project gates).
6. PATH persistence: append `~/.cargo/bin` to `~/.profile` (once).
7. Verify: print versions (machine-readable with `--quiet`).

### `check` behavior (read-only, exit code is the API)
Probes, each reported as OK/MISSING:
- `rustc --version`, `cargo --version`
- `rustup component list --installed` contains `clippy`, `rustfmt`
- Linux: `pkg-config --exists gtk+-3.0`, `webkit2gtk-4.1`, `libsoup-3.0`
- macOS: `xcode-select -p`
- Toolchain channel matches `rust-toolchain.toml` (if present)
Exit 0 = build-ready; exit 1 = list what's missing (and hint the fix command).

### `help` behavior
Top-level help lists commands + global flags (like `gitmap --help`).
`install --help` / `check --help` show command-specific help.

## Implementation notes
- POSIX `sh` compatible (keep `#!/bin/sh`), no bashisms — the current file
  claims `#!/usr/bin/env bash` but only uses POSIX features; keep it that way.
- Keep the colored output helpers; honor `--quiet` by silencing steps.
- Keep backward compat: bare invocation (no subcommand) == `install`
  (this is how `run.sh` calls it today).
- Do NOT change the `.ps1` in this plan (separate follow-up for parity).

## Testing (this machine)
1. `bash -n` syntax check.
2. `./install-rust-toolchain.sh help` and `--help` render correctly.
3. `./install-rust-toolchain.sh check` reports OK on this machine
   (toolchain installed; native GTK probes expected MISSING — honest output).
4. `./install-rust-toolchain.sh install --skip-native-deps --quiet`
   is idempotent (detects existing install, exits 0).

## Deliverables
- Rewritten `scripts/install-rust-toolchain.sh` (v2).
- Commit + push to `origin/main`.

## Out of scope (listed for user, not implemented)
Other installable toolchain pieces — see Task-04 list in chat:
Node.js LTS, pnpm, sccache, cargo-watch, Tauri CLI, GitHub CLI, `just`,
ldd/mold already covered, lldb.
