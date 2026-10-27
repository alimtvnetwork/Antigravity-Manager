# Canonical Tree View Design System & Component Guidelines

> **/goal** Establish the canonical, authoritative design specification and component guidelines for Tree Views across Antigravity Manager so any engineer or AI coding assistant can build, style, and maintain tree views matching the master UI/UX standard.
> **/learn** Master compact panel geometry (280px–480px), 4-tier prompt origin classification, segmented pill capsules, zero-clipping filter rows, hover physics without layout displacement, and interactive dark-glass truncation banners.

---

## 🎯 Actionable CI/CD & Agent Checklist

- [ ] `/goal` Enforce panel width bounds: minimum 280px, maximum 480px, default 340px, with responsive zero-clipping overflow behavior.
- [ ] `/learn` Adhere strictly to the 4-tier prompt origin classification badges (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`) and tree branching glyphs (`├──`, `└──`, `└── ↳`).
- [ ] `/goal` Wrap adjacent toolbar and filter controls in segmented pill capsules (`rounded-full`, shared border, subtle divider lines, dark-glass styling) with transparent borderless inner `<select>` elements.
- [ ] `/learn` Implement hover and active physics with 300ms CSS3 brightness transitions (`hover:brightness-110`), `.btn-tactile` micro-interactions (`active:scale-[0.985]`), and zero layout displacement.
- [ ] `/goal` Transform raw context omission markers (`<truncated N bytes>` / `<truncated N lines>`) into interactive dark-glass omission banners (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`).
- [ ] `/learn` Use strictly repository-relative Git paths across all documentation and comments.

---

## Document Metadata

| Attribute | Value |
| :--- | :--- |
| **Document ID** | `02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md` |
| **Canonical Category** | Coding Guidelines / Application UI Design System |
| **Version** | `4.272.0` |
| **Updated** | 2026-10-27 |
| **AI Confidence** | Authoritative (Verified & Grounded) |
| **Status** | ACTIVE CANONICAL SPECIFICATION |
| **Cross-References** | [01-index.md](../../24-app-ui-design-system/01-index.md), [03-tree-view-design-system-spec.md](../../21-app/231-anti-gravity-rerun-and-ui-fixes/03-tree-view-design-system-spec.md) |

---

## 1. Architectural Purpose & Design Philosophy

The **Prompt Tree View** is the primary navigation and hierarchical inspection surface in Antigravity Manager (`agm`). It visualizes repositories, multi-turn conversations, autonomous subagent cascades, diagnostic tool outputs, and in-flight prompt executions across multiple isolated IDE instances.

Because Antigravity Manager operates simultaneously as a desktop application (Tauri + React) and a developer control center, its Tree View components must adhere to four core design principles:

1. **Extreme Information Density Without Visual Clutter**: Developers must be able to view projects, conversations, turns, and real-time execution states at a glance without feeling overwhelmed.
2. **Deterministic Responsive Adaptability (Zero-Clipping Invariant)**: Controls must adapt smoothly between the minimum width (280px) and maximum width (480px). No control may ever be clipped, pushed off-screen, or hidden behind unwanted horizontal scrollbars.
3. **4-Tier Semantic Origin Transparency**: Every node in the tree must clearly communicate its origin: human user input, autonomous subagent directive, system initialization preamble, or diagnostic tool output.
4. **Physical Tactile Feedback**: Micro-interactions must feel physical, responsive, and grounded, using CSS3 brightness scaling and transform scaling without causing neighboring elements to shift or jitter.

---

## 2. Visual Anatomy & Panel Geometry

