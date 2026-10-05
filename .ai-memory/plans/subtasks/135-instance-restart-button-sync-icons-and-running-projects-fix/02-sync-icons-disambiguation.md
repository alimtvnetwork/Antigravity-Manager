# Subtask 02: Semantic Sync Icons Disambiguation & Anti-Confusion Invariant

- **Subtask Identifier**: `02-sync-icons-disambiguation`
- **Parent Task**: `135-instance-restart-button-sync-icons-and-running-projects-fix`
- **Specification References**:
  - Architecture Spec: [`02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md`](file:///d:/work/Antigravity-Manager/02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md)
  - Component Spec: [`02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md`](file:///d:/work/Antigravity-Manager/02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md)
- **Status**: Ready for Execution

---

## 1. Objectives & User Requirements

The user specified:
> "And there are a couple of sync buttons, sync icons, feels like restart. Try to have a different sync icon, I believe, that would be making more sense."

### The Anti-Confusion Invariant:
- Circular rotation arrows (`RotateCw`, `RefreshCw`, `RotateCcw`) universally denote "Restart" or "Reload".
- When read-only or background metadata sync buttons use circular rotation icons, users believe their active Antigravity IDE instances will be rebooted or closed.
- **Rule**: `RotateCcw` is **strictly and exclusively reserved** for Restart operations.
- All synchronization, quota evaluation, credential maintenance, and setting synchronization actions must use dedicated, semantically clear domain icons.

---

## 2. Icon Transformation Matrix

| Action / Button | Current Confusing Icon | New Distinct Semantic Icon | Target Component / Location | Rationale |
|---|---|---|---|---|
| **Sync All** | `RotateCw` (circular arrow) | `FolderSync` | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L657) (Top Toolbar) | Distinguishes full multi-workspace / profile synchronization from instance restarts. |
| **Eval Quota** | `RotateCw` (circular arrow) | `Sparkles` | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L741) (Top Toolbar) | Represents intelligent AI quota evaluation and smart auto-rotation candidate selection. |
| **Wipe Credentials** | `RotateCcw` (restart arrow) | `KeyRound` | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L1721) (Card Dropdown) | Cleans auth tokens and session keys; `RotateCcw` falsely implied profile reboot. |
| **Sync PID & Quota** | `Cpu` (already hardened) | `Cpu` | [`src/components/instances/InstanceTable.tsx`](file:///d:/work/Antigravity-Manager/src/components/instances/InstanceTable.tsx#L525) & Card Actions | Microprocessor icon accurately reflects reading OS process table PIDs. |
| **Account Switch** | `ArrowLeftRight` | `ArrowLeftRight` | Primary Action Pill in Table and Card views | Bidirectional arrows clearly represent switching between accounts. |
| **Instance Restart** | Missing | `RotateCcw` (**Exclusively Reserved**) | Primary Split Capsule `[Square \| RotateCcw]` | Counter-clockwise circular arrow is the unambiguous desktop symbol for restart. |

---

## 3. File Locations & Detailed Changes

### 3.1 `src/pages/Instances.tsx`
1. **Import Verification**:
   - Ensure `FolderSync`, `Sparkles`, `KeyRound`, `Cpu`, `ArrowLeftRight`, `RotateCcw` are imported from `'lucide-react'`:
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
     *(Note: `FolderSync` with pulse gives clear visual activity without mimicking a rotating reboot).*
3. **"Eval Quota" Button in Toolbar** (around line 741):
   - Replace `<RotateCw className={cn("w-3.5 h-3.5", isLoading && "animate-spin")} />`
   - With:
     ```tsx
     <Sparkles className={cn("w-3.5 h-3.5 text-amber-500", isLoading && "animate-pulse")} />
     ```
4. **"Wipe Credentials" Button in Card More Dropdown** (around line 1721):
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

---

## 4. Visual Aesthetics & Styling Constraints

1. **Light & Dark Mode Consistency**:
   - Ensure text and icon colors maintain high contrast in both Tailwind light (`bg-white`, `text-slate-700`) and dark glass (`dark:bg-[#071a27]`, `dark:text-slate-200`) environments.
2. **Micro-Interactions**:
   - For in-flight background operations, prefer `animate-pulse` or a dedicated spinning ring rather than spinning a non-circular icon like `FolderSync` or `KeyRound` out of its axis.
3. **Tooltip Clarity**:
   - Keep tooltips descriptive:
     - "Sync All": *"Synchronize process PIDs and account quotas across all instances"*
     - "Eval Quota": *"Evaluate rolling quota across all monitored instances and auto-rotate any low quota accounts"*
     - "Wipe Credentials": *"Clear saved tokens, authentication state, and session keys"*

---

## 5. Verification & Acceptance Criteria

- [ ] Audit every occurrence of `RotateCcw` across `src/pages/` and `src/components/instances/` to confirm it is used ONLY for Restart actions.
- [ ] In `Instances.tsx` toolbar, "Sync All" displays `FolderSync` icon.
- [ ] In `Instances.tsx` toolbar, "Eval Quota" displays `Sparkles` icon.
- [ ] In `Instances.tsx` card dropdown, "Wipe Credentials" displays `KeyRound` icon.
- [ ] In `InstanceTable.tsx` dropdown, "Sync PID & Quota" displays `Cpu` icon.
- [ ] Icons render crisply with proper colors and contrast in both light and dark modes.
