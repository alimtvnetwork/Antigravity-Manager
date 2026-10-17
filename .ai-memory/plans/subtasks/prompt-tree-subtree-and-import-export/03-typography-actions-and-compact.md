---
plan: prompt-tree-subtree-and-import-export
subtask: "03"
title: Typography (Poppins & Ubuntu), Header Copy Button, Sequence Badges, Conversation Actions, and Charming Markdown Studio
domain: frontend-and-theming
depends_on: []
citations:
  architecture_spec: ../../../../02-spec/21-app/prompt-tree-subtree-and-import-export/03-typography-actions-and-compact-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - index.html
  - tailwind.config.js
  - src/App.css
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/ConversationNode.tsx
  - src/components/instances/PromptTreeView/ProjectNode.tsx
  - src/components/instances/PromptTreeView/ConversationContextMenu.tsx
  - src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
  - src/components/instances/PromptTreeView/verificationTemplates.ts
  - src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx
  - src/components/instances/PromptTreeView/PromptTreeViewModal.tsx
status: pending
---

# Subtask 03: Typography (Poppins & Ubuntu), Header Copy Button, Sequence Badges, Conversation Actions, and Charming Markdown Studio

## 1. Context & Operational Objectives

This subtask guides **Worker 03** through implementing the frontend typography, title actions, sequence clarifications, conversation lifecycle operations, and the charming markdown studio as specified in `02-spec/21-app/prompt-tree-subtree-and-import-export/03-typography-actions-and-compact-spec.md`.

Key Objectives:
1. **Typography Standard**: Enforce **Ubuntu** font (`font-ubuntu`) for all headers and titles (`h1`–`h4`, modal headers, project titles, conversation titles). Enforce **Poppins** font (`font-poppins`) for body text, prompt text, previews, badges, and controls. Configure `fontFamily.poppins` and `fontFamily.ubuntu` in `tailwind.config.js`, load fonts in `index.html`, and update `src/App.css`.
2. **1-Click Header Copy Button**: Mount a dedicated 1-click Copy button directly beside the conversation title `<h3>` in `DetailHeader.tsx` with copy feedback toast.
3. **Sequence Code Tooltip Affordances**: Clarify that `C` = Conversation sequential index and `P` = Project sequential index (stored in `agm_conversation_sequences` and `agm_project_sequences` in `repo_prompts.db`). Add informative hover tooltips like `Conversation #226 (Sequential AGM Index)` and `Project #003 (Sequential Project Index)`.
4. **Conversation Lifecycle Actions**:
   - Triple-trigger renaming: inline double-click, F2 keyboard hotkey, and right-click context menu in `ConversationContextMenu.tsx` and `ConversationNode.tsx`.
   - Multi-Prompt Resend submenu: "Resend All Prompts", "Resend Last 5 Prompts", "Resend Last 10 Prompts", and "Resend Custom N Prompts..." (with numeric modal).
   - "Verify Last 10 Prompts & Enqueue": Synthesize a structured 5-Chapter prompt querying task completion, test validation, and release readiness, enqueuing it directly back to the active instance queue.
   - "Clone Conversation & Compact Memory": Handoff generator creating a compacted summary prompt to start a fresh session with complete context.
5. **Charming Markdown Preview & Studio**: Dual-pane studio layout (Input Prompt Instruction vs Output Results & Tool Pipeline) featuring syntax-highlighted codeblocks with copy buttons, collapsible reasoning `<thinking>` blocks in translucent lavender glass, and rich markdown rendering.

---

