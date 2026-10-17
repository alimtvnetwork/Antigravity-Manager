# Subtask 06: Prompt Parameter Context Menu & Reset Filter Controls

**Parent Plan:** `.ai-memory/plans/antigravity-manager-agm-codebase-review-and-refactoring.md`  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md)  
**Owner:** Worker 02  
**Status:** Queued  

---

## 1. Owned Files
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/pages/instances/InstancesStates.tsx`
- `src/components/settings/EmailNotifications/SshGitmapSection.tsx`

---

## 2. Implementation Steps

### 2.1 Prompt Parameter Badges Context Menu in `DetailHeader.tsx`
1. In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - Create a parameter context menu state: `{ isOpen: boolean; x: number; y: number; type: 'seq' | 'tier' | 'context'; value: string }`.
   - Attach `onContextMenu` to:
     - **Sequence Badge (`rawSeq`):**
       - Right-click menu options:
         - *Copy Sequence Code* (e.g. `C001`)
         - *Copy Dual AGM/GitMap Code* (e.g. `[AGM:C001 | GM:<short_id>]`)
     - **Tier Badge (`tierInfo.tier`):**
       - Right-click menu options:
         - *Copy Origin Tier* (e.g. `USER_PROMPT`)
         - *Filter Prompts by this Tier*
     - **Instance Context Badge:**
       - Right-click menu options:
         - *Copy Instance Name / ID*
   - Prevent default browser menu (`e.preventDefault()`, `e.stopPropagation()`) and render lightweight dark-glass popover.

### 2.2 1-Click Search Clear Controls in `InstancesStates.tsx`
1. In `InstancesFilterBar` (`src/pages/instances/InstancesStates.tsx`):
   - Inside the search input container, check `searchQuery.length > 0`.
   - If true, display an interactive `✕` button that triggers `setSearchQuery('')`.
2. In `InstancesNoResultsState` (`src/pages/instances/InstancesStates.tsx`):
   - Add a primary outline button:
     ```tsx
     <button
         type="button"
         onClick={() => setSearchQuery('')}
         className="btn btn-sm btn-outline gap-1.5 mt-3 rounded-[5px] text-xs font-semibold"
     >
         <span>Clear Search Filter</span>
     </button>
     ```

### 2.3 Remote SSH Prompt Dispatcher Reset to Local
1. In `src/components/settings/EmailNotifications/SshGitmapSection.tsx`:
   - In Section 6 (Remote Machine & Instance Prompt Dispatcher Bar), locate Machine & Instance selector capsule.
   - Add a dedicated **"Reset to Local"** button:
     ```tsx
     <button
         type="button"
         onClick={() => {
             setTargetMachine('local');
             setTargetInstance('default');
             showToast('Reset prompt dispatch to local default instance', 'info');
         }}
         className="px-2 py-0.5 rounded-full text-[10px] font-mono text-slate-600 dark:text-slate-300 bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 dark:hover:bg-slate-700 border border-slate-200 dark:border-slate-700 transition-colors cursor-pointer"
         title="Reset dispatch target back to local machine"
     >
         Reset to Local
     </button>
     ```

---

## 3. Verification & Quality Gates
- `npm run build` compiles with zero errors.
- Visual inspection confirms right-clicking Sequence, Tier, or Context badges in DetailHeader opens the custom parameter context menu.
- Visual inspection confirms typing in the instance search bar renders an interactive `✕` that clears input on click.
- Visual inspection confirms "Reset to Local" in SshGitmapSection immediately sets target machine to `'local'` and target instance to `'default'`.