```
+-----------------------------------------------------------------------------+
| TreePanel Container (w-[280px..480px], default 340px, flex flex-col)        |
+-----------------------------------------------------------------------------+
| 1. Header Toolbar: Instance Identity + Minimize/Collapse + Expand-All        |
|    - Height: 40px (h-10)                                                    |
|    - Dark-glass backdrop: bg-slate-900/80 / bg-white/80, backdrop-blur-xs  |
+-----------------------------------------------------------------------------+
| 2. Search & Filter Bar (Dense, 2-Row Layout)                                |
|    - Row 1: Search Input (w-full) + Clear Button                             |
|    - Row 2: Status Capsule [Run | Queue | Pin | Img] + Scope Capsule [Auto|Arc] |
|    - Combined Row 2 Footprint: <= 195px (fits in 280px min panel effortlessly) |
+-----------------------------------------------------------------------------+
| 3. Tree Content Area (flex-1 overflow-y-auto overflow-x-hidden)             |
|    - Project Node: Accordion toggle, Repo badge, Conversation count         |
|      ├── Conversation Node: Origin icon, Seq code (#C001), Title, Status    |
|      │   ├── Prompt Turn Node: Turn badge (T1), Timestamp, Status orb       |
|      │   └── Subagent Node: Hierarchical branch (└── ↳), Bot icon, Role     |
+-----------------------------------------------------------------------------+
| 4. Tree Footer / Quick Action Bar                                           |
|    - Active conversation indicator, filter reset pill, batch actions        |
+-----------------------------------------------------------------------------+
```

### 2.1 Panel Geometry Constraints

| Metric | Dimension | Behavior / CSS Implementation |
| :--- | :--- | :--- |
| **Minimum Width** | `280px` (`min-w-[280px]`) | Absolute hard floor. All filter capsules, search inputs, and badges must remain 100% visible and unclipped. |
| **Default Width** | `340px` (`w-[340px]`) | Balanced default for standard desktop resolutions (1080p, 1440p, 4K). |
| **Maximum Width** | `480px` (`max-w-[480px]`) | Upper limit when resizing split-pane layout. |
| **Splitter Handle** | `4px` (`w-1 hover:w-1.5`) | Interactive cursor `col-resize` with subtle hover accent (`hover:bg-sky-500/50`). |
| **Padding** | `px-2 py-1.5` | Dense, compact inner gutters maximizing readable horizontal label space. |
| **Overflow Rule** | `overflow-y-auto overflow-x-hidden` | Strict zero horizontal scrollbar rule. Content must truncate (`truncate`) with title tooltips. |

### 2.2 Segmented Pill Capsules

Toolbar actions, status filters, and window controls must **always** be grouped into contiguous segmented pill capsules rather than rendered as loose, detached circular or square buttons.

#### Anatomical Anatomy of a Segmented Pill Capsule:

```
+--------------------------------------------------------------------+
|  [ (•) Run ]  |  [ (•) Queue ]  |  [ (📌) Pin ]  |  [ (🖼️) Image ]   |
+--------------------------------------------------------------------+
   ^               ^                  ^               ^
   Segment 1       Divider            Segment 2       Segment 3
```

#### Capsule CSS Construction Rules:

1. **Outer Capsule Wrapper**:
   - `inline-flex items-center rounded-full p-0.5`
   - Border: `border border-slate-200/80 dark:border-slate-800/80`
   - Dark-Glass Backdrop: `bg-white/80 dark:bg-slate-900/80 backdrop-blur-xs`
   - Shadows: `shadow-2xs`
   - Internal Dividers: `divide-x divide-slate-200/60 dark:divide-slate-800/60`
2. **Inner Segment Button**:
   - First segment: `rounded-l-full`
   - Middle segments: `rounded-none`
   - Last segment: `rounded-r-full`
   - Padding: `px-2 py-0.5` (compact) or `px-1.5 py-0.5` (icon-only)
   - Font: `text-[10px] font-medium whitespace-nowrap`
   - Transition: `transition-all duration-300`
3. **Active vs Inactive Segment States**:
   - Active: Solid contrast tint (e.g. `bg-emerald-600 text-white shadow-2xs font-semibold`).
   - Inactive: Subdued text with subtle background tint on hover (`text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-100/60 dark:hover:bg-slate-800/60`).
4. **Transparent Borderless Dropdown Rule (No Nested White Pill)**:
   Any inline `<select>` element mounted inside a segmented pill capsule MUST use:
   ```html
   <select className="appearance-none bg-transparent border-0 focus:ring-0 focus:outline-none text-[10px] font-medium text-slate-700 dark:text-slate-300 cursor-pointer pr-3 py-0">
   ```
   **FORBIDDEN**: Applying `border`, `bg-white`, `rounded-full`, or `shadow` to an inner `<select>`. This causes a jarring "white pill nested inside another pill" visual defect.

---

## 3. 4-Tier Prompt Origin Classification & Hierarchy

