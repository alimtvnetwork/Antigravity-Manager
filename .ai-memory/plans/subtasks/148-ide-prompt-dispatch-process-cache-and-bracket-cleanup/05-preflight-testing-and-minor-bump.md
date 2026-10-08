# Subtask 05: Pre-flight Quality Gates, Testing & Minor Bump (v4.166.0)

- **Subtask ID**: `148-05`
- **Parent Task**: `148-ide-prompt-dispatch-process-cache-and-bracket-cleanup`
- **Target Files**:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `changelog.md`
  - `changelog_en.md`
  - `readme.md`
  - `readme_en.md`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: READY_FOR_EXECUTION

---

## 1. Objective

Execute comprehensive pre-flight quality verification across frontend and Rust backend codebases, ensure zero clippy warnings and formatting discrepancies, and conduct the atomic release ceremony for minor version `v4.166.0` with attribution strictly to `@aukgit`.

---

## 2. Pre-flight Quality Gates

Before any version bump or release tagging, the following gates must be executed and verified green:

```bash
# 1. Rust formatting verification
cd src-tauri && cargo fmt -- --check

# 2. Rust comprehensive compiler and clippy gate
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Targeted Rust unit tests for touched modules
cd src-tauri && cargo test modules::instance
cd src-tauri && cargo test modules::repo_db
cd src-tauri && cargo test modules::cli

# 4. Frontend production bundle verification
npm run build
```

---

## 3. Release Ceremony Protocol (`v4.166.0`)

### 3.1 Atomic Version Bump
Execute the automated version synchronization script to bump minor version to `v4.166.0`:
```bash
npm run bump minor
```
This updates:
- `package.json` (`version: "4.166.0"`)
- `package-lock.json`
- `src-tauri/Cargo.toml` (`version = "4.166.0"`)
- `src-tauri/Cargo.lock`
- Generates changelog skeleton entries in `changelog.md` and `changelog_en.md`.

### 3.2 Attribution Invariant
Attribution in all release materials must strictly reference `@aukgit`:
```markdown
*(Thanks to @aukgit)*
```
Do **NOT** include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else.

### 3.3 Changelog Synchronization (`changelog.md` & `changelog_en.md`)

