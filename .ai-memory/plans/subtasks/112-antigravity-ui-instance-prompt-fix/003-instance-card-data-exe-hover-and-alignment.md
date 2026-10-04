# Subtask 003 — Instance Card Data & Exe Path Display, Hover Effects & Strict Header Alignment

## Status: Pending
## Priority: High
## Assigned Worker: Worker 01 (Frontend Specialist — UI, Progress Bar, Themes & Cards)

---

## 1. Objectives & User Feedback Analysis

In user feedback for Plan 112:
1. **Default Instance Executable Path Parity**:
   - Cloned/custom instances display both their Data directory and their Executable path.
   - However, the **Default instance** currently only shows the Data directory and hides the Executable path section because `inst.config.executable_path` is empty/null by default.
   - Requirement: For the Default profile, display both the Data directory AND the Executable path (falling back to the configured IDE path or default system executable path), ensuring visual parity across all instance cards.
2. **Card Hover Dark Mode Transition**:
   - Add a subtle, sleek darker tone when hovering over non-active cards in dark mode (mimicking VS Code's editor focus behavior: `dark:hover:bg-[#061421]`) with CSS3 transition (`transition-all duration-200`).
   - In light mode, provide a crisp subtle tone: `hover:bg-slate-50/90`.
3. **Email Section Card Hover Interaction**:
   - When hovering over the card, the email container should transition to a darker background (`dark:group-hover:bg-[#050f18]`), with an amber border highlight (`group-hover:border-amber-400/50`).
   - The email text inside should highlight in amber/yellow (`group-hover:text-amber-500 dark:group-hover:text-amber-300`) to provide instant visual feedback on account assignment.
4. **Header Control Alignment & Elimination of Wrapping**:
   - Currently, instance card headers use `flex-wrap`. When a card displays `#1`, `Instance Name`, `Prompt Active`, and `Set Default`, the items wrap onto a second line, making that card taller and causing vertical misalignment across the card grid.
   - Requirement: Prevent header row wrapping (`flex-nowrap h-6`), standardize the height and padding of the "Set Default" button to match the `DEFAULT` badge identically, and align all controls on the same horizontal plane.

---

## 2. File Targets

| File Path | Description of Changes |
| :--- | :--- |
| `src/pages/Instances.tsx` | Fallback executable path resolution for Default instance, CSS3 hover effects for card and email section, `flex-nowrap h-6` header alignment, badge/button standardization. |

---

## 3. Exact Implementation Specifications

### 3.1 Resolving Effective Executable Path for Default Instance

In `src/pages/Instances.tsx`, inside `filteredInstances.map((inst, index) => { ... })`:
```tsx
// Resolve effective executable path for display
const defaultExePath =
    config?.antigravity_ide_executable ||
    config?.antigravity_executable ||
    'Antigravity IDE (Default System Path)';

const effectiveExePath =
    inst.config.executable_path || (inst.config.is_default ? defaultExePath : null);
```

Render the executable badge whenever `effectiveExePath` is present:
```tsx
{/* Executable path badge (renders for custom instances AND default instance) */}
{effectiveExePath ? (
    <div className="flex items-center justify-between gap-1.5 px-2 py-1 rounded-[5px] bg-purple-50/30 dark:bg-slate-900/60 border border-slate-200/80 dark:border-[#15334d] text-[10px] group/exe transition-colors">
        <div className="flex items-center gap-1 min-w-0 flex-1 text-slate-700 dark:text-slate-300" title={effectiveExePath}>
            <Cpu className="w-3 h-3 shrink-0 text-cyan-500" />
            <span className="truncate font-mono" title={effectiveExePath}>
                {truncatePath(effectiveExePath)}
            </span>
        </div>
        <button
            type="button"
            onClick={async (e) => {
                e.stopPropagation();
                try {
                    await navigator.clipboard.writeText(effectiveExePath);
                    showToast('Executable path copied to clipboard', 'info');
                } catch {
                    showToast('Failed to copy executable path', 'error');
                }
            }}
            className="opacity-60 group-hover/exe:opacity-100 p-0.5 rounded-[5px] hover:bg-slate-200 dark:hover:bg-[#15334d] text-slate-500 dark:text-slate-300 transition-opacity cursor-pointer shrink-0"
            title="Copy executable path"
        >
            <Copy className="w-3 h-3" />
        </button>
    </div>
) : null}
```

---

### 3.2 Non-Active Card Dark Hover & CSS3 Transitions

In `src/pages/Instances.tsx`, update the card container's `className`:
```tsx
className={cn(
    "group rounded-xl border transition-all duration-200 flex flex-col justify-between bg-white dark:bg-[#0a1e30] overflow-hidden shadow-xs backdrop-blur-xs",
    isActive
        ? "border-blue-500 shadow-lg ring-2 ring-blue-500/30 bg-blue-50/15 dark:bg-[#0c2438]"
        : "border-gray-200/50 dark:border-[#15334d]/60 hover:border-gray-300/80 dark:hover:border-blue-500/40 hover:bg-slate-50/90 dark:hover:bg-[#061421]"
)}
```

---

### 3.3 Email Section Hover Highlight & Yellow Accent

In `src/pages/Instances.tsx`, update the Bound Account / Email Section:
```tsx
{/* Bound Account / Email Section */}
<div className={cn(
    "py-1 px-2.5 rounded-md bg-gray-50/80 dark:bg-[#0c2438]/90 border border-gray-100 dark:border-[#15334d] mb-2.5 flex items-center justify-between gap-1.5 transition-all duration-200",
    "group-hover:border-amber-400/40 dark:group-hover:border-amber-400/50 dark:group-hover:bg-[#050f18]"
)}>
    <div className="flex items-center gap-1.5 min-w-0 flex-1">
        <Mail className="w-3 h-3 text-gray-400 group-hover:text-amber-400 transition-colors shrink-0" />
        <span className="text-[10px] text-gray-500 dark:text-gray-400 font-medium shrink-0">
            Account:
        </span>
        {displayEmail ? (
            <span
                className={cn(
                    "px-1.5 py-0.5 rounded-md text-[11px] font-semibold font-mono border flex items-center gap-1 min-w-0 shadow-2xs transition-colors",
                    theme.emailPill,
                    "group-hover:text-amber-600 dark:group-hover:text-amber-300"
                )}
                title={displayEmail}
            >
                <span className={cn("w-1.5 h-1.5 rounded-full shrink-0", theme.dot)} />
                <span className="truncate">{displayEmail}</span>
            </span>
        ) : (
            <span className="text-[11px] text-gray-400 italic">
                Unassigned
            </span>
        )}
    </div>
    {/* Tier badge on the right remains stable */}
</div>
```

---

### 3.4 Unify Header Alignment & Standardize "Set Default" Button

#### Problem
The header currently wraps with `flex-wrap`, causing uneven heights across grid rows.
The `DEFAULT` badge and `Set Default` button have mismatched dimensions and padding:
- `DEFAULT` badge: `px-1.5 py-0.5 text-[9px] font-bold bg-indigo-500/15`
- `Set Default` button: loose padding, causing jitter.

#### Solution
1. Lock header row height to `h-6` with `flex-nowrap`:
```tsx
<div className="flex items-center justify-between gap-2 mb-2.5 h-6 flex-nowrap min-w-0">
    <div className="flex items-center gap-1.5 min-w-0 flex-1 flex-nowrap overflow-hidden">
        <span
            className={cn(
                "w-2.5 h-2.5 rounded-full shrink-0",
                inst.is_running ? "bg-emerald-500 shadow-xs shadow-emerald-500/50 animate-pulse" : "bg-gray-300 dark:bg-gray-600"
            )}
        />
        <span className="px-1.5 py-0.5 rounded-[5px] text-xs font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 shrink-0">
            #{seqNumber}
        </span>
        <h3 className={cn("font-bold text-xs truncate shrink min-w-0", isActive ? "text-blue-900 dark:text-blue-100" : "text-gray-900 dark:text-base-content")} title={inst.config.name}>
            {inst.config.name}
        </h3>
        {hasActiveTask && (
            <button
                type="button"
                onClick={() => setPromptTreeInstance({ id: inst.config.id, name: inst.config.name })}
                className="inline-flex items-center gap-1 px-1.5 h-5 rounded-[5px] text-[9px] font-bold bg-cyan-500/15 hover:bg-cyan-500/25 text-cyan-700 dark:text-cyan-300 border border-cyan-500/30 shrink-0 cursor-pointer transition-colors"
                title="Active prompt/task running - Click to open Prompt Tree"
            >
                <span className="w-1.5 h-1.5 rounded-full bg-cyan-500 animate-pulse shrink-0" />
                <span>Prompt</span>
            </button>
        )}
        {inst.config.is_default ? (
            <span className="h-5 px-1.5 rounded-[5px] text-[9px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-400/30 flex items-center justify-center shrink-0">
                DEFAULT
            </span>
        ) : (
            <button
                onClick={async () => {
                    try {
                        await setDefaultInstance(inst.config.id);
                        showToast(t('instances.set_default_toast', 'Default profile updated successfully'), 'success');
                    } catch (e: any) {
                        setActionError(e?.toString() || 'Failed to set default profile');
                    }
                }}
                className="h-5 px-1.5 rounded-[5px] text-[9px] font-medium text-gray-400 hover:text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-950/30 border border-dashed border-gray-300 dark:border-[#15334d] transition-colors cursor-pointer flex items-center gap-1 shrink-0"
                title="Set as default profile"
            >
                <Star className="w-2.5 h-2.5" />
                <span>Set Default</span>
            </button>
        )}
    </div>
    {/* Right controls: Active status and action icons */}
    <div className="shrink-0 flex items-center gap-1">
        ...
    </div>
</div>
```

---

## 4. Acceptance Criteria & Verification

1. **Default Instance Executable Display**:
   - Inspect the Default instance card.
   - Verify that both the Data directory (`Folder` icon) AND the Executable path (`Cpu` icon) are visible.
   - Verify the copy button on the executable path copies the path successfully.
2. **Card Hover Behavior**:
   - In Dark mode, hover over non-active cards.
   - Verify card background smoothly darkens to `#061421` over 200ms without jitter.
   - Verify the email sub-card turns darker `#050f18` with an amber border, and the email text highlights in yellow `#fde047` (`amber-300`).
3. **Card Grid Visual Alignment**:
   - Create 4+ instances with various name lengths.
   - Verify card headers never wrap onto multiple lines (`h-6 flex-nowrap`).
   - Verify all "Set Default" buttons and "DEFAULT" badges align perfectly on the same vertical baseline.