## 2. Target Files & Symbol Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `index.html` | `<head>` links | Add Google Fonts link loading Poppins (weights 300, 400, 500, 600, 700) alongside existing Ubuntu font. |
| `tailwind.config.js` | `theme.extend.fontFamily` | Add `poppins: ['Poppins', 'sans-serif']` and `ubuntu: ['Ubuntu', 'sans-serif']`. Set `sans` fallback. |
| `src/App.css` | Typography rules | Enforce `h1..h4` and `.font-heading` use `'Ubuntu'`; `body`, paragraphs, badges use `'Poppins'`. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeader` | Add 1-click Copy button directly beside `<h3>` title with copy feedback toast. Apply `font-ubuntu` to `<h3>`. |
| `src/components/instances/PromptTreeView/ConversationNode.tsx` | `ConversationNode` | 1. Add descriptive tooltip to sequence code badge (`Conversation #226 (Sequential AGM Index)`).<br/>2. Add `F2` keydown handler to trigger inline title rename.<br/>3. Wire right-click context menu options. |
| `src/components/instances/PromptTreeView/ProjectNode.tsx` | `ProjectNode` | Add descriptive tooltip to project sequence badge (`Project #003 (Sequential Project Index)`). |
| `src/components/instances/PromptTreeView/ConversationContextMenu.tsx` | `ConversationContextMenu` | Add menu items: Rename Conversation (`F2`), Resend Prompts Submenu (All, 5, 10, Custom N), "Verify Last 10 & Enqueue", "Clone Conversation & Compact Memory". |
| `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | Action functions | Implement `dispatchMultiPromptResend`, `handleVerifyLastPromptsAndEnqueue`, and `handleCloneAndCompactConversation`. |
| `src/components/instances/PromptTreeView/verificationTemplates.ts` | Template generators | Add `buildMultiPromptVerificationPrompt(records)` producing structured 5-Chapter audit text. |
| `src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx` | `PromptMarkdownPreview`, `CharmingCodeBlock`, `LavenderThinkingCard` | Syntax-highlighted codeblocks with copy buttons, translucent lavender glass `<thinking>` cards, Poppins typography. |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | `PromptTreeViewModal` | Dual-pane studio layout (Input vs Output) support and modal integrations. |

---

## 3. Concrete Step-by-Step Implementation Checklist

### Step 3.1: Typography Configuration
- [ ] In `index.html`:
  - Locate Google Fonts `<link>` at line 11.
  - Update href to load Poppins and Ubuntu:
    ```html
    <link href="https://fonts.googleapis.com/css2?family=Poppins:ital,wght@0,300;0,400;0,500;0,600;0,700;1,300;1,400;1,500;1,600;1,700&family=Ubuntu:ital,wght@0,300;0,400;0,500;0,700;1,300;1,400;1,500;1,700&family=Ubuntu+Mono:ital,wght@0,400;0,700;1,400;1,700&display=swap" rel="stylesheet">
    ```
- [ ] In `tailwind.config.js`:
  - Locate `theme.extend.fontFamily`.
  - Add `poppins` and verify `ubuntu`:
    ```javascript
    fontFamily: {
        sans: ['Poppins', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        poppins: ['Poppins', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        ubuntu: ['Ubuntu', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['Ubuntu Mono', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'Monaco', 'Consolas', 'monospace'],
        'ubuntu-mono': ['Ubuntu Mono', 'monospace'],
    },
    ```
- [ ] In `src/App.css`:
  - Enforce headers use Ubuntu:
    ```css
    h1, h2, h3, h4, .font-heading, .font-ubuntu {
      font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
    }
    ```
  - Enforce body text and UI components use Poppins:
    ```css
    body, p, span, div, input, textarea, button, .font-poppins {
      font-family: 'Poppins', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
    }
    ```

### Step 3.2: 1-Click Header Copy Button in `DetailHeader.tsx`
- [ ] In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
  - Add state `const [isHeaderCopied, setIsHeaderCopied] = useState(false);`.
  - Add handler `handleQuickCopyHeader`:
    ```typescript
    const handleQuickCopyHeader = async (e: React.MouseEvent) => {
        e.stopPropagation();
        const text = activePromptText || selectedConversation?.prompt_preview_200w || '';
        if (!text) return;
        const ok = await copyPromptWithRichImages(text, (msg) => {
            setIsHeaderCopied(true);
            setTimeout(() => setIsHeaderCopied(false), 2000);
        });
        if (ok) {
            setIsHeaderCopied(true);
            setTimeout(() => setIsHeaderCopied(false), 2000);
        }
    };
    ```
  - Mount copy button directly beside the `<h3>` title element inside Row 1 (around line 314):
    ```tsx
    <button
        type="button"
        onClick={handleQuickCopyHeader}
        data-ui-name="header-quick-copy-btn"
        className="p-1 rounded-md text-slate-400 hover:text-purple-600 dark:hover:text-purple-300 hover:bg-purple-50 dark:hover:bg-purple-950/40 transition-all duration-150 ease-out active:scale-95 cursor-pointer shrink-0"
        title="Copy full prompt text & attachments (1-click)"
    >
        {isHeaderCopied ? (
            <Check className="w-3.5 h-3.5 text-emerald-500 animate-in zoom-in-75 duration-150" />
        ) : (
            <Copy className="w-3.5 h-3.5" />
        )}
    </button>
    ```
  - Ensure the `<h3>` title has `font-ubuntu font-bold`.

### Step 3.3: Sequence Code Clarification & Tooltips
- [ ] In `src/components/instances/PromptTreeView/ConversationNode.tsx`:
  - Update sequence badge title attribute (line 128):
    ```tsx
    title={`Conversation #${conv.seq_code?.replace(/^C/i, '') || '001'} (Sequential AGM Index · ${conv.short_id || 'ID'})`}
    ```
- [ ] In `src/components/instances/PromptTreeView/ProjectNode.tsx`:
  - Update project sequence badge title attribute:
    ```tsx
    title={`Project #${project.seq_code?.replace(/^#?P/i, '') || '001'} (Sequential Project Index · ${project.repo_path})`}
    ```
- [ ] In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
  - Update Row 1 sequence code badge tooltip:
    ```tsx
    title={`Conversation #${selectedConversation.seq_code?.replace(/^C/i, '') || '001'} (Sequential AGM Index)`}
    ```

### Step 3.4: Conversation Actions Enhancement
- [ ] In `src/components/instances/PromptTreeView/ConversationNode.tsx`:
  - Add `tabIndex={0}` to conversation node container.
  - Add keyboard listener `onKeyDown`:
    ```tsx
    onKeyDown={(e) => {
        if (e.key === 'F2') {
            e.preventDefault();
            e.stopPropagation();
            setIsEditingTitle(true);
            setEditingTitle(conv.title || '');
        }
    }}
    ```
  - Ensure double-clicking title text enables inline editing.
- [ ] In `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`:
  - Expand props interface to accept:
    ```typescript
    onResendPrompts: (count: number | 'all') => void;
    onVerifyLastPrompts: (count: number) => void;
    onCloneAndCompact: () => void;
    ```
  - Render menu items:
    1. **Rename Conversation (`F2`)** with `<Pencil className="w-3.5 h-3.5 text-cyan-500" />`.
    2. **Resend Prompts Submenu** with `<RotateCw className="w-3.5 h-3.5 text-emerald-500" />`:
       - Sub-options: "Resend All Prompts", "Resend Last 5 Prompts", "Resend Last 10 Prompts", "Resend Custom N Prompts...".
    3. **Verify Last 10 & Enqueue** with `<ShieldCheck className="w-3.5 h-3.5 text-purple-500" />`.
    4. **Clone & Compact Memory** with `<CopyCheck className="w-3.5 h-3.5 text-blue-500" />`.
    5. **Inspect Transcript** with `<FileSearch className="w-3.5 h-3.5" />`.
    6. **Copy Conversation ID** with `<Copy className="w-3.5 h-3.5" />`.
- [ ] Add Custom N Numeric Modal / Popover if user selects "Resend Custom N Prompts...".

### Step 3.5: Action Handlers & Templates
- [ ] In `src/components/instances/PromptTreeView/verificationTemplates.ts`:
  - Export `buildMultiPromptVerificationPrompt(prompts: string[], convTitle: string)`:
    - Generates 5-Chapter prompt auditing task completion, git diff verification, test execution, and documentation.
- [ ] In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
  - Implement `handleVerifyLastPromptsAndEnqueue(convId: string, count: number)`.
  - Implement `handleMultiPromptResend(convId: string, count: number | 'all')`.
  - Implement `handleCloneAndCompactConversation(convId: string)`.

### Step 3.6: Charming Markdown Preview & Dual-Pane Studio
- [ ] In `src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx`:
  - Verify `CharmingCodeBlock`: language header with uppercase badge, one-click copy button, and copy state toast.
  - Verify `LavenderThinkingCard`: translucent lavender glass styling (`border-purple-500/30 bg-purple-950/20 backdrop-blur-md`), collapsible chevron, word count badge, `font-ubuntu` header, and `font-poppins` text.
- [ ] In `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`:
  - Provide a responsive layout toggle: `[Split View | Input | Output]`.
  - On viewports $\ge 1024\text{px}$, render input prompt instruction alongside execution results in a dual-pane studio.

---

## 4. Verification & Testing Matrix

| Test ID | Area | Verification Command / Action | Pass Criteria |
| :--- | :--- | :--- | :--- |
| **V-01** | Font Compilation | `npm run build` | Builds without CSS or Tailwind font errors. |
| **V-02** | Font Inspection | Inspect DOM elements in browser DevTools. | `font-ubuntu` applied to headings; `font-poppins` applied to body text. |
| **V-03** | Title Copy Button | Click copy button beside conversation title `<h3>`. | Clipboard receives full prompt + images; checkmark icon displays for 2s. |
| **V-04** | Sequence Tooltips | Hover over `C...` and `P...` badges. | Descriptive text explains Conversation index and Project index. |
| **V-05** | F2 Hotkey | Focus conversation row and press `F2`. | Inline title rename input appears. |
| **V-06** | Context Menu Resend | Right-click conversation -> Resend Last 5. | Dispatches last 5 prompts sequentially. |
| **V-07** | Verify & Enqueue | Right-click conversation -> Verify Last 10 & Enqueue. | Synthesizes 5-chapter prompt and enqueues to instance. |
| **V-08** | Codeblock Copy | Click "Copy" on a fenced code block in markdown preview. | Code text copied to clipboard, displays "Copied". |
| **V-09** | Thinking Card | Expand/collapse `<thinking>` block. | Smooth toggle of translucent lavender glass card with word count. |
| **V-10** | Pre-flight Gate | `cd src-tauri && cargo fmt -- --check && cargo clippy --all-targets --all-features` | Zero warnings, zero errors. |
