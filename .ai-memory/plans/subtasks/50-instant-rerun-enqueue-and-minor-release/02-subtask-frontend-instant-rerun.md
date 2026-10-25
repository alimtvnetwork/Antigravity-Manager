---
plan: 50-instant-rerun-enqueue-and-minor-release
subtask: "02"
title: Frontend Instant Rerun Enqueue, Pacing Delay Removal & Immediate Tree Synchronization
domain: frontend-react-typescript
depends_on:
  - 02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/01-architecture-spec.md
  - 02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/02-component-spec.md
citations:
  architecture_spec: ../../../../02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/01-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/02-component-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/PromptTreeViewModal.tsx
  - src/components/instances/PromptTreeView/PromptTreeDetail.tsx
  - src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts
status: pending
---

# Subtask 02: Frontend Instant Rerun Enqueue, Pacing Delay Removal & Immediate Tree Synchronization

## 1. Context & Motivation

When operators click "Rerun Now" in the Prompt Tree View detail header, the prompt turns should enqueue instantaneously into the FIFO prompt queue, and the conversation tree should immediately reflect the enqueued items.

However, three frontend bottlenecks combined to cause noticeable delays and poor UX:
1. **Artificial Pacing Sleep in `dispatchRerunMessages`**:
   In `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`:
   ```typescript
   if (i < requested - 1) {
       await new Promise((r) => setTimeout(r, 300));
   }
   ```
   This inserted 300ms of dead time between every round (e.g. 1.2s for 5 rounds, 2.1s for 8 rounds). Because backend SQLite enqueue is atomic and monotonic, this artificial pacing sleep is completely unnecessary and harmful.
2. **Repeated Sequential Heavy Clipboard Operations**:
   `copyPromptWithRichImages` was awaited within the loop on every single iteration. Synthesizing base64 HTML blobs and accessing OS clipboard APIs repeatedly inside the enqueue loop introduced OS-level clipboard churn and UI thread stalls.
3. **Missing Immediate Cache Invalidation & Tree Reload**:
   In `src/components/instances/PromptTreeView/DetailHeader.tsx` (`handleRerun`), successful enqueuing did not invoke `invalidate_prompt_tree_cache` nor `loadTree(true, true)`. Consequently, the conversation tree did not show the enqueued turns until the next background poll (up to 30 seconds later).

This subtask removes the pacing delay, defers rich clipboard copy to a single post-dispatch operation, plumbs `loadTree` into `DetailHeader`, invalidates the prompt tree cache, and triggers an immediate tree refresh, while decomposing long functions into clean $\le 15$ line helpers.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Functions | Planned Modifications |
| :--- | :--- | :--- |
| `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | `dispatchRerunMessages` | 1. Remove `await new Promise((r) => setTimeout(r, 300))`.<br/>2. Move `copyPromptWithRichImages` out of loop; prime rich clipboard once at the end of the dispatch batch.<br/>3. Preserve non-fatal error tracking via `useErrorStore.getState().trackWarning`. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeaderProps`, `handleRerun`, helper functions | 1. Add `loadTree?: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>` to `DetailHeaderProps`.<br/>2. Import `isTauri` from `../../../utils/env`.<br/>3. When `result.dispatched > 0`, call `invalidate_prompt_tree_cache` / `reset_prompt_tree_cache` and `loadTree(true, true)`.<br/>4. Decompose `handleRerun` into $\le 15$ line helper functions (`resolveEffectivePromptText`, `resolveMergedMediaList`, `refreshTreeAfterEnqueue`, `handleRerunDispatchResult`). |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | `PromptTreeViewModal` JSX | Pass `loadTree={loadTree}` to `<DetailHeader ... />` at line 1097. |
| `src/components/instances/PromptTreeView/PromptTreeDetail.tsx` | `PromptTreeDetailProps` | Ensure `loadTree` seamlessly passes through `headerProps`. |
| `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts` | Unit tests for `dispatchRerunMessages` | Verify that all rounds are enqueued immediately, without delay, and that tests run significantly faster. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Remove Pacing Sleep & Defer Clipboard Copy in `usePromptTreePromptActions.ts`

