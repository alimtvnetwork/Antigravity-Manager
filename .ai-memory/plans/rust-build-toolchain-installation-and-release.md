# Execution Plan: Rust Build Toolchain Installation and Release

## User Request (Verbatim)
# Rust Build Toolchain Installation and Release: high priority instruction, non-negotiable task

Install the toolchain for the Rust build from the scripts fixture properly. So Cargo, Rust, these are the two things you need to install only. And then you try to build it locally to see what the issue is and try to fix. Can you please do that for me? And then finally, do a minor bump and do a release

## slug: rust-build-toolchain-installation-and-release

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/rust-build-toolchain-installation-and-release/ and enqueue plan task in .ai-memory/plans/rust-build-toolchain-installation-and-release.md (subtasks in .ai-memory/plans/subtasks/rust-build-toolchain-installation-and-release/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Install Cargo and Rust from the scripts fixture
7. Build the project locally to identify and fix any issues
8. Perform a minor version bump and release

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

---

## 1. Architecture Overview & Execution Strategy

### 1.1 Toolchain Installation from Fixture
- Script fixture: `scripts/install-rust-toolchain.ps1`
- Target items: Rust toolchain (rustup, cargo, rustc, clippy, rustfmt) only (`-Only rust` or `-Profile minimal`)
- Environment setup: Ensure `$env:USERPROFILE\.cargo\bin` is loaded into current session PATH.
- Verification: Validate `cargo --version` and `rustc --version`.

### 1.2 Local Build & Issue Resolution
- Run `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` to diagnose any compilation issues across library and binary targets.
- Fix any compilation or type check errors identified.
- Run `cargo fmt -- --check` and `cargo clippy --all-targets --all-features` in `src-tauri` as required by AGENTS.md pre-flight checks.

### 1.3 Minor Bump & Release Ceremony
- Synchronize all manifests using `npm run bump minor` (or node `scripts/bump-version.mjs minor`).
- Update `changelog.md` and `changelog_en.md` with release highlights, strictly adhering to the `@aukgit` attribution invariant: strictly `(Thanks to @aukgit)`, zero other `@...` handles.
- Synchronize release summary in `README.md` ("## 📝 更新日志") and `README_EN.md` ("## 📝 Changelog").
- Perform git push and tag `vX.Y.Z` as required.

---

## 2. Work Breakdown & Subtask Ownership

### Wave 1: Specification Authoring (A = 2 Subagents)
- **Spec 01**:
  - `02-spec/21-app/rust-build-toolchain-installation-and-release/01-architecture-spec.md`
  - `.ai-memory/plans/subtasks/rust-build-toolchain-installation-and-release/01-toolchain-and-build-diagnostics.md`
- **Spec 02**:
  - `02-spec/21-app/rust-build-toolchain-installation-and-release/02-component-spec.md`
  - `.ai-memory/plans/subtasks/rust-build-toolchain-installation-and-release/02-release-ceremony-plan.md`

### Wave 2: Execution & Remediation (A = 2 Workers)
- **Worker 01**:
  - Toolchain installation from `scripts/install-rust-toolchain.ps1 -Only rust`.
  - Path initialization and verification (`rustc --version`, `cargo --version`).
  - Diagnostic run of `cargo check` on `src-tauri`.
- **Worker 02**:
  - Code fixes for any compiler or linter errors identified by local build.
  - Verification of `cargo fmt` and `cargo clippy`.

### Phase 3: Release Ceremony & Atomic Push (Lead Agent)
- Run `npm run bump minor`.
- Audit and finalize `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md`.
- Run secrets gate and relative path checks.
- Atomic commit & push.
