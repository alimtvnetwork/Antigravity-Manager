---
plan: prompt-restore-labeling-and-tree-overhaul
subtask: "03"
title: Prompt Tree Toolbar Action Merging, Suffix Template Library, and Charming Markdown Inspector
domain: frontend-and-backend
depends_on: []
citations:
  architecture_spec: ../../../../02-spec/21-app/prompt-restore-labeling-and-tree-overhaul/03-toolbar-suffix-and-markdown-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/verificationTemplates.ts
  - src/components/instances/PromptTreeView/SuffixTemplateModal.tsx
  - src/components/instances/PromptTreeView/InspectorModal.tsx
  - src/components/instances/PromptTreeView/markdown.tsx
  - src/components/instances/PromptTreeView/ResultsView.tsx
  - src-tauri/src/modules/suffix_templates.rs
  - src-tauri/src/modules/repo_db/schema.rs
  - src-tauri/src/bin/agm/prompt_cmds.rs
  - src-tauri/src/commands/mod.rs
status: pending
---

# Subtask 03: Prompt Tree Toolbar Action Merging, Suffix Template Library, and Charming Markdown Inspector

## 1. Context & Operational Objectives

This subtask guides **Worker 03** through the implementation of:
1. **Toolbar Actions Merging**: Merging the three disconnected floating action capsules (`Copy All`, `Send N / Enqueue`, `Rerun ×N`) into a single contiguous segmented dark-glass pill capsule (`rounded-full`, shared border, `divide-x`, backdrop-blur) in `src/components/instances/PromptTreeView/DetailHeader.tsx`.
2. **Suffix Template Library**:
   - Replacing the jarring native white HTML `<select>` dropdown with a custom dark-glass popover dropdown matching AGM design system tokens.
   - Expanding the built-in presets from 4 to 8 domain-specific checklists in `verificationTemplates.ts` (`is-it-done`, `is-it-released`, `are-you-confident`, `edge-cases-and-tests`, `security-audit`, `pr-review-ready`, `test-and-coverage`, `refactor-dry`).
   - Adding SQLite storage for custom user templates in `repo_prompts.db` via `src-tauri/src/modules/suffix_templates.rs`.
   - Creating a template management modal `SuffixTemplateModal.tsx` for adding, editing, and deleting custom templates from the UI.
   - Enforcing complete CLI command parity via `agm prompt suffix list/get/add/remove/set-default` and `--suffix=<id>` flag support in `agm prompt send` and `agm prompt queue`.
3. **Charming Markdown Preview & Dual Input/Output Inspector**:
   - Upgrading `InspectorModal.tsx` into a dual-pane studio supporting both Side-by-Side (wide screens `>= 1024px`) and Stacked/Tabbed (narrow screens `< 1024px`) views.
   - Enhancing `Input Card` with 4-tier origin badges, word/byte counters, file chips, and click-to-zoom image Lightbox.
   - Enhancing `Output Card` with live status capsules, collapsible reasoning `<thinking>` cards in lavender glass, charming rich markdown with codeblock copy buttons, and tool execution accordions.
   - Integrating a one-click **AI Task Verification Prompt Generator** that synthesizes a structured 5-Chapter prompt for automated validation in the IDE.

---