In `dispatchRerunMessages`:
1. Initialize tracking variables before the loop:
   ```typescript
   let lastFormattedPrompt: string | null = null;
   let lastRerunMedia: string[] = [];
   ```
2. Inside the `for (let i = 0; i < requested; i += 1)` loop:
   - Remove the inner `copyPromptWithRichImages` try/catch block.
   - Record `lastFormattedPrompt = formattedPrompt;` and `lastRerunMedia = rerunMedia;` upon successful enqueue.
   - Remove the trailing sleep:
     ```typescript
     // REMOVE THIS:
     // if (i < requested - 1) {
     //     await new Promise((r) => setTimeout(r, 300));
     // }
     ```
3. After the loop finishes:
   ```typescript
   if (lastFormattedPrompt) {
       try {
           await copyPromptWithRichImages(
               lastFormattedPrompt,
               undefined,
               undefined,
               lastRerunMedia.length > 0 ? lastRerunMedia : undefined
           );
       } catch (clipErr: unknown) {
           useErrorStore.getState().trackWarning(clipErr, {
               source: 'usePromptTreePromptActions.dispatchRerunMessages',
               triggerAction: 'copy_rerun_rich_images_once',
           });
       }
   }
   ```

### 3.2 Step 2: Extend `DetailHeaderProps` in `DetailHeader.tsx`

1. In `src/components/instances/PromptTreeView/DetailHeader.tsx`, update `interface DetailHeaderProps`:
   ```typescript
   export interface DetailHeaderProps {
       // ... existing props
       loadTree?: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>;
   }
   ```
2. Import `isTauri`:
   ```typescript
   import { isTauri } from '../../../utils/env';
   ```

### 3.3 Step 3: Extract Focused Helper Functions ($\le 15$ Lines)

Extract the following single-responsibility helper functions above `DetailHeader`:

```typescript
function resolveEffectivePromptText(activeText: string, conv?: AgmConversationNode): string {
    const directText = activeText.trim();
    if (directText.length > 0) return directText;
    if (!conv?.prompts || conv.prompts.length === 0) {
        return (conv?.full_prompt_text || conv?.prompt_preview_200w || '').trim();
    }
    const matchedTurn = conv.selected_turn_number != null
        ? conv.prompts.find((p) => p.turn_number === conv.selected_turn_number)
        : undefined;
    if (matchedTurn?.prompt_text?.trim()) return matchedTurn.prompt_text.trim();
    const runningTurn = conv.prompts.find((p) => Boolean(p.is_running) && Boolean(p.prompt_text?.trim()));
    if (runningTurn?.prompt_text?.trim()) return runningTurn.prompt_text.trim();
    const lastTurn = [...conv.prompts].reverse().find((p) => Boolean(p.prompt_text?.trim()));
    return lastTurn?.prompt_text?.trim() || (conv.full_prompt_text || '').trim();
}

function resolveMergedMediaList(convMedia?: string[], attached?: AttachedImageState | null): string[] {
    const rawList: string[] = [
        ...(convMedia || []),
        ...(attached ? [attached.dataUri, attached.path] : []),
    ];
    return Array.from(new Set(rawList)).filter((p): p is string => Boolean(p && typeof p === 'string' && p.trim().length > 0));
}

async function refreshTreeAfterEnqueue(
    targetInstId: string,
    loadTreeFn?: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>
): Promise<void> {
    if (isTauri()) {
        try {
            await invoke('invalidate_prompt_tree_cache', { instanceId: targetInstId }).catch(async () => {
                await invoke('reset_prompt_tree_cache', { instanceId: targetInstId });
            });
        } catch (cacheErr: unknown) {
            useErrorStore.getState().trackWarning(cacheErr, {
                source: 'DetailHeader.handleRerun',
                triggerAction: 'invalidate_prompt_tree_cache',
            });
        }
    }
    if (loadTreeFn) {
        try {
            await loadTreeFn(true, true);
        } catch (loadErr: unknown) {
            useErrorStore.getState().trackWarning(loadErr, {
                source: 'DetailHeader.handleRerun',
                triggerAction: 'loadTree',
            });
        }
    }
}
```

