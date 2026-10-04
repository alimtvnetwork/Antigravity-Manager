# Subtask 03: Filter Out and Merge Untitled 0-Word Empty Conversations

**Task ID:** `128-03-untitled-empty-conversations-filter-and-merge`  
**Target File:** `src/components/instances/PromptTreeViewModal.tsx`  
**Owner:** Worker 01 (Frontend Specialist)  
**Status:** READY  
**Prerequisites:** Subtask 01, Subtask 02  

---

## 1. Context & User Directive (Verbatim)

```text
And if we go into the `gitmap` section. You want to get into the `gitmap` section, the Git maps prompt. If we open, the problem here is that it still has prompts which has untitled conversation and has no prompt content. I asked you not to show this in the list while you are seeing this... Lots of untitled conversations. I asked you to merge these untitled conversations which has no prompt. Try to merge this. Yet understood
```

---

## 2. Technical Objective

1. Unconditionally purge 0-word, empty-content "Untitled Conversation" ghost nodes from active conversation tree lists in `PromptTreeViewModal.tsx` (specifically targeting repositories like `gitmap`).
2. Consolidate non-ghost empty or historical sessions into a unified, collapsible `Archived / Stale Prompts (N)` accordion section inside each project tree node to maintain an organized, clutter-free navigation sidebar.

---

## 3. Implementation Blueprint

### 3.1 Ghost Conversation Detection Logic

- **Location:** `src/components/instances/PromptTreeViewModal.tsx` (~Lines 176–210).
- **Predicate Definitions:**
  ```typescript
  // Identify true ghost conversations: untitled and zero content
  export function isGhostConversation(conv: AgmConversationNode): boolean {
      const title = (conv.title || '').trim().toLowerCase();
      const isUntitled =
          !title ||
          title === 'untitled' ||
          title.startsWith('untitled conversation') ||
          title === 'new conversation' ||
          title === 'conversation' ||
          title === (conv.short_id || '').toLowerCase();
      const isEmptyPrompt =
          conv.prompt_word_count === 0 ||
          !conv.prompt_preview_200w ||
          conv.prompt_preview_200w.trim().length === 0;
      return isUntitled && isEmptyPrompt;
  }

  // Identify stale or empty conversations (exempt if actively running with content)
  export function isStaleOrEmptyConversation(conv: AgmConversationNode): boolean {
      if (isGhostConversation(conv)) {
          return true;
      }
      const isEmptyPrompt =
          conv.prompt_word_count === 0 ||
          !conv.prompt_preview_200w ||
          conv.prompt_preview_200w.trim().length === 0;
      if (isEmptyPrompt && !conv.is_running) {
          return true;
      }
      return false;
  }
  ```

---

### 3.2 Filtering Active Project Conversations & Prompt Counters

- **Location:** `src/components/instances/PromptTreeViewModal.tsx` in `renderProjectNode` (~Lines 1565–1600).
- **Filtering Implementation:**
  ```typescript
  // Exclude ghost conversations from the total count and active view
  const filteredConvs = project.conversations.filter((c) => {
      if (activeFilter === 'archived') return true;
      return !isGhostConversation(c);
  });

  const totalProjectPrompts = filteredConvs.reduce(
      (sum, c) => sum + (c.step_count > 0 ? c.step_count : 1),
      0
  );

  const sortedConvs = sortConversations(project.conversations);
  const activeConversations: AgmConversationNode[] = [];
  const staleConversations: AgmConversationNode[] = [];

  sortedConvs.forEach((conv) => {
      if (isGhostConversation(conv) && activeFilter !== 'archived') {
          return; // Strictly drop 0-word untitled ghost nodes
      }
      if (isStaleOrEmptyConversation(conv)) {
          staleConversations.push(conv);
      } else {
          activeConversations.push(conv);
      }
  });
  ```

---

### 3.3 Merging Empty / Stale Sessions into Collapsible Accordion

- **Location:** `src/components/instances/PromptTreeViewModal.tsx` in `renderProjectNode` (~Lines 1675–1715).
- **Accordion UI Rendering:**
  ```tsx
  {/* Active Conversations */}
  <div className="space-y-0.5 pl-3">
      {activeConversations.map((conv) => renderConversationNode(project, conv))}
  </div>

  {/* Collapsible Merged Group: Archived / Stale Prompts */}
  {staleConversations.length > 0 && (
      <div className="mt-1 pt-1 border-t border-slate-200/50 dark:border-[#15334d]/50 pl-3">
          <button
              type="button"
              onClick={(e) => {
                  e.stopPropagation();
                  setExpandedStaleGroups((prev) => ({
                      ...prev,
                      [project.project_id]: !prev[project.project_id],
                  }));
              }}
              className="flex items-center justify-between w-full px-2 py-1 text-[11px] text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 rounded hover:bg-slate-100 dark:hover:bg-[#0c2438] transition-colors"
          >
              <div className="flex items-center gap-1.5">
                  {isStaleGroupExpanded ? (
                      <ChevronDown className="w-3 h-3 text-slate-400" />
                  ) : (
                      <ChevronRight className="w-3 h-3 text-slate-400" />
                  )}
                  <span>Archived / Stale Prompts</span>
              </div>
              <span className="font-mono text-[10px] bg-slate-200 dark:bg-slate-800 px-1.5 py-0.2 rounded-full">
                  {staleConversations.length}
              </span>
          </button>

          {isStaleGroupExpanded && (
              <div className="space-y-0.5 mt-1 pl-2 border-l border-slate-200 dark:border-[#15334d]">
                  {staleConversations.map((conv) => renderConversationNode(project, conv))}
              </div>
          )}
      </div>
  )}
  ```

---

## 4. Verification Checklist & Definition of Done (DoD)

- [ ] Ghost conversations (0 words + untitled) never appear in the active conversation list for `gitmap` or any project node.
- [ ] Project header badges display accurate total prompt counts excluding ghost conversations.
- [ ] Non-ghost empty or stale conversations are merged under "Archived / Stale Prompts (N)".
- [ ] Expanding the "Archived / Stale Prompts" accordion renders the enclosed conversation items smoothly.
- [ ] Switching filter pills to "Archived" allows explicit inspection of archived nodes when desired.
- [ ] ZERO git commands used.
