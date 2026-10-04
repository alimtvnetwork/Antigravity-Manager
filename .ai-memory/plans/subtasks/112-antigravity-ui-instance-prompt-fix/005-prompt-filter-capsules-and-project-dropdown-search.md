---
plan: 112-antigravity-ui-instance-prompt-fix
subtask: "005"
title: Streamline prompt filter capsules and add project dropdown search
domain: frontend
depends_on:
  - 004-prompt-tree-view-turn-cleanup-and-sorting.md
citations:
  app_spec: .ai-memory/plans/112-antigravity-ui-instance-prompt-fix.md
  coding_guidelines: AGENTS.md
  strictly_avoid: 02-spec/02-coding-guidelines/01-strictly-avoid.md
target_files:
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# 005 — Streamline Prompt Filter Capsules and Add Project Dropdown Search

## 1. Context & Rationale
In the current implementation of `src/components/instances/PromptTreeViewModal.tsx`, the search and filter header in the left-hand navigation pane renders a horizontal list of 6 loose filter buttons (`All`, `Running`, `Latest Conv`, `Latest Prompt`, `Pinned`, `Archived`). This clutters the UI, causes horizontal overflow on standard resolutions, and contradicts the segmented pill capsule styling mandated by `AGENTS.md`.

Furthermore, users navigating large workspaces containing dozens of projects currently have to scroll through all projects or rely on a global text search. There is no scoped project selector dropdown to isolate conversations and searches to a single specific project (e.g. searching only inside a selected repository).

## 2. Target Files and Symbols
- **`src/components/instances/PromptTreeViewModal.tsx`**:
  - `activeFilter`: Refactor type and state to prioritize `running`, `latest`, `pinned`, while delegating `all`, `archived`, and granular options to a dropdown.
  - `selectedProjectIdFilter`: New state (`string`, default `'all'`) to scope the tree and search to a specific project.
  - Filter Bar JSX: Replace the loose button list with a contiguous segmented pill capsule (`[Running]`, `[Latest]`, `[Pinned]`) plus a compact dropdown (`More Filters`).
  - Search Header JSX: Introduce a project selector dropdown above or alongside the search input defaulting to `"All Projects"`.

## 3. Detailed Technical Requirements

### 3.1. Segmented Pill Capsule Design
As defined in `AGENTS.md` ("Minimalist & Contextual UI: When rendering adjacent toolbar actions or desktop window controls, always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons"):
- Consolidate the primary quick-filter actions into a contiguous segmented pill:
  - **Running**: Filters projects containing active running conversations.
  - **Latest**: Filters projects with the most recently updated conversations (collapses `latest_conv` and `latest_prompt` into an intuitive top-tier filter).
  - **Pinned**: Filters pinned projects.
- Place a compact dropdown button next to the capsule for secondary filters:
  - `All Projects` (resets filter)
  - `Archived Projects` (shows archived list with count badge)
  - `Latest Conversations`
  - `Latest Prompts`

### 3.2. Scoped Project Selector Dropdown
- Add a dropdown selector positioned right above or beside the search bar:
  - Value: `selectedProjectIdFilter` (`'all'` by default).
  - Options:
    - `"All Projects"` (`value="all"`)
    - Dynamic options mapped from `treeData` sorted by name: `<option value={proj.project_id}>{proj.repo_name} ({proj.conversations.length})</option>`.
- **Filtering Behavior**:
  - When `selectedProjectIdFilter !== 'all'`, `activeProjects` is filtered strictly to `project.project_id === selectedProjectIdFilter`.
  - When searching with `searchQuery`, search matches are evaluated **only within the scoped project(s)**.
  - Changing the project dropdown to a specific project automatically expands that project node in the tree and sets `selectedProject`.

### 3.3. Keyboard & Visual Polish
- Ensure dropdowns have dark mode styling consistent with Antigravity design system:
  `bg-white dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border-slate-200 dark:border-[#15334d]`.
- Provide smooth transition effects when switching filter capsules.

## 4. Exact Implementation Details

### 4.1. State Additions in `PromptTreeViewModal.tsx`
```tsx
// Scoped Project Filter
const [selectedProjectScope, setSelectedProjectScope] = useState<string>('all');

// Streamlined Filter Mode
type PrimaryFilter = 'running' | 'latest' | 'pinned';
type ExtendedFilter = 'all' | 'running' | 'latest' | 'pinned' | 'archived';
const [activeFilter, setActiveFilter] = useState<ExtendedFilter>('all');
```

### 4.2. Filter Logic Refactoring
```tsx
// Compute filtered projects respecting both project dropdown scope and search query
const filteredProjects = useMemo(() => {
    let list = activeFilter === 'archived' ? archivedProjects : activeProjects;

    // 1. Apply Project Dropdown Scope
    if (selectedProjectScope !== 'all') {
        list = list.filter((p) => p.project_id === selectedProjectScope);
    }

    // 2. Apply Search Query
    if (!searchQuery.trim()) return list;

    const query = searchQuery.toLowerCase();
    return list.filter((p) => {
        const matchesRepo = p.repo_name.toLowerCase().includes(query);
        const matchesConv = p.conversations.some(
            (c) =>
                c.title?.toLowerCase().includes(query) ||
                c.short_id?.toLowerCase().includes(query) ||
                c.prompt_preview_200w?.toLowerCase().includes(query)
        );
        return matchesRepo || matchesConv;
    });
}, [activeProjects, archivedProjects, activeFilter, selectedProjectScope, searchQuery]);
```

