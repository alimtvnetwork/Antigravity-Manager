# Subtask 01: UI Line Gap Preservation via `<br>` Tags and Markdown Formatting

**Task ID:** `128-01-ui-linegap-br-and-markdown-formatting`  
**Target File:** `src/components/instances/PromptTreeViewModal.tsx`  
**Owner:** Worker 01 (Frontend Specialist)  
**Status:** READY  
**Prerequisites:** None  

---

## 1. Context & User Directive (Verbatim)

```text
Okay. So here, if you look into this, the UI does not look okay. So first of all, the line gap, you don't have the line gap. How you get the text, you don't have the line gap. Fix the line gap in terms of display and everywhere. Okay? That's the first. Second, the issue here, because the line gap, you need to fix it with a BR tag only. That's the first thing. Okay. So apply that, and then format it for the markdown.
```

---

## 2. Technical Objective

Enforce strict, deterministic vertical paragraph and line spacing in `src/components/instances/PromptTreeViewModal.tsx` across both **Rich Markdown Preview Mode** and **Raw Monospace View Mode** using explicit HTML break tags (`<br className="my-1.5 block select-none" />`), eliminating browser CSS margin collapsing.

---

## 3. Implementation Blueprint

### 3.1 Pre-formatting Function: `formatPromptForMarkdown`
- **Location:** `src/components/instances/PromptTreeViewModal.tsx` (~Lines 156–174).
- **Transformation Rules:**
  1. Normalize all line breaks: convert Windows `\r\n` and legacy Mac `\r` to standard `\n`:
     ```typescript
     let formatted = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
     ```
  2. Disentangle inline markdown headings conjoined to sentence text:
     ```typescript
     // e.g. "# High Priority Instruction Okay. So here..." -> "# High Priority Instruction\n\nOkay. So here..."
     formatted = formatted.replace(
         /^(#{1,4}\s+[A-Za-z0-9_\-\s]{2,40}?)([\.\:\!\?])\s+([A-Z])/gm,
         '$1$2\n\n$3'
     );
     formatted = formatted.replace(
         /^(#{1,4}\s+High Priority Instruction|#{1,4}\s+Instruction|#{1,4}\s+Overview|#{1,4}\s+Notice|#{1,4}\s+Task|#{1,4}\s+Plan)\s+([A-Z])/gm,
         '$1\n\n$2'
     );
     ```
  3. Return the sanitized string for downstream rendering.

### 3.2 Rich Markdown Preview: `RichMarkdownRenderer`
- **Location:** `src/components/instances/PromptTreeViewModal.tsx` (~Lines 345–600).
- **Transformation Rules:**
  1. **Empty Lines:** When iterating through lines and `!trimmed` is encountered, inject:
     ```tsx
     if (!trimmed) {
         elements.push(<br key={`br-${i}`} className="my-1.5 block select-none" />);
         continue;
     }
     ```
  2. **Standard Paragraphs:** Wrap paragraphs in a styled container that appends an explicit block-level select-none `<br />`:
     ```tsx
     elements.push(
         <div key={`p-wrap-${i}`} className="my-1.5 leading-relaxed">
             <p className="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-words">
                 {parseInlineMarkdown(line)}
             </p>
             <br className="my-1.5 block select-none" />
         </div>
     );
     ```
  3. **Trailing Ellipsis Lines:** Preserve line gap alongside the interactive expansion badge:
     ```tsx
     elements.push(
         <div key={`p-wrap-${i}`} className="my-1.5 leading-relaxed">
             <p className="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-words inline">
                 {parseInlineMarkdown(cleanLine)}
             </p>
             <span
                 onClick={(e) => {
                     e.stopPropagation();
                     onToggleExpand();
                 }}
                 title="Click to expand full prompt text"
                 className="cursor-pointer font-bold text-cyan-600 dark:text-cyan-400 hover:underline px-1.5 py-0.5 rounded bg-cyan-500/10 hover:bg-cyan-500/20 transition-colors ml-1.5 inline-block select-none"
             >
                 {showAllWords ? '... [Collapse]' : '... [Expand Full Text]'}
             </span>
             <br className="my-1.5 block select-none" />
         </div>
     );
     ```

### 3.3 Raw Monospace View Tab
- **Location:** `src/components/instances/PromptTreeViewModal.tsx` (~Lines 2376–2415).
- **Transformation Rules:**
  - In `viewMode === 'raw'`, format `activePromptText` through `formatPromptForMarkdown`, split into lines, and inject `<br className="my-1.5 block select-none" />`:
    ```tsx
    {activePromptText ? (
        formatPromptForMarkdown(activePromptText).split('\n').map((line, idx, arr) => (
            <span key={`raw-line-${idx}`} className="block">
                {line || <br className="my-1.5 block select-none" />}
                {idx < arr.length - 1 && <br className="my-1.5 block select-none" />}
            </span>
        ))
    ) : (
        'No prompt content recorded.'
    )}
    ```

---

## 4. Verification Checklist & Definition of Done (DoD)

- [ ] Multi-line prompts render with clear, comfortable vertical space between paragraphs in Preview Mode.
- [ ] No adjacent lines merge or collapse into single unbroken blocks.
- [ ] Raw View displays distinct line gaps separated by `<br className="my-1.5 block select-none" />`.
- [ ] Headings (`#`, `##`, `###`) do not glue directly onto paragraph text.
- [ ] Select-none styling (`select-none`) prevents `<br />` tags from injecting rogue blank tokens into user copy-paste selections.
- [ ] ZERO git commands used.
