# Subtask 004 — Prompt Tree View Synthetic Turn Strip & Strict Descending Activity Sorting

## Status: Pending
## Priority: High
## Assigned Worker: Worker 02 (Prompts, Backend IPC & Cross-Platform Specialist)

---

## 1. Objectives & User Feedback Analysis

In user feedback for Plan 112:
1. **Elimination of Synthetic Layer 3 "Turn" Nodes**:
   - The user noted:
     > *"What is this turn? I don't understand. If something is turn- I think you can skip that from keeping it in the display. Only the prompts which is given by the user. Terms are, I think, is done by the IDE, not by us. Checkpoint of the conversation. If that is the case, then I request you to take out the terms. Only display the prompts which you find as a text, you display that. And this is how you proceed further."*
   - Currently, `PromptTreeViewModal.tsx` contains a function `getPromptTurns` that fabricates artificial turn objects:
     `[Turn #1] Checkpoint prompt in conversation <id>...`
     This clutters the tree hierarchy with confusing fake placeholder nodes that never existed in the user's conversation.
   - Requirement: **Strip out synthetic Layer 3 turn nodes entirely.** The tree view becomes a clean, intuitive 2-tier tree:
     - **Level 1**: Project / Workspace
     - **Level 2**: Conversation / Real User Prompt
   - Clicking a conversation directly displays the authentic user-submitted prompt text in the preview and editor panels.
2. **Strict Descending Sort by `last_modified`**:
   - The user emphasized:
     > *"And the latest one, we always want to have the top in the tree view of the tree view. Does this make sense?"*
   - Currently, projects with no recent conversations or equal timestamps can shuffle or rely on alphabetical fallbacks.
   - Requirement: Enforce **strict descending sort by `last_modified`** across both projects and conversations.
   - The project with the most recently updated conversation is placed at the very top of the tree.
   - Within each project, conversations are strictly sorted so that the newest conversation is at the top.
   - On opening the modal, auto-selection immediately highlights the newest prompt at the very top.

---

## 2. File Targets

| File Path | Description of Changes |
| :--- | :--- |
| `src/components/instances/PromptTreeViewModal.tsx` | Remove `getPromptTurns` synthetic logic, eliminate Layer 3 DOM rendering, bind conversation clicks directly to real prompt text, and enforce strict descending timestamp sorting. |

---

## 3. Exact Implementation Specifications

### 3.1 Removing Synthetic Turn Generation & Simplifying Tree to 2 Tiers

#### In `src/components/instances/PromptTreeViewModal.tsx`:
1. Remove `PromptTurnNode` interface and `getPromptTurns` function:
```typescript
// REMOVE THIS SYNTHETIC FUNCTION:
// function getPromptTurns(conv: AgmConversationNode): PromptTurnNode[] { ... }
```
2. Remove `selectedTurnNumber` state and its references:
```typescript
// Replace:
// const [selectedTurnNumber, setSelectedTurnNumber] = useState<number>(1);
// With direct conversation selection:
```
3. Update `selectConversation`:
```typescript
const selectConversation = useCallback((conv: AgmConversationNode, project: AgmProjectTreeNode) => {
    setSelectedProject(project);
    setSelectedConversation(conv);
    const text = conv.prompt_preview_200w || '';
    setActivePromptText(text);
    setEditedPromptText(text);
    setShowAllWords(false);
}, []);
```
4. Refactor `renderConversationNode`:
   - Eliminate the collapsible Layer 3 turns container (`isConvExpanded && promptTurns.map(...)`).
   - A conversation row represents the prompt directly:
```tsx
const renderConversationNode = (conv: AgmConversationNode, project: AgmProjectTreeNode) => {
    const isConvSelected = selectedConversation?.conversation_id === conv.conversation_id;
    const isRunning = conv.is_running || conv.status === 'RUNNING';

    return (
        <div
            key={conv.conversation_id}
            onClick={() => selectConversation(conv, project)}
            onDoubleClick={() => openInspector(conv, project.repo_path)}
            className={cn(
                'group flex items-center justify-between rounded-[6px] px-2.5 py-1.5 text-xs cursor-pointer transition-colors border',
                isConvSelected
                    ? 'bg-blue-600 text-white font-medium border-blue-500 shadow-xs'
                    : 'text-slate-700 dark:text-slate-300 border-transparent hover:bg-slate-100 dark:hover:bg-[#0c2438]'
            )}
            title="Click to view prompt; double-click for Full inspector"
        >
            <div className="flex items-center gap-2 min-w-0 flex-1">
                <MessageSquare className={cn('h-3.5 w-3.5 shrink-0', isConvSelected ? 'text-white' : 'text-blue-500 opacity-80')} />
                <span className="truncate text-xs font-medium">
                    {conv.title || conv.short_id || conv.conversation_id.slice(0, 8)}
                </span>
            </div>
            <div className="flex items-center gap-1.5 shrink-0">
                {isRunning ? (
                    <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-[4px] text-[9px] font-bold bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 animate-pulse">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shrink-0" />
                        RUNNING
                    </span>
                ) : (
                    <span className={cn(
                        "text-[9px] font-mono",
                        isConvSelected ? "text-blue-100" : "text-slate-400"
                    )}>
                        {conv.last_modified ? new Date(conv.last_modified).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : ''}
                    </span>
                )}
            </div>
        </div>
    );
};
```

