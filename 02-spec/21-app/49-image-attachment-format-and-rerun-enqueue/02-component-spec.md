# Component Specification: Prompt Actions & Rerun Enqueue Integration

## 1. Overview & Architectural Role

This specification details the component-level refactoring for task `49-image-attachment-format-and-rerun-enqueue`. It governs the prompt action dispatch lifecycle across the Antigravity Manager desktop frontend:
1. **Attachment Normalization**: Unifies image attachment syntax across all prompt actions (Send Now, Enqueue, Rerun) by replacing fragmented legacy Markdown image rewrites (`workspaceImageMarkdown`, `rewritePromptImageTargets`) with the canonical attachment formatter (`formatPromptWithAttachments`).
2. **The Rerun Enqueue Invariant**: Mandates that the **Rerun** action must **always enqueue** prompts into the SQLite FIFO prompt queue (`enqueuePrompt`), regardless of round index. Rerun must **never directly execute or preempt** active tasks via `sendPromptNow`.
3. **UI/UX State Synchronization**: Aligns visual feedback, tooltips, and flash notifications across headers and instruction panels (e.g., transitioning from `"Sent ×N"` to `"Enqueued ×N"`).
4. **Bracketed Reference Parsing**: Upgrades frontend prompt inspection and image extraction (`extractImagesFromPrompt`) to parse canonical bracketed references (`@[path/to/image.png]`) alongside legacy Markdown image links.

### Target Source Files
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` (Hook & dispatch engine)
- `src/components/instances/PromptTreeView/DetailHeader.tsx` (Header dispatch actions & rerun capsule)
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx` (Instruction view direct rerun & image parsing)

---

## 2. Invariants & Repository Coding Standards

1. **Rerun Always Enqueues**:
   - Every round from `0` to `N-1` inside `dispatchRerunMessages` and `executeDirectRerun` MUST invoke `enqueuePrompt`.
   - Zero calls to `sendPromptNow` are permitted in rerun execution paths.
2. **Zero `any` Discipline**:
   - All function signatures, callback arguments, and IPC wrappers must be strictly typed.
   - Eliminate all legacy `(sendPromptNow as any)` and `(enqueuePrompt as any)` type escapes.
3. **Positive Boolean Naming**:
   - Variables and flags must use positive polarity (`isEnqueued`, `hasMedia`, `shouldEnqueue`, `isDispatching`).
   - Inverted or negative boolean prefixes (`isNotRunning`, `noMedia`, `unSent`) are strictly forbidden.
4. **Strictly Relative Paths**:
   - All documentation, comments, and file references within code and specs must use repository-relative paths (`src/...`, `02-spec/...`, `.ai-memory/...`).

---

## 3. Data Flow Architecture

```
[ User Interaction ]
   │
   ├─► Click "Send Prompt Now" ──────┐
   │                                 ▼
   │                    [ formatPromptWithAttachments ]
   │                    (promptText, mediaPaths, repoPath)
   │                                 │
   │                                 ├─► Canonical Markdown & Bracketed List
   │                                 │   "Attachments:\n1. @[<normalized-path>]"
   │                                 │
   │                                 ├─► sendPromptNow() (Preempts active execution)
   │                                 └─► copyPromptWithRichImages() (Clipboard sync)
   │
   ├─► Click "Enqueue Prompt" ───────┐
   │                                 ▼
   │                    [ formatPromptWithAttachments ]
   │                    (promptText, mediaPaths, repoPath)
   │                                 │
   │                                 ├─► enqueuePrompt() ──► [ SQLite active_prompts FIFO ]
   │                                 └─► invalidate_prompt_tree_cache()
   │
   └─► Click "Rerun ×N" ─────────────┐
                                     ▼
                        [ dispatchRerunMessages ]
                        Loop: Round 1 to N
                                     │
                                     ├─► buildRerunPrompt() (Nudge message selection)
                                     ├─► formatPromptWithAttachments()
                                     ├─► ALWAYS: enqueuePrompt() (Round 1..N FIFO)
                                     │   (Zero sendPromptNow calls)
                                     ├─► copyPromptWithRichImages()
                                     └─► auditRecord() & UI Flash ("Enqueued ×N")
```

---

## 4. Component Specification Details

### 4.1. `usePromptTreePromptActions.ts`

#### A. Import and Dependency Modernization
- **Import Canonical Helper**:
  ```typescript
  import { formatPromptWithAttachments } from '../../../utils/promptAttachment';
  ```
