# Subtask 02: Prompt Origin Distinction and Subagent Hierarchy

**Parent Plan:** [145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md](../../145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md)  
**Parent Spec:** [01-architecture-and-ui-spec.md](../../../../02-spec/21-app/145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/01-architecture-and-ui-spec.md)  
**Status:** `COMPLETED`  
**Target Component:** `src/components/instances/PromptTreeViewModal.tsx` & classification engine  
**Lead Specialist:** Antigravity UI & UX Architecture Specialist  

---

## 1. Context & Objectives

Users reported that conversations in the Prompt Tree View mix direct human prompts with internal AI instructions, subagent directives, and non-prompt system artifacts:
> *"Can this distinguish what is prompt, what is not prompt? Because some of those are prompt, some of those are not. It's by the AI instruction. So we need to distinguish those, and those could be a sub point or things like that that you need to work on in the UI."*

Previous implementations only supported a crude binary flag (`USER_PROMPT` vs `SUBAGENT_INSTRUCTION`), failing to isolate raw tool outputs and system messages, and rendering all items at the same flat hierarchy level without parent-child sub-point relationships.

### Core Objectives:
1. Implement a **4-Tier Categorization Engine**:
   - `USER_PROMPT`: Human-initiated prompt (Primary Root Node).
   - `SUBAGENT_INSTRUCTION`: Autonomous subagent instruction (Nested Sub-Point Node).
   - `SYSTEM_MESSAGE`: System messages, environment preambles, and identity directives.
   - `TOOL_OUTPUT`: Tool execution logs, command outputs, and function results.
2. Establish a **Nested Sub-Point Hierarchy** in the left panel tree:
   - Identify child subagent conversations spawned by parent tasks.
   - Render them nested with tree branch connector glyphs (`↳`, `└──`) and subtle left indentation (`pl-6`).
3. Create distinct **Role & Origin Badges**:
   - Sky pill for `User Prompt` (`bg-sky-500/15 text-sky-700 dark:text-cyan-300`).
   - Purple pill for `AI Subagent` (`bg-purple-500/15 text-purple-700 dark:text-purple-300`).
   - Zinc pill for `System` (`bg-zinc-500/15 text-zinc-700 dark:text-zinc-300`).
   - Amber pill for `Tool Output` (`bg-amber-500/15 text-amber-700 dark:text-amber-300`).
4. Overhaul the **Category Filter Capsule**:
   - Filter seamlessly by `All Types`, `User Prompts`, `AI Subagents`, and `System/Tools`.
   - Default to hiding non-prompts from root explorer to prevent clutter.

---

## 2. Technical Architecture & Algorithm

### 2.1 4-Tier Classification Types & Engine

In `src/components/instances/PromptTreeViewModal.tsx`:

```typescript
export type PromptTier = 'USER_PROMPT' | 'SUBAGENT_INSTRUCTION' | 'SYSTEM_MESSAGE' | 'TOOL_OUTPUT';

export interface TierClassificationResult {
    tier: PromptTier;
    label: string;
    roleBadge: string;
    badgeStyle: string;
    iconName: 'user' | 'bot' | 'terminal' | 'wrench';
    isSubagent: boolean;
    isNonPrompt: boolean;
    confidence: number;
    matchedPattern?: string;
    subagentRole?: string;
}

export function classifyPromptTier(
    text?: string,
    title?: string,
    metadata?: Record<string, any>
): TierClassificationResult {
    const raw = (text || '').trim();
    const cleanTitle = (title || '').trim().toLowerCase();

    // 1. Tool Output Classification (Highest Specificity)
    const isToolOutput = 
        raw.startsWith('[Tool Result]') ||
        raw.startsWith('{"tool_call_id":') ||
        raw.startsWith('Tool returned:') ||
        raw.includes('<tool_response>') ||
        raw.includes('<tool_calls>') ||
        (raw.startsWith('```') && (cleanTitle.includes('output') || cleanTitle.includes('result')));

    if (isToolOutput) {
        return {
            tier: 'TOOL_OUTPUT',
            label: 'Tool Output',
            roleBadge: 'TOOL',
            badgeStyle: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-400/30',
            iconName: 'wrench',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.98,
            matchedPattern: 'tool_output_marker'
        };
    }

    // 2. System Message Classification
    const isSystemMessage =
        raw.startsWith('<SYSTEM_MESSAGE>') ||
        raw.includes('<conversation_transcript>') ||
        raw.includes('<artifacts>') ||
        cleanTitle.startsWith('system:');

    if (isSystemMessage && !raw.includes('invoked by a caller agent') && !raw.includes('You are')) {
        return {
            tier: 'SYSTEM_MESSAGE',
            label: 'System Message',
            roleBadge: 'SYSTEM',
            badgeStyle: 'bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30',
            iconName: 'terminal',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.95,
            matchedPattern: 'system_message_tag'
        };
    }

    // 3. AI Subagent Instruction Classification
    const subagentRoleMatch = 
        raw.match(/Role:\s*([A-Za-z0-9_\-\s]{3,30})/i) ||
        raw.match(/You are (?:the )?([A-Za-z0-9_\-\s]{3,30}) for Task/i);

    const isSubagent =
        raw.includes('invoked by a caller agent') ||
        raw.includes('<subagent_reminder>') ||
        raw.includes('send_message to communicate all results') ||
        /Role:\s*(?:Codebase Researcher|Database Debugger|QA Tester|Subagent)/i.test(raw) ||
        cleanTitle.includes('subagent') ||
        cleanTitle.includes('worker-') ||
        cleanTitle.includes('author-') ||
        metadata?.is_subagent === true;

    if (isSubagent) {
        const detectedRole = subagentRoleMatch ? subagentRoleMatch[1].trim() : 'AI Subagent';
        return {
            tier: 'SUBAGENT_INSTRUCTION',
            label: 'AI Subagent Instruction',
            roleBadge: detectedRole,
            badgeStyle: 'bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-400/30',
            iconName: 'bot',
            isSubagent: true,
            isNonPrompt: false,
            confidence: 0.96,
            matchedPattern: 'subagent_signature',
            subagentRole: detectedRole
        };
    }

    // 4. Default: USER_PROMPT (Direct Human Request)
    return {
        tier: 'USER_PROMPT',
        label: 'User Prompt',
        roleBadge: 'USER',
        badgeStyle: 'bg-sky-500/15 text-sky-700 dark:text-cyan-300 border-sky-400/30',
        iconName: 'user',
        isSubagent: false,
        isNonPrompt: false,
        confidence: 0.90
    };
}
```

---

### 2.2 Nested Sub-Point Tree Assembly

Conversations within a project are structured into a 2-level hierarchy:
- **Root Level:** Primary `USER_PROMPT` nodes.
- **Sub-Point Level:** Attached `SUBAGENT_INSTRUCTION` nodes executed in service of the root task.

```typescript
export interface HierarchicalConversationNode {
    primaryNode: AgmConversationNode;
    classification: TierClassificationResult;
    subagents: HierarchicalConversationNode[];
    isExpanded: boolean;
}

export function assembleConversationHierarchy(
    conversations: AgmConversationNode[]
): HierarchicalConversationNode[] {
    const rootNodes: HierarchicalConversationNode[] = [];
    let currentRoot: HierarchicalConversationNode | null = null;

    for (const conv of conversations) {
        const classification = classifyPromptTier(conv.prompt_preview_200w, conv.title);

        const node: HierarchicalConversationNode = {
            primaryNode: conv,
            classification,
            subagents: [],
            isExpanded: true
        };

        if (classification.tier === 'SUBAGENT_INSTRUCTION') {
            if (currentRoot) {
                // Attach as child sub-point to active root prompt
                currentRoot.subagents.push(node);
            } else {
                // If no root has occurred yet, treat as standalone root
                rootNodes.push(node);
            }
        } else {
            // New User Prompt or System Message becomes new root
            rootNodes.push(node);
            if (classification.tier === 'USER_PROMPT') {
                currentRoot = node;
            }
        }
    }

    return rootNodes;
}
```

---

### 2.3 Nested Tree View UI Rendering

```tsx
{/* Rendering Parent Root Node */}
<div className="space-y-0.5">
    {renderConversationNode(root.primaryNode, project)}

    {/* Rendering Child Sub-Points (Indented with Branch Glyphs) */}
    {root.subagents.length > 0 && (
        <div className="pl-5 space-y-0.5 border-l border-slate-200 dark:border-[#15334d]/80 ml-3.5 my-0.5">
            {root.subagents.map((subNode) => (
                <div
                    key={subNode.primaryNode.conversation_id}
                    onClick={() => selectConversation(subNode.primaryNode, project)}
                    className={cn(
                        "relative flex items-center justify-between rounded-[5px] px-2 py-1 text-xs cursor-pointer transition-colors group",
                        selectedConversation?.conversation_id === subNode.primaryNode.conversation_id
                            ? "bg-purple-600 text-white font-medium shadow-2xs"
                            : "text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#0c2438]"
                    )}
                >
                    <div className="flex items-center gap-1.5 min-w-0">
                        {/* Branch Connector Glyph */}
                        <span className="text-slate-400 dark:text-slate-500 font-mono text-[10px] select-none">↳</span>
                        <Bot className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                        <span className="px-1.5 py-0.2 rounded-full text-[8.5px] font-mono font-bold uppercase tracking-wider bg-purple-500/15 text-purple-700 dark:text-purple-300 border border-purple-400/30 shrink-0">
                            {subNode.classification.roleBadge}
                        </span>
                        <span className="truncate text-[11px]">
                            {subNode.primaryNode.title || 'Subagent Task'}
                        </span>
                    </div>
                </div>
            ))}
        </div>
    )}
