---
plan: 135-instance-restart-button-sync-icons-and-running-projects-fix
subtask: "02"
title: Semantic Sync Icons Disambiguation & Anti-Confusion Invariant
domain: frontend-react-ui-icons
depends_on:
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
citations:
  architecture_spec: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md
  component_spec: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  root_cause_analysis: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
  coding_guidelines: 02-spec/02-coding-guidelines/readme.md
target_files:
  - src/pages/Instances.tsx
  - src/components/instances/InstanceTable.tsx
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# Subtask 02: Semantic Sync Icons Disambiguation & Anti-Confusion Invariant

## 1. Objectives & User Requirements

### User Prompt Verbatim:
> "And there are a couple of sync buttons, sync icons, feels like restart. Try to have a different sync icon, I believe, that would be making more sense."

### The Anti-Confusion Invariant:
- In desktop operating systems and web applications, circular rotation arrows (`RotateCw`, `RefreshCw`, `RotateCcw`) universally symbolize "Restart", "Reboot", or "Reload".
- When read-only or background metadata synchronization buttons use circular rotating glyphs, users mistakenly believe the application is restarting their active IDE instances and terminate running workflows out of caution.
- **Strict Invariant**: `RotateCcw` is **strictly and exclusively reserved** for Instance Restart operations.
- All background synchronizations, quota evaluations, credential purges, and configuration syncs must use dedicated, semantically unmistakable domain icons.

---

## 2. Icon Transformation Matrix

| Action / Button | Old Ambiguous Icon | New Distinct Semantic Icon | Target Component / Location | Rationale |
|---|---|---|---|---|
| **Instance Restart** | N/A (Missing) | `RotateCcw` (**Exclusively Reserved**) | Primary Split Capsule `[Square \| RotateCcw]` | Counter-clockwise circular arrow is the standard desktop glyph for restarting an instance on its bound account. |
| **Sync All** | `RotateCw` (circular arrow) | `FolderSync` | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L657) (Top Toolbar) | Folder-synchronization glyph accurately represents refreshing workspaces, process PIDs, and quota caches without rebooting. |
| **Eval Quota** | `RotateCw` (circular arrow) | `Sparkles` | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L741) (Top Toolbar) | Sparkles glyph communicates intelligent AI quota evaluation and auto-rotation analysis. |
| **Sync PID & Quota** | `RotateCw` / `RefreshCw` | `Cpu` | [`src/components/instances/InstanceTable.tsx`](file:///d:/work/Antigravity-Manager/src/components/instances/InstanceTable.tsx#L525) & Card Actions | Microprocessor glyph accurately reflects querying OS process table PIDs. |
| **Wipe Credentials** | `RotateCcw` (restart arrow) | `KeyRound` | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L1721) (Card Dropdown) | Key glyph clearly conveys purging authentication credentials, eliminating any false implication of an instance restart. |
| **Account Switch** | `ArrowLeftRight` | `ArrowLeftRight` | Primary Action Pill in Table and Card views | Bidirectional arrows clearly represent account selection and switching. |
| **Settings & Sync** | `SlidersHorizontal` | `SlidersHorizontal` | Dropdown Menus | Sliders indicate profile configuration and synchronization options. |

---

## 3. Granular Implementation Steps

### 3.1 `src/pages/Instances.tsx`
1. **Verify Lucide-React Imports**:
   - Ensure `FolderSync`, `Sparkles`, `KeyRound`, `Cpu`, `ArrowLeftRight`, and `RotateCcw` are imported:
     ```typescript
     import {
         // ...
         FolderSync,
         Sparkles,
         KeyRound,
         Cpu,
         ArrowLeftRight,
         RotateCcw,
         // ...
     } from 'lucide-react';
     ```
