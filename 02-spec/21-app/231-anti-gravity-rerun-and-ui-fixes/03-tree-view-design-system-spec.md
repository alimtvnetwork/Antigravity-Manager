# Feature & Architectural Spec: Tree View Design System & Filter Row Compaction

> **/goal** Define the complete architectural blueprint, component changes, and quality gates for Subtask 03 of task `231-anti-gravity-rerun-and-ui-fixes`: repairing filter row clipping in `TreePanel.tsx`, formalizing the tree view design system, and ensuring zero-overflow responsive adaptability.
> **/learn** Master the 190px compact filter row architecture, icon-first status buttons, icon-only scope toggles, 4-tier origin hierarchy badges, `.btn-tactile` micro-interactions, and dark-glass truncation banners.

---

## 🎯 Actionable CI/CD & Agent Checklist

- [ ] `/goal` Repair filter row clipping in `src/components/instances/PromptTreeView/TreePanel.tsx` by reducing Row 2 width from 451px to ~190px.
- [ ] `/learn` Replace verbose status labels with icon-first buttons (`Run`, `Queue`, `Pin`, Lucide `Image`).
- [ ] `/goal` Replace bulky inline `<select>` archive dropdown with sleek icon-only toggle buttons (`Archive`, `EyeOff`/`Eye`).
- [ ] `/learn` Enforce 4-tier prompt origin distinction (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`) and hierarchical branch glyphs (`└── ↳`).
- [ ] `/goal` Verify zero horizontal scrollbars and zero clipped controls at minimum 280px panel width.
- [ ] `/learn` Use strictly repository-relative Git paths across all files and tests.

---

## 1. Document Metadata & Task Tracking

| Field | Value |
| :--- | :--- |
| **Task ID** | `231-anti-gravity-rerun-and-ui-fixes` |
| **Subtask ID** | `subtask-231-03` |
| **Code** | `SUB-03` |
| **Slug** | `tree-filter-compaction-design-system` |
| **Document Path** | `02-spec/21-app/231-anti-gravity-rerun-and-ui-fixes/03-tree-view-design-system-spec.md` |
| **Status** | APPROVED SPECIFICATION |
| **Current App Version** | `4.271.0` |
| **Target Release Version** | `4.272.0` (Minor Bump) |
| **Parent Plan** | [.ai-memory/plans/pending/231-anti-gravity-rerun-and-ui-fixes.md](../../../.ai-memory/plans/pending/231-anti-gravity-rerun-and-ui-fixes.md) |
| **Subtask Plan** | [.ai-memory/plans/subtasks/231-anti-gravity-rerun-and-ui-fixes/03-subtask-tree-filter-compaction-and-design-system.md](../../../.ai-memory/plans/subtasks/231-anti-gravity-rerun-and-ui-fixes/03-subtask-tree-filter-compaction-and-design-system.md) |
| **Canonical Guideline** | [02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md](../../02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md) |

---

## 2. Problem Statement & Root Cause Analysis

### 2.1 The Filter Row Clipping Defect
In `src/components/instances/PromptTreeView/TreePanel.tsx`, the search and filter header contains three vertical rows:
- **Row 1**: Text search box + project scope dropdown.
- **Row 2**: Status filter segmented capsule (Running, Queued, Pinned, Images) + Scope filter capsule (Hide Auto toggle + native `<select>` dropdown for All vs Archived).
- **Row 3**: Filter reset & batch resend buttons (rendered conditionally).

#### The Root Cause:
1. **Excessive Horizontal Footprint in Row 2**:
   - The left status capsule contained four verbose items: `[ (•) Running ]`, `[ (•) Queued ]`, `[ (📌) Pinned ]`, and `[ 🖼️ Images ]`. Minimum rendered width: **~268px**.
   - The right scope capsule contained a text toggle button `[ 👁️ Hide Auto ]` (minimum width: **~95px**) and a native `<select>` dropdown `[ All (N) v ]` (minimum width: **~88px**).
   - Minimum combined width: `268px + 6px (gap) + 95px + 88px = 457px`.
2. **Deficit at Standard & Minimum Panel Widths**:
   - At the default panel width of **340px** (inner width ~324px), there was a **133px deficit**.
   - At the minimum resizable panel width of **280px** (inner width ~264px), there was a **193px deficit**.
   - As a result, the rightmost controls in Row 2 were severely clipped or pushed completely outside the visible viewport boundary, making it impossible for users to toggle automated prompts or select archived projects on standard displays.
3. **Inconsistent Emojis vs Iconography**:
   - The "Images" filter used a raw OS-dependent Unicode emoji `🖼️`, causing inconsistent line-heights, fuzzy emoji rendering on Linux, and misaligned baselines.
4. **Nested White Pill Visual Defect**:
   - The `<select>` element in the right capsule occasionally rendered browser-default white backgrounds and borders, creating an unsightly "white pill nested inside a dark-glass pill" appearance.

---

## 3. Architecture & Remediation Design

### 3.1 Mathematical Layout Budget Comparison

```
BEFORE (Width: 457px -> Severe Clipping at 340px & 280px):
+-----------------------------------------------------------------------------------------------+
| [ (•) Running ] [ (•) Queued ] [ (📌) Pinned ] [ 🖼️ Images ] | [ 👁️ Hide Auto ] [ All (12) v ] |
+-----------------------------------------------------------------------------------------------+
  <-------------------------- 268px ------------------------->   <--- 95px --->   <--- 88px --->

AFTER (Width: 190px -> Zero Clipping, +74px Safety Margin at 280px):
+------------------------------------------------------+   +-----------------------+
| [ (•) Run ] [ (•) Queue ] [ (📌) Pin ] [ (🖼) Image ] |   | [ 👁‍🗨 Auto ] [ 📦 Arc ]  |
+------------------------------------------------------+   +-----------------------+
  <---------------------- 135px ----------------------->     <-------- 55px -------->
```

| Component Element | Legacy Implementation | Compact Remediated Design | Width Savings |
| :--- | :--- | :--- | :--- |
| **Running Filter** | `<span>Running</span>` (px-2.5) | `<span>Run</span>` (px-2) | -14px |
| **Queued Filter** | `<span>Queued</span>` (px-2.5) | `<span>Queue</span>` (px-2) | -12px |
| **Pinned Filter** | `<span>Pinned</span>` (px-2) | `<span>Pin</span>` (px-1.5) | -10px |
| **Images Filter** | `<span>🖼️ Images</span>` (px-2) | Lucide `Image` icon (w-2.5 h-2.5) + `<span>Img</span>` | -18px |
| **Left Capsule Subtotal** | **~268px** | **~135px** | **-133px** |
| **Automated Prompts Toggle** | `EyeOff` + `<span>Hide Auto</span>` (~95px) | Sleek icon button: `EyeOff`/`Eye` (w-3 h-3) with rich tooltip (~26px) | -69px |
| **Archive Selection** | Native `<select>` `All (N)` / `Archived (N)` (~88px) | Direct toggle button: `Archive` icon (w-3 h-3) with counter badge (~28px) | -60px |
| **Right Capsule Subtotal** | **~183px** | **~55px** | **-128px** |
| **Combined Row 2 Width** | **457px** | **~190px** | **-267px (58% reduction)** |

### 3.2 Component Changes in `TreePanel.tsx`

#### 1. Status Capsule Implementation (Lines 466-524):
```tsx
{/* Status Capsule: Icon-first, short text, zero-clip footprint */}
<div className="inline-flex items-center rounded-full border border-slate-200/80 dark:border-slate-800/80 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xs p-0.5 shadow-2xs divide-x divide-slate-200/60 dark:divide-slate-800/60 shrink-0">
    {/* Running */}
    <button
        type="button"
        onClick={() => setActiveFilter(activeFilter === 'running' ? 'all' : 'running')}
        className={cn(
            "px-2 py-0.5 text-[10px] font-medium rounded-l-full transition-all duration-300 whitespace-nowrap cursor-pointer flex items-center gap-1 active:scale-[0.985]",
            activeFilter === 'running'
                ? "bg-emerald-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-slate-800/60"
        )}
        aria-label="Active running prompts"
        title="Filter active running prompts"
    >
        <span className={cn("w-1.5 h-1.5 rounded-full", activeFilter === 'running' ? "bg-white" : "bg-emerald-500 animate-pulse")} />
        <span>Run</span>
    </button>

    {/* Queued */}
    <button
        type="button"
        onClick={() => setActiveFilter(activeFilter === 'queued' ? 'all' : 'queued')}
        className={cn(
            "px-2 py-0.5 text-[10px] font-medium rounded-none transition-all duration-300 whitespace-nowrap cursor-pointer flex items-center gap-1 active:scale-[0.985]",
            activeFilter === 'queued'
                ? "bg-amber-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-slate-800/60"
        )}
        aria-label="Queued prompts"
        title="Filter queued prompts awaiting execution"
    >
        <span className={cn("w-1.5 h-1.5 rounded-full", activeFilter === 'queued' ? "bg-white" : "bg-amber-500")} />
        <span>Queue</span>
    </button>

    {/* Pinned */}
    <button
        type="button"
        onClick={() => setActiveFilter(activeFilter === 'pinned' ? 'all' : 'pinned')}
        className={cn(
            "px-1.5 py-0.5 text-[10px] font-medium rounded-none transition-all duration-300 whitespace-nowrap cursor-pointer flex items-center gap-0.5 active:scale-[0.985]",
            activeFilter === 'pinned'
                ? "bg-blue-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-slate-800/60"
        )}
        aria-label="Pinned projects"
        title="Filter pinned favorite projects"
    >
        <Pin className="w-2.5 h-2.5 shrink-0" />
        <span>Pin</span>
    </button>

    {/* Images (Lucide Icon replacing raw unicode emoji) */}
    <button
        type="button"
        onClick={() => imageFilterEmitter.set(!effectiveImageFilter)}
        className={cn(
            "px-1.5 py-0.5 text-[10px] font-medium rounded-r-full transition-all duration-300 whitespace-nowrap cursor-pointer flex items-center gap-1 active:scale-[0.985]",
            effectiveImageFilter
                ? "bg-sky-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-slate-800/60"
        )}
        aria-label="Filter prompts with attached images"
        title={effectiveImageFilter ? "Showing conversations with images (Click to clear)" : "Filter conversations with images"}
    >
        <Image className="w-2.5 h-2.5 shrink-0" />
        <span>Img</span>
    </button>
</div>
```

#### 2. Scope Capsule Implementation (Lines 526-558):
```tsx
{/* Scope Capsule: Sleek icon buttons with rich tooltips */}
<div className="inline-flex items-center rounded-full border border-slate-200/80 dark:border-slate-800/80 bg-white/80 dark:bg-slate-900/80 backdrop-blur-xs p-0.5 shadow-2xs divide-x divide-slate-200/60 dark:divide-slate-800/60 shrink-0">
    {/* Automated Prompts Toggle */}
    <button
        type="button"
        onClick={handleToggleAutomated}
        className={cn(
            "p-1 text-[10px] font-medium rounded-l-full transition-all duration-300 whitespace-nowrap cursor-pointer active:scale-[0.985] flex items-center justify-center",
            effectiveAutomatedHidden
                ? "bg-purple-500/15 text-purple-700 dark:text-purple-300 font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-slate-800/60"
        )}
        aria-label="Toggle automated prompts"
        title={effectiveAutomatedHidden ? "Automated subagent prompts hidden (Click to show)" : "Automated subagent prompts visible (Click to hide)"}
    >
        {effectiveAutomatedHidden ? (
            <EyeOff className="w-3 h-3 text-purple-500" />
        ) : (
            <Eye className="w-3 h-3 text-slate-400" />
        )}
    </button>

    {/* Archive Toggle Button */}
    <button
        type="button"
        onClick={() => setActiveFilter(activeFilter === 'archived' ? 'all' : 'archived')}
        className={cn(
            "p-1 text-[10px] font-medium rounded-r-full transition-all duration-300 whitespace-nowrap cursor-pointer active:scale-[0.985] flex items-center gap-0.5",
            activeFilter === 'archived'
                ? "bg-slate-700 dark:bg-slate-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-slate-800/60"
        )}
        aria-label="Toggle archived projects"
        title={activeFilter === 'archived' ? `Viewing archived projects (${archivedCount}) · Click to show all` : `Filter archived projects (${archivedCount})`}
    >
        <Archive className="w-3 h-3 shrink-0" />
        {archivedCount > 0 && (
            <span className="text-[9px] font-mono font-bold leading-none px-0.5">
                {archivedCount}
            </span>
        )}
    </button>
</div>
```

---

## 4. 4-Tier Origin Hierarchy in Tree Nodes

The tree view visualizes multi-turn conversations and subagent execution flows using the 4-tier model.

### 4.1 Tier Mapping Reference Table

| Origin Tier | Badge Class | Icon Component | Role Glyph | Purpose |
| :--- | :--- | :--- | :--- | :--- |
| `USER_PROMPT` | `bg-sky-500/10 text-sky-400 border-sky-500/20` | `User` | `USER` | Primary human conversation turns |
| `SUBAGENT_INSTRUCTION` | `bg-purple-500/10 text-purple-400 border-purple-500/20` | `Bot` | `SUBAGENT` | Autonomous delegated worker tasks |
| `SYSTEM_MESSAGE` | `bg-zinc-500/10 text-zinc-400 border-zinc-500/20` | `Terminal` | `SYSTEM` | Environment & system preambles |
| `TOOL_OUTPUT` | `bg-amber-500/10 text-amber-400 border-amber-500/20` | `Wrench` | `TOOL` | Execution stdout/stderr & tool calls |

### 4.2 Subagent Indentation & Branching Glyphs

Subagents must be nested beneath their parent conversation turn:

```tsx
{/* Subagent nesting glyph in ConversationNode.tsx */}
{promptCategory.tier === PromptOriginTier.SubagentInstruction && (
    <span
        className="text-purple-400 font-mono text-[10px] select-none shrink-0"
        title="Autonomous Subagent Worker"
    >
        └── ↳
    </span>
)}
```

---

## 5. Interaction Physics & Hover Contracts

All tree view nodes, capsules, and buttons must comply with three mechanical rules:

1. **Brightness Scaling**: `transition-all duration-300 hover:brightness-110 active:brightness-95`
2. **Tactile Compression**: `active:scale-[0.985] transition-transform duration-100 ease-out`
3. **Zero Layout Displacement**: Bounding dimensions (padding, margins, border widths) must remain perfectly stationary when transitioning between active, hovered, and selected states.

---

## 6. Acceptance Criteria & Test Verification

### AC-231-03-01: Filter Row Zero-Clipping Invariant (280px)
- **Given** The `TreePanel` mounted at minimum panel width `280px` (`style={{ width: 280 }}`).
- **When** Rendered with active projects, archived projects, running prompts, and image filters.
- **Then** Row 2 controls measure <= 195px combined width, `clientWidth >= scrollWidth`, and all buttons (`Run`, `Queue`, `Pin`, `Img`, `Auto`, `Archive`) are fully visible and clickable without horizontal scrollbars.

### AC-231-03-02: Scope Capsule Button Functionality
- **Given** The scope capsule in `TreePanel`.
- **When** Clicking the `Archive` toggle button.
- **Then** `activeFilter` toggles between `'archived'` and `'all'`, updating the tree view project listing immediately.
- **When** Clicking the `Auto` toggle button.
- **Then** `effectiveAutomatedHidden` toggles between `true` and `false`, showing or hiding automated subagent turns.

### AC-231-03-03: Lucide Icon Standardization
- **Given** The image filter button in the status capsule.
- **When** Inspected in the DOM.
- **Then** It renders Lucide `Image` icon SVG instead of raw unicode emoji `🖼️`, ensuring pixel-crisp alignment and uniform line heights.

### AC-231-03-04: Subagent Branch Glyph Rendering
- **Given** A conversation containing subagent worker turns.
- **When** Displayed in `ConversationNode.tsx`.
- **Then** The node renders purple bot iconography, `SUBAGENT` role badge, and the `└── ↳` branching glyph.

### AC-231-03-05: Dark-Glass Truncation Callout Expandability
- **Given** A transcript with omitted bytes or lines.
- **When** Rendered via `TruncatedContextCallout.tsx`.
- **Then** It displays the `⚡ [Omitted N KB/MB transcript context]` banner and invokes `onExpandFull` when clicked or activated via keyboard (`Enter`/`Space`).

---

## 7. Verification Commands

```bash
# 1. Frontend type checking
npx tsc --noEmit

# 2. Frontend linting
npm run lint

# 3. Production Vite asset compilation
npm run build
```

---

## 8. Cross-References

- Canonical Guideline: [02-tree-view-design-system.md](../../02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md)
- App UI Design System Index: [01-index.md](../../24-app-ui-design-system/01-index.md)
- Master Plan 231: [.ai-memory/plans/pending/231-anti-gravity-rerun-and-ui-fixes.md](../../../.ai-memory/plans/pending/231-anti-gravity-rerun-and-ui-fixes.md)
- Subtask 03 Plan: [.ai-memory/plans/subtasks/231-anti-gravity-rerun-and-ui-fixes/03-subtask-tree-filter-compaction-and-design-system.md](../../../.ai-memory/plans/subtasks/231-anti-gravity-rerun-and-ui-fixes/03-subtask-tree-filter-compaction-and-design-system.md)
