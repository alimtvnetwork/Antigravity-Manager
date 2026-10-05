---
plan: 135-instance-restart-button-sync-icons-and-running-projects-fix
subtask: "04"
title: Preflight Quality Gates, Minor Version Bump & Release Ceremony
domain: release-orchestration-and-verification
depends_on:
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
  - .ai-memory/plans/subtasks/135-instance-restart-button-sync-icons-and-running-projects-fix/01-instance-restart-and-split-button.md
  - .ai-memory/plans/subtasks/135-instance-restart-button-sync-icons-and-running-projects-fix/02-sync-icons-disambiguation.md
  - .ai-memory/plans/subtasks/135-instance-restart-button-sync-icons-and-running-projects-fix/03-running-projects-and-prompts-detection-deep-fix.md
citations:
  component_spec: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  root_cause_analysis: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
  release_guidelines: docs/release_guide.md
  project_guidelines: AGENTS.md
target_files:
  - package.json
  - package-lock.json
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
  - src-tauri/tauri.conf.json
  - Casks/antigravity-tools.rb
  - CHANGELOG.md
  - CHANGELOG_EN.md
  - README.md
  - README_EN.md
  - src/components/layout/MiniView.tsx
  - src/pages/Settings.tsx
  - src/components/common/SuggestionDeleteThinkingModal.tsx
status: pending
---

# Subtask 04: Preflight Quality Gates, Minor Version Bump & Release Ceremony

## 1. Context & Release Governance

This subtask defines the comprehensive quality assurance verification, atomic multi-file minor version bump, changelog documentation, strict attribution governance, and release ceremony for **135-instance-restart-button-sync-icons-and-running-projects-fix**.

### 1.1 Non-Negotiable Release Invariants (from `AGENTS.md`)
1. **Strict Attribution Invariant**:
   - Attribution in `CHANGELOG.md` and `CHANGELOG_EN.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list contains only `aukgit` and none else.
2. **README Changelog Synchronization**:
   - For stable releases on `main`, the release summary **MUST** be updated in both `README.md` (under `## 📝 更新日志`) and `README_EN.md` (under `## 📝 Changelog`). Never update only `CHANGELOG.md` while leaving `README.md` / `README_EN.md` with outdated release notes.
3. **Atomic 14-Location Version Sync**:
   - Version bump must be executed using the automated script `npm run bump minor`, synchronizing all 14 project manifests and files simultaneously from `v4.154.0` to `v4.155.0`.
4. **Thinking Cache Invalidation Control**:
   - In `src/components/common/SuggestionDeleteThinkingModal.tsx`: keep `SUGGESTION_DELETE_THINKING_STORE = false`. Routine releases must not invalidate user thinking caches without architectural schema breaks.
5. **Subagent Git Execution Ban**:
   - Subagents operate under a total ban on Git commands (`git add`, `git commit`, `git push`, `git tag`, `git diff`, `git status`). Preflight verification and file synchronization are executed directly; git operations and release tags are managed exclusively by the parent orchestrator or maintainer.

---

## 2. 3-Tier Preflight Verification Gate

Before initiating the version bump, all preflight quality gates must pass cleanly:

### 2.1 Gate 1: Rust Code Formatting Check
```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```
- **Pass Criteria**: Clean zero-exit code with no formatting diffs across `src-tauri/src/modules/instance.rs`, `src-tauri/src/commands/instance.rs`, and `src-tauri/src/modules/repo_db.rs`.

### 2.2 Gate 2: Rust Clippy & Compilation Gate
```powershell
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```
- **Pass Criteria**: Exits with code 0 and zero warnings or errors. Comprehensive gate covers compilation, borrow checker validation, and clippy lints.

### 2.3 Gate 3: Frontend TypeScript & Vite Production Build
```powershell
npm run build
```
- **Pass Criteria**: `tsc` succeeds with 0 type errors; Vite compiles frontend bundle without missing symbol references, broken imports, or circular dependencies.

---