### 4.3. JSX Layout for Search & Filter Capsule
```tsx
{/* Search Input & Project Scope Header */}
<div className="p-3 border-b border-slate-200 dark:border-[#15334d] shrink-0 space-y-2">
    {/* Project Selector Dropdown */}
    <div className="flex items-center gap-1.5">
        <label className="text-[11px] font-semibold text-slate-500 dark:text-slate-400 shrink-0">
            Project:
        </label>
        <select
            value={selectedProjectScope}
            onChange={(e) => {
                const val = e.target.value;
                setSelectedProjectScope(val);
                if (val !== 'all') {
                    setExpandedProjects((prev) => ({ ...prev, [val]: true }));
                    const target = treeData.find((p) => p.project_id === val);
                    if (target) setSelectedProject(target);
                }
            }}
            className="w-full rounded-[6px] bg-white dark:bg-[#0c2438] text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] text-xs px-2 py-1 focus:outline-none focus:ring-1 focus:ring-blue-500 cursor-pointer truncate"
        >
            <option value="all">All Projects ({treeData.length})</option>
            {treeData.map((p) => (
                <option key={p.project_id} value={p.project_id}>
                    {p.repo_name} ({p.conversations.length} convos)
                </option>
            ))}
        </select>
    </div>

    {/* Search Input */}
    <div className="relative">
        <Search className="absolute left-3 top-2.5 h-3.5 w-3.5 text-slate-400" />
        <input
            type="text"
            placeholder={
                selectedProjectScope === 'all'
                    ? "Search all projects, convos, or prompts..."
                    : `Search inside ${treeData.find(p => p.project_id === selectedProjectScope)?.repo_name || 'project'}...`
            }
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full rounded-[6px] bg-white dark:bg-[#0c2438] pl-9 pr-3 py-1.5 text-xs text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] focus:outline-none focus:ring-2 focus:ring-blue-500/30"
        />
    </div>

    {/* Segmented Filter Capsule + Compact Dropdown */}
    <div className="flex items-center justify-between pt-0.5">
        {/* Contiguous Segmented Pill Capsule */}
        <div className="inline-flex rounded-full bg-slate-100 dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] p-0.5 shadow-2xs divide-x divide-slate-200 dark:divide-[#15334d]">
            <button
                type="button"
                onClick={() => setActiveFilter(activeFilter === 'running' ? 'all' : 'running')}
                className={cn(
                    "px-2.5 py-0.5 text-[10px] font-medium rounded-full transition-colors cursor-pointer",
                    activeFilter === 'running'
                        ? "bg-blue-600 text-white shadow-xs font-semibold"
                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                )}
            >
                Running
            </button>
            <button
                type="button"
                onClick={() => setActiveFilter(activeFilter === 'latest' ? 'all' : 'latest')}
                className={cn(
                    "px-2.5 py-0.5 text-[10px] font-medium rounded-full transition-colors cursor-pointer",
                    activeFilter === 'latest'
                        ? "bg-blue-600 text-white shadow-xs font-semibold"
                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                )}
            >
                Latest
            </button>
            <button
                type="button"
                onClick={() => setActiveFilter(activeFilter === 'pinned' ? 'all' : 'pinned')}
                className={cn(
                    "px-2.5 py-0.5 text-[10px] font-medium rounded-full transition-colors cursor-pointer",
                    activeFilter === 'pinned'
                        ? "bg-blue-600 text-white shadow-xs font-semibold"
                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                )}
            >
                Pinned
            </button>
        </div>

        {/* More Filters Dropdown */}
        <select
            value={activeFilter === 'running' || activeFilter === 'latest' || activeFilter === 'pinned' ? '' : activeFilter}
            onChange={(e) => setActiveFilter(e.target.value as ExtendedFilter)}
            className="rounded-[6px] bg-slate-100 dark:bg-[#0c2438] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#15334d] text-[10px] px-2 py-0.5 focus:outline-none cursor-pointer"
        >
            <option value="all">Filter: All</option>
            <option value="archived">Archived ({archivedCount})</option>
        </select>
    </div>
</div>
```

## 5. Constraints & Non-Negotiables
- Must strictly use contiguous segmented pill capsule styling (`rounded-full`, shared border, divider lines) for adjacent buttons.
- No loose unbordered button bars.
- Boolean variables must follow positive prefixes (`is...`, `has...`).
- Zero build warnings or TypeScript compile errors.
- Do not bump version or update changelog in this subtask.

## 6. Out of Scope
- Backend modifications (handled in subtasks 007 and 008).
- Progress bar and Instance Card theming (handled by Worker 01 in 001-003).

## 7. Verification & Acceptance Tests
1. **Pill Capsule Visual Check**: Open PromptTreeViewModal. The filter bar renders as a clean pill capsule containing `Running`, `Latest`, and `Pinned`, with subtle divider lines and active state highlights.
2. **Project Scope Dropdown Check**: Select a specific project from the dropdown. The list immediately filters to show only the selected project.
3. **Scoped Search Check**: Type a search term that exists in multiple projects. Only results within the selected project are shown.
4. **Build Gate**: Run `npm run build` to verify clean frontend compilation.
