# Subtask 03 — Refactor Frontend Tree Conditionals

> **Subtask Code:** `Task-03`  
> **Subtask Name:** `03-refactor-frontend-tree-conditionals`  
> **Status:** `PENDING`  
> **Assigned Worker:** `Worker 03`  
> **Owned Files (Exclusive):**  
> - `src/components/instances/PromptTreeView/usePromptTreeData.ts`  
> - `src/components/instances/PromptTreeView/tierClassification.ts`  
> **Read-Only Context Files:**  
> - `02-spec/21-app/boolean-and-complex-conditions-remediation/03-migration-and-verification-spec.md`  
> - `src/components/instances/PromptTreeView/types.ts`  
> - `src/components/instances/PromptTreeView/conversationGuards.ts`  
> - `02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/`  
> **Verification Command:** `npm run build` (Exit code: 0)  

---

## 1. Intent & Scope

Refactor boolean conditionals, mixed polarity expressions, explicit boolean comparisons (`=== true`), and inline comparators in the React frontend Prompt Tree View. Extract 15 affirmative predicate helpers adhering to coding guidelines (`02-spec/02-coding-guidelines/01-cross-language/02-boolean-principles/`).

### In-Scope Deliverables:
1. **`src/components/instances/PromptTreeView/usePromptTreeData.ts`**:
   - Extract and export affirmative helpers:
     - `isDefaultInstanceAlias(instanceId?: string | null): boolean`
     - `isAllInstancesFilter(instanceId?: string | null): boolean`
     - `isInstanceCompatible(nodeInstanceId?: string | null, targetInstanceId?: string | null): boolean`
     - `isProjectRunning(project: AgmProjectTreeNode): boolean`
     - `compareProjectsPriority(a: AgmProjectTreeNode, b: AgmProjectTreeNode, pinnedProjectIds: string[]): number`
   - Refactor `performAutoSelection` to use `compareProjectsPriority` and `isProjectRunning`.
   - Refactor `loadTree` filtering using `isAllInstancesFilter` and `isInstanceCompatible`.
   - Refactor `invalidatePromptTreeCache` and `renameConversationOptimistic` to use `isDefaultInstanceAlias`.

2. **`src/components/instances/PromptTreeView/tierClassification.ts`**:
   - Extract and export affirmative helpers:
     - `isToolOutputPayload(text: string, cleanTitle: string): boolean`
     - `hasUserContent(text: string): boolean`
     - `isSystemMessageDirective(text: string, cleanTitle: string): boolean`
     - `isSubagentTitle(cleanTitle: string): boolean`
     - `hasSubagentSignature(text: string): boolean`
     - `isSubagentDirective(text: string, cleanTitle: string, metadata?: Record<string, any>): boolean`
     - `isRecentChildTurn(nodeTime: number, rootTime: number, maxGapMs?: number): boolean`
     - `isHierarchicalNodeRunning(node: HierarchicalConversationNode): boolean`
     - `isHierarchicalNodeQueued(node: HierarchicalConversationNode): boolean`
     - `compareHierarchicalNodes(a: HierarchicalConversationNode, b: HierarchicalConversationNode): number`
   - Eliminate `metadata?.is_subagent === true` (Rule 1 violation) -> replace with `Boolean(metadata?.is_subagent)`.
   - Refactor `classifyPromptTier` using decomposed stage helpers.
   - Refactor `assembleConversationHierarchy` using `isRecentChildTurn` and `compareHierarchicalNodes`.
   - Fix bug in legacy sorting where `bQueued` referenced `s.primaryNode` inappropriately.

---

## 2. Step-by-Step Execution Plan

### Step 1: Refactor `usePromptTreeData.ts`
1. Define and export `isDefaultInstanceAlias`, `isAllInstancesFilter`, `isInstanceCompatible`, `isProjectRunning`, and `compareProjectsPriority`.
2. Update `invalidatePromptTreeCache`:
   - Replace `targetInstanceId === 'default' || targetInstanceId === '__default__'` with `isDefaultInstanceAlias(targetInstanceId)`.
3. Update `renameConversationOptimistic`:
   - Replace inline ternary `targetInstanceId === 'default' || targetInstanceId === '__default__' ? ['default', '__default__'] : []` with clean branch using `isDefaultInstanceAlias`.
4. Update `performAutoSelection`:
   - Replace inline sort comparator on `targetPool` with `compareProjectsPriority(a, b, currentPinnedIds)`.
   - Replace inline running checks with `isProjectRunning(p)`.
5. Update `loadTree`:
   - Replace complex boolean chain `isTargetDefault` and `isFilterAll` with `isAllInstancesFilter` and `isInstanceCompatible`.

### Step 2: Refactor `tierClassification.ts`
1. Define and export `isToolOutputPayload`, `hasUserContent`, and `isSystemMessageDirective`.
2. Define and export `isSubagentTitle`, `hasSubagentSignature`, and `isSubagentDirective`.
   - Ensure `metadata?.is_subagent === true` is replaced with `Boolean(metadata?.is_subagent)`.
3. Define and export `isRecentChildTurn`, `isHierarchicalNodeRunning`, `isHierarchicalNodeQueued`, and `compareHierarchicalNodes`.
4. Refactor `classifyPromptTier`:
   - Delegate tool output classification to `isToolOutputPayload`.
   - Delegate system message classification to `isSystemMessageDirective`.
   - Delegate subagent classification to `isSubagentDirective`.
5. Refactor `assembleConversationHierarchy`:
   - Replace inline time math with `isRecentChildTurn`.
   - Replace 15-line inline reverse sort with `rootNodes.reverse().sort(compareHierarchicalNodes)`.

### Step 3: Verification & Quality Gate
1. Run preflight verification check:
   ```bash
   npm run build
   ```
2. Verify exit code is 0 and no TypeScript compilation errors occur.
3. Confirm clean exports and zero regressions in Prompt Tree View.

---

## 3. Negative Boundaries & Invariants

- **TOTAL BAN ON GIT COMMANDS:** Subagents and workers NEVER run `git add`, `git commit`, `git push`, `git status`, `git diff`, etc.
- **Strict Disjoint Ownership:** Only edit `src/components/instances/PromptTreeView/usePromptTreeData.ts` and `src/components/instances/PromptTreeView/tierClassification.ts`. Never touch backend Rust files or other components.
- **No Absolute Paths:** Use strict relative repository paths only.
- **No Explicit `=== true`:** All boolean checks must be implicit or use `Boolean(...)`.
- **Preserve SWR and State Invariants:** Keep `promptTreeSwrCache` semantics and optimistic renaming fully functional.

---

## 4. Verification Evidence Contract

- **Target Command:** `npm run build`
- **Expected Result:** Exit code 0, clean Vite build output.
- **AST Check:** No unused imports or variables, zero linter warnings.
