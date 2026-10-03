# Institutional Memory: 28 - Navbar Pill Grouping & Instance Settings Parity

## 1. Domain & Classification
- **Domain**: UI Header Controls & Multi-Instance Engine Parity.
- **Classification**: Architectural Invariants & UI UX Design Rules.

---

## 2. Invariants & Rules

### 2.1 UI Segmented Control Grouping Invariant
- **Rule**: When grouping related micro-actions (e.g., Quick Clean `↺` alongside Preferences `🌙 EN ⌵`) or window controls (Minimize `—`, Maximize/Restore `🗗`, Close `✕`), do NOT render them as loose, disconnected individual circular buttons.
- **Implementation**: Wrap them into a unified, contiguous segmented pill container:
  ```tsx
  <div className="flex items-center rounded-full bg-gray-100 dark:bg-[#0c2438]/90 border border-gray-200/60 dark:border-[#15334d] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs z-50">
      {/* Individual buttons with rounded-l-full, rounded-r-full or borderless segments */}
  </div>
  ```
- **Rationale**: Segmented capsules eliminate visual clutter, keep desktop toolbars neat and aligned, and provide clear spatial grouping between distinct control sets.

### 2.2 Duplication & Settings Continuity Invariant (CLI & UI Parity)
- **Rule**: Any instance duplication initiated from the CLI (`agm instance duplicate / clone`) or frontend UI must invoke the identical underlying Rust service (`instance::copy_instance_with_options`).
- **Workspace Continuity**: Duplication must support preserving open projects and workspace configurations (`workspaceStorage`, `state.vscdb`, `storage.json`) by default (`copy_projects: true`).
- **Deep Settings Synchronization**: Cross-instance settings copy (`instance::copy_instance_settings` / `agm instance copy-settings`) must deep-merge theme configurations, Antigravity operational toggles (`turboMode`, `planReviewAlwaysProceed`), and execution policies without corrupting user customizations.
- **Baseline Defaults Enforcement**: The system must provide simple mechanisms (`enforce_default_settings` / `agm instance settings enforce-defaults`) to enforce verified performance baseline configurations across single or all instances.