### 3.4 Step 4: Implement `handleRerun` and `handleRerunDispatchResult`

Implement `handleRerunDispatchResult` ($\le 15$ lines):
```typescript
async function handleRerunDispatchResult(
    result: RerunDispatchResult,
    targetInstId: string,
    stateOps: {
        setRerunFlash: (flash: string | null) => void;
        setActionMsg?: (msg: string | null) => void;
        setAttachedImage: (img: AttachedImageState | null) => void;
        setRerunHistory: React.Dispatch<React.SetStateAction<string[]>>;
        rerunHistory: string[];
        loadTree?: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>;
    }
): Promise<void> {
    if (result.failed > 0) {
        reportPromptActionError(new Error(`Rerun: ${result.failed} of ${result.requested} dispatch(es) failed`), 'DetailHeader.rerunSection');
        return;
    }
    if (result.dispatched > 0) {
        const previewMsg = result.messages?.[0] ? ` ("${result.messages[0]}")` : '';
        stateOps.setRerunFlash(`Enqueued ×${result.dispatched}`);
        stateOps.setActionMsg?.(`Rerun ×${result.dispatched} enqueued in prompt queue!${previewMsg}`);
        setTimeout(() => stateOps.setRerunFlash(null), 2500);
        stateOps.setAttachedImage(null);
        if (result.messages && result.messages.length > 0) {
            stateOps.setRerunHistory((prev) => [...prev, ...result.messages!].slice(-10));
        }
        await refreshTreeAfterEnqueue(targetInstId, stateOps.loadTree);
    }
}
```

Implement `handleRerun` ($\le 15$ lines):
```typescript
const handleRerun = async (): Promise<void> => {
    if (isRerunning) return;
    handleCloseDispatchMenu();
    setIsRerunning(true);
    setRerunFlash(null);
    try {
        const targetInstId = selectedConversation.instance_id?.trim() || propInstanceId?.trim() || 'default';
        const targetConvId = dispatchConversationId(conversationTargetMode, selectedConversation.conversation_id, Boolean(selectedConversation.is_running));
        const effectivePrompt = resolveEffectivePromptText(activePromptText, selectedConversation);
        const effectiveRepo = rerunRepoPath || (selectedConversation as { repo_path?: string })?.repo_path || '.';
        const mergedMedia = resolveMergedMediaList(selectedConversation?.media_paths, attachedImage);
        const formattedPrompt = formatPromptWithAttachments(effectivePrompt, mergedMedia, { repoPath: effectiveRepo });
        const history = extractUsedNudgesFromConversation(selectedConversation, rerunHistory);
        const result = await dispatchRerunMessages({
            targetInstId, repoPath: effectiveRepo, convId: targetConvId, count: rerunCount,
            messageOption: rerunMessageOption || 'randomized', originalPrompt: formattedPrompt,
            source: 'DetailHeader.rerunSection', mediaPaths: mergedMedia, history,
        });
        await handleRerunDispatchResult(result, targetInstId, {
            setRerunFlash, setActionMsg, setAttachedImage, setRerunHistory, rerunHistory, loadTree: props.loadTree,
        });
    } catch (err: unknown) {
        reportPromptActionError(err, 'DetailHeader.rerunSection');
    } finally {
        setIsRerunning(false);
    }
};
```

### 3.5 Step 5: Plumb `loadTree` in `PromptTreeViewModal.tsx`

