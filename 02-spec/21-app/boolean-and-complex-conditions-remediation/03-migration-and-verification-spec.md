# Migration and Verification Specification: Frontend Tree Conditionals

> **Specification:** 03-migration-and-verification-spec.md  
> **Target Subsystem:** Frontend Prompt Tree View  
> **Target Files:**  
> - `src/components/instances/PromptTreeView/usePromptTreeData.ts`  
> - `src/components/instances/PromptTreeView/tierClassification.ts`  
> **Reference Guidelines:** `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/`  
> **Verification Command:** `npm run build` (Exit code: 0)  

---

## 1. Executive Summary & Intent

The Prompt Tree View is a core GUI component in Antigravity-Manager, responsible for organizing, classifying, and rendering conversations, projects, and subagent hierarchies across multiple instance profiles.

Prior implementations introduced several coding guideline violations:
1. **Compound Multi-Clause Conditions:** Expressions chaining 3 to 7 conditional operands using mixed `&&` and `||` without intermediate affirmative decomposition (violating Principle 4: 2-operand limit).
2. **Explicit Boolean Comparisons:** Anti-patterns such as `metadata?.is_subagent === true` violating Principle 5 and Rule 1 (mandatory implicit boolean evaluation).
3. **Mixed Polarity Checks:** Blending affirmative guards with negated expressions (e.g., `isSystemMessage && !raw.includes(...)`) forcing cognitive context-switching (violating Principle 6).
4. **Nested Inline Ternaries:** Inline conditional nesting for instance alias branching and stale group tracking reducing legibility.

This specification details the extraction of 15 strongly typed predicate helper functions across `usePromptTreeData.ts` and `tierClassification.ts`, standardizing boolean naming prefixes (`is*`, `has*`), decomposing compound expressions into single-intent booleans, eliminating explicit `=== true` checks, and enforcing clean early-exit guard structures.

---

## 2. Target File 1: `usePromptTreeData.ts` Specification

### 2.1 Issues Identified

- **Instance Matching Logic:** `loadTree` contained repeated, multi-clause inline condition chains:
  `!instanceId || instanceId === 'default' || instanceId === '__default__' || instanceId === 'all' || instanceId.includes('default')`
  and deep nested checks inside `.filter((p) => ...)` and `.filter((c) => ...)`.
- **Project Running Evaluation:** Inline checks `Boolean(a.is_running) || a.conversations.some((c) => Boolean(c.is_running))` duplicated in sort callbacks and auto-selection finders.
- **Project Sorting Complexity:** An inline 20-line sort comparator combining pinned status, running state, timestamp resolution, and lexical name sorting.
- **SWR Cache Invalidation Branching:** Repeated checks for default instance aliases across cache eviction paths.

### 2.2 Helper Functions Specification

#### 1. `isDefaultInstanceAlias`
Determines whether an instance identifier corresponds to the primary default profile alias.

```typescript
/**
 * Evaluates whether an instance identifier is empty or matches default instance aliases.
 */
export function isDefaultInstanceAlias(instanceId?: string | null): boolean {
    if (!instanceId) {
        return true;
    }
    const normalized = instanceId.trim().toLowerCase();
    const isDefaultExact = normalized === 'default' || normalized === '__default__';
    const hasDefaultSubstring = normalized.includes('default');
    return isDefaultExact || hasDefaultSubstring;
}
```

#### 2. `isAllInstancesFilter`
Evaluates whether an instance filter indicates aggregating all instances.

```typescript
/**
 * Evaluates whether the instance filter targets all instances without profile scoping.
 */
export function isAllInstancesFilter(instanceId?: string | null): boolean {
    if (!instanceId) {
        return true;
    }
    return instanceId.trim().toLowerCase() === 'all';
}
```

#### 3. `isInstanceCompatible`
Determines if a project or conversation's instance ID matches the target instance filter.

```typescript
/**
 * Determines whether a node's instance identifier matches the active instance target.
 * Resolves exact matches and cross-alias default compatibility.
 */
export function isInstanceCompatible(
    nodeInstanceId?: string | null,
    targetInstanceId?: string | null
): boolean {
    if (nodeInstanceId === targetInstanceId) {
        return true;
    }
    const isTargetDefault = isDefaultInstanceAlias(targetInstanceId);
    const isNodeDefault = isDefaultInstanceAlias(nodeInstanceId);
    return isTargetDefault && isNodeDefault;
}
```