</div>
```

---

### 2.4 4-Tier Category Filter Capsule

Overhaul the left-panel filter bar into a 4-tier segmented pill capsule:

```tsx
{/* 4-Tier Category Filter Capsule */}
<div className="w-full inline-flex items-center rounded-full border border-slate-200/80 dark:border-[#15334d]/90 bg-white/80 dark:bg-[#0c2438]/80 backdrop-blur-xs p-0.5 shadow-2xs divide-x divide-slate-200/70 dark:divide-[#15334d]/80">
    {/* All */}
    <button
        type="button"
        onClick={() => setTierFilter('all')}
        className={cn(
            "flex-1 py-1 text-[10px] font-medium rounded-l-full transition-colors text-center cursor-pointer",
            tierFilter === 'all'
                ? "bg-blue-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
        )}
    >
        All
    </button>

    {/* User Prompts */}
    <button
        type="button"
        onClick={() => setTierFilter('user')}
        className={cn(
            "flex-1 py-1 text-[10px] font-medium transition-colors flex items-center justify-center gap-1 cursor-pointer",
            tierFilter === 'user'
                ? "bg-sky-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
        )}
        title="Human User Prompts"
    >
        <User className="w-2.5 h-2.5 shrink-0" />
        <span>User</span>
    </button>

    {/* AI Subagents */}
    <button
        type="button"
        onClick={() => setTierFilter('subagent')}
        className={cn(
            "flex-1 py-1 text-[10px] font-medium transition-colors flex items-center justify-center gap-1 cursor-pointer",
            tierFilter === 'subagent'
                ? "bg-purple-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
        )}
        title="Autonomous Subagent Directives"
    >
        <Bot className="w-2.5 h-2.5 shrink-0" />
        <span>Subagent</span>
    </button>

    {/* System / Tools */}
    <button
        type="button"
        onClick={() => setTierFilter('system')}
        className={cn(
            "flex-1 py-1 text-[10px] font-medium rounded-r-full transition-colors flex items-center justify-center gap-1 cursor-pointer",
            tierFilter === 'system'
                ? "bg-zinc-600 text-white shadow-2xs font-semibold"
                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
        )}
        title="System Messages and Tool Diagnostics"
    >
        <Terminal className="w-2.5 h-2.5 shrink-0" />
        <span>System</span>
    </button>
</div>
```

---

## 3. Step-by-Step Implementation Checklist

1. [ ] **Classification Types & Logic:**
   - Define `PromptTier` and `TierClassificationResult` interfaces.
   - Implement `classifyPromptTier` replacing the legacy binary helper.
2. [ ] **Hierarchy Builder:**
   - Create `assembleConversationHierarchy` grouping subagents under parent human prompts.
3. [ ] **Left Panel Rendering:**
   - Update `renderConversationListWithGrouping` to support nested subagent children with `↳` branch glyphs.
   - Attach visual role badges based on detected subagent roles (Researcher, Worker, Debugger).
4. [ ] **Filter Controls:**
   - Replace binary filter button with the 4-tier segmented capsule (`All`, `User`, `Subagent`, `System`).
   - Add state filtering logic in `filteredProjects` calculation.
5. [ ] **Validation:**
   - Confirm subagent tasks are cleanly grouped under their parent prompts.
   - Verify selecting a subagent node correctly displays its instruction text in the right preview pane.

---

## 4. Acceptance Criteria & Verification

- **AC-02.1:** Classifier reliably categorizes `USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, and `TOOL_OUTPUT`.
- **AC-02.2:** Subagent nodes render indented under their parent task with `↳` connector glyphs.
- **AC-02.3:** Subagent badge shows detected role (e.g. `RESEARCHER`, `WORKER`, `SUBAGENT`).
- **AC-02.4:** 4-Tier Category Filter cleanly isolates User Prompts from Subagent and System turns.
- **AC-02.5:** Pre-flight checks (`cargo fmt`, `cargo clippy`, `npm run build`) pass cleanly.
