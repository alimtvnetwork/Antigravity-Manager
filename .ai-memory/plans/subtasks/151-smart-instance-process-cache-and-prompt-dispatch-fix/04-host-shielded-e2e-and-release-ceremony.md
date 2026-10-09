# Subtask 04: Host-Shielded E2E Testing, Pre-flight Verification & Minor Bump Release Ceremony (`v4.172.0`)

- **Subtask ID**: `151-04`
- **Parent Task**: `151-smart-instance-process-cache-and-prompt-dispatch-fix`
- **Specification Reference**: `02-spec/21-app/151-smart-instance-process-cache-and-prompt-dispatch-fix/00-master-audit-ledger.md`
- **Target Files**:
  - `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `changelog.md`
  - `changelog_en.md`
  - `README.md`
  - `README_EN.md`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: `[READY_FOR_EXECUTION]`

---

## 1. Objectives & Quality Gates

This subtask defines the comprehensive verification, pre-flight gate, and release ceremony pipeline for Task 151:
1. **Safe Host-Shielded E2E Test Execution**:
   - Run `python 03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py` across all 5 test cases (`TC01` through `TC05`) with 100% pass rate.
   - Enforce zero disruption to active host Antigravity IDE instances via process shielding (`inventory_host_protected_pids`).
2. **Pre-flight Quality Gates**:
   - Format check: `cd src-tauri && cargo fmt -- --check`.
   - Frontend production build: `npm run build`.
   - Comprehensive Rust clippy gate: `cd src-tauri && cargo clippy --all-targets --all-features`.
3. **Atomic Version Bump & Documentation**:
   - Execute `npm run bump minor` to advance version from `4.171.0` to `4.172.0` synchronously across `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`.
   - Update `changelog.md` and `changelog_en.md` with release notes strictly crediting `@aukgit` (`(Thanks to @aukgit)`).
   - Synchronize release highlights into `README.md` (`## 📝 更新日志`) and `README_EN.md` (`## 📝 Changelog`).
4. **GitMap Atomic Commit, Tagging & Push**:
   - Commit changes via GitMap atomic commit: `gitmap cpf "chore(release): bump version to 4.172.0 and update changelog"`.
   - Create Git tag `v4.172.0` and push tag to remote tracking branch.
   - Create official GitHub release via `gh release create v4.172.0`.

---

## 2. Phase 1: Safe Host-Shielded E2E Verification

### 2.1 Test Suite Breakdown (`03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`)

| Test Case | Description | Verification Logic | Shielding Guarantee |
| :--- | :--- | :--- | :--- |
| **TC01** | Cache Hit (No Relaunch Verification) | Verifies that when an IDE instance PID is active and alive, prompt dispatch reuses the running process without spawning duplicate instances or calling `close_instance()`. | Active host PIDs are inventoried in memory and shielded from signals or termination. |
| **TC02** | Process Death Check & Cold Launch Path | Verifies that if an instance PID is terminated or closed, the two-tier re-scan queries the OS process table and only triggers cold launch when confirmed 100% closed. | Uses isolated dummy subprocesses, never signals host processes. |
| **TC03** | Strict FIFO Queue Storage & Ordering | Verifies that `enqueue_prompt` persists prompts to `active_prompts` with status `queued` in strict `ORDER BY created_at ASC, id ASC` order. | Operates on dedicated test SQLite fixtures. |
| **TC04** | False-Positive Ghost Running Elimination | Verifies that completed conversation transcripts (with terminal markers) never report pulsing `RUNNING` status. | Transcript mock parser tests against completed events. |
| **TC05** | PromptTreeViewModal UI Compaction | Verifies elimination of bracket tokens `[Collapse Full Text]`, clean dual badges, and button word counts. | AST / static regex inspection on TypeScript source. |

### 2.2 Execution Command
```bash
python 03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py
```
**Expected Exit Code**: `0` with all 5 tests showing `[PASS]`.

---

## 3. Phase 2: Pre-flight Compilation & Linter Gates

Execute standard project pre-flight gates:
```bash
# 1. Rust code formatting check
cd src-tauri && cargo fmt -- --check

# 2. Rust clippy comprehensive check
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Frontend TypeScript & Vite production build
cd .. && npm run build
```

**Pass Invariant**:
- Zero compiler errors in Rust (`cargo clippy`).
- Zero formatting errors (`cargo fmt`).
- Zero TypeScript typecheck errors in Vite build (`npm run build`).

---

## 4. Phase 3: Version Bump & Release Documentation

