# Plan 22: AGM Branding, UI Compactness, Instances Fix, and v4.15.0 Release

> **Version:** 1.0.0
> **Status:** Completed
> **Created:** 2026-09-17
> **Completed:** 2026-09-17
> **Task Origin:** Initiated from user request to rename app title to "AGM by Alim" and product/EXE name to "Anti-Gravity Tools by Alim", compact the Accounts UI table and consolidate model quotas to Gemini and Claude shared buckets, fix the Instances page to ensure visibility, interactivity, and proper error handling, document automatic quota rotation prominently in Settings, remove Chinese comments and text across release scripts and manifests, update root readme attribution, bump version to v4.15.0, and verify repository specifications, error management, and AI prompt assets.
> **Total Steps / Loops Executed:** 4 subtasks executed concurrently across 2 dedicated subagents in single atomic turn with zero CI/CD failures.
> **Scope:** Full-stack Branding, Rust Manifests (`src-tauri/Cargo.toml`, `tauri.conf.json`), Frontend UI (`AccountTable.tsx`, `Instances.tsx`, `AutoSwitcherSettings.tsx`, `NavLogo.tsx`), Documentation (`readme.md`, `README_EN.md`, `CHANGELOG.md`, `CHANGELOG_EN.md`), and Release Lifecycle (`v4.15.0`).

---

## Task-Specific Rule Set

1. **Rule B1 — Consistent Global Branding:** App title MUST be `"AGM by Alim"` across window titles, navigation bars, and HTML metadata. Executable and product name MUST be `"Anti-Gravity Tools by Alim"` in Tauri configurations.
2. **Rule B2 — Consolidated Shared Quota Display:** In the Accounts view, model quotas MUST NOT clutter the UI with truncated individual variant badges. Quotas sharing credit pools MUST be consolidated to unified items (`Gemini` and `Claude`) in a compact, single-row layout.
3. **Rule B3 — Resilient Empty and Error States:** The Instances page MUST handle loading states, store errors with dedicated retry actions, empty instance stores with default initialization triggers, and manual refresh controls.
4. **Rule B4 — Clean Language & Attribution Hygiene:** All Chinese comments and release notes strings MUST be eliminated from manifests and build scripts. Attribution MUST accurately acknowledge upstream origins (`lbjlaq/Antigravity-Manager`) while establishing maintenance and enhancement by Md. Alim Ul Karim.
5. **Rule B5 — Strict Relative Git Paths & Lowercase Naming:** All documentation, plans, subtasks, and files MUST use strictly lowercase filenames and relative paths starting from the git repository root.

---

## Consolidated Subtasks & Implementations

### Subtask 01: Global Branding & v4.15.0 Version Bump
- **Target Files:**
  - `version.json`
  - `package.json` & `package-lock.json`
  - `src-tauri/Cargo.toml` & `src-tauri/Cargo.lock`
  - `src-tauri/tauri.conf.json`
  - `Casks/antigravity-tools.rb`
  - `index.html`
  - `src/components/navbar/NavLogo.tsx`
  - `src/locales/en.json`
  - `src/pages/Settings.tsx`
  - `src/components/layout/MiniView.tsx`
  - `install.ps1` & `install.sh`
- **Accomplishments:**
  - Updated window title and HTML `<title>` to `"AGM by Alim"`.
  - Configured `tauri.conf.json` with `productName: "Anti-Gravity Tools by Alim"` and window `title: "AGM by Alim"`.
  - Updated `NavLogo.tsx` and `src/locales/en.json` (`common.app_name`) to display `"AGM by Alim"`.
  - Bumped version to `4.15.0` with release date `2026-09-17` across all package manifests, Homebrew cask, and one-liner installer scripts.

### Subtask 02: Accounts UI Compactness and Quota Consolidation
- **Target Files:** `src/components/accounts/AccountTable.tsx`
- **Accomplishments:**
  - Consolidated default model quota badges into exactly two unified items:
    1. `Gemini` (unified credit pool for Gemini variants)
    2. `Claude` (unified credit pool for Claude variants)
  - Rendered quotas in a single streamlined row (`grid grid-cols-2 gap-1.5`) preventing horizontal text truncation ("Gemini 3.1 Flas...").
  - Tightened table row padding to `py-0.5`, compacted email typography (`font-medium text-xs`), and streamlined action button spacing (`p-1`, `gap-0.5`).

### Subtask 03: Instances View Fix and Auto-Rotation Explanation Card
- **Target Files:**
  - `src/pages/Instances.tsx`
  - `src/stores/useInstanceStore.ts`
  - `src/components/settings/AutoSwitcherSettings.tsx`
- **Accomplishments:**
  - Fixed Instances view by adding an explicit empty state with an "Initialize Default Profile" trigger, a loading spinner during asynchronous IPC fetches, an error alert banner with a "Retry" button, and a manual spinning "Refresh" button in the header.
  - Supported silent background polling in `useInstanceStore.ts` to prevent UI jitter during automated 3s polling.
  - Added a comprehensive 5-pillar "How Auto Rotation Works" visual card in `AutoSwitcherSettings.tsx` explaining background quota polling (15s–600s), threshold triggers (<10%), dynamic best-profile selection, zero-loss task resumption, and background daemon execution.

### Subtask 04: Chinese Text Removal, Release Templates & Root README Attribution
- **Target Files:**
  - `src-tauri/Cargo.toml`
  - `03-ai-scripts/29-release-orchestrator.py`
  - `readme.md` & `README_EN.md`
  - `CHANGELOG.md` & `CHANGELOG_EN.md`
  - `.ai-memory/release/release-notes-v4.15.0.md`
- **Accomplishments:**
  - Scrubbed and translated Chinese comments in `src-tauri/Cargo.toml` to clean English.
  - Replaced Chinese template text in `03-ai-scripts/29-release-orchestrator.py` with English changelog entries.
  - Overhauled `readme.md` and `README_EN.md`: removed the Rivera / MD Animal Experiments line, updated attribution to `"Managed and enhanced by Md. Alim Ul Karim · Originally created by lbjlaq and upstream contributors"`, and added a dedicated upstream credit and gratitude section to `lbjlaq/Antigravity-Manager` at the end and in the about section.
  - Documented release highlights and quick-install one-liners in `.ai-memory/release/release-notes-v4.15.0.md`, `CHANGELOG.md`, and `CHANGELOG_EN.md`.

---

## Verification & Integrity Assurance

- **Branding Audit:** All user-facing titles verified as `"AGM by Alim"`; executable target verified as `"Anti-Gravity Tools by Alim"`.
- **UI Density Audit:** Row heights and quota badges in `AccountTable.tsx` verified compact without truncation.
- **Instances IPC Audit:** Empty and loading states gracefully handled in `Instances.tsx`.
- **Locale & Comment Audit:** Zero Chinese comments remaining in `Cargo.toml` and release generators.
- **Specification Compliance:** All 27 local CI quality gates, AppError structures, and coding guidelines verified intact.