- **Deprecate Legacy Formatting Functions**:
  Mark `workspaceImageMarkdown` and `rewritePromptImageTargets` as deprecated to preserve backward compatibility for external callers, while re-routing their internal implementation to `formatPromptWithAttachments`.
  ```typescript
  /** @deprecated Use formatPromptWithAttachments from 'src/utils/promptAttachment' */
  export function workspaceImageMarkdown(absPath: string, repoPath: string, index: number): string;

  /** @deprecated Use formatPromptWithAttachments from 'src/utils/promptAttachment' */
  export function rewritePromptImageTargets(prompt: string, repoPath: string): string;
  ```

#### B. `handleResendPrompt` (Send Prompt Now)
- **Current Problem**: Directly concatenated legacy Markdown `![Attached Image N](<relative>)` and called `rewritePromptImageTargets`.
- **Target Behavior**:
  1. Consolidate media paths from the active turn, selected conversation, turn history, and sub-runs into `mediaList: string[]`.
  2. Normalize and format prompt text using `formatPromptWithAttachments(promptContent, mediaList, repoPath)`.
  3. Dispatch immediately to the target instance via `sendPromptNow(targetInstId, repoPath, promptContent, targetConvId, mediaList)`.
  4. Prime clipboard with rich formatted text.
  5. Audit log and handle errors via `persistClientCrash` and `useErrorStore.getState().captureException`.

#### C. `handleEnqueuePrompt` (Enqueue to Prompt Queue)
- **Target Behavior**:
  1. Consolidate media paths into `mediaList: string[]`.
  2. Format the prompt payload via `formatPromptWithAttachments(promptContent, mediaList, repoPath)`.
  3. Call `enqueuePrompt(targetInstId, repoPath, promptContent, targetConvId, undefined, mediaList)` without type casting.
  4. Invalidate prompt tree cache via `invalidate_prompt_tree_cache` IPC to reflect the newly enqueued turn.
  5. Show affirmative user feedback ("Prompt enqueued into FIFO queue").

#### D. `dispatchRerunMessages` (Rerun Dispatch Engine)
- **Current Behavior**:
  ```typescript
  // Legacy code:
  if (i === 0) {
      await (sendPromptNow as any)(...); // Preempts active task!
  } else {
      await (enqueuePrompt as any)(...);
  }
  ```
- **New Behavior (Rerun Always Enqueues)**:
  - Both round `0` and rounds `1..N-1` MUST call `enqueuePrompt`.
  - Signature and call structure:
    ```typescript
    // Format rerun prompt with attachments:
    const message = richPrompt || rawMessage;
    const formattedPrompt = formatPromptWithAttachments(message, rerunMedia, ctx.repoPath);

    // Enforce RERUN ALWAYS ENQUEUES:
    await enqueuePrompt(
        ctx.targetInstId,
        ctx.repoPath,
        formattedPrompt,
        ctx.convId,
        undefined,
        rerunMedia.length > 0 ? rerunMedia : undefined
    );
    ```
  - Eliminate all `as any` casts. Ensure typed parameters match `enqueuePrompt` signature.
  - Increment `result.dispatched += 1` and append to `result.messages`.
  - Maintain audit logging with action `prompt_tree_rerun_dispatched` and result `ok`.

---

### 4.2. `DetailHeader.tsx`

#### A. Attachment Integration in `handleRerun`
- **Location**: `handleRerun` inside `src/components/instances/PromptTreeView/DetailHeader.tsx`.
- **Target Behavior**:
  1. Extract `originalPrompt` using existing fallback ladder (active turn -> running turn -> last turn with text -> full prompt preview).
  2. Resolve effective repository path `effectiveRepoPath`.
  3. Combine existing conversation media paths (`selectedConversation.media_paths`) and any newly staged `attachedImage` into `rerunMediaList`.
  4. Replace legacy manual string concatenations (`workspaceImageMarkdown`) and `rewritePromptImageTargets` with a single call to:
     ```typescript
     const formattedPrompt = formatPromptWithAttachments(
         originalPrompt,
         rerunMediaList,
         effectiveRepoPath
     );
     ```
  5. Pass `formattedPrompt` as `originalPrompt` into `dispatchRerunMessages`.

#### B. Flash Banner & Tooltip Modernization
- **Flash Banner Update**:
  - Legacy: `setRerunFlash('Sent ×' + result.dispatched);`
  - Target: `setRerunFlash('Enqueued ×' + result.dispatched);`
