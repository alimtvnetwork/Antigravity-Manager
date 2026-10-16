---
name: agm-cicd-and-preflight-gates
description: Specialized skill for managing Antigravity-Manager pre-flight quality gates, Zero-CI quarantine standards, local runner workflows (03-ai-scripts/06-cicd-local-runner.py), and GitHub Actions CI/CD release pipelines.
---

# AGM CI/CD & Pre-Flight Quality Gates

This skill defines the pre-flight verification gates, quarantine isolation standards for heavy integration tests, local test runner procedures, and GitHub Actions CI/CD pipeline architecture for Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

Antigravity-Manager enforces strict pre-flight gates to guarantee zero compile errors, clean linting, and absolute safety for developer environments:

```
+----------------------------------------------------------------------------------------------------+
|                                      Pre-Flight Quality Gates                                      |
|    1. Rust Formatting Check:             cargo fmt -- --check                                      |
|    2. Rust Clippy & Compilation Gate:    cargo clippy --all-targets --all-features                 |
|    3. Frontend TypeScript & Vite Bundle: npm run build                                             |
+-------------------------------------------------+--------------------------------------------------+
                                                  │
                                                  ▼
+----------------------------------------------------------------------------------------------------+
|                            Zero-CI Quarantine Standard (Invariant 6)                               |
|    - Heavy tests spawning Electron/Antigravity processes, modifying state.vscdb, or rotating      |
|      accounts MUST be decorated with #[ignore = "local_only_e2e"]                                  |
|    - Dual Guard: Harness level (#[ignore]) + Runtime level (RUN_TEMP_E2E=1 check)                  |
|    - CI runners compile all test targets with 100% green status but NEVER execute heavy tests      |
+-------------------------------------------------+--------------------------------------------------+
                                                  │
                                                  ▼
+----------------------------------------------------------------------------------------------------+
|                        Local CI/CD Runner (03-ai-scripts/06-cicd-local-runner.py)                  |
|    - Multi-phase automated execution: Cargo fmt, Clippy, Node build, and fast unit tests           |
|    - Bounded stack traces (max 50 lines) with structured 4-part Root Cause Analysis (RCA)          |
|    - Clean temporary build caches (03-ai-scripts/19-artifact-remover.py)                           |
+-------------------------------------------------+--------------------------------------------------+
                                                  │
                                                  ▼
+----------------------------------------------------------------------------------------------------+
|                                GitHub Actions Release Pipeline (.github/)                          |
|    - Multi-OS matrix: Windows (x86_64), macOS (x86_64, aarch64), Ubuntu (x86_64)                 |
|    - scripts/before-bundle.js: Merges macOS x86_64 + aarch64 native CLI via lipo into universal agm|
|    - scripts/package_dmg.sh & Fix_Damaged.command: DMG quarantine removal and disk image bundle     |
|    - src-tauri/hooks.nsh: NSIS installer hooks and Windows registry branding                        |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Core Architectural Invariants

### Invariant 1: Comprehensive Rust Clippy Gate
Running `cargo check` alone is **strictly prohibited** as a pre-flight completion check. Developers and agents must run:
```bash
cd src-tauri && cargo clippy --all-targets --all-features
```
This single command:
1. Compiles all library targets (`antigravity_tools_lib`).
2. Compiles both binaries (`agm-alim` GUI and `agm` native CLI).
3. Compiles all test and benchmark targets without executing them.
4. Enforces strict zero-warning clippy lints across all feature flags.

### Invariant 2: Zero-CI Quarantine Standard
Any test that spawns child processes, interacts with SQLite `state.vscdb`, or touches instance sandboxes must implement the dual-layer quarantine guard:
```rust
#[test]
#[ignore = "local_only_e2e"]
fn test_local_e2e_instance_switch_and_prompt_restore() {
    // Runtime Guard: even if -- --ignored is passed, abort if RUN_TEMP_E2E != 1
    if std::env::var("RUN_TEMP_E2E").as_deref() != Ok("1") {
        eprintln!("Skipping quarantined test: RUN_TEMP_E2E != 1");
        return;
    }
    // ... test logic ...
}
```
**Why Quarantine is Mandatory:**
- Shared CI/CD runners lack GPU contexts, display servers, and real Antigravity IDE binaries.
- Running OS-modifying tests in CI causes intermittent timeouts, orphaned processes, and false-positive pipeline failures.

### Invariant 3: Host IDE Safety Protection
All test scripts (`scripts/test-instance-e2e.ps1`, `scripts/test-instance-isolation.ps1`) and CLI verification commands (`agm test-instance-flow`) must discover and protect the developer's active host IDE processes before any action:
```powershell
$protectedPids = Get-Process -Name "agm-alim", "Antigravity" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id
```
Any kill operation targeting a PID in `$protectedPids` triggers an immediate fatal abort.

### Invariant 4: Frontend Production Build Gate
Whenever files in `src/`, `package.json`, or frontend configuration files are modified, `npm run build` must be executed:
```bash
npm run build
```
This runs `tsc` (TypeScript compiler) followed by `vite build` (`--max-old-space-size=4096`), ensuring zero type mismatches, correct asset bundling, and clean production artifacts in `dist/`.

---

## 3. Pre-Flight Execution Protocol

Before creating a commit, submitting a pull request, or tagging a release, execute the following steps in sequence:

```bash
# Step 1: Rust Code Formatting
cd src-tauri && cargo fmt -- --check

# Step 2: Rust Compilation & Clippy Linting
cd src-tauri && cargo clippy --all-targets --all-features

# Step 3: Frontend TypeScript & Vite Compilation (from root)
npm run build

# Step 4 (Optional - Local Only): On-Demand Quarantined E2E Verification
$env:RUN_TEMP_E2E="1"; cd src-tauri; cargo test --lib instance test_local_e2e -- --ignored --nocapture
```

---

## 4. Local CI/CD Runner (`03-ai-scripts/06-cicd-local-runner.py`)

The repository includes an autonomous Python CI/CD runner:
```bash
# Run complete pre-flight suite locally
python 03-ai-scripts/06-cicd-local-runner.py

# Run specific phase
python 03-ai-scripts/06-cicd-local-runner.py --phase clippy
python 03-ai-scripts/06-cicd-local-runner.py --phase frontend
```

### Features:
- **Clean Execution Environment**: Automatically triggers `03-ai-scripts/19-artifact-remover.py` to purge temporary files and Cargo build artifacts when needed.
- **Bounded Error Extraction**: Limits error dumps to 50 lines, preventing context buffer flooding.
- **Root Cause Analysis (RCA)**: Automatically formats failures into standard 4-part RCA blocks (Trigger, Root Cause, Blast Radius, Verified Fix).

---

## 5. Troubleshooting Common Gate Failures

1. **`cargo fmt` failed**:
   - Run `cd src-tauri && cargo fmt` to auto-format Rust code.
2. **`cargo clippy` unused import / dead code warning**:
   - If intentional during staged refactors, use `#[allow(unused_imports)]` or prefix with `_`.
   - Remove unused variables or clean up dead code.
3. **Frontend `tsc` type error**:
   - Check `src/types/` and ensure IPC event payloads match Rust `#[derive(Serialize, Deserialize)]` DTOs.
4. **Quarantined test accidentally triggered in CI**:
   - Verify that the test function is annotated with `#[ignore = "local_only_e2e"]`.
   - Ensure GitHub Actions workflow runs `cargo test` without the `-- --ignored` flag.
