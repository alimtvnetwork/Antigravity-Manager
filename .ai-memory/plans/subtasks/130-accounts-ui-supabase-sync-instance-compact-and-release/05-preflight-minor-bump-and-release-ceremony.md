# Subtask 05: Preflight Quality Gates, Minor Version Bump, and Release Ceremony

- **Parent Task**: `130-accounts-ui-supabase-sync-instance-compact-and-release`
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

This subtask governs the quality assurance verification, atomic multi-file version bump, release documentation, attribution governance, and release ceremony for task **130-accounts-ui-supabase-sync-instance-compact-and-release**.

### 1.1 Non-Negotiable Release Invariants (from `AGENTS.md`)
1. **Strict Attribution Invariant**:
   - Attribution in `CHANGELOG.md` and `CHANGELOG_EN.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes. This ensures the GitHub release page contributor list contains only `aukgit` and none else.
2. **README Changelog Synchronization**:
   - For stable releases on `main`, the release summary **MUST** be updated in both `README.md` (under `## 📝 更新日志`) and `README_EN.md` (under `## 📝 Changelog`). Never update only `CHANGELOG.md` while leaving README files outdated.
3. **Atomic 14-Location Version Sync**:
   - Version bump must be executed using the atomic script `npm run bump minor`, synchronizing all 14 project manifests and files simultaneously.
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
Execute the automated version synchronization script:
```powershell
npm run bump minor
```
The script automatically synchronizes the new version across all 14 project locations:
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
## [v<NEW_VERSION>] - 2026-10-05

### ✨ 新增与优化
- **账号配额视觉升级**: 配额进度条全面升级为 VS Code 蓝青色系（`teal-500` / `cyan-500` / `sky-500`），去除高饱和荧光绿与刺眼发光点；表格行增加清晰边框，配额列建立微底色分组以防数据混淆。(Thanks to @aukgit)
- **多实例表格紧凑化与无横向滚动**: 紧凑化实例列表间距与列宽，合并名称与邮箱为上下层叠单元格，路径智能缩短为 `...\<parent>\<leaf>` 并支持一键复制，彻底消除 1280px+ 视口下的横向滚动条。(Thanks to @aukgit)
- **对话树 Prompts 胶囊按钮整合**: 将 Prompts 对话树入口收拢进实例操作胶囊栏，按钮圆角全量统一为紧凑规范的 5–6px（`rounded-[5px]`）。(Thanks to @aukgit)
- **卡片网格规范与轮换提示**: 实例卡片在桌面端统一为 4 列网格（`xl:grid-cols-4`），操作按钮规范为 2 行整齐排列；“轮换至最佳”按钮提供动态悬浮提示，明确标识当前轮换的目标实例。(Thanks to @aukgit)
- **Supabase 跨节点分布式租约与冷冻池回退**: 新节点启动自动发现 `repo-secrets` 目录凭据并开启同步；轮换前执行分布式独占锁校验，并在所有可用账号进入冷冻期时代偿回退至最久未使用的健康账号，彻底避免多机冲突与调度死锁。(Thanks to @aukgit)
```

#### 3.2 `CHANGELOG_EN.md` (English Entry)
```markdown
## [v<NEW_VERSION>] - 2026-10-05

### ✨ Enhancements & Fixes
- **Modernized Accounts Quota Palette**: Replaced harsh neon green tracks with calming VS Code teal/cyan/sky gradients (`teal-500 / cyan-500 / sky-500`) and softened checkpoint nodes. Restored crisp row borders and subtle quota column grouping. (Thanks to @aukgit)
- **Compact Instance Table with Zero Horizontal Scroll**: Compacted column dimensions and padding, merged Profile Name and Bound Email into a single stacked cell, and truncated directory paths to `...\<parent>\<leaf>` with 1-click clipboard copy, eliminating horizontal scrollbars on 1280px+ displays. (Thanks to @aukgit)
- **Integrated Prompts Capsule Action**: Merged Prompts conversation tree button directly into the segmented instance action capsule and standardized button border radii to strict 5–6px (`rounded-[5px]`). (Thanks to @aukgit)
- **4-Column Card Grid & Dynamic Rotation Tooltip**: Standardized instance cards to a 4-column layout (`xl:grid-cols-4`) with partitioned 2-row action toolbars. Added dynamic target tooltip for the "Rotate to Next Best" action. (Thanks to @aukgit)
- **Distributed Supabase Leases & Cooldown Fallback**: Enabled automatic discovery of root credentials from `repo-secrets`. Introduced pre-switch distributed lease locks and a graceful fallback to the oldest account in the cooldown pool, eliminating multi-instance race conditions and rotation deadlocks. (Thanks to @aukgit)
```

---

### Step 4: Synchronize Homepage README Changelog Summaries
Synchronize the release summary directly into both root README documents:
1. In `README.md` under `## 📝 更新日志`:
   - Prepend the new version heading and bullet points.
2. In `README_EN.md` under `## 📝 Changelog`:
   - Prepend the matching English version heading and bullet points.

---

### Step 5: Verify Thinking Cache Invalidation Flag
Inspect `src/components/common/SuggestionDeleteThinkingModal.tsx`:
```typescript
export const SUGGESTION_DELETE_THINKING_STORE = false;
```
Ensure the flag remains `false` since this release does not alter the underlying thinking storage serialization schema.

---

### Step 6: Final Pre-flight Re-Verification
Before completing the subtask, re-run pre-flight validation on the bumped files:
```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features
npm run build
```
Verify that the build outputs bundle without errors.

---

### Step 7: Delegate Git Commit & Release Tagging
As a subagent with a total ban on Git execution:
1. Provide the list of modified and bumped files to the parent agent / maintainer.
2. Request the parent agent or maintainer to execute:
   ```bash
   # Delegated to Parent / Maintainer
   git add -A
   git commit -m "chore(release): bump version to v<NEW_VERSION> and release"
   git tag v<NEW_VERSION>
   git push origin main --tags
   ```

---

## 3. Verification & Sign-off Checklist

- [ ] All pre-flight gates (`cargo fmt`, `cargo clippy`, `npm run build`) pass cleanly.
- [ ] Version bump is synchronized across all 14 files via `npm run bump minor`.
- [ ] Attribution in `CHANGELOG.md` and `CHANGELOG_EN.md` strictly references `@aukgit` and no other handles.
- [ ] Both `README.md` and `README_EN.md` changelog sections are fully synchronized.
- [ ] `SUGGESTION_DELETE_THINKING_STORE` is confirmed `false`.
- [ ] No git commands were executed by this subagent.
