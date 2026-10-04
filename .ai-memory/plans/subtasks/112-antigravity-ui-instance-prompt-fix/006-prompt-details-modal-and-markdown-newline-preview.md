---
plan: 112-antigravity-ui-instance-prompt-fix
subtask: "006"
title: Declutter prompt header with details modal and fix rich markdown newline rendering
domain: frontend
depends_on:
  - 005-prompt-filter-capsules-and-project-dropdown-search.md
citations:
  app_spec: .ai-memory/plans/112-antigravity-ui-instance-prompt-fix.md
  coding_guidelines: AGENTS.md
  strictly_avoid: 02-spec/02-coding-guidelines/01-strictly-avoid.md
target_files:
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# 006 — Declutter Prompt Header with Details Modal and Fix Rich Markdown Newline Rendering

## 1. Context & Rationale
In `src/components/instances/PromptTreeViewModal.tsx`:
1. **Right-Hand Panel Header Clutter**: The prompt view header is currently overwhelmed by metadata: raw `Conv ID: 6a84f3...`, `Modified: 2026-10-04...`, idle state badges (`QUEUED / IDLE`), and process churn indicators. This takes away visual focus from the conversation title and actions. As requested by the user, these technical details should be relocated into an inspectable "Details" modal/popover, keeping the primary header lean and clean.
2. **Broken Markdown Preview & Missing Newlines**: In `RichMarkdownRenderer`, blank lines are dropped due to:
   ```tsx
   if (!trimmed) {
       continue;
   }
   ```
   When a user writes markdown text containing empty lines between paragraphs or preceding markdown headings (`# Title`), the parser strips all whitespace separation, causing headers and paragraphs to jam together into a single block.
3. **Typography & Selection Colors**: Markdown selection and inline tokens use outdated harsh pink colors instead of the polished VS Code / Antigravity dark-slate color palette. There is also a need for an explicit "Detail / More" toggle to seamlessly view the full prompt without arbitrary limits.

## 2. Target Files and Symbols
- **`src/components/instances/PromptTreeViewModal.tsx`**:
  - `RichMarkdownRenderer`:
    - Fix blank line handling: Insert visual spacing nodes (`<div className="h-2.5" />`) instead of skipping empty lines.
    - Wrap paragraph contents with `whitespace-pre-wrap` and `break-words`.
    - Improve heading typography (`# `, `## `, `### `) with top margin separation when preceded by newlines.
    - Modernize inline code syntax highlighting (cyan/blue/emerald tones instead of pink).
    - Add selection color styling (`selection:bg-cyan-500/30 selection:text-white`).
  - Prompt Header:
    - Remove raw `Conv ID: ...` and `Modified: ...` text lines and `QUEUED / IDLE` badges from default view.
    - Retain only `ACTIVE RUNNING` badge when actively running, conversation title, and turn badge.
    - Add an inspectable "Details" button (`<Info className="w-3.5 h-3.5" /> Details`) opening a clean modal dialog with full conversation metadata and copy buttons.
  - Preview Truncation & "Detail / More":
    - Enhance the expandable "Show All / Detail" toggle button with word count indicators.

## 3. Detailed Technical Requirements

### 3.1. Decluttering the Right-Hand Panel Header
- **Before**:
  Header displays:
  - Title
  - `ACTIVE RUNNING` / `QUEUED / IDLE`
  - `Turn #X`
  - `Conv ID: 6a84f3... · Modified: 2026-10-04...`
- **After**:
  Header displays:
  - Title (truncated cleanly with tooltip)
  - `ACTIVE RUNNING` badge (only when `conv.is_running === true` or `conv.status === 'RUNNING'`)
  - `Turn #X` badge
  - A compact **"Details"** button in the action bar:
    ```tsx
    <button
        type="button"
        onClick={() => setIsDetailsModalOpen(true)}
        className="flex items-center gap-1 px-2.5 py-1.5 rounded-[5px] bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer"
        title="View conversation and instance details"
    >
        <Info className="w-3.5 h-3.5 text-blue-500" />
        <span>Details</span>
    </button>
    ```