## 2. Target Files & Symbol Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeader`, `DetailHeaderProps` | 1. Merge 3 action capsules into 1 contiguous segmented dark-glass capsule.<br/>2. Adjust border radiuses: left `rounded-l-full`, middle `rounded-none`, right `rounded-r-full`.<br/>3. Replace OS white `<select>` with custom dark-glass popover dropdown.<br/>4. Mount "+ Manage Templates" trigger button. |
| `src/components/instances/PromptTreeView/verificationTemplates.ts` | `VERIFICATION_OPTIONS`, `VerificationOptionMeta`, `generateVerificationPrompt` | 1. Expand presets to 8 options (add `security-audit`, `pr-review-ready`, `test-and-coverage`, `refactor-dry`).<br/>2. Export helper `generateVerificationPrompt(context)` producing 5-chapter prompt. |
| `src/components/instances/PromptTreeView/SuffixTemplateModal.tsx` | `SuffixTemplateModal` (New) | Create modal for adding, editing, deleting, and testing custom suffix templates stored in SQLite. |
| `src/components/instances/PromptTreeView/InspectorModal.tsx` | `InspectorModal`, `InspectorModalProps` | 1. Implement dual-pane Side-by-Side and Stacked responsive layout.<br/>2. Add view toggle (`[Split | Input | Output]`).<br/>3. Embed click-to-zoom Lightbox for images.<br/>4. Render Output card with thinking and tools.<br/>5. Embed AI Verification Prompt generator bar. |
| `src/components/instances/PromptTreeView/markdown.tsx` | `RichMarkdownRenderer`, `parseInlineMarkdown` | 1. Add lavender glass `<thinking>` block parser and renderer.<br/>2. Polish codeblock language tags and copy button states.<br/>3. Refine blockquotes and link styling. |
| `src/components/instances/PromptTreeView/ResultsView.tsx` | `ResultsView` | Export shared thinking block and tool call accordion renderers for reuse in `InspectorModal.tsx`. |
| `src-tauri/src/modules/repo_db/schema.rs` | `init_tables` | Execute `CREATE TABLE IF NOT EXISTS prompt_suffix_templates` during database initialization. |
| `src-tauri/src/modules/suffix_templates.rs` | `SuffixTemplate`, `list_suffix_templates`, `save_suffix_template`, `delete_suffix_template` (New) | Implement SQLite CRUD operations for custom suffix templates. |
| `src-tauri/src/commands/mod.rs` | `list_suffix_templates`, `save_suffix_template`, `delete_suffix_template` | Register Tauri IPC invoke commands and route through `AppError`. |
| `src-tauri/src/bin/agm/prompt_cmds.rs` | `dispatch_prompt_domain`, `cmd_prompts_suffix`, `cmd_prompts_send`, `cmd_prompts_queue` | 1. Add `agm prompt suffix list/get/add/remove/set-default` CLI commands.<br/>2. Support `--suffix=<id>` in send and queue commands. |

---

## 3. Concrete Step-by-Step Implementation Checklist

### Step 3.1: Merge Toolbar Action Groups in `DetailHeader.tsx`
- [ ] Locate the two separate capsule wrappers in `src/components/instances/PromptTreeView/DetailHeader.tsx` (lines 256 and 685).
- [ ] Remove the secondary capsule wrapper (`ml-1.5`) around the Rerun section.
- [ ] Wrap all 3 action groups into a single parent container:
  ```tsx
  <div className="relative inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
  ```
- [ ] Ensure Segment 1 (`Copy All` split button) has `rounded-l-full` on the main copy button and `rounded-none` on the chevron.
- [ ] Ensure Segment 2 (`Send N / Enqueue` split button) has `rounded-none` on both the main action button and the chevron.
- [ ] Ensure Segment 3 (`Rerun ×N` split button) has `rounded-none` on the main rerun button and `rounded-r-full` on the chevron.
- [ ] Verify that flyouts and popovers align properly without overlapping or getting clipped.

### Step 3.2: SQLite Table & Backend Module `suffix_templates.rs`
- [ ] In `src-tauri/src/modules/repo_db/schema.rs`, add table creation:
  ```rust
  conn.execute(
      "CREATE TABLE IF NOT EXISTS prompt_suffix_templates (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          description TEXT,
          category TEXT NOT NULL DEFAULT 'custom',
          template TEXT NOT NULL,
          is_default INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL
      )",
      [],
  ).map_err(|e| format!("Failed to create prompt_suffix_templates table: {}", e))?;
  ```
- [ ] Create `src-tauri/src/modules/suffix_templates.rs` defining:
  - `pub struct SuffixTemplate { pub id: String, pub name: String, pub description: Option<String>, pub category: String, pub template: String, pub is_default: bool, pub created_at: String, pub updated_at: String }`
  - `pub fn list_suffix_templates() -> Result<Vec<SuffixTemplate>, AppError>`
  - `pub fn get_suffix_template(id: &str) -> Result<Option<SuffixTemplate>, AppError>`
  - `pub fn save_suffix_template(t: &SuffixTemplate) -> Result<(), AppError>`
  - `pub fn delete_suffix_template(id: &str) -> Result<bool, AppError>`
  - `pub fn set_default_suffix_template(id: &str) -> Result<(), AppError>`
