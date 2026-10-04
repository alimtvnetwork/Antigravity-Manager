---
plan: 128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix
subtask: "05"
title: Prompt View Header Sequence, Tail Snippet & Instance Trio Capsule
domain: frontend-react-typescript-tailwind
depends_on:
  - 01-architecture-spec.md
  - 02-component-spec.md
  - 03-root-cause-analysis.md
citations:
  app_spec: 02-spec/21-app/128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix/02-component-spec.md
  root_cause_analysis: 02-spec/21-app/128-prompt-tree-linegap-expansion-hotkey-send-now-and-running-activity-fix/03-root-cause-analysis.md
  coding_guidelines: 02-spec/02-coding-guidelines/readme.md
target_files:
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# Subtask 05: Prompt View Header Sequence, Tail Snippet & Instance Trio Capsule

## 1. Context & User Requirement

The user explicitly requested:
> *"And also, in this UI, the prompt view, I need to see a little bit of prompts sequence, prompts ending text, and also the name of the instance of the Id. So again, Id sequence, Id exe name, and also Id instance name. These three things I need to see here in the UI nicely."*

The prompt inspector header previously showed basic conversation details, but lacked:
1. Clear **Prompt Sequence** badge visibility (`#P001`).
2. An **Ending Text Snippet** (`“… ending with: '<snippet>'”`) giving immediate context on what command or instruction concluded the prompt.
3. The unified **Instance Identity Trio** (`[#1 · Antigravity.exe · Default]`) combining instance sequence number, executable name, and instance profile name into an integrated capsule.

---

## 2. Target Files & Symbols

- `src/components/instances/PromptTreeViewModal.tsx`:
  - `getPromptTailSnippet`: Helper function extracting the terminal 10–12 words of the prompt.
  - Inspector header rendering block (~lines 1400–1480): Add the trio capsule, prompt sequence badge, and tail snippet.
  - Data mapping from `selectedConversation`: Bind `seq_code`, `instance_seq_num`, `instance_exe_name`, and `instance_name`.

---

## 3. Granular Implementation Steps

### Step 3.1: Concluding Prompt Tail Snippet Extraction
Ensure `getPromptTailSnippet` reliably produces a 10–12 word terminal slice:
```typescript
function getPromptTailSnippet(text: string, fallbackSnippet?: string): string {
    if (fallbackSnippet && fallbackSnippet.trim()) {
        return fallbackSnippet.trim();
    }
    const trimmed = text.trim();
    if (!trimmed) return '';
    const words = trimmed.split(/\s+/).filter(Boolean);
    if (words.length <= 12) {
        return words.join(' ');
    }
    return words.slice(-12).join(' ');
}
```

### Step 3.2: Construct Instance Identity Trio Capsule
Format the three required properties into a contiguous segmented pill capsule:
```tsx
{/* Instance Identity Trio: [#Seq · Exe · Name] */}
<div
    className="flex items-center gap-1.5 px-2.5 py-1 rounded-[5px] bg-purple-500/10 dark:bg-purple-950/30 border border-purple-500/20 text-purple-700 dark:text-purple-300 font-mono text-[11px]"
    title={`Instance Sequence #${selectedConversation.instance_seq_num ?? 1} | Executable: ${selectedConversation.instance_exe_name ?? 'Antigravity.exe'} | Profile: ${selectedConversation.instance_name ?? 'Default'}`}
>
    <span className="font-bold">
        #{selectedConversation.instance_seq_num ?? 1}
    </span>
    <span className="opacity-40">·</span>
    <span className="text-[10.5px]">
        {selectedConversation.instance_exe_name ?? 'Antigravity.exe'}
    </span>
    <span className="opacity-40">·</span>
    <span className="font-semibold text-purple-800 dark:text-purple-200">
        {selectedConversation.instance_name ?? 'Default'}
    </span>
</div>
```

### Step 3.3: Render Prompt Sequence Badge & Concluding Tail Snippet
1. Prompt Sequence Badge:
   ```tsx
   <span className="px-2 py-0.5 rounded-[5px] bg-blue-500/10 text-blue-600 dark:text-cyan-400 font-mono font-bold text-xs border border-blue-500/20">
       #{selectedConversation.seq_code || 'P001'}
   </span>
   ```
2. Concluding Tail Snippet:
   ```tsx
   {tailSnippet && (
       <div
           className="hidden md:flex items-center gap-1 text-[11px] text-slate-500 dark:text-slate-400 italic truncate max-w-sm lg:max-w-md"
           title={`Ending snippet: ${tailSnippet}`}
       >
           <span className="opacity-60">… ending with:</span>
           <span className="font-medium text-slate-700 dark:text-slate-300 truncate">
               '{tailSnippet}'
           </span>
       </div>
   )}
   ```

### Step 3.4: Layout Integration in Header Bar
- Place the sequence badge directly before the conversation title.
- Place the instance identity trio capsule adjacent to the project directory pill.
- Place the ending tail snippet in the secondary metadata row with subtle truncation handling (`truncate max-w-md`).

---

## 4. Visual & Ergonomic Specifications

- **Capsule Structure**: Contiguous segmented pill (`rounded-[5px]`, shared border, subtle divider dots `·`, dark-glass styling) conforming strictly to project UI design conventions in `AGENTS.md`.
- **Theme Support**: High-contrast legibility across both light (`bg-purple-500/10 text-purple-700`) and dark (`dark:bg-purple-950/30 dark:text-purple-300`) themes.
- **Responsive Adaptability**: On narrow modal widths (`< 768px`), tail snippet hides smoothly while the sequence badge and instance trio remain visible.

---

## 5. Verification & Acceptance Criteria

- [ ] Selecting any conversation renders the prompt sequence badge (e.g. `#{selectedConversation.seq_code || 'P001'}`).
- [ ] Header displays the instance identity trio capsule formatted as `[#1 · Antigravity.exe · Default]` (or custom instance values).
- [ ] Concluding tail snippet displays the ending 10–12 words preceded by `… ending with:`.
- [ ] Hover tooltips on the trio capsule and snippet show full metadata.
- [ ] `npm run build` succeeds without TypeScript or Vite errors.