### 3.2. Conversation Details Modal
When `isDetailsModalOpen` is true, render a modal popover:
- **Title**: Conversation Details
- **Fields**:
  - Conversation Title
  - Full Conversation ID (with one-click copy button)
  - Target Project Name & Repository Path
  - Target Instance Name & ID (and PID if active)
  - Status (`RUNNING` / `IDLE`)
  - Last Modified Timestamp
  - Total Turns / Step Count
  - Total Word Count
- Close on Escape or click outside.

### 3.3. Fixing `RichMarkdownRenderer` Blank Lines & Newlines
- When encountering an empty line (`!trimmed`), do NOT drop it if it is outside code blocks. Instead:
  ```tsx
  if (!trimmed) {
      // Preserve vertical newline breathing room by rendering a spacer
      elements.push(<div key={`spacer-${i}`} className="h-3 min-h-[0.75rem]" />);
      continue;
  }
  ```
- **Paragraph & Text Rendering**:
  Ensure paragraph lines apply `whitespace-pre-wrap break-words leading-relaxed`:
  ```tsx
  elements.push(
      <p key={`p-${i}`} className="text-xs text-slate-800 dark:text-slate-200 my-1 leading-relaxed whitespace-pre-wrap break-words">
          {parseInlineMarkdown(line)}
      </p>
  );
  ```
- **Heading Rendering**:
  Ensure `# `, `## `, `### ` headings have crisp spacing and typography:
  ```tsx
  if (trimmed.startsWith('# ')) {
      elements.push(
          <h1 key={`h1-${i}`} className="text-sm font-bold mt-4 mb-2 text-slate-900 dark:text-white pb-1 border-b border-slate-200 dark:border-[#15334d] first:mt-1">
              {parseInlineMarkdown(trimmed.slice(2))}
          </h1>
      );
      continue;
  }
  ```
- **Inline Code Styling**:
  Replace `text-pink-600 dark:text-pink-400` with `text-cyan-700 dark:text-cyan-300 bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d]`.
- **Text Selection Styling**:
  Add `selection:bg-cyan-500/25 selection:text-cyan-900 dark:selection:text-cyan-100` to the preview container.

### 3.4. Expandable "Detail / More" Full Inspection
- In the preview area footer, ensure an expandable button toggles between preview truncation (500 words) and full view:
  ```tsx
  {isTruncated && (
      <button
          type="button"
          onClick={() => setShowAllWords(!showAllWords)}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-blue-50 dark:bg-[#0c2438] text-blue-600 dark:text-cyan-300 border border-blue-200 dark:border-[#15334d] hover:bg-blue-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
      >
          {showAllWords ? (
              <>
                  <ChevronUp className="w-3.5 h-3.5" />
                  <span>Show Less</span>
              </>
          ) : (
              <>
                  <ChevronDown className="w-3.5 h-3.5" />
                  <span>Detail / More ({totalWords} words)</span>
              </>
          )}
      </button>
  )}
  ```

## 4. Constraints & Non-Negotiables
- Respect empty lines in Markdown; never allow empty lines to be discarded.
- No harsh magenta/pink inline code tags; stick to Antigravity cyan/blue theme.
- The Details modal must be fully keyboard accessible (Escape to close).
- Code changes must be self-contained in `src/components/instances/PromptTreeViewModal.tsx`.

## 5. Verification & Acceptance Tests
1. **Header Clutter Test**: Verify the main prompt header no longer contains raw UUIDs, `QUEUED / IDLE` badges, or timestamp clutter.
2. **Details Modal Test**: Click "Details" button. Verify the modal opens and displays full Conversation ID with functional copy button.
3. **Markdown Newline Test**: Render a prompt containing:
   ```text
   Line 1

   # Section Header

   Line 2
   ```
   Verify that `Line 1` and `# Section Header` have a visible vertical spacer between them and do not collide.
4. **Detail / More Test**: Click "Detail / More" on a long prompt. Verify the entire text expands smoothly without truncation.
5. **Build Gate**: Run `npm run build` to verify clean compilation.