#### English Changelog Entry (`changelog_en.md`):
```markdown
## [v4.166.0] - 2026-10-09

### 🚀 Core Architecture & Process Reliability
- **Smart Instance Process Cache & Closed-PID Re-Scan**: Implemented user-mandated 3-step lifecycle:
  1. Scan and cache active Antigravity Tools Manager instances and PIDs.
  2. If cached PID is closed, re-scan OS process table to detect reloaded instances before restarting.
  3. Reopen IDE and dispatch prompt only when confirmed down after re-scan.
- **Fixed `close_instance` Process Argument Refresh**: Updated `sysinfo` process refresh to include command-line arguments (`.with_cmd(...)`), ensuring target instances close cleanly.
- **Injected `--user-data-dir` into `spawn_prompt_via_agy`**: Ensured `agy` background prompt execution connects to cloned instance sockets rather than defaulting to `$HOME/.config/Antigravity`.
- **System Clipboard Dispatch Fallback**: Integrated native OS clipboard population upon prompt dispatch.

### 🎨 UI & UX Modernization
- **Total Elimination of Bracket Tag Clutter**: Removed heavy bracket wrappers (`[AGM:P006 | GM:#6]`, `[AGM:C025 | GM:antigrav]`) across the prompt tree view, replacing them with sleek `#6`, `P006`, and `C025` sequence indicators.
- **Header Sequence Display Polish**: Cleaned up the conversation detail header to display `C025` without redundant `#C025` double prefixes.
- **Dark-Glass Context Omission Banner**: Replaced bracketed text with interactive callout banners (`⚡ Omitted {formattedSize} transcript context · Click to expand`).
- **Eradicated Frontend Double-Launch Race**: Removed redundant window focus/launch call in `handleResendPrompt` to eliminate duplicate IDE window launches.

### 🧹 Integrity & Ghost State Eradication
- **Purged Synthetic Crash Recovery Prompts**: Eliminated artificial prompt manufacturing in `auto_resume_recent_prompts`, preventing phantom tasks from polluting `active_prompts`.
- **Empirical Worker/Transcript Verification**: Enforced live worker process and transcript verification in `compute_project_conversation_tree`, eliminating false `1 RUNNING` badges on idle projects.

### 💻 CLI Parity
- **Full CLI Command Routing**: Added match arms and implementations for `agm prompts ls`, `tree`, `send`, `enqueue`, `backup`, `restore`, and `agm doctor` with both human-readable and `--json` envelope support.

*(Thanks to @aukgit)*
```

#### Chinese Changelog Entry (`changelog.md`):
```markdown
## [v4.166.0] - 2026-10-09

### 🚀 核心架构与进程生命周期可靠性
- **智能实例进程缓存与关闭 PID 重新扫描**：严格实现三步执行生命周期：
  1. 扫描并缓存运行中的 Antigravity Tools Manager 实例与 PID 计数。
  2. 若缓存 PID 已关闭，先重新扫描系统进程表排查重新启动的 PID，避免误重启。
  3. 仅在重新扫描确认未运行时，才重启 IDE 实例并发送 Prompt。
- **修复 `close_instance` 进程参数刷新缺失**：为 `sysinfo` 添加命令行参数刷新（`.with_cmd(...)`），确保正确匹配 `--user-data-dir` 并彻底关闭目标实例。
- **`spawn_prompt_via_agy` 注入 `--user-data-dir`**：确保 `agy` 后台任务连接克隆实例目录，而非回退至 `$HOME/.config/Antigravity`。
- **系统剪贴板调度兜底**：在发送 Prompt 时原生写入系统剪贴板，支持立即快捷粘贴。

### 🎨 UI / UX 现代化与视觉降噪
- **彻底去除方括号标签冗余**：清理 Prompt 树状视图中庞大的 `[AGM:P006 | GM:#6]` 与 `[AGM:C025 | GM:antigrav]` 标签，换用极简 `#6`、`P006` 与 `C025` 序号标识。
- **头部序号双重前缀修复**：优化对话详情面板头部，避免生成 `#C025` 重复前缀。
- **暗色毛玻璃上下文省略横幅**：替换方括号纯文本为交互式折叠横幅（`⚡ Omitted {formattedSize} transcript context · Click to expand`）。
- **根除前端双重启动竞态**：移除 `handleResendPrompt` 中重复调用的窗口聚焦命令，防止瞬间并发弹出两个 IDE 窗口。

### 🧹 幽灵运行状态根除
- **清除合成崩溃恢复 Prompt**：彻底移除 `auto_resume_recent_prompts` 中虚构的崩溃恢复 Prompt，避免污染 `active_prompts`。
- **真实 Worker / 记录校验**：在 `compute_project_conversation_tree` 中强制核验实际后台 Worker 进程及对话记录活跃度，彻底消除空闲项目上的误报 `1 RUNNING`。

### 💻 CLI 命令行能力对齐
- **全面补齐命令行路由**：支持 `agm prompts ls`、`tree`、`send`、`enqueue`、`backup`、`restore` 及 `agm doctor`，全面支持交互式输出与 `--json` 规范数据信封。

*(Thanks to @aukgit)*
```

### 3.4 Root Readme Synchronization (`README.md` & `README_EN.md`)
Synchronize the release notes directly under `## 📝 更新日志` in `README.md` and `## 📝 Changelog` in `README_EN.md`. Never update only `CHANGELOG.md` while leaving root README files with outdated version notes.

---

## 4. Invariants & Constraints

- **Strict Relative Git Paths**: All paths cited in code and documentation must be relative (e.g. `changelog.md`, `src-tauri/Cargo.toml`).
- **Strict Single Attribution**: `@aukgit` only.
- **Exact Tag Matching**: Release tag `v4.166.0` must match the changelog heading character-for-character.
- **Positive Booleans**: Exclusively use positive boolean identifiers (`is_passed`, `is_clean`, `is_tagged`).
- **No Git Commands**: The spec author must not execute git commands.

---

## 5. Done When

- [ ] All pre-flight quality checks pass (`cargo fmt`, `cargo clippy`, `npm run build`).
- [ ] Version manifests (`package.json`, `Cargo.toml`) bumped to `4.166.0`.
- [ ] `changelog.md` and `changelog_en.md` synchronized with complete release notes attributed strictly to `@aukgit`.
- [ ] `README.md` and `README_EN.md` updated with release notes under their respective changelog headings.
