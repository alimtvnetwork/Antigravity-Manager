# Subtask 02: Interactive Dot-Dot (`...`) Click Expansion and Hotkey `N` / "Send Now" Dispatch

**Task ID:** `128-02-dot-dot-click-expansion-and-hotkey-send-now`  
**Target File:** `src/components/instances/PromptTreeViewModal.tsx`  
**Owner:** Worker 01 (Frontend Specialist)  
**Status:** READY  
**Prerequisites:** Subtask 01  

---

## 1. Context & User Directive (Verbatim)

```text
Then when I click on dot dot, it does not expand to the full text. That is a problem, and also N key, or also Send Now does not work at all. So you need to test it live here, then it goes there, and you can trace back.
```

---

## 2. Technical Objective

1. Convert trailing ellipsis indicators (`...` / `…`) into fully interactive, clickable inline badges that toggle between truncated preview and full prompt instructions across Markdown Preview, Raw Mode, and Full-Screen Inspector.
2. Bind the `N` keyboard shortcut at modal scope and harden the "Send Now" button workflow to physically generate `.antigravity_resume_task.json`, invoke IDE resume IPC, copy text to clipboard, focus the target instance window, and provide traceable toast feedback.

---

## 3. Implementation Blueprint

### 3.1 Interactive Dot-Dot (`...`) Click-to-Expand Badge

- **State Management:**
  ```typescript
  const [showAllWords, setShowAllWords] = useState<boolean>(false);
  ```
- **Ellipsis Detection & Badge Rendering:**
  In `RichMarkdownRenderer`, identify lines ending in ellipsis (`...` or `…`) when text is truncated:
  ```tsx
  const isTrailingEllipsis = trimmed.endsWith('...') || trimmed.endsWith('…');
  if (isTrailingEllipsis && onToggleExpand) {
      const cleanLine = line.replace(/(\.{3}|…)\s*$/, '');
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
  }
  ```
- **Universal Component Forwarding:**
  Ensure `showAllWords` and `onToggleExpand={() => setShowAllWords(!showAllWords)}` are passed into:
  1. Main Markdown Preview panel.
  2. Full-Screen Inspector dialog modal (`InspectorModal`).
  3. Raw View toggle pill button.

---

### 3.2 Modal-Scoped Hotkey `N` Listener

- **Listener Hook:**
  ```typescript
  useEffect(() => {
      if (!isOpen) return;

      const handleKeyDown = (e: KeyboardEvent) => {
          // Guard against modifier keys
          if (e.ctrlKey || e.altKey || e.metaKey) return;

          // Guard against interactive input elements
          const target = e.target as HTMLElement | null;
          if (target) {
              const tagName = target.tagName?.toLowerCase();
              if (
                  tagName === 'input' ||
                  tagName === 'textarea' ||
                  tagName === 'select' ||
                  target.isContentEditable ||
                  target.getAttribute('contenteditable') === 'true'
              ) {
                  return;
              }
          }

          if (e.key === 'n' || e.key === 'N') {
              e.preventDefault();
              handleResendPrompt();
          }
      };

      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, handleResendPrompt]);
  ```

---

### 3.3 "Send Now" & `handleResendPrompt` Live Execution Pipeline

- **Fallback Resolution:**
  If user clicked a project header without selecting a child conversation, automatically resolve the latest non-empty conversation:
  ```typescript
  let conv = selectedConversationRef.current || selectedConversation;
  let proj = selectedProjectRef.current || selectedProject;

  if (!conv && proj && proj.conversations.length > 0) {
      const candidates = proj.conversations.filter(
          (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
      );
      if (candidates.length > 0) {
          const sorted = [...candidates].sort((a, b) => {
              const aTime = new Date(a.last_modified).getTime() || 0;
              const bTime = new Date(b.last_modified).getTime() || 0;
              return bTime - aTime;
          });
          conv = sorted[0];
      }
  }
  ```

- **Physical File Generation (`.antigravity_resume_task.json`):**
  ```typescript
  if (repoPath) {
      const taskPath = `${repoPath.replace(/[\\/]+$/, '')}/.antigravity_resume_task.json`;
      const payload = {
          prompt_id: conv?.conversation_id || `prompt-${Date.now()}`,
          project_id: proj?.project_id || '',
          instance_id: proj?.instance_id || instanceId || 'default',
          repo_path: repoPath,
          prompt_content: promptContent,
          model: 'gemini-2.5-pro',
          auto_boot: true,
          status: 'dispatched',
          resumed_at: Math.floor(Date.now() / 1000),
      };
      await invoke('save_text_file', {
          path: taskPath,
          content: JSON.stringify(payload, null, 2),
      });
  }
  ```

- **IPC Trigger, Clipboard Copy & Window Focus:**
  ```typescript
  const targetInstId = proj?.instance_id || instanceId || 'default';
  await invoke('resume_recent_project_prompts', { instanceId: targetInstId, maxAgeSeconds: 3600 });
  await navigator.clipboard.writeText(promptContent);
  await focusOrLaunchInstance(targetInstId);
  setActionMsg("Prompt Dispatched & Focused IDE (via Hotkey 'N' / Send Now)!");
  setTimeout(() => setActionMsg(null), 3500);
  ```

---

## 4. Verification Checklist & Definition of Done (DoD)

- [ ] Clicking `... [Expand Full Text]` toggles full text immediately without re-rendering jumps.
- [ ] Clicking `... [Collapse]` truncates back to 120 words cleanly.
- [ ] Pressing `N` or `n` dispatches the prompt and flashes "Send Now" spinner.
- [ ] Typing 'n' or 'N' inside search bars, edit textarea, or select inputs does NOT trigger dispatch.
- [ ] `.antigravity_resume_task.json` file is written to the project root with valid JSON metadata.
- [ ] Target IDE instance window is brought to foreground via `focusOrLaunchInstance`.
- [ ] Action feedback toast appears for 3.5 seconds.
- [ ] ZERO git commands used.