Every prompt turn and conversation item in Antigravity Manager belongs to one of four distinct semantic tiers. The UI must clearly differentiate these tiers using canonical color ramps, Lucide icons, and role badges.

```
Conversation Root
├── Turn #1 (USER_PROMPT) [Sky Badge, User Icon]
│   ├── Subagent #1 (SUBAGENT_INSTRUCTION) [Purple Badge, Bot Icon] ── ↳
│   │   └── Tool Step #1 (TOOL_OUTPUT) [Amber Badge, Wrench Icon]
│   └── Tool Step #2 (TOOL_OUTPUT) [Amber Badge, Wrench Icon]
└── Turn #2 (SYSTEM_MESSAGE) [Zinc Badge, Terminal Icon]
```

### 3.1 The 4 Canonical Origin Tiers

| Origin Tier | Semantic Meaning | Badge Tokens (Light / Dark) | Icon | Role Badge | Example Payloads |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `USER_PROMPT` | Direct human prompt submitted via UI chat, CLI, or task request | `bg-sky-500/10 text-sky-700 dark:text-sky-400 border-sky-500/20` | `User` (Lucide) | `USER` | "Refactor the database queries", "Run e2e tests" |
| `SUBAGENT_INSTRUCTION` | Autonomous worker prompt or delegate agent directive | `bg-purple-500/10 text-purple-700 dark:text-purple-400 border-purple-500/20` | `Bot` (Lucide) | `SUBAGENT` | `invoke_subagent`, "You are Spec Author 03..." |
| `SYSTEM_MESSAGE` | System initialization directive or runtime environment preamble | `bg-zinc-500/10 text-zinc-700 dark:text-zinc-400 border-zinc-500/20` | `Terminal` (Lucide) | `SYSTEM` | `<SYSTEM_MESSAGE>`, agent identity initialization |
| `TOOL_OUTPUT` | Diagnostic tool execution results, CLI stdout/stderr, or linter logs | `bg-amber-500/10 text-amber-700 dark:text-amber-400 border-amber-500/20` | `Wrench` (Lucide) | `TOOL` | `run_command` output, JSON tool result, stack traces |

### 3.2 Hierarchical Branching Glyphs

To convey parent-child relationships clearly without heavy indentation margins, use monospace unicode branching glyphs:

- Intermediary sibling: `├── ` (font-mono, text-slate-400 dark:text-purple-400/80)
- Terminal sibling: `└── ` (font-mono, text-slate-400 dark:text-purple-400/80)
- Subagent worker nesting: `└── ↳ ` (font-mono, text-purple-500 dark:text-purple-400)

Subagents **must** be nested hierarchically beneath their parent conversation turn rather than displayed as flat, disconnected items at the root level of the conversation tree.

---

## 4. Compact Filter Rows & Zero-Clipping Rules

### 4.1 The Filter Overflow Problem

In previous iterations, the tree view filter row (Row 2) contained verbose labels (`Running`, `Queued`, `Pinned`, `Images`, `Hide Auto`, and a bulky `<select>` dropdown with `All (N)` and `Archived (N)`). This layout demanded a minimum of **451px** of horizontal space. When displayed in the standard **340px** panel (or when resized to the **280px** minimum), this created a 123px–171px deficit, forcibly truncating or clipping the right-hand controls (`Hide Auto` and the archive selector).

### 4.2 Compact Segmented Architecture

To achieve zero clipping down to 280px, the filter row is partitioned into two compact capsules:

```
Row 2 Container: flex items-center justify-between gap-1.5 pt-0.5 min-w-0
+---------------------------------------+   +-----------------------+
| Status Capsule (~135px)               |   | Scope Capsule (~55px) |
| [• Run] [• Queue] [📌 Pin] [🖼 Image] |   | [ 👁‍🗨 Auto ] [ 📦 Arc ]  |
+---------------------------------------+   +-----------------------+
```

#### Left Status Capsule (Icon-First with Concise Text):
- **Run**: Emerald dot (pulsing when active) + "Run" (replaces verbose "Running") -> `px-2 py-0.5`
- **Queue**: Amber dot + "Queue" (replaces verbose "Queued") -> `px-2 py-0.5`
- **Pin**: Lucide `Pin` (w-2.5 h-2.5) + "Pin" -> `px-1.5 py-0.5`
- **Image**: Lucide `Image` (w-2.5 h-2.5) replacing raw unicode emoji `🖼️` -> `px-1.5 py-0.5`
- *Total width: ~135px*.

