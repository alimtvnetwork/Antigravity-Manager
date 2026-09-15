# CI Pipeline Workflow Specification

## Overview

The Continuous Integration (CI) pipeline for **Antigravity-Manager** (`.github/workflows/ci.yml`) validates every push and pull request targeting the `main` and `master` branches. It enforces rigorous quality gates across the React 19 / TypeScript frontend, native Rust backend, and cross-platform Tauri desktop application across Linux, Windows, and macOS.

---

## Trigger and Concurrency

### Trigger Configuration

```yaml
on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]
```

### Concurrency Control

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

Superseded runs on active branches are automatically cancelled to conserve CI runner capacity and prevent queue buildup.

---

## Pipeline Job Topology

```
┌────────────────────────────────────────────────────────────────────────┐
│                        GitHub Actions CI Workflow                      │
├──────────────────────┬─────────────────────────┬───────────────────────┤
│    build-frontend    │       check-rust        │      build-tauri      │
│  (ubuntu-latest)     │ (ubuntu/windows/macos)  │(ubuntu/windows/macos) │
│                      │                         │                       │
│ 1. Node 20 Setup     │ 1. OS Native Libs (apt) │ 1. OS Native Libs     │
│ 2. npm install       │ 2. Rust Stable Setup    │ 2. Rust Stable Setup  │
│ 3. npx tsc --noEmit  │ 3. rust-cache           │ 3. Node 20 + npm deps │
│ 4. npm run build     │ 4. cargo fmt --check    │ 4. Disable updater    │
│                      │ 5. cargo clippy         │ 5. Tauri build (debug)│
│                      │ 6. cargo check          │                       │
│                      │ 7. cargo test (backend) │                       │
└──────────────────────┴─────────────────────────┴───────────────────────┘
```

---

## 1. Job: Frontend Verification (`build-frontend`)

Runs on `ubuntu-latest` with a 20-minute timeout. Validates TypeScript types and Vite production bundling.

### Verification Steps

1. **Checkout Repository:** `actions/checkout@v4`
2. **Node.js Setup:** `actions/setup-node@v4` with Node 20 and npm caching
3. **Dependency Installation:**
   ```bash
   npm install --legacy-peer-deps
   ```
4. **TypeScript Typecheck Gate:**
   ```bash
   npx tsc --noEmit
   ```
   Ensures zero type errors, strict interface contracts with Tauri IPC commands, and valid JSX typing.
5. **Frontend Production Build Gate:**
   ```bash
   npm run build
   ```
   Compiles React 19 frontend assets into `dist/` via Vite, validating tree-shaking, CSS token bindings, and asset compression.

---

## 2. Job: Rust Backend Verification (`check-rust`)

Runs across a 3-platform matrix (`ubuntu-latest`, `windows-2025`, `macos-latest`) with a 30-minute timeout. Validates code style, linter compliance, type safety, and unit test execution.

### Verification Steps

1. **Checkout Repository:** `actions/checkout@v4`
2. **System Dependencies (Linux):** Installs WebKitGTK 4.1, GTK3, AppIndicator3, OpenSSL, and NetworkManager headers.
3. **Rust Toolchain:** `dtolnay/rust-toolchain@stable`
4. **Dependency Caching:** `Swatinem/rust-cache@v2` scoped to `src-tauri` workspace
5. **Code Formatting Gate:**
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
   Guarantees uniform code formatting matching standard Rust style rules.
6. **Clippy Linter Gate:**
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
   Guarantees zero warnings on dead code, unhandled conditions, and anti-patterns.
7. **Compilation Check Gate:**
   ```bash
   cd src-tauri && cargo check
   ```
   Validates fast compilation across all target architectures.
8. **Automated Backend Test Gate:**
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```
   Executes native Rust unit tests and integration tests for proxy mappers, session managers, quota parsers, and database models.

---

## 3. Job: Tauri Desktop Build (`build-tauri`)

Runs across the 3-platform matrix (`ubuntu-latest`, `windows-2025`, `macos-latest`) with a 45-minute timeout. Verifies end-to-end linking of Rust backend with compiled frontend.

### Verification Steps

1. **System & Toolchain Setup:** Installs native WebKit/GTK libraries, stable Rust, and Node 20.
2. **Frontend Dependencies:** `npm install --legacy-peer-deps`
3. **CI Config Sanitization:** Modifies `tauri.conf.json` via PowerShell to disable updater signing artifacts for unsigned CI builds:
   ```pwsh
   $config = Get-Content -Raw "src-tauri/tauri.conf.json" | ConvertFrom-Json
   $config.bundle.createUpdaterArtifacts = $false
   $config | ConvertTo-Json -Depth 32 | Set-Content -Path "src-tauri/tauri.conf.json" -Encoding UTF8
   ```
4. **Tauri Debug Build Gate:**
   ```bash
   npm run tauri build -- --debug --no-bundle
   ```
   Verifies that the Tauri IPC command registry, native dependencies, and static web assets bundle without linker errors.

---

## Quality Gates & Failure Handling

| Gate | Command | Failure Impact |
|---|---|---|
| **Frontend Typecheck** | `npx tsc --noEmit` | Blocks merge on invalid types or missing imports |
| **Frontend Asset Build** | `npm run build` | Blocks merge on bundle syntax errors or broken assets |
| **Rust Formatting** | `cargo fmt -- --check` | Blocks merge on unformatted code |
| **Rust Linting** | `cargo clippy --all-targets --all-features` | Blocks merge on lint warnings or unsafe idioms |
| **Rust Compilation** | `cargo check` | Blocks merge on compiler errors |
| **Rust Backend Tests** | `cargo test --manifest-path src-tauri/Cargo.toml` | Blocks merge on failing unit or integration tests |
| **Tauri App Build** | `npm run tauri build -- --debug --no-bundle` | Blocks merge on packaging or IPC compilation errors |

---

## Pipeline Rules & Constraints

1. **Never Disable CI/CD:** Workflows, jobs, and steps must never be commented out or bypassed to force a build to pass.
2. **Zero False Positives:** All checks must execute against real code and exit with return code 0.
3. **Matrix Isolation:** `fail-fast: false` ensures full multi-platform telemetry across Linux, Windows, and macOS.
4. **Deterministic Dependencies:** Uses pinned Node 20 and stable Rust toolchains with workspace caching.
5. **Strict Relative Paths:** All specification citations and build scripts must reference repository paths relatively.

---

*CI Pipeline Workflow Specification — Antigravity-Manager v4.7.0*
