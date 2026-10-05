# Subtask 05: Preflight Quality Gates, Minor Version Bump, and Release Ceremony

- **Parent Task**: `134-accounts-ui-supabase-instance-compact`
- **Subtask ID**: `05-preflight-minor-bump-and-release-ceremony`
- **Target Files**:
  - `package.json` & `package-lock.json` (Atomic version sync)
  - `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`
  - `Casks/antigravity-tools.rb`
  - `CHANGELOG.md` & `CHANGELOG_EN.md` (Release changelogs with strict `@aukgit` attribution)
  - `README.md` & `README_EN.md` (Synchronized homepage changelog summaries)
  - `src/components/layout/MiniView.tsx` & `src/pages/Settings.tsx`
  - `src/components/common/SuggestionDeleteThinkingModal.tsx` (Cache invalidation control)
- **Status**: Ready for Execution
- **Dependencies**: Subtasks 01, 02, 03, 04

---

## 1. Context & Release Governance

This subtask governs the quality assurance verification, atomic multi-file version bump, release documentation, attribution governance, and release ceremony for task **134-accounts-ui-supabase-instance-compact**.

### 1.1 Non-Negotiable Release Invariants (from `AGENTS.md`)
1. **Strict Attribution Invariant**:
   - Attribution in `CHANGELOG.md` and `CHANGELOG_EN.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes. This ensures the GitHub release page contributor list contains only `aukgit` and none else.
2. **README Changelog Synchronization**:
   - For stable releases on `main`, the release summary **MUST** be updated in both `README.md` (under `## 📝 更新日志`) and `README_EN.md` (under `## 📝 Changelog`). Never update only `CHANGELOG.md` while leaving README files outdated.
3. **Atomic 14-Location Version Sync**:
   - Version bump must be executed using the atomic script `npm run bump minor`, synchronizing all 14 project manifests and files simultaneously from `v4.153.0` to `v4.154.0`.
4. **Thinking Cache Invalidation Toggle**:
   - In `src/components/common/SuggestionDeleteThinkingModal.tsx`: for routine UI and sync updates, keep `SUGGESTION_DELETE_THINKING_STORE = false`.
5. **Subagent Git Execution Ban**:
   - Subagents operate under a total ban on Git commands (`git add`, `git commit`, `git push`, `git tag`). Pre-flight verification and file synchronization are performed directly; actual Git commits and tags are delegated to the parent orchestrator or repository maintainer.

---

## 2. Step-by-Step Release Protocol

### Step 1: Pre-flight Verification Gate
Execute the standard 3-tier pre-flight checks to guarantee zero linting, formatting, or compilation regressions:

1. **Rust Formatting**:
   ```powershell
   cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
   ```
   *Pass Criteria*: Zero formatting diffs.

2. **Rust Clippy Compilation Gate**:
   ```powershell
   cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
   ```
   *Pass Criteria*: Clean exit with zero warnings or errors.

3. **Frontend TypeScript & Vite Build**:
   ```powershell
   npm run build
   ```
   *Pass Criteria*: `tsc` passes without type errors; Vite bundle completes successfully.

---

### Step 2: Atomic Version Bump (`npm run bump minor`)
Execute the automated version synchronization script to transition from `4.153.0` to `4.154.0`:
```powershell
npm run bump minor
```
The script automatically synchronizes the new version (`4.154.0`) across all 14 project locations:
1. `package.json`
2. `package-lock.json` (root version mirror, 2 locations)
3. `src-tauri/Cargo.toml`
4. `src-tauri/Cargo.lock`
5. `src-tauri/tauri.conf.json`
6. `Casks/antigravity-tools.rb`
7. `README.md` (header version badge & install commands)
8. `README_EN.md` (header version badge & install commands)
9. `src/components/layout/MiniView.tsx`
10. `src/pages/Settings.tsx`
11. `CHANGELOG.md` (inserts version heading skeleton)
12. `CHANGELOG_EN.md` (inserts version heading skeleton)

---

### Step 3: Populate Changelogs with Strict `@aukgit` Attribution
Populate the newly inserted version skeletons in `CHANGELOG.md` and `CHANGELOG_EN.md`:

#### 3.1 `CHANGELOG.md` (Chinese Entry)
```markdown
## [v4.154.0] - 2026-10-05

### ✨ 新增与优化
- **账号配额视觉升级**: 配额进度条全面升级为 VS Code 蓝青色系（`teal-500` / `cyan-500` / `sky-500`），去除高饱和荧光绿与刺眼发光点；表格行增加清晰边框，配额列建立微底色分组以防数据混淆。(Thanks to @aukgit)
- **多实例表格紧凑化与无横向滚动**: 紧凑化实例列表间距与列宽，合并名称与邮箱为上下层叠单元格，路径智能缩短为 `...\<parent>\<leaf>` 并支持一键复制，彻底消除 1280px+ 视口下的横向滚动条。(Thanks to @aukgit)
- **对话树 Prompts 胶囊按钮整合**: 将 Prompts 对话树入口收拢进实例操作胶囊栏与操作菜单，按钮圆角全量统一为紧凑规范的 5–6px（`rounded-[5px]`）。(Thanks to @aukgit)
- **卡片网格规范与轮换提示**: 实例卡片在桌面端统一为 4 列网格（`xl:grid-cols-4`），操作按钮规范为 2 行整齐排列；“轮换至最佳”按钮提供动态悬浮提示，明确标识当前轮换的目标实例。(Thanks to @aukgit)
- **Supabase 跨节点分布式租约与冷冻池回退**: 新节点启动自动发现 `repo-secrets` 目录凭据并开启同步；轮换前执行分布式独占锁校验，并在所有可用账号进入冷冻期时代偿回退至最久未使用的健康账号，彻底避免多机冲突与调度死锁。(Thanks to @aukgit)
```

#### 3.2 `CHANGELOG_EN.md` (English Entry)
```markdown
## [v4.154.0] - 2026-10-05

### ✨ Features & Improvements
- **Accounts Quota Visual Upgrade**: Upgraded quota progress bars to VS Code teal/cyan/sky palette (`teal-500` / `cyan-500` / `sky-500`), eliminating high-saturation neon greens and glaring halos; added clear table row borders and subtle quota column grouping. (Thanks to @aukgit)
- **Instances Table Compacting & Zero Horizontal Scroll**: Compacted instance table spacing and column widths, merged profile name and bound email into a single stacked cell, and truncated directory paths to `...\<parent>\<leaf>` with 1-click copy, fully eliminating horizontal scrolling on 1280px+ viewports. (Thanks to @aukgit)
- **Integrated Prompts Action Button**: Consolidated Prompt Tree actions into the segmented action capsule and dropdown menu, standardizing all interactive button radii to 5–6px (`rounded-[5px]`). (Thanks to @aukgit)
- **Card Grid Standardization & Rotate Tooltip**: Standardized instance cards to an exact 4-column desktop grid (`xl:grid-cols-4`) with structured 2-row action buttons; added dynamic contextual candidate tooltips to "Rotate to Next Best". (Thanks to @aukgit)
- **Supabase Distributed Leases & Cooldown Fallback**: Automatic discovery of root credentials from `repo-secrets` on fresh startups; enforced pre-switch exclusive distributed locks and implemented graceful fallback to the oldest account when all healthy profiles are in cooldown. (Thanks to @aukgit)
```

---

### Step 4: Synchronize Homepage README Release Summaries
Ensure that the latest `v4.154.0` summary is synchronized into both `README.md` and `README_EN.md`:

#### 4.1 In `README.md` under `## 📝 更新日志`:
Insert the `v4.154.0` release notes at the top of the update log section.

#### 4.2 In `README_EN.md` under `## 📝 Changelog`:
Insert the `v4.154.0` English release notes at the top of the update log section.

---

### Step 5: Verify Thinking Cache Invalidation Setting
In `src/components/common/SuggestionDeleteThinkingModal.tsx`:
- Confirm `SUGGESTION_DELETE_THINKING_STORE = false`.
- Routine UI updates do not require clearing user thinking stores.

---

### Step 6: Git Tag & Release Ceremony (Parent Orchestrator / Maintainer)
*Note: Due to the Subagent Git Ban, the following commands are executed by the Parent Agent or Maintainer:*
```bash
git add .
git commit -m "chore(release): bump version to v4.154.0 - accounts UI, supabase sync, and instances compacting"
git tag v4.154.0
git push origin main
git push origin v4.154.0
```

---

## 3. Verification & Validation Steps

1. **Manifest Version Check**:
   - Verify `package.json` reads `"version": "4.154.0"`.
   - Verify `src-tauri/tauri.conf.json` reads `"version": "4.154.0"`.
   - Verify `src-tauri/Cargo.toml` reads `version = "4.154.0"`.
2. **Attribution Audit**:
   - Check `CHANGELOG.md` and `CHANGELOG_EN.md` to ensure only `@aukgit` is mentioned.
3. **README Sync Check**:
   - Check that `README.md` and `README_EN.md` feature the `v4.154.0` notes.
4. **Pre-flight Execution**:
   - `cargo fmt -- --check` passes.
   - `cargo clippy` passes.
   - `npm run build` completes cleanly.

---

## 4. Done When Checklist

- [ ] Pre-flight checks (`cargo fmt`, `cargo clippy`, `npm run build`) all pass cleanly.
- [ ] Atomic version bump updates all 14 project manifest files to `v4.154.0`.
- [ ] `CHANGELOG.md` and `CHANGELOG_EN.md` updated with strict `@aukgit` attribution.
- [ ] `README.md` and `README_EN.md` updated with release summary.
- [ ] `SUGGESTION_DELETE_THINKING_STORE` remains `false`.
- [ ] Subagent git ban strictly observed.