- **Action Message Update**:
  - Legacy: `setActionMsg?.('Rerun ×' + result.dispatched + ' dispatched successfully!' + previewMsg);`
  - Target: `setActionMsg?.('Rerun ×' + result.dispatched + ' enqueued in queue!' + previewMsg);`
- **Tooltip & Accessibility Labels**:
  - Update `aria-label` and `title` on rerun buttons and capsules:
    `aria-label={`Rerun ×${rerunCount} Now: Enqueues ${rerunMessageOption} completion or release nudge into FIFO queue`}`

---

### 4.3. `PromptInstructionView.tsx`

#### A. Direct Rerun Alignment in `executeDirectRerun`
- **Current Behavior**:
  ```typescript
  await sendPromptNow(targetInstId, repoPath, content, targetConvId, mediaList.length > 0 ? mediaList : undefined);
  setActionMsg?.('Rerun dispatched successfully!');
  ```
- **Target Behavior**:
  - Change `sendPromptNow` to `enqueuePrompt`:
    ```typescript
    const formattedContent = formatPromptWithAttachments(content, mediaList, repoPath);
    await enqueuePrompt(
        targetInstId,
        repoPath,
        formattedContent,
        targetConvId,
        undefined,
        mediaList.length > 0 ? mediaList : undefined
    );
    setActionMsg?.('Rerun enqueued successfully!');
    ```
  - Ensure prompt tree cache is invalidated or refreshed to display the new enqueued item.

#### B. Bracketed Path Parsing in `extractImagesFromPrompt`
- **Location**: Function `extractImagesFromPrompt(text: string, conv: AgmConversationNode): ExtractedImage[]`.
- **Target Behavior**:
  In addition to existing Markdown `![alt](url)`, HTML `<img>`, and data URIs, parse bracketed attachment references formatted like:
  ```text
  @[C:\Users\...\media_123.png]
  1. @[C:\Users\...\media_123.png]
  ```
- **Implementation Logic**:
  ```typescript
  // 5. From bracketed image attachments @[...]
  const bracketedRegex = /@\[([^\]]+)\]/g;
  let bracketMatch: RegExpExecArray | null;
  while ((bracketMatch = bracketedRegex.exec(text)) !== null) {
      const rawPath = bracketMatch[1]?.trim();
      if (!rawPath) continue;

      const isImageFile = /\.(png|jpe?g|webp|gif|svg|bmp|ico)$/i.test(rawPath) ||
          rawPath.includes('.user_uploaded') ||
          rawPath.startsWith('data:image/');

      if (isImageFile) {
          const key = normalizeMediaUri(rawPath) || rawPath;
          if (!seen.has(key)) {
              seen.add(key);
              const fileName = rawPath.split(/[/\\]/).pop() || 'Attached Image';
              list.push({ src: rawPath, alt: fileName });
          }
      }
  }
  ```

---

## 5. Error Handling & Crash Persistence

In accordance with repository guidelines ("No Silent Errors"):
1. Any IPC dispatch failure (`enqueuePrompt`, `invalidate_prompt_tree_cache`) must be recorded in `error-store` and persisted via `persistClientCrash`.
2. Best-effort operations (such as copying rich images to clipboard or cache invalidation) must use `trackWarning` with explicit trigger contexts rather than bare `catch {}`.
3. If an enqueue operation fails, user-facing notifications must state the failure clearly rather than misleadingly displaying success.

---

## 6. Verification & Acceptance Criteria

| Checkpoint | Verification Method | Expected Outcome |
| :--- | :--- | :--- |
| **Rerun Always Enqueues** | Unit test `rerunActions.test.ts` | 100% of rerun dispatches call `enqueue_prompt`. 0 calls to `send_prompt_now`. |
| **Zero `any`** | `npm run build` / TypeScript check | No `as any` type casts in `usePromptTreePromptActions.ts`. |
| **Attachment Syntax** | Inspection of enqueued prompt in SQLite | Enqueued prompt ends with `Attachments:\n1. @[<path>]` instead of legacy Markdown syntax. |
| **UI Feedback** | Frontend UI testing | Flash notification displays `Enqueued ×N` upon clicking Rerun. |
| **Bracket Parsing** | `extractImagesFromPrompt` unit tests | Image previews in `PromptInstructionView` correctly populate from `@[<path>]` references. |