In `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`:
Add `loadTree={loadTree}` to the `<DetailHeader ... />` component invocation:
```tsx
<DetailHeader
    previewTab={previewTab}
    setPreviewTab={setPreviewTab}
    showAllWords={showAllWords}
    setShowAllWords={setShowAllWords}
    isTruncated={isTruncated}
    setIsDetailsModalOpen={setIsDetailsModalOpen}
    selectedConversation={selectedConversation}
    activePromptText={activePromptText}
    confirmationSuffix={confirmationSuffix}
    setConfirmationSuffix={setConfirmationSuffix}
    conversationTargetMode={conversationTargetMode}
    setConversationTargetMode={setConversationTargetMode}
    isCopiedText={false}
    setIsCopiedText={() => {}}
    isCopiedRaw={false}
    isCopied={isCopied}
    setIsCopied={setIsCopied}
    isCopiedAll={isCopiedAll}
    setIsCopiedAll={setIsCopiedAll}
    isEnqueueing={isEnqueueing}
    isResending={isResending}
    isFocusing={false}
    isCopyMenuOpen={isCopyMenuOpen}
    setIsCopyMenuOpen={setIsCopyMenuOpen}
    isSendMenuOpen={isSendMenuOpen}
    setIsSendMenuOpen={setIsSendMenuOpen}
    isEnqueueMenuOpen={isEnqueueMenuOpen}
    setIsEnqueueMenuOpen={setIsEnqueueMenuOpen}
    isExportMenuOpen={isExportMenuOpen}
    setIsExportMenuOpen={setIsExportMenuOpen}
    hasImages={hasImages}
    instanceId={instanceId || selectedConversation?.instance_id || selectedProject?.instance_id}
    instancePid={instancePid}
    instanceSeqNum={instanceSeqNum}
    instanceExeName={instanceExeName}
    instanceNameDisplay={instanceNameDisplay}
    formatDuration={formatDuration}
    elapsedSeconds={elapsedSeconds}
    onCopyAllWithRichImages={promptActions.handleCopyAllWithRichImages}
    onCopyCleanText={() => {
        const cleanText = stripImagesFromPrompt(activePromptText);
        copyToClipboard(cleanText);
    }}
    onCopyWithImages={promptActions.handleCopyWithImagesRich}
    onSaveImages={promptActions.handleSaveImages}
    onExport={promptActions.handleExport}
    onFocusIde={promptActions.handleFocusIde}
    onResend={promptActions.handleResendPrompt}
    onEnqueue={promptActions.handleEnqueuePrompt}
    rerunRepoPath={selectedProject?.repo_path || ''}
    onRenameConversation={handleRenameConversation}
    setActionMsg={setActionMsg}
    onOpenInspector={(conv) => promptActions.openInspector(conv, selectedProject?.repo_path || '')}
    loadTree={loadTree}
/>
```

---

## 4. Strict Invariants & Anti-Patterns to Avoid

- **No Artificial Sleep Loops**: Never insert `setTimeout` or `Promise((r) => setTimeout(r, ...))` between sequential database enqueue operations.
- **Positive Booleans**: Use positive identifiers (`isRerunning`, `isEnqueueing`, `isInitialLoad`, `isForce`). Never use double-negative flags like `isNotRerunning` or `disableRefresh`.
- **Zero `any`**: Strictly enforce typed interfaces. Catch clauses must use `err: unknown` and be converted via `err instanceof Error ? err : new Error(String(err))`.
- **Function Limit ($\le 15$ lines)**: Decompose all complex handlers into discrete, single-purpose helper functions.
- **Strictly Relative Paths**: Never use Windows drive letters (`C:\...`, `D:\...`) or absolute paths in error messages, comments, or documentation.
- **No Silent Errors**: Ensure every `catch` routes failures through `useErrorStore.getState().captureException` or `trackWarning`.

---

## 5. Verification Commands

1. **Frontend Unit Tests**:
   ```bash
   node scripts/run-frontend-tests.mjs rerunActions
   ```
   Must pass all tests in sub-second time (significant speedup from previous ~4s execution).
2. **Frontend Type & Build Verification**:
   ```bash
   npm run build
   ```
   Must pass with zero TypeScript compilation errors and zero lint warnings.