#### 4. `isProjectRunning`
Evaluates whether a project node has active execution, either on the project itself or within any child conversations.

```typescript
/**
 * Returns true if a project is marked running or contains any active conversations.
 */
export function isProjectRunning(project: AgmProjectTreeNode): boolean {
    const isNodeRunning = Boolean(project.is_running);
    const hasRunningConversation = project.conversations.some((c) => Boolean(c.is_running));
    return isNodeRunning || hasRunningConversation;
}
```

#### 5. `compareProjectsPriority`
A deterministic comparator sorting projects by: (1) pinned priority, (2) active running status, (3) most recent conversation timestamp, (4) alphabetical repository name.

```typescript
/**
 * Comparator sorting projects by pinned status, running activity, last modified time, and name.
 */
export function compareProjectsPriority(
    a: AgmProjectTreeNode,
    b: AgmProjectTreeNode,
    pinnedProjectIds: string[]
): number {
    const isAPinned = pinnedProjectIds.includes(a.project_id);
    const isBPinned = pinnedProjectIds.includes(b.project_id);
    if (isAPinned !== isBPinned) {
        return isAPinned ? -1 : 1;
    }

    const isARunning = isProjectRunning(a);
    const isBRunning = isProjectRunning(b);
    if (isARunning !== isBRunning) {
        return isARunning ? -1 : 1;
    }

    const aLatest = Math.max(0, ...a.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
    const bLatest = Math.max(0, ...b.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
    if (bLatest !== aLatest) {
        return bLatest - aLatest;
    }

    return a.repo_name.localeCompare(b.repo_name);
}
```

### 2.3 Refactoring Call Sites

- In `invalidatePromptTreeCache`:
  ```typescript
  export function invalidatePromptTreeCache(targetInstanceId?: string): void {
      if (!targetInstanceId) {
          promptTreeSwrCache.clear();
          return;
      }
      promptTreeSwrCache.delete(targetInstanceId);
      if (isDefaultInstanceAlias(targetInstanceId)) {
          promptTreeSwrCache.delete('default');
          promptTreeSwrCache.delete('__default__');
      }
  }
  ```
- In `performAutoSelection`:
  Replace inline sorting of `targetPool` with:
  ```typescript
  const prioritized = [...targetPool].sort((a, b) =>
      compareProjectsPriority(a, b, currentPinnedIds)
  );
  ```
  And replace `Boolean(p.is_running) && p.conversations.length > 0` with `isProjectRunning(p) && p.conversations.length > 0`.
- In `loadTree`:
  Replace complex filter chains with:
  ```typescript
  const isFilterAll = isAllInstancesFilter(instanceId);
  const relevant = isFilterAll
      ? data
      : data
            .filter((p) => isInstanceCompatible(p.instance_id, instanceId))
            .map((p) => ({
                ...p,
                conversations: (p.conversations || []).filter((c) =>
                    isInstanceCompatible(c.instance_id, instanceId)
                ),
            }));
  ```

---

## 3. Target File 2: `tierClassification.ts` Specification

### 3.1 Issues Identified

- **Tool Output Detection:** A 6-clause compound condition combining prefix checks and title substring checks in one expression.
- **System Message Detection:** Mixed polarity checks combining positive prefixes (`raw.startsWith('<SYSTEM_MESSAGE>')`), negation of user content (`!hasUserContent`), and exclusions (`!raw.includes('invoked by a caller agent')`).
- **Subagent Detection:**
  - 13-clause compound condition chaining regex, prefix, metadata, and string tests.
  - **Rule 1 Violation:** `metadata?.is_subagent === true` violates affirmative implicit evaluation guidelines.
- **Hierarchy Assembly Sorting:** Repeated inline subagent status queries and a copy-paste bug in the queued comparator (`s.primaryNode` referenced across `bStatus`).

### 3.2 Helper Functions Specification

#### 1. `isToolOutputPayload`
Evaluates whether a message payload or title matches tool execution output markers.

```typescript
/**
 * Evaluates whether text and title indicate diagnostic or tool output.
 */
export function isToolOutputPayload(text: string, cleanTitle: string): boolean {
    const isResultPrefix = text.startsWith('[Tool Result]') || text.startsWith('Tool returned:');
    if (isResultPrefix) return true;

    const isJsonCallId = text.startsWith('{"tool_call_id":');
    if (isJsonCallId) return true;

    const hasToolTags = text.includes('<tool_response>') || text.includes('<tool_calls>');
    if (hasToolTags) return true;

    const isCodeBlock = text.startsWith('```');
    const hasOutputKeyword = cleanTitle.includes('output') || cleanTitle.includes('result');
    return isCodeBlock && hasOutputKeyword;
}
```

#### 2. `hasUserContent`
Detects whether a prompt text includes embedded user request delimiters or section headers.

```typescript
/**
 * Detects presence of explicit user request tags or section markers.
 */
