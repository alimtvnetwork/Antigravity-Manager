# Subtask 001: Frontend Line Gap with `<br />` Tags, Markdown Formatting & Dot-Dot Expand

## Owner: Worker 01 (Frontend Specialist)
## Target File: `src/components/instances/PromptTreeViewModal.tsx`

### Objective
1. **Vertical Line Gaps via `<br />` Tags**:
   - In `RichMarkdownRenderer`:
     - When rendering standard paragraphs (lines 431-436), ensure paragraphs have generous vertical spacing and render explicit `<br />` tags when empty lines occur instead of skipping or collapsing.
     - In `parseInlineMarkdown`: Ensure `\n` characters are converted to `<br className="my-1" />` elements so that multi-line markdown preserves explicit breaks.
   - In Raw View (lines 2030-2050):
     - Split `activePromptText` by `\n` and render each line with a dedicated `<br />` tag between lines:
       ```tsx
       {activePromptText.split('\n').map((line, idx, arr) => (
           <span key={`raw-line-${idx}`}>
               {line}
               {idx < arr.length - 1 && <br className="my-1.5" />}
           </span>
       ))}
       ```
2. **"..." (Dot Dot) Click to Expand**:
   - At lines 1970–2010 where prompt text is displayed truncated:
     - Wrap any trailing `...` in an interactive clickable badge:
       ```tsx
       <span
           onClick={(e) => {
               e.stopPropagation();
               setShowAllWords(true);
           }}
           className="cursor-pointer font-bold text-cyan-600 dark:text-cyan-400 hover:underline px-1 py-0.5 rounded bg-cyan-500/10 hover:bg-cyan-500/20 transition-colors ml-1"
           title="Click to expand full prompt text"
       >
           ... [Expand Full Text]
       </span>
       ```
     - Also make clicking the preview box or clicking a prominent `[Expand / Collapse]` pill toggle `showAllWords`.