#### Right Scope Capsule (Icon-Only with Tooltips):
- **Auto Toggle**: Lucide `EyeOff` (when hidden) / `Eye` (when visible) button with tooltip "Toggle automated prompts" -> `p-1`
- **Archive Toggle**: Direct icon button with Lucide `Archive` (w-3 h-3) and counter badge instead of a native select dropdown -> `p-1`
- *Total width: ~55px*.

#### Combined Width Verification:
- Total width: `135px + 6px (gap) + 55px = 196px`.
- Available space at 280px min panel: `280px - 16px (padding) = 264px`.
- **Safety Margin**: `+68px surplus space`. Zero clipping, zero horizontal scrollbars.

---

## 5. Hover, Focus & Interaction Physics

All interactive elements (buttons, tree rows, pill segments, accordion headers) must adhere to standardized micro-interaction physics:

### 5.1 CSS3 Brightness Scaling
Instead of abrupt background color swaps, buttons and capsules use smooth brightness scaling:
```css
transition-all duration-300 hover:brightness-110 active:brightness-95
```

### 5.2 Tactile Compression (`.btn-tactile`)
Interactive controls must provide immediate physical feedback upon click:
```css
active:scale-[0.985] transition-transform duration-100 ease-out
```

### 5.3 Zero Layout Displacement Invariant
**Strict Invariant**: An element's bounding box (width, height, margin, padding, border-width) must **never** change on hover, focus, or selection state.
- **FORBIDDEN**: Adding a 1px border on hover when inactive had 0px border. (Causes neighboring elements to shift by 1px).
- **REQUIRED**: Maintain a permanent border with `border-transparent` on inactive states, transitioning only `border-color` on hover or selection:
  ```tsx
  className={cn(
      "border border-transparent transition-colors duration-150",
      isSelected ? "border-sky-500/40 bg-sky-500/10" : "hover:border-slate-300/40 dark:hover:border-slate-700/40"
  )}
  ```

### 5.4 Focus Ring Contracts
Keyboard navigation (Tab, Arrow keys, Enter, Space) must render high-visibility, non-displacing focus rings:
```css
focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/50 focus-visible:ring-offset-1 dark:focus-visible:ring-offset-slate-900
```

---

## 6. Interactive Dark-Glass Truncation Callouts

When displaying large conversation transcripts, raw truncation markers like `<truncated 19247 of 29091 bytes>` or `> ✂ [Omitted 500 lines]` must **never** be rendered as unstyled plain text or raw markdown blockquotes.

### 6.1 The TruncatedContextCallout Component Specification

Every truncation point must be converted into an interactive dark-glass omission banner:

```
+-------------------------------------------------------------------------------+
| ⚡ Omitted 18.8 KB transcript context · Click to expand            [Expand Full v] |
+-------------------------------------------------------------------------------+
```

#### Visual Styling Specs:
- **Surface**: `bg-amber-500/10 dark:bg-amber-950/40 backdrop-blur-md`
- **Border**: `border border-amber-500/30 dark:border-amber-600/40 rounded-xl`
- **Typography**: `font-mono text-xs text-amber-900 dark:text-amber-100 font-medium`
- **Icon**: High-voltage lightning bolt `⚡` (`text-amber-500`)
- **Action Button**: Compact high-contrast pill `[Expand Full v]` (`bg-amber-500 text-white dark:text-slate-900 font-bold hover:bg-amber-600 text-[10px] rounded-md px-2.5 py-1`)
- **Keyboard Handling**: Accessible with `role="button"`, `tabIndex={0}`, responding to `Enter` and `Space`.

#### Size Formatting Helper:
```typescript
export function formatOmissionSize(bytes?: number, lines?: number): string {
    if (bytes) {
        return bytes >= 1024 * 1024
            ? `${(bytes / (1024 * 1024)).toFixed(1)} MB`
            : `${(bytes / 1024).toFixed(1)} KB`;
    }
    if (lines) {
        return `${lines} lines`;
    }
    return 'transcript context';
}
```

---

## 7. TypeScript Implementation Blueprint

Frontend developers and AI coding assistants implementing tree view components should adhere to the following canonical data types:

```typescript
// 02-spec/02-coding-guidelines/24-app-ui-design-system/02-tree-view-design-system.md - Types
export enum PromptOriginTier {
    UserPrompt = 'USER_PROMPT',
    SubagentInstruction = 'SUBAGENT_INSTRUCTION',
    SystemMessage = 'SYSTEM_MESSAGE',
    ToolOutput = 'TOOL_OUTPUT',
}

export interface TierClassificationResult {
    tier: PromptOriginTier;
    category: string;
    label: string;
    roleBadge: 'USER' | 'SUBAGENT' | 'SYSTEM' | 'TOOL';
    badgeStyle: string;
    iconName: 'user' | 'bot' | 'terminal' | 'wrench';
    isSubagent: boolean;
    isNonPrompt: boolean;
    confidence: number;
    description: string;
}

export interface AgmPromptTurnNode {
    turn_number: number;
    prompt_text?: string;
    preview_200w?: string;
    status: 'RUNNING' | 'QUEUED' | 'COMPLETED' | 'ERROR';
    created_at?: number;
    timestamp?: string;
    media_paths?: string[];
    is_running?: boolean;
    is_queued?: boolean;
}

export interface AgmConversationNode {
    conversation_id: string;
    short_id?: string;
    seq_code?: string;
    title?: string;
    step_count: number;
    is_running?: boolean;
    is_queued?: boolean;
    turns: AgmPromptTurnNode[];
    origin_tier?: PromptOriginTier;
}

export interface AgmProjectTreeNode {
    repo_name: string;
    repo_path: string;
    instance_id: string;
    is_pinned?: boolean;
    is_archived?: boolean;
    conversations: AgmConversationNode[];
}
```

---

## 8. Verification & Acceptance Criteria

### AC-CG-TREE-001: Minimum Width No-Clip Invariant (280px)
- **Given** The tree view panel rendered at its absolute minimum width of `280px`.
- **When** All status filters (Run, Queue, Pin, Image) and scope toggles (Auto, Archive) are mounted.
- **Then** The total combined width of the filter row does not exceed `200px`, no element is clipped, and the parent container has zero horizontal scrollbars (`scrollWidth <= clientWidth`).

### AC-CG-TREE-002: 4-Tier Origin Discrimination & Subagent Branch Glyphs
- **Given** A multi-turn conversation containing user prompts, subagents, and tool outputs.
- **When** The tree renders conversation nodes and prompt turns.
- **Then** Each node renders its exact origin badge and icon (`USER_PROMPT` = Sky/User, `SUBAGENT_INSTRUCTION` = Purple/Bot, `SYSTEM_MESSAGE` = Zinc/Terminal, `TOOL_OUTPUT` = Amber/Wrench), and subagents are nested with `└── ↳`.

### AC-CG-TREE-003: Segmented Pill Capsule Dropdown Styling
- **Given** Any `<select>` dropdown placed within a segmented pill capsule.
- **When** Inspected in DOM developer tools across light and dark themes.
- **Then** The `<select>` element carries `bg-transparent border-0 focus:ring-0 focus:outline-none`, with zero nested white-pill borders.

### AC-CG-TREE-004: Interactive Truncation Callout Rendering
- **Given** A conversation transcript containing omitted context tokens.
- **When** The transcript preview mounts in the UI.
- **Then** It renders an interactive `TruncatedContextCallout` banner with amber glass styling, displaying exact formatted bytes (`N KB` or `N MB`), and expands when clicked or activated with Enter/Space.

### AC-CG-TREE-005: Micro-Interaction Physics & Zero Layout Displacement
- **Given** Any button, capsule segment, or tree row.
- **When** The user hovers or clicks the element.
- **Then** The element transitions with 300ms brightness (`hover:brightness-110`) and 100ms scale (`active:scale-[0.985]`) with zero displacement in neighboring margins, padding, or borders.

---

## 9. Verification Commands

To verify styling, TypeScript compilation, and design system contracts across all tree view components:

```bash
# 1. TypeScript syntax & typing gate
npx tsc --noEmit

# 2. ESLint UI component rules
npm run lint

# 3. Production asset compilation check
npm run build
```

_Guideline last updated: 2026-10-27 for Antigravity Manager v4.272.0_