- [ ] Register module in `src-tauri/src/modules/mod.rs`.

### Step 3.3: Tauri IPC Commands Registration
- [ ] In `src-tauri/src/commands/mod.rs` (or a dedicated `cmd_suffix.rs`):
  - Add Tauri commands: `list_suffix_templates`, `save_suffix_template`, `delete_suffix_template`, `set_default_suffix_template`.
  - Route all fallible calls to `AppError` and log via `tracing`.
  - Add to `tauri::generate_handler![...]`.

### Step 3.4: Expand Built-in Presets in `verificationTemplates.ts`
- [ ] In `src/components/instances/PromptTreeView/verificationTemplates.ts`, expand `VerificationTemplateType` and `VERIFICATION_OPTIONS`:
  - `is-it-done`: Completion audit checklist & scorecard
  - `is-it-released`: 14-location SemVer sync, `@aukgit` attribution, README sync, CI pre-flight
  - `are-you-confident`: <=15-line functions, positive booleans, error routing, DRYness
  - `edge-cases-and-tests`: Concurrency, failure ladder, bounds checking, mock isolation
  - `security-audit`: Secrets scrub, credential protection, input validation, zero-trust
  - `pr-review-ready`: Single problem scope, clean commits, PR template compliance
  - `test-and-coverage`: Unit test execution, mock isolation, error branch coverage, no OS mutation
  - `refactor-dry`: Code DRYness, duplicated logic extraction, helper consolidation
- [ ] Export `generateVerificationPrompt(context: VerificationTemplateContext): string` combining chapters with prompt text and touched files.

### Step 3.5: Dark-Glass Suffix Popover & Modal in Frontend
- [ ] In `DetailHeader.tsx`, remove `<select>` and implement a custom listbox popover:
  - Trigger pill: Displays selected preset icon, label, and chevron.
  - Dropdown menu: `bg-[#171025] border-[#40285e] shadow-2xl rounded-xl p-1.5`.
  - Grouping: "Built-in Presets" and "Custom Templates" sections.
  - Action footer: "+ Manage Suffix Templates" button opening `SuffixTemplateModal`.
- [ ] Create `src/components/instances/PromptTreeView/SuffixTemplateModal.tsx`:
  - Lists custom templates fetched via Tauri IPC.
  - Provides inline editor (id, name, description, template text).
  - Validates required fields and saves to SQLite.

### Step 3.6: CLI Command Suite in `prompt_cmds.rs`
- [ ] In `src-tauri/src/bin/agm/prompt_cmds.rs`, add `"suffix"` subcommand handler:
  ```rust
  "suffix" => cmd_prompts_suffix(args),
  ```
- [ ] Implement `cmd_prompts_suffix(args: &[String])` supporting:
  - `list` / `ls`: Outputs formatted table or JSON array.
  - `get <id>`: Displays metadata and template content.
  - `add <id> --name "<name>" [--desc "<desc>"] (--file <path> | "<content>")`: Inserts or updates template.
  - `remove` / `rm <id>`: Deletes custom template.
  - `set-default <id>`: Marks template as default.
- [ ] In `cmd_prompts_send` and `cmd_prompts_queue`, parse `--suffix=<id>` or `--suffix <id>`:
  - Lookup template by ID (checking built-ins first, then SQLite table).
  - Append synthesized template envelope to prompt body before dispatching.

### Step 3.7: Charming Dual-Pane Inspector (`InspectorModal.tsx`)
- [ ] Update `InspectorModal.tsx` layout:
  - Top header: Sequence badge, conversation title, instance breadcrumb, layout switcher (`Split`, `Input`, `Output`), copy buttons, and close button.
  - Main body:
    - Grid layout: `grid-cols-1 lg:grid-cols-2 gap-6` when in split mode; full width when single pane selected.
    - Left Pane (Input Card):
      * 4-Tier origin badge with exact color tokens (`USER_PROMPT` Sky, `SUBAGENT` Purple, `SYSTEM` Zinc, `TOOL` Amber).
      * Word count & byte count pills.
      * Embedded image gallery with click-to-zoom Lightbox modal (keyboard ESC dismissal, zoom controls).
      * Referenced context files chip list.
      * Rendered markdown with truncation callouts.
    - Right Pane (Output Card):
      * Live running banner or completed status capsule.
      * Collapsible `<thinking>` reasoning card in lavender glass (`bg-purple-950/20 border-purple-500/30`).
      * AI Assistant Response in charming rich markdown (`RichMarkdownRenderer`) with copyable codeblocks.
      * Collapsible tool execution accordion showing tool name, arguments, exit code, and outputs.