## 3. Atomic Version Bump Execution (`npm run bump minor`)

Execute the atomic version synchronization script:
```powershell
npm run bump minor
```

### 14 Manifest Locations Synchronized:
1. `package.json`: `"version": "4.155.0"`
2. `package-lock.json`: `"version": "4.155.0"` (root mirror, 2 locations)
3. `src-tauri/Cargo.toml`: `version = "4.155.0"`
4. `src-tauri/Cargo.lock`: `version = "4.155.0"`
5. `src-tauri/tauri.conf.json`: `"version": "4.155.0"`
6. `Casks/antigravity-tools.rb`: `version "4.155.0"`
7. `README.md`: Version badges and installation curl/irm strings
8. `README_EN.md`: Version badges and installation curl/irm strings
9. `src/components/layout/MiniView.tsx`: Version constants
10. `src/pages/Settings.tsx`: Version display badge
11. `version.json`: Runtime version declaration
12. `CHANGELOG.md`: Skeleton entry for `## [v4.155.0] - 2026-10-05`
13. `CHANGELOG_EN.md`: Skeleton entry for `## [v4.155.0] - 2026-10-05`
14. System metadata / installer manifests

---

## 4. Changelog & Documentation Population

Populate the newly generated skeleton entries in `CHANGELOG.md` and `CHANGELOG_EN.md` adhering strictly to `@aukgit` attribution.

### 4.1 `CHANGELOG.md` Entry (Chinese)
```markdown
## [v4.155.0] - 2026-10-05

### ✨ 新增与优化
- **实例原子化重启按钮与分段胶囊**: 在多实例表格与卡片视图中，为运行中实例新增原子化重启按钮与 `[停止 | 重启]` 分段胶囊栏（`rounded-l-[5px]`，5–6px 紧凑圆角与细微分割线）；支持在当前绑定账号上一键优雅重启，自动轮询等待旧进程退出与套接字释放（<1.5s）并清除提示词树缓存后安全启动，保持账号绑定与配置不变。(Thanks to @aukgit)
- **同步图标语义解耦与反混淆**: 全面将原有的旋转箭头图标（`RotateCw` / `RefreshCw`）解耦，进程 PID 同步采用语义专属的 `Cpu` 图标，配置与配额同步采用 `SlidersHorizontal` 图标，将旋转图标 `RotateCcw` 严格专用于实例重启操作，彻底消除用户界面操作歧义。(Thanks to @aukgit)
- **运行中项目与提示词深度探测根因修复**:
  - **宿主进程存活门禁 (Gate 0)**: 在 `detect_running_projects` 与 `is_prompt_running_for_project` 顶层强制校验实例 OS 进程存活，实例终止或崩溃时立即上报 `INSTANCE_PROCESS_DEAD` 并阻断运行态，根除幽灵运行状态。(Thanks to @aukgit)
  - **思考模型自适应 10 分钟窗口**: 针对 o1/o3-mini、Claude 3.7 Thinking、Gemini 2.5 Flash Thinking 等长时间深度推理场景，将第 4 门禁的轮次活跃窗口由僵硬的 60s/120s 扩展至自适应 600s（10分钟），避免模型推理中途被误判为空闲；严格保留显式空闲霸权（`not_fully_idle == 0` 或 `status == IDLE` 立即置为非活跃）。(Thanks to @aukgit)
  - **毫秒/微秒级时间戳全格式解析**: 升级 SQLite 时间戳解析器，全面兼容 RFC 3339、ISO 8601 毫秒微秒格式（`%Y-%m-%dT%H:%M:%S%.f`）及标准 SQL 空格分隔格式，杜绝因解析失败回退至 0 导致的误判。(Thanks to @aukgit)
  - **空白幽灵对话过滤**: 自动识别并过滤 IDE 启动时生成的 0 词无标题草稿会话，防止虚假对话挤占运行中项目统计。(Thanks to @aukgit)
  - **严格实例归属隔离与启动僵尸重置**: 在前端 `isNodeOwnedByInstance` 中强制要求精确实例 ID 匹配，剔除末尾字符模糊匹配；在后端 `purge_corrupted_running_projects` 启动清理中增加 `UPDATE running_projects SET is_running = 0`，消除异常关机留下的僵尸标记。(Thanks to @aukgit)
```