export function hasUserContent(text: string): boolean {
    const hasUserTag = text.includes('<USER_REQUEST>') || text.includes('<user_request>');
    const hasUserHeader = text.includes('User Requests') || text.includes('# User Requests');
    return hasUserTag || hasUserHeader;
}
```

#### 3. `isSystemMessageDirective`
Determines if a turn represents an authentic system initialization directive rather than an embedded user request or subagent instructions.

```typescript
/**
 * Evaluates whether a payload constitutes a system-level initialization directive.
 */
export function isSystemMessageDirective(text: string, cleanTitle: string): boolean {
    const hasSystemPrefix = text.startsWith('<SYSTEM_MESSAGE>') || text.startsWith('System:');
    const hasTitlePrefix = cleanTitle.startsWith('system:');
    const isCandidate = hasSystemPrefix || hasTitlePrefix;

    if (!isCandidate) {
        return false;
    }
    if (hasUserContent(text)) {
        return false;
    }
    const isCallerPreamble = text.includes('invoked by a caller agent') || text.includes('You are');
    return !isCallerPreamble;
}
```

#### 4. `isSubagentTitle`
Checks whether a conversation title begins with subagent or worker prefix markers.

```typescript
/**
 * Checks whether the normalized conversation title indicates a subagent or worker.
 */
export function isSubagentTitle(cleanTitle: string): boolean {
    const isSubagentPrefix =
        cleanTitle.startsWith('subagent:') ||
        cleanTitle.startsWith('subagent-') ||
        cleanTitle.startsWith('subagent ') ||
        cleanTitle.startsWith('[subagent]');
    if (isSubagentPrefix) return true;

    const isWorkerPrefix =
        cleanTitle.startsWith('worker:') ||
        cleanTitle.startsWith('worker-') ||
        cleanTitle.startsWith('worker ');
    return isWorkerPrefix;
}
```

#### 5. `hasSubagentSignature`
Detects runtime subagent signatures, tool directives, and role declarations in prompt text.

```typescript
/**
 * Detects multi-agent invocation signatures and operational reminders in prompt text.
 */
export function hasSubagentSignature(text: string): boolean {
    const hasInvocationMarker =
        text.includes('invoked by a caller agent') ||
        text.includes('<subagent_reminder>') ||
        text.includes('send_message to communicate all results');
    if (hasInvocationMarker) return true;

    const rolePattern = /Role:\s*(?:Codebase Researcher|Database Debugger|QA Tester|Subagent|Worker|Architect)/i;
    return rolePattern.test(text);
}
```

#### 6. `isSubagentDirective`
Aggregates signature checks, title prefixes, and metadata indicators to classify subagent directives.
**Eliminates `metadata?.is_subagent === true` in favor of `Boolean(metadata?.is_subagent)`.**

```typescript
/**
 * Determines whether the message originates from an autonomous subagent worker.
 */
export function isSubagentDirective(
    text: string,
    cleanTitle: string,
    metadata?: Record<string, any>
): boolean {
    if (hasSubagentSignature(text)) {
        return true;
    }
    if (isSubagentTitle(cleanTitle)) {
        return true;
    }
    const isMetadataFlagged = Boolean(metadata?.is_subagent);
    const isCategorySubagent = metadata?.prompt_category === 'subagent';
    return isMetadataFlagged || isCategorySubagent;
}
```

#### 7. `isRecentChildTurn`
Evaluates whether a child subagent node falls within the chronological attachment window (default 2 hours) of a root prompt.

```typescript
/**
 * Checks if a child conversation's timestamp falls within the correlation window of a root prompt.
 */
export function isRecentChildTurn(
    nodeTime: number,
    rootTime: number,
    maxGapMs: number = 2 * 3600 * 1000
): boolean {
    if (rootTime === 0 || nodeTime === 0) {
        return false;
    }
    const timeDelta = Math.abs(nodeTime - rootTime);
    return timeDelta <= maxGapMs;
}
```

#### 8. `isHierarchicalNodeRunning`
Determines if a hierarchical node or any of its nested subagents is actively running.

```typescript
/**
 * Evaluates whether a hierarchical root node or any attached subagents are actively running.
 */