2. **"Sync All" Button in Toolbar** (around line 657):
   - Replace `<RotateCw className={cn("w-3.5 h-3.5 text-cyan-500", isSyncingAll && "animate-spin")} />`
   - With:
     ```tsx
     <FolderSync className={cn("w-3.5 h-3.5 text-cyan-500", isSyncingAll && "animate-pulse")} />
     ```
   - *Design rationale*: `animate-pulse` signifies active background I/O without spinning an asymmetrical folder glyph off-axis.
3. **"Eval Quota" Button in Toolbar** (around line 741):
   - Replace `<RotateCw className={cn("w-3.5 h-3.5", isLoading && "animate-spin")} />`
   - With:
     ```tsx
     <Sparkles className={cn("w-3.5 h-3.5 text-amber-500", isLoading && "animate-pulse")} />
     ```
4. **"Wipe Credentials" Button in Card More Options Dropdown** (around line 1721):
   - Replace `<RotateCcw className="w-3.5 h-3.5" />`
   - With:
     ```tsx
     <KeyRound className="w-3.5 h-3.5 text-amber-500" />
     ```

### 3.2 `src/components/instances/InstanceTable.tsx`
1. **Verify Dropdown Action Icons**:
   - "Sync PID & Quota": `<Cpu className="w-3.5 h-3.5 text-teal-500" />`
   - "Restart Instance": `<RotateCcw className="w-3.5 h-3.5 text-amber-500" />`
   - "Settings & Sync": `<SlidersHorizontal className="w-3.5 h-3.5 text-blue-500" />`
   - "Clone Profile": `<Copy className="w-3.5 h-3.5 text-indigo-500" />`
2. **Verify Primary Action Split Capsule**:
   - Left button: Stop (`Square`)
   - Right button: Restart (`RotateCcw`)
   - Adjacent button: Switch (`ArrowLeftRight`)

### 3.3 `src/components/instances/PromptTreeViewModal.tsx`
1. Ensure tree refresh button uses either `FolderSync` or `RefreshCw` paired with an explicit text label *"Refresh Tree"* to prevent ambiguity with instance lifecycle operations.

---

## 4. Visual Aesthetics, Styling & Accessibility

1. **Dark Glass & High-Contrast Support**:
   - Ensure all replaced icons maintain high contrast across light backgrounds (`text-slate-700 hover:text-slate-900`) and dark glass themes (`text-slate-300 hover:text-white`).
2. **Micro-Interactions**:
   - Background sync operations utilize `animate-pulse` rather than `animate-spin` when using non-circular icons (`FolderSync`, `Sparkles`, `KeyRound`).
   - Only truly circular glyphs (e.g. `RotateCw` used as a transient loading spinner inside a button) may spin.
3. **Descriptive Accessible Tooltips**:
   - "Sync All": `title="Synchronize process PIDs and account quotas across all instances"`
   - "Eval Quota": `title="Evaluate rolling quota across all monitored instances and auto-rotate low quota accounts"`
   - "Wipe Credentials": `title="Clear saved tokens, authentication state, and session keys"`
   - "Restart Instance": `title="Restart Instance on Current Account"`

---

## 5. Verification & Acceptance Criteria

- [ ] Audit every occurrence of `RotateCcw` across `src/pages/` and `src/components/instances/` to confirm it is used ONLY for Restart actions.
- [ ] In `Instances.tsx` toolbar, "Sync All" renders `FolderSync` icon.
- [ ] In `Instances.tsx` toolbar, "Eval Quota" renders `Sparkles` icon.
- [ ] In `Instances.tsx` card dropdown, "Wipe Credentials" renders `KeyRound` icon.
- [ ] In `InstanceTable.tsx` dropdown, "Sync PID & Quota" renders `Cpu` icon.
- [ ] In both Table and Card modes, "Restart Instance" renders `RotateCcw` icon.
- [ ] In both Table and Card modes, "Switch Account" renders `ArrowLeftRight` icon.
- [ ] Icons render crisply with high contrast in both light and dark themes.
