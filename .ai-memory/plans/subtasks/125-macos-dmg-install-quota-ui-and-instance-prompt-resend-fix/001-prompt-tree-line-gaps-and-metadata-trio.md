# Subtask 001: Prompt Tree View Line Gaps with `<br />` Tags, Dot-Dot Expand & Header Metadata Trio

## Owner: Worker 01 (Frontend Specialist)
## Target File: `src/components/instances/PromptTreeViewModal.tsx`

### Requirements
1. **Vertical Line Gaps via `<br />` Tags**:
   - In `RichMarkdownRenderer`: Ensure empty lines emit `<br className="my-2" />` elements and paragraph spacing is expanded (`my-2 leading-relaxed`).
   - In `parseInlineMarkdown`: Ensure `\n` characters are converted to `<br className="my-1" />` elements.
   - In Raw View: Split `activePromptText` by `\n` and render lines separated by `<br className="my-1.5" />` tags.
2. **"..." Dot-Dot Expand**:
   - Make any trailing `...` ellipsis interactively clickable:
     `<span onClick={() => setShowAllWords(!showAllWords)} title="Click to expand full prompt text">{showAllWords ? '... [Collapse]' : '... [Expand Full Text]'}</span>`.
   - Include a dedicated `[Expand (Full Text)]` / `[Collapse]` toggle button in the Prompt Instruction toolbar.
3. **Header Metadata Trio & Concluding Tail Snippet**:
   - Display Prompt Sequence badge: `#${selectedConversation.seq_code || 'P001'}`.
   - Display Instance Identity Trio:
     `[#${selectedConversation.instance_seq_num ?? selectedProject.instance_seq_num ?? 1} · ${selectedConversation.instance_exe_name ?? selectedProject.instance_exe_name ?? 'Antigravity.exe'} · ${selectedConversation.instance_name ?? selectedProject.instance_name ?? 'default'}]`.
   - Display Tail Excerpt snippet: `“… ending with: '${tailSnippet}'”`.
