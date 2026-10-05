---
plan: 135-instance-restart-button-sync-icons-and-running-projects-fix
subtask: "04"
title: Preflight Quality Gates, Naming Linter & Release Ceremony
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

# Subtask 04: Preflight Quality Gates, Naming Linter & Release Ceremony

## 1. Context & Release Governance

This subtask governs the preflight quality assurance gates, naming linter verification, atomic multi-file minor version bump, changelog documentation, strict `@aukgit` attribution adherence, and release ceremony for **135-instance-restart-button-sync-icons-and-running-projects-fix**.

### 1.1 Non-Negotiable Release Invariants (from `AGENTS.md`)
1. **Strict Attribution Invariant**:
   - Attribution in `CHANGELOG.md` and `CHANGELOG_EN.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list contains only `aukgit` and none else.
2. **README Changelog Synchronization**:
   - For stable releases on `main`, the release summary **MUST** be synchronized in both `README.md` (under `## 📝 更新日志`) and `README_EN.md` (under `## 📝 Changelog`). Never update only `CHANGELOG.md` while leaving `README.md` / `README_EN.md` with outdated release notes.
3. **Atomic 14-Location Version Sync**:
   - Version bump must be executed using the automated script `npm run bump minor`, synchronizing all 14 project manifests simultaneously from `v4.154.0` to `v4.155.0`.
4. **Thinking Cache Invalidation Control**:
   - In `src/components/common/SuggestionDeleteThinkingModal.tsx`: keep `SUGGESTION_DELETE_THINKING_STORE = false`. Routine releases must not invalidate user thinking caches without architectural schema breaks.
5. **Subagent Git Execution Ban**:
   - Subagents operate under a total ban on Git commands (`git add`, `git commit`, `git push`, `git tag`, `git diff`, `git status`). All operations must use direct file tools and GitMap utilities.

---

## 2. 4-Tier Preflight Verification Gates

Before initiating the version bump, all preflight quality gates must pass cleanly:

### 2.1 Gate 1: Rust Code Formatting Gate
```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```
- **Pass Criteria**: Clean zero-exit code with no formatting diffs across `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/email_watcher.rs`, and `src-tauri/src/commands/instance.rs`.

### 2.2 Gate 2: Rust Clippy & Compilation Gate
```powershell
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
```
- **Pass Criteria**: Exits with code 0 and zero warnings or errors. Comprehensive gate covers compilation, borrow checker validation, and clippy lints.

### 2.3 Gate 3: Frontend TypeScript & Vite Production Build Gate
```powershell
npm run build
```
- **Pass Criteria**: `tsc` succeeds with 0 type errors; Vite compiles frontend bundle without missing symbol references, broken imports, or circular dependencies.

### 2.4 Gate 4: Naming Linter & Sequence Integrity Gate
```powershell
python 03-ai-scripts/08-naming-autofixer.py --check
python 03-ai-scripts/21-sequence-integrity-linter.py
```
- **Pass Criteria**: Validates that all newly authored spec and plan files follow lowercase hyphen-separated naming conventions, sequence integrity numbers are contiguous, and no forbidden suffixes exist.

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
- **实例原子化重启分段胶囊**: 在多实例表格与卡片视图中，为运行中实例新增原子化重启按钮与 `[停止 | 重启]` 分段胶囊栏（`rounded-l-[5px]`，紧凑圆角与细微分割线）；支持在当前绑定账号上一键优雅重启，自动轮询等待旧进程退出与文件锁释放（<1.5s）并清除提示词树缓存后安全启动，保持账号绑定与配置连续性。(Thanks to @aukgit)
- **同步图标语义解耦与反混淆**: 全面解耦原有的旋转箭头图标（`RotateCw` / `RefreshCw`），将旋转图标 `RotateCcw` 严格专用于实例重启操作；进程 PID 同步采用语义专属的 `Cpu` 图标，配置与配额同步采用 `SlidersHorizontal` 图标，全量同步采用 `FolderSync`，配额评估采用 `Sparkles`，凭证擦除采用 `KeyRound`，彻底消除用户界面操作歧义。(Thanks to @aukgit)
- **运行中项目与提示词深度探测 6 项根因修复**:
  - **沙盒目录隔离修复 (Defect 1)**: 修复 `gemini_dirs_for_instance` 中命名实例失败时误退回用户主目录的漏洞，杜绝沙盒实例与默认实例之间的目录碰撞。(Thanks to @aukgit)
  - **路径与仓库标识双层匹配 (Defect 2)**: 修复第 4 门禁中由于绝对路径与仓库短 ID / 名称不匹配导致活跃会话被误判为空闲的缺陷，全面支持绝对路径、仓库基名与前缀模糊匹配。(Thanks to @aukgit)
  - **消除全局进程假活判定 (Defect 3)**: 剔除第 0/4 门禁中调用的全局 `is_antigravity_running(None)`，改为基于各实例 `--user-data-dir` 独立校验 OS PID，防止克隆实例运行导致已关闭的默认实例被误判存活。(Thanks to @aukgit)
  - **思考模型自适应 10 分钟窗口与微秒解析 (Defect 4)**: 针对 o1/o3-mini、Claude 3.7 Thinking、Gemini 2.5 Flash Thinking 等深度推理场景，将活跃窗口由僵硬的 60s/120s 扩展至自适应 600s（10分钟）；升级 SQLite 时间戳解析器全面兼容 RFC 3339、微秒格式（`%Y-%m-%dT%H:%M:%S%.f`）及标准 SQL 格式；严格执行显式空闲霸权（`not_fully_idle == 0` 或 `status == IDLE` 立即置为非活跃）。(Thanks to @aukgit)
  - **空闲邮件传感器解除屏蔽 (Defect 5)**: 剔除 `is_any_prompt_actively_running` 中直接检测 IDE 进程存活的误判逻辑，恢复对真实活跃会话的精准检测，彻底解除对 `email_watcher.rs` 空闲告警的错误屏蔽。(Thanks to @aukgit)
  - **提示词树缓存契约化驱逐与僵尸重置 (Defect 6)**: 在 `close_instance`、`launch_instance`、`save_or_requeue_prompt` 及前端 `fetchRunningTasks({ force: true })` 中全面挂载 `invalidate_prompt_tree_cache`，消除 5 秒 TTL 缓存滞后；在启动清理中增加 `UPDATE running_projects SET is_running = 0`，消除异常关机留下的僵尸标记。(Thanks to @aukgit)
