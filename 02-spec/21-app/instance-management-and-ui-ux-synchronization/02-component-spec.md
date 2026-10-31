# Instance Management & UI/UX Synchronization — Component Specification

## 1. Overview
This specification details the frontend component structures, layout contracts, design token compliance, and interaction patterns required for instance list view rows, switch account dialogs, and instance audit modals.

## 2. Component Layout & Interaction Contracts

### 2.1 Instance List Table Row (`InstanceRowCells.tsx`)
- **Combined Status, Process & Environment Cell:**
  - Merge the legacy separate `StatusCell` and `IdeExeAndDataCell` into a unified `StatusAndPathsCell`.
  - Layout structure:
    ```tsx
    <td className="px-2 py-1 whitespace-nowrap min-w-[340px] max-w-[500px]">
      <div className="flex items-center justify-between gap-2 text-[11px] font-mono flex-nowrap">
        <div className="flex items-center gap-2 min-w-0 flex-1 truncate flex-nowrap">
          {/* 1. Green Running Indicator / PID Pill */}
          {inst.is_running ? (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/30 shrink-0">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 dark:bg-[#1af18d] animate-pulse shrink-0" />
              <span>{inst.pid ? `PID: ${inst.pid}` : 'Running'}</span>
            </span>
          ) : (
            <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-slate-100/90 text-slate-600 dark:bg-[#090611]/90 dark:text-slate-400 border border-slate-200 dark:border-[#40285e] shrink-0">
              Idle
            </span>
          )}

          <span className="text-slate-400 dark:text-slate-600 select-none text-[10px]">·</span>

          {/* 2. Trailing Executable Path */}
          <div className="flex items-center gap-1 min-w-0 shrink-0 text-slate-700 dark:text-slate-200 font-semibold truncate max-w-[140px]" title={`IDE: ${exeFull || exeEnding}`}>
            <Terminal className="w-3.5 h-3.5 text-slate-400 shrink-0" />
            <span className="truncate">{exeEnding}</span>
          </div>

          <span className="text-slate-400 dark:text-slate-600 select-none text-[10px]">·</span>

          {/* 3. Trailing Data Directory */}
          <div className="flex items-center gap-1 min-w-0 text-[10px] text-slate-500 dark:text-slate-400 truncate max-w-[150px]" title={`Data: ${fullPath}`}>
            <Folder className="w-3.5 h-3.5 text-slate-400 shrink-0" />
            <span className="truncate">{dataEnding}</span>
          </div>
        </div>

        {/* 4. Contiguous Segmented Copy Action Capsule */}
        <div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#40285e] bg-slate-100/80 dark:bg-[#090611]/80 p-0.5 shrink-0 ml-1 shadow-2xs">
          <button onClick={handleCopyExe} title="Copy executable path" className="p-1 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/60 dark:hover:bg-[#1f1535] rounded-full transition-colors">
            <Terminal className="w-3 h-3" />
          </button>
          <span className="w-px h-2.5 bg-slate-200 dark:bg-[#40285e] shrink-0" />
          <button onClick={onCopy} title="Copy data path" className="p-1 text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-200/60 dark:hover:bg-[#1f1535] rounded-full transition-colors">
            <Copy className="w-3 h-3" />
          </button>
        </div>
      </div>
    </td>
    ```

### 2.2 Switch Account Dialog (`SwitchAccountDialog.tsx`)
- **Royal Violet Tokens:**
  - Base modal background: `dark:bg-[var(--ui-surface-1)]/95` (`#171025`).
  - Subtle border: `dark:border-[var(--ui-border-subtle)]` (`#40285e`).
  - Brand interactive accent: `bg-[var(--ui-interactive)]` (`#7c3aed`), focus ring: `focus:ring-[var(--ui-focus-ring)]`.
  - Success badge / current account: `text-[var(--ui-success)]` (`#1af18d`).
- **Interactive Capabilities:**
  - Whole-card row selection: clicking anywhere on `.ui-interactive-row` selects account.
  - Keyboard accessibility: `Escape` safely dismisses dialog when not switching; `Enter`/`Space` selects card.
  - Quota Visualization Capsule:
    - Display compact quota bars alongside a segmented capsule header:
      `4H: <val>% | Weekly: <val>% | Total: <val>%`.

### 2.3 Instance Audit Trail Modal (`InstanceAuditTrailModal.tsx`)
- **Dual-Tab Segmented Control:**
  - `Switch Cycles ({totalSwitches})` vs `All Operations ({totalOperations})`.
  - Smart Auto-Tab Switch: if `recentSwitches.length === 0` and `totalOperations > 0`, default to `operations` tab automatically.
- **Action Badge Styling:**
  - Comprehensive badges for action codes 1–14 (LaunchInstance: Sky, StopInstance: Amber, RestartInstance: Purple, SwitchAccount: Teal, etc.).
