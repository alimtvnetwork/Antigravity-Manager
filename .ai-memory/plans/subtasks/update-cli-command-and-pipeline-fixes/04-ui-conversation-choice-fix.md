# Subtask Implementation Plan: UI Conversation Target Choice (Current vs New Conversation)

> **Subtask ID:** `04-ui-conversation-choice-fix`  
> **Parent Slug:** `update-cli-command-and-pipeline-fixes`  
> **Spec Reference:** [02-spec/21-app/update-cli-command-and-pipeline-fixes/02-component-spec.md](../../../02-spec/21-app/update-cli-command-and-pipeline-fixes/02-component-spec.md)  
> **AI Instructions Reference:** [02-spec/21-app/update-cli-command-and-pipeline-fixes/03-ai-instructions.md](../../../02-spec/21-app/update-cli-command-and-pipeline-fixes/03-ai-instructions.md)  
> **Target Files:**
> - `src/components/instances/PromptTreeView/DetailHeader.tsx`
> - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
> - `src/components/instances/PromptTreeView/types.ts`
> - `src/components/instances/PromptTreeView/usePromptTreeState.ts`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

The Prompt Tree View modal (`PromptTreeViewModal`) provides operators with inspection, editing, and dispatch controls for agent prompts.

Currently, clicking "Send" (Hotkey `N`) or "Queue" automatically defaults to binding the prompt to `selectedConversation.conversation_id`. Operators cannot choose whether to continue the existing conversation turn or spawn a completely clean conversation session in the target IDE workspace.

This subtask introduces a conversation target mode selector (`targetMode`: `'current'` vs `'new'`) seamlessly embedded inside the header's dark-glass action capsule, wiring it directly to the Send and Queue dispatch services.

---

## 2. Concrete Implementation Steps

### Step 1: Define `ConversationTargetMode` Type in `types.ts`
- File: `src/components/instances/PromptTreeView/types.ts`
- Add:
  ```typescript
  export type ConversationTargetMode = 'current' | 'new';
  ```

### Step 2: Manage `targetMode` State in `usePromptTreeState.ts`
- File: `src/components/instances/PromptTreeView/usePromptTreeState.ts`
- Add state variable and setter:
  ```typescript
  const [targetMode, setTargetMode] = useState<ConversationTargetMode>('current');
  ```
- Expose `targetMode` and `setTargetMode` in the returned state interface.

### Step 3: Embed Target Selector in `DetailHeader.tsx`
- File: `src/components/instances/PromptTreeView/DetailHeader.tsx`
- Add `targetMode: ConversationTargetMode` and `setTargetMode: (mode: ConversationTargetMode) => void` to `DetailHeaderProps`.
- In Capsule 2 (Execution & Workflow), insert a transparent borderless `<select>` beside `confirmationSuffix`:
  ```tsx
  {/* Conversation Target Mode Selector */}
  <div className="flex items-center px-1.5 py-0.5">
      <select
          value={targetMode}
          onChange={(e) => setTargetMode(e.target.value as ConversationTargetMode)}
          className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[95px] truncate pr-1 py-0"
          title="Target current conversation or open a new conversation on Send / Queue"
      >
          <option value="current" className="bg-white dark:bg-[#0c2438]">Target: Current</option>
          <option value="new" className="bg-white dark:bg-[#0c2438]">Target: New Conv</option>
      </select>
  </div>
  ```

### Step 4: Update Send and Queue Handlers in `usePromptTreePromptActions.ts`
- File: `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- In `handleResendPrompt`:
  - When `targetMode === 'new'`, pass `undefined` for `conversationId` into `sendPromptNow`.
  - When `targetMode === 'current'`, pass `conv?.conversation_id`.
- In `handleEnqueuePrompt`:
  - When `targetMode === 'new'`, pass `undefined` for `conversationId` into `enqueue_prompt`.
  - When `targetMode === 'current'`, pass `selectedConversation.conversation_id`.

---

## 3. UI Invariants & Styling Guidelines

- **Dark-Glass Capsule**: Must remain within the existing `rounded-full` segmented container with `divide-x` and subtle border.
- **Transparent Styling**: Must strictly use `bg-transparent border-0 focus:ring-0 focus:outline-none` on the inline `<select>` to prevent white pill-in-pill artifacts per `AGENTS.md`.
- **Positive Booleans**: Use positive polarity for any conditional styling or props (`is_current_target`).

---

## 4. Verification Checkpoints

1. Run `npm run build` to verify TypeScript types and JSX compilation.
2. Verify visual appearance: no layout displacement or broken pill borders.
3. Test switching between "Target: Current" and "Target: New Conv" in the UI.
4. Verify that selecting "Target: New Conv" dispenses prompts with `undefined` conversation ID, creating a new session.