### 4.1 Atomic Version Bump
Run the project automated version bumper:
```bash
npm run bump minor
```
This updates:
- `package.json`: `"version": "4.172.0"`
- `src-tauri/Cargo.toml`: `version = "4.172.0"`
- `src-tauri/tauri.conf.json`: `"version": "4.172.0"`
- Auto-generates initial release header skeletons in `changelog.md` and `changelog_en.md`.

### 4.2 Changelog Synchronization (`changelog.md` & `changelog_en.md`)

Update `changelog.md` with:
```markdown
## [4.172.0] - 2026-10-10

### 🚀 新特性与系统优化 (Features & Enhancements)
- **智能实例进程缓存与防重开机制**: 重构实例发送/入队逻辑，优先复用已运行 IDE 实例，彻底消除重复启动与误杀后台进程问题。(Thanks to @aukgit)
- **已关闭 PID 两级重扫与自愈**: 当缓存 PID 退出时，自动通过 OS 进程表进行两级二次确认，仅在确认完全退出后才安全冷启动。(Thanks to @aukgit)
- **深度日志扫描与虚假运行消除**: 增强日志倒序扫描深度，精准识别终态指令，消除 finished 任务误报常驻 RUNNING 状态的 Ghost 现象。(Thanks to @aukgit)
- **Prompt Tree View 界面精简与括号降噪**:
  - 移除 `[Collapse Full Text]` 与 `[Expand Full Text]` 等字面量方括号噪音，采用优雅内联排版。
  - 精简序列徽章为单一行号 `C001` / `P001`，GitMap SHA 与序号收拢至悬浮提示。
  - 规范字数展开按钮文本为 `Show All (${n} words)`。
  - 实例表格状态单元格优化为 `Running · {pid}` 对称点号分隔。
```

Update `changelog_en.md` with:
```markdown
## [4.172.0] - 2026-10-10

### 🚀 Features & Enhancements
- **Smart Instance Process Cache & Zero-Relaunch Guarantee**: Refactored instance prompt dispatch and enqueue pipelines to reuse active IDE instances, eliminating duplicate window launches and accidental host process termination. (Thanks to @aukgit)
- **Closed-PID Two-Tier OS Re-Scan**: Added robust process vitality checks; when a cached PID exits, the system scans the OS process table to verify whether worker instances remain before triggering a cold launch. (Thanks to @aukgit)
- **Deep Transcript Scanning & Ghost Running Elimination**: Enhanced transcript reverse-scanning depth to detect terminal completion markers, eradicating false-positive pulsing RUNNING indicators on finished conversations. (Thanks to @aukgit)
- **Prompt Tree View UI Compaction & Bracket De-Cluttering**:
  - Stripped literal bracket tokens `[Collapse Full Text]` and `[Expand Full Text]` in favor of clean typography.
  - Compacted sequence badges to clean `C001` / `P001` codes, moving GitMap commit SHAs into native tooltips.
  - Standardized expand button labels to `Show All (${n} words)`.
  - Streamlined Instance Table status indicators to `Running · {pid}`.
```

### 4.3 Root README Synchronization (`README.md` & `README_EN.md`)
Per `AGENTS.md` release rules, the latest stable/minor release summary **MUST** be mirrored under `## 📝 更新日志` in `README.md` and `## 📝 Changelog` in `README_EN.md`.

---

## 5. Phase 4: GitMap Atomic Commit, Tagging & GitHub Release

```bash
# 1. Atomic staging, commit and push via GitMap
gitmap cpf "chore(release): bump version to 4.172.0 and update changelog"

# 2. Create version tag
git tag v4.172.0

# 3. Push tag to upstream
git push origin v4.172.0

# 4. Create official GitHub Release
gh release create v4.172.0 --title "v4.172.0 - Smart Instance Process Cache & Prompt Dispatch Fix" --notes-file changelog_en.md
```

---

## 6. Execution Verification Matrix

| Checkpoint | Gate Command | Target Output |
| :--- | :--- | :--- |
| **E2E Suite** | `python 03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py` | All 5 test cases `[PASS]`, exit code `0` |
| **Rust Format** | `cd src-tauri && cargo fmt -- --check` | Clean check, exit code `0` |
| **Rust Clippy** | `cd src-tauri && cargo clippy --all-targets --all-features` | Zero warnings/errors, exit code `0` |
| **Frontend Build** | `npm run build` | Built without errors, exit code `0` |
| **Attribution Audit** | `grep -i "@" changelog.md changelog_en.md` | Only `@aukgit` appears in changelogs |
| **Tag Matching** | `git tag -l "v4.172.0"` | Exact match `v4.172.0` present |