export function isHierarchicalNodeRunning(node: HierarchicalConversationNode): boolean {
    const isPrimaryRunning = getConversationStatus(node.primaryNode).isRunning;
    if (isPrimaryRunning) {
        return true;
    }
    return node.subagents.some((s) => getConversationStatus(s.primaryNode).isRunning);
}
```

#### 9. `isHierarchicalNodeQueued`
Determines if a hierarchical node or any of its nested subagents is in a queued state.

```typescript
/**
 * Evaluates whether a hierarchical root node or any attached subagents are in queued status.
 */
export function isHierarchicalNodeQueued(node: HierarchicalConversationNode): boolean {
    const isPrimaryQueued = getConversationStatus(node.primaryNode).isQueued;
    if (isPrimaryQueued) {
        return true;
    }
    return node.subagents.some((s) => getConversationStatus(s.primaryNode).isQueued);
}
```

#### 10. `compareHierarchicalNodes`
Comparator for hierarchical conversation nodes enforcing priority: (1) running state, (2) queued state, (3) descending modification time.

```typescript
/**
 * Sorts hierarchical nodes by running status first, queued status second, and newest time third.
 */
export function compareHierarchicalNodes(
    a: HierarchicalConversationNode,
    b: HierarchicalConversationNode
): number {
    const isARunning = isHierarchicalNodeRunning(a);
    const isBRunning = isHierarchicalNodeRunning(b);
    if (isARunning !== isBRunning) {
        return isARunning ? -1 : 1;
    }

    const isAQueued = isHierarchicalNodeQueued(a);
    const isBQueued = isHierarchicalNodeQueued(b);
    if (isAQueued !== isBQueued) {
        return isAQueued ? -1 : 1;
    }

    const aTime = new Date(a.primaryNode.last_modified).getTime() || 0;
    const bTime = new Date(b.primaryNode.last_modified).getTime() || 0;
    return bTime - aTime;
}
```

### 3.3 Refactoring Call Sites

- In `classifyPromptTier`:
  - Step 1:
    ```typescript
    if (isToolOutputPayload(raw, cleanTitle)) { ... }
    ```
  - Step 2:
    ```typescript
    if (isSystemMessageDirective(raw, cleanTitle)) { ... }
    ```
  - Step 3:
    ```typescript
    if (isSubagentDirective(raw, cleanTitle, metadata)) { ... }
    ```
- In `assembleConversationHierarchy`:
  - Simplify child turn attachment:
    ```typescript
    const rootTime = currentRoot ? new Date(currentRoot.primaryNode.last_modified).getTime() || 0 : 0;
    const nodeTime = new Date(conv.last_modified).getTime() || 0;
    const isChildCorrelated = currentRoot !== null && isRecentChildTurn(nodeTime, rootTime);

    if (isChildCorrelated && currentRoot) {
        currentRoot.subagents.push(node);
    } else {
        rootNodes.push(node);
    }
    ```
  - Simplify root node sorting:
    ```typescript
    return rootNodes.reverse().sort(compareHierarchicalNodes);
    ```

---

## 4. Invariants & Negative Guardrails

1. **Strictly Relative Paths:** No absolute paths (`C:\...`, `/home/...`) or `file:///` URIs anywhere in the code or specs.
2. **Implicit Boolean Invariant:** Zero instances of `=== true` or `=== false`. Use `Boolean(val)` or implicit truthy checks.
3. **No Mixed Polarity:** Never combine `&&` and `||` in the same unparenthesized conditional expression. Break complex checks into named intermediate variables.
4. **Preserve Auto-Selection Invariant:** Ensure the active conversation is retained when refreshed via SWR without flickering or losing focus.
5. **No Solo Git Execution:** Subagents and workers NEVER execute git commands. Commits are reserved exclusively for the lead orchestrator via GitMap.

---

## 5. Verification Protocol & Quality Gates

The implementation must pass the frontend production build gate:

```bash
npm run build
```

### Verification Criteria:
- **TypeScript Compilation:** Exit code `0` with zero diagnostic errors across `usePromptTreeData.ts` and `tierClassification.ts`.
- **Vite Bundle Optimization:** All assets and modules bundled successfully into `dist/`.
- **Runtime Integrity:** Zero syntax errors, circular dependencies, or undefined helper exports.