### Step 3.8: AI Verification Prompt Generator Integration
- [ ] At the bottom of `InspectorModal.tsx`, mount the AI Verification Action Bar:
  - Primary button: `[Sparkles] Generate AI Verification Prompt`.
  - Secondary button: `[Copy] Copy Verification Prompt`.
  - Tertiary button: `[RotateCw] Dispatch Audit to IDE`.
- [ ] When triggered, synthesize a targeted 5-Chapter prompt embedding the current prompt instruction and response, ready for the user to verify the agent's work.

### Step 3.9: Pre-flight Quality Verification Gate
- [ ] Run `cargo fmt -- --check` in `src-tauri`.
- [ ] Run `cargo clippy --all-targets --all-features` in `src-tauri` (ensuring 0 errors and 0 warnings).
- [ ] Run `npm run build` in root (ensuring TypeScript compilation and Vite bundling succeed).

---

## 4. Edge Cases & Defensive Invariants

1. **Z-Index Popover Layering**:
   - The toolbar capsule is mounted at `z-30`. Popovers (`isCopyMenuOpen`, `isSendMenuOpen`, `isRerunMenuOpen`, `isSuffixMenuOpen`) must use `z-50` with click-outside and `onMouseLeave` listeners to prevent sticky overlapping popups.
2. **Empty or Missing Response Handling**:
   - When inspecting an in-flight prompt or a prompt without recorded assistant responses, the Output Pane must render the live animated pulsing card (`Live Execution in Progress` with elapsed time) or an empty state card (`No execution results recorded yet`) with a one-click button to focus the IDE window.
3. **Large Base64 Images in Inspector**:
   - Images exceeding 500 KB must be rendered with `loading="lazy"` and thumbnail constraints in the gallery to prevent UI thread lockup.
4. **SQLite Concurrency & WAL Busy Timeout**:
   - `repo_db::connect_db()` enforces 5000ms busy timeout and WAL mode. Suffix template reads/writes must avoid holding persistent locks.
5. **No Silent Errors Rule**:
   - All errors in `suffix_templates.rs` return `Result<T, AppError>`.
   - All async catches in `DetailHeader.tsx` and `InspectorModal.tsx` call `useErrorStore.getState().captureException(err, ...)` or `trackWarning(err, ...)`.

---

## 5. Acceptance Criteria & Verification Tests

- [ ] **Visual Merging Verified**: The action controls toolbar in `DetailHeader.tsx` renders as a single seamless capsule with zero margin gaps between Copy, Send, and Rerun.
- [ ] **Dark-Glass Suffix Dropdown**: Clicking Suffix opens a custom dark-glass popover with 8 built-in presets and custom templates; no native OS white dropdown appears.
- [ ] **Custom Suffix Templates**: Custom templates can be added, updated, and deleted through `SuffixTemplateModal.tsx`, and persist in `repo_prompts.db`.
- [ ] **CLI Parity**: Running `agm prompt suffix list`, `agm prompt suffix add ...`, and `agm prompt send ... --suffix=is-it-done` works flawlessly.
- [ ] **Dual-Pane Inspector**: Opening Full Inspector displays side-by-side Input and Output cards on wide screens, with interactive Lightbox for images, collapsible `<thinking>` reasoning in lavender glass, and collapsible tool executions.
- [ ] **AI Verification Generator**: Clicking "Generate AI Verification Prompt" synthesizes a comprehensive 5-Chapter prompt.
- [ ] **Pre-flight Clean**: `cargo fmt`, `cargo clippy`, and `npm run build` all pass with zero errors.
