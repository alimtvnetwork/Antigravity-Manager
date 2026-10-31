# Subtask 08: Pre-Flight Verification, Minor Version Bump, and Release Ceremony

**Module:** `cli-and-ui-testing-for-antigravity-backups`
**Version:** 4.239.0
**Updated:** 2026-10-20
**Status:** Ready for Execution

---

## 1. Context & Objectives

This subtask defines the rigorous quality gate verification, atomic version synchronization, documentation alignment, and release ceremony for the multimodal running prompt backup, restoration, and CLI testing capabilities.

### Core Targets:
- Pre-Flight Quality Gates: `cargo fmt`, `cargo clippy`, `npm run build`
- Version Bump Automation: `scripts/bump-version.mjs` via `npm run bump minor`
- Documentation Artifacts: `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md`

---

## 2. Technical Invariants & Guardrails

1. **Strict Attribution Invariant**:
   - Attribution in `changelog.md` and `changelog_en.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - Under NO circumstance may any other GitHub handle (`@...`) appear in release changelogs or release notes.
2. **Synchronize README Changelog**:
   - Every stable release MUST update both `README.md` (under `## 📝 更新日志`) and `README_EN.md` (under `## 📝 Changelog`). Updating only `changelog.md` is strictly forbidden.
3. **Strictly Relative Git Paths Invariant**:
   - Zero absolute paths (e.g. `D:\...`, `C:\...`, `/home/...`) in commit messages, changelogs, READMEs, specs, or plans.
4. **Pre-flight Gate Invariant**:
   - Zero warnings or errors in `cargo clippy --all-targets --all-features`.
   - Zero formatting differences in `cargo fmt -- --check`.
   - Zero build errors in `npm run build`.

---

## 3. Step-by-Step Execution Plan

### Step 1: Pre-Flight Quality Gates
Run the three essential local pre-flight gates before bumping versions:
1. Rust formatting verification:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
2. Comprehensive Rust clippy gate:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
3. Frontend TypeScript check and production build:
   ```bash
   npm run build
   ```

### Step 2: Atomic Version Synchronization
Execute the atomic version bump script for a minor release:
```bash
npm run bump minor
```
This updates 14 manifest files synchronously:
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `version.json`
- Core documentation version headers

### Step 3: Changelog Authoring with Strict `@aukgit` Attribution
Update `changelog.md` and `changelog_en.md` with the new version section:

#### `changelog.md` Example:
```markdown
## [4.239.0] - 2026-10-20

### 新增与优化
- **多模态运行中提示词备份与恢复**:
  - 支持 Base64 格式图像（`data:image/`）与本地磁盘图片路径提取与无损恢复 (Thanks to @aukgit)。
  - 自动提取并持久化截图关联标识（`SS_ID_*`）(Thanks to @aukgit)。
  - 提示词预览词数限制配置（`prompt_preview_words`），支持 100 至 200 词调节 (Thanks to @aukgit)。
- **CLI 与 UI 深度对齐**:
  - 新增 `agm running`、`agm backup running` 与 `agm restore running` 原生命令行指令 (Thanks to @aukgit)。
  - 统一 `[AGM:P001 | GM:#1]` 双序列标识与终端格式化高亮 (Thanks to @aukgit)。
- **自动化端到端测试套件**:
  - 引入 6 关卡 PowerShell 端到端测试脚本（`scripts/test-agm-gitmap-running-backup-e2e.ps1`），自带会话 PID 保护与零残留环境隔离 (Thanks to @aukgit)。
```

#### `changelog_en.md` Example:
```markdown
## [4.239.0] - 2026-10-20

### Added & Improved
- **Multimodal Running Prompt Backup & Restoration**:
  - Full support for Base64 image URIs (`data:image/`) and disk file references across backup and restore cycles (Thanks to @aukgit).
  - Screenshot token detection and persistence (`SS_ID_*`) (Thanks to @aukgit).
  - Configurable prompt preview word limit (`prompt_preview_words`) with a slider range of 100 to 200 words (Thanks to @aukgit).
- **CLI & UI Alignment**:
  - Added native `agm running`, `agm backup running`, and `agm restore running` commands (Thanks to @aukgit).
  - Unified dual sequence identifiers `[AGM:P001 | GM:#1]` and formatted terminal previews (Thanks to @aukgit).
- **Automated End-to-End Test Suite**:
  - Introduced 6-gate PowerShell E2E test suite (`scripts/test-agm-gitmap-running-backup-e2e.ps1`) with session PID protection and zero-residual sandbox hygiene (Thanks to @aukgit).
```

### Step 4: Synchronize README Files
Update the release summary in both `README.md` and `README_EN.md` under their respective Changelog / 更新日志 sections to ensure immediate home-page visibility.

### Step 5: Relative Path Audit
Audit the working tree diff to verify zero absolute filesystem paths:
- Verify all links use `02-spec/...`, `.ai-memory/...`, `src/...`, or `scripts/...`.
- Verify no `C:\...`, `D:\...`, or `/home/...` paths exist in documentation or code.

### Step 6: Atomic Commit and Release Tagging
Commit all changes in a single atomic commit:
```bash
# Note: Subagents do not execute git commands. This step defines the maintainer protocol:
# git add -A
# git commit -m "feat(backup): multimodal running prompt backup, preview word limits, and E2E verification"
# git tag v4.239.0
# git push origin main && git push origin v4.239.0
```

---

## 4. Verification Checklist

- [ ] `cargo fmt -- --check` reports zero differences.
- [ ] `cargo clippy --all-targets --all-features` produces zero warnings.
- [ ] `npm run build` succeeds cleanly.
- [ ] 14 version manifest files are synchronized to identical version number.
- [ ] `changelog.md` and `changelog_en.md` contain strictly `@aukgit` attribution.
- [ ] `README.md` and `README_EN.md` home-page changelogs are updated.
- [ ] Zero absolute paths in any modified files.
- [ ] `scripts/test-agm-gitmap-running-backup-e2e.ps1` passes all 6 gates.
