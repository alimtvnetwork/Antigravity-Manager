# Subtask Implementation Plan: Pre-Flight Verification & Release Ceremony (`v4.186.0`)

> **Subtask ID:** `05-preflight-release-ceremony`  
> **Parent Slug:** `update-cli-command-and-pipeline-fixes`  
> **Spec Reference:** [02-spec/21-app/update-cli-command-and-pipeline-fixes/02-component-spec.md](../../../02-spec/21-app/update-cli-command-and-pipeline-fixes/02-component-spec.md)  
> **AI Instructions Reference:** [02-spec/21-app/update-cli-command-and-pipeline-fixes/03-ai-instructions.md](../../../02-spec/21-app/update-cli-command-and-pipeline-fixes/03-ai-instructions.md)  
> **Target Files:**
> - `package.json`
> - `src-tauri/Cargo.toml`
> - `src-tauri/tauri.conf.json`
> - `README.md`
> - `README_EN.md`
> - `changelog.md`
> - `changelog_en.md`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

Following completion of all implementation subtasks (Subtasks 01–04), the codebase must pass all mandatory pre-flight quality gates and execute the atomic release ceremony bumping the version from `v4.185.0` to `v4.186.0`.

Per `AGENTS.md` and release discipline:
- All Rust files must adhere to `cargo fmt` and pass `cargo clippy --all-targets --all-features`.
- Frontend code must build cleanly via `npm run build`.
- Release version bump must synchronize all 14 project manifests via `npm run bump patch`.
- Documentation and changelogs must attribute changes strictly to `@aukgit` (`(Thanks to @aukgit)`). No other user handles may appear.
- Stable releases require full synchronization across both `README.md` ("## 📝 更新日志") and `README_EN.md` ("## 📝 Changelog").

---

## 2. Concrete Execution Steps

### Step 1: Pre-Flight Quality Gates
Run local quality gates before initiating any version bump or release commits:
1. **Rust Formatting Gate**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
2. **Rust Clippy Gate**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
3. **Frontend Build Gate**:
   ```bash
   npm run build
   ```

### Step 2: Atomic Version Synchronization
Execute the atomic version bump script:
```bash
npm run bump patch
```
*Verifies synchronized transition from `4.185.0` to `4.186.0` across `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, and all 14 manifests.*

### Step 3: Documentation & Changelog Synchronization
Update release notes for `v4.186.0` across 4 documentation files:

1. **`changelog.md`**:
   Add section `## [v4.186.0]`:
   - Summary of prompt backup & restore across switching and profile rotation.
   - CLI flags `--conversation`, `--existing`, `--new`, `--repo`, and `qe` alias.
   - Prompt Tree View header conversation target mode selector (Current vs New Conv).
   - Strict attribution: `(Thanks to @aukgit)`.

2. **`changelog_en.md`**:
   Add English release notes matching `changelog.md` with identical `@aukgit` attribution.

3. **`README.md`**:
   Update section `## 📝 更新日志` with the `v4.186.0` release summary.

4. **`README_EN.md`**:
   Update section `## 📝 Changelog` with the `v4.186.0` release summary.

---

## 3. Invariants & Guardrails

- **Strict Attribution Invariant**: Maintainer attribution is strictly `@aukgit`. Zero other GitHub usernames or handles (`@...`).
- **Relative Paths Only**: All markdown links must use clean relative paths.
- **Thinking Cache Invalidation**: Routine release — keep `SUGGESTION_DELETE_THINKING_STORE = false` in `src/components/common/SuggestionDeleteThinkingModal.tsx`.
- **Pre-flight Before Tagging**: All compilation and linter gates must be 100% green before tagging.

---

## 4. Verification Checkpoints

1. Pre-flight checks pass with zero warnings: `cargo fmt`, `cargo clippy`, `npm run build`.
2. `package.json` displays `"version": "4.186.0"`.
3. `src-tauri/Cargo.toml` displays `version = "4.186.0"`.
4. `src-tauri/tauri.conf.json` displays `"version": "4.186.0"`.
5. Changelogs and READMEs consistently describe `v4.186.0` with `@aukgit` attribution.