### 4.2 `CHANGELOG_EN.md` Entry (English)
```markdown
## [v4.155.0] - 2026-10-05

### ✨ Features & Improvements
- **Atomic Instance Restart & Segmented Split Capsule**: Added a dedicated restart action and `[Stop | Restart]` segmented split pill capsule (`rounded-l-[5px]` with subtle divider line) in both Table view and Card view; enables seamless 1-click restarting on the currently bound account, polling OS process exit and lockfile release (<1.5s), flushing prompt tree cache, and safely relaunching without losing profile bindings. (Thanks to @aukgit)
- **Sync Icon Disambiguation & Semantic Separation**: Replaced circular rotation sync icons (`RotateCw` / `RefreshCw`) with domain-specific icons (`Cpu` for PID Sync, `SlidersHorizontal` for Profile/Quota Sync), reserving `RotateCcw` exclusively for instance restart operations to eliminate visual confusion. (Thanks to @aukgit)
- **Deep Running Projects & Prompts Detection Resolution**:
  - **Host Process Liveness Gate (Gate 0)**: Enforced strict OS process liveness verification in `detect_running_projects` and `is_prompt_running_for_project`; dead or crashed instances immediately report `INSTANCE_PROCESS_DEAD` and set all projects to idle. (Thanks to @aukgit)
  - **Adaptive 10-Minute Thinking Window**: Extended Gate 4 turn recency window from rigid 60s/120s to an adaptive 600s (10 minutes) for extended reasoning models (o1, o3-mini, Claude 3.7 Thinking, Gemini 2.5 Flash Thinking), preventing active thinking tasks from prematurely flipping to idle while preserving strict idle supremacy (`not_fully_idle == 0` or status `IDLE`). (Thanks to @aukgit)
  - **Multi-Format Fractional Timestamp Parser**: Upgraded SQLite timestamp parser to robustly handle ISO 8601 with fractional seconds (`%Y-%m-%dT%H:%M:%S%.f`), RFC 3339, and space-separated SQL datetimes, eliminating parser failures defaulting timestamps to 0. (Thanks to @aukgit)
  - **0-Word Ghost Conversation Pruning**: Automatically filters out untitled, 0-word scratch sessions spawned during IDE startup. (Thanks to @aukgit)
  - **Strict Instance Ownership & Startup Zombie Reset**: Enforced exact instance ID matching in `isNodeOwnedByInstance` to eliminate cross-card node bleed; added `UPDATE running_projects SET is_running = 0` in `purge_corrupted_running_projects` on startup to reset zombie flags from abnormal terminations. (Thanks to @aukgit)
```

---

## 5. Homepage README Synchronization

Synchronize the summary entries to:
1. `README.md` under `## 📝 更新日志`:
   - Prepend `v4.155.0` update highlights at the top.
2. `README_EN.md` under `## 📝 Changelog`:
   - Prepend `v4.155.0` English update highlights at the top.

---

## 6. Release Verification Checklist

- [ ] `cargo fmt -- --check` completes with 0 errors.
- [ ] `cargo clippy --all-targets --all-features` completes with 0 errors.
- [ ] `npm run build` compiles frontend successfully.
- [ ] `npm run bump minor` synchronizes all 14 project manifest files to `4.155.0`.
- [ ] `CHANGELOG.md` and `CHANGELOG_EN.md` populated with strict `@aukgit` attribution.
- [ ] `README.md` and `README_EN.md` synchronized.
- [ ] `SUGGESTION_DELETE_THINKING_STORE` verified as `false` in `SuggestionDeleteThinkingModal.tsx`.
- [ ] Subagent git ban strictly maintained.