---

### 3.2 Enforcing Strict Descending Activity Sorting

#### 1. Project Sorting by Most Recent Conversation Timestamp:
```typescript
const getProjectLatestTime = (project: AgmProjectTreeNode): number => {
    if (!project.conversations || project.conversations.length === 0) return 0;
    return Math.max(
        0,
        ...project.conversations.map((c) => {
            const time = new Date(c.last_modified).getTime();
            return Number.isFinite(time) ? time : 0;
        })
    );
};

const sortFn = (a: AgmProjectTreeNode, b: AgmProjectTreeNode) => {
    // 1. Pinned projects have priority if pinned
    const aPinned = pinnedProjectIds.includes(a.project_id);
    const bPinned = pinnedProjectIds.includes(b.project_id);
    if (aPinned !== bPinned) return aPinned ? -1 : 1;

    // 2. Actively running projects stay on top
    const aRunning = a.is_running || a.conversations.some((c) => c.is_running || c.status === 'RUNNING');
    const bRunning = b.is_running || b.conversations.some((c) => c.is_running || c.status === 'RUNNING');
    if (aRunning !== bRunning) return aRunning ? -1 : 1;

    // 3. Strict Descending by last_modified timestamp
    const aLatest = getProjectLatestTime(a);
    const bLatest = getProjectLatestTime(b);
    if (bLatest !== aLatest) return bLatest - aLatest;

    // 4. Stable tie-breaker
    return a.repo_name.localeCompare(b.repo_name);
};
```

#### 2. Conversation Sorting Within Each Project:
```typescript
const sortConversations = useCallback(
    (convs: AgmConversationNode[]) => {
        let sorted = [...convs];
        if (activeFilter === 'running') {
            sorted = sorted.filter((c) => c.is_running || c.status === 'RUNNING');
        }
        return sorted.sort((a, b) => {
            // Running conversations first
            const aRunning = a.is_running || a.status === 'RUNNING';
            const bRunning = b.is_running || b.status === 'RUNNING';
            if (aRunning !== bRunning) {
                return aRunning ? -1 : 1;
            }
            // Strict descending by last_modified timestamp
            const aTime = new Date(a.last_modified).getTime() || 0;
            const bTime = new Date(b.last_modified).getTime() || 0;
            return bTime - aTime;
        });
    },
    [activeFilter]
);
```

#### 3. Auto-Selection of the Topmost Item on Modal Open:
```typescript
const performAutoSelection = useCallback(
    (data: AgmProjectTreeNode[], currentArchivedIds: string[], currentPinnedIds: string[]) => {
        if (data.length === 0) {
            setSelectedProject(null);
            setSelectedConversation(null);
            return;
        }

        const nonArchived = data.filter((p) => !currentArchivedIds.includes(p.project_id));
        const targetPool = nonArchived.length > 0 ? nonArchived : data;

        // Sort projects using strict sortFn
        const prioritized = [...targetPool].sort(sortFn);
        const topProject = prioritized[0];

        if (topProject) {
            setSelectedProject(topProject);
            setExpandedProjects({ [topProject.project_id]: true });

            if (topProject.conversations && topProject.conversations.length > 0) {
                const sortedConvs = sortConversations(topProject.conversations);
                const topConv = sortedConvs[0];
                selectConversation(topConv, topProject);
            }
        }
    },
    [sortConversations, selectConversation]
);
```

---

## 4. Acceptance Criteria & Verification

1. **No Synthetic Turn Nodes**:
   - Open Prompt Tree View modal on any instance.
   - Inspect the tree structure.
   - Verify there are NO `[Turn #1] Checkpoint prompt...` or artificial `Turn #N` expandable child nodes under conversations.
   - Verify clicking a conversation directly loads its authentic prompt text into the right preview panel.
2. **Top-Level Sorting by Activity**:
   - Open the modal with multiple projects and conversations.
   - Verify the project with the most recent `last_modified` timestamp is at the very top of the list.
   - Expand the topmost project; verify its conversations are strictly ordered from newest to oldest.
   - Verify the auto-selected prompt is the newest conversation at the top of the tree.