```

### 4.2 `CHANGELOG_EN.md` Entry (English)
```markdown
## [v4.155.0] - 2026-10-05

### ✨ Features & Improvements
- **Atomic Instance Restart & Segmented Split Capsule**: Added a dedicated restart action and `[Stop | Restart]` segmented split pill capsule (`rounded-l-[5px]` with subtle divider line) in both Table view and Card view; enables seamless 1-click restarting on the currently bound account, polling OS process exit and lockfile release (<1.5s), flushing prompt tree cache, and safely relaunching without losing profile bindings. (Thanks to @aukgit)
- **Sync Icon Disambiguation & Semantic Separation**: Replaced circular rotation sync icons (`RotateCw` / `RefreshCw`) with domain-specific icons (`Cpu` for PID Sync, `SlidersHorizontal` for Profile/Quota Sync, `FolderSync` for Sync All, `Sparkles` for Eval Quota, `KeyRound` for Wipe Credentials), reserving `RotateCcw` exclusively for instance restart operations to eliminate visual confusion. (Thanks to @aukgit)
- **Deep Running Projects & Prompts Detection Resolution (6 Architectural Defects)**:
  - **Sandbox Home Collision Remedy (Defect 1)**: Fixed `gemini_dirs_for_instance` fallback to user home directory on named profile lookup failure, guaranteeing strict filesystem sandbox isolation. (Thanks to @aukgit)
  - **Two-Tier Path & Basename Matching in Gate 4 (Defect 2)**: Resolved string mismatch where absolute workspace paths failed against short repo IDs/names; implemented direct, basename, and prefix matching. (Thanks to @aukgit)
  - **Eliminated Global Process False Alive (Defect 3)**: Replaced global `is_antigravity_running(None)` calls with strict `--user-data-dir` PID checks, preventing closed default instances from appearing active when secondary instances run. (Thanks to @aukgit)
  - **Adaptive 10-Minute Thinking Window & Microsecond Parser (Defect 4)**: Extended Gate 4 recency window from rigid 60s/120s to an adaptive 600s (10 minutes) for extended reasoning models (o1, o3-mini, Claude 3.7 Thinking, Gemini 2.5 Flash Thinking), preventing active thinking tasks from prematurely flipping to idle while preserving strict idle supremacy; upgraded timestamp parser to handle microsecond ISO 8601 formats. (Thanks to @aukgit)
  - **Unblocked Idle Sensor in Email Watcher (Defect 5)**: Removed blind IDE process check from `is_any_prompt_actively_running`, preventing open editors from permanently blocking idle email alerts in `email_watcher.rs`. (Thanks to @aukgit)
  - **Cache Invalidation Contract & Startup Zombie Reset (Defect 6)**: Enforced `invalidate_prompt_tree_cache` across `close_instance`, `launch_instance`, `save_or_requeue_prompt`, and frontend `fetchRunningTasks({ force: true })`, eliminating 5-second TTL cache lag; added startup reset `UPDATE running_projects SET is_running = 0` to clear abnormal shutdown states. (Thanks to @aukgit)
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
- [ ] `python 03-ai-scripts/08-naming-autofixer.py --check` passes cleanly.
- [ ] `npm run bump minor` synchronizes all 14 project manifest files to `4.155.0`.
- [ ] `CHANGELOG.md` and `CHANGELOG_EN.md` populated with strict `@aukgit` attribution.
- [ ] `README.md` and `README_EN.md` synchronized.
- [ ] `SUGGESTION_DELETE_THINKING_STORE` verified as `false` in `SuggestionDeleteThinkingModal.tsx`.
- [ ] Subagent git ban strictly maintained.
