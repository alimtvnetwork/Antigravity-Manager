# Subtask Plan 02: Prompt Actions & Rerun Enqueue Implementation

**Identifier**: `02-subtask-prompt-actions-and-rerun`  
**Parent Plan**: `.ai-memory/plans/pending/49-image-attachment-format-and-rerun-enqueue.md`  
**Component Spec**: `02-spec/21-app/49-image-attachment-format-and-rerun-enqueue/02-component-spec.md`  
**Assigned Worker**: Worker 02  
**Target Files**:
- `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`

---

## 1. Objectives & Scope

1. **Replace Legacy Image Formatting**:
   - Transition from manual Markdown formatting (`workspaceImageMarkdown`, `rewritePromptImageTargets`) to `formatPromptWithAttachments` from `src/utils/promptAttachment.ts`.
   - Ensure all prompt actions (Send Prompt Now, Enqueue Prompt, Rerun) produce canonical bracketed attachment blocks:
     ```text
     Attachments:
     1. @[C:\path\to\image1.png]
     2. @[C:\path\to\image2.png]
     ```
2. **Enforce Rerun Always Enqueues**:
   - In `usePromptTreePromptActions.ts` (`dispatchRerunMessages`), change Round 0 dispatch from `sendPromptNow` to `enqueuePrompt`. Every round (0 to N-1) must enqueue sequentially into SQLite FIFO queue.
   - In `PromptInstructionView.tsx` (`executeDirectRerun`), replace `sendPromptNow` with `enqueuePrompt`.
   - Completely eliminate `(sendPromptNow as any)` and `(enqueuePrompt as any)` type escapes.
3. **UI/UX State and Flash Updates**:
   - Update `DetailHeader.tsx` flash notification from `"Sent ×N"` to `"Enqueued ×N"`.
   - Update `PromptInstructionView.tsx` action message to `"Rerun enqueued successfully!"`.
4. **Bracketed Reference Parsing**:
   - Update `extractImagesFromPrompt` in `PromptInstructionView.tsx` to detect and parse `@[...]` bracketed image paths so image thumbnails render properly in the UI.

---

## 2. Implementation Steps

### Phase 1: Refactor `usePromptTreePromptActions.ts`

- [ ] **Step 1.1: Import canonical attachment formatter**
  ```typescript
  import { formatPromptWithAttachments } from '../../../utils/promptAttachment';
  ```

- [ ] **Step 1.2: Deprecate legacy helpers with backward compatibility**
  Retain `workspaceImageMarkdown` and `rewritePromptImageTargets` as deprecated exports to prevent breaking external modules, delegating internally or marking with `@deprecated`.

- [ ] **Step 1.3: Update `handleResendPrompt` (Send Prompt Now)**
  Replace manual image block concatenation and `rewritePromptImageTargets` with `formatPromptWithAttachments`:
  ```typescript
  if (mediaList.length > 0) {
      promptContent = formatPromptWithAttachments(promptContent, mediaList, repoPath);
  }
  await sendPromptNow(targetInstId, repoPath, promptContent, targetConvId, mediaList);
  ```

- [ ] **Step 1.4: Update `handleEnqueuePrompt` (Enqueue Prompt)**
  Format prompt content before calling `enqueuePrompt`:
  ```typescript
  if (mediaList.length > 0) {
      promptContent = formatPromptWithAttachments(promptContent, mediaList, repoPath);
  }
  await enqueuePrompt(targetInstId, repoPath, promptContent, targetConvId, undefined, mediaList);
  ```

- [ ] **Step 1.5: Enforce Rerun Enqueue Invariant in `dispatchRerunMessages`**
  Modify loop inside `dispatchRerunMessages`:
  ```typescript
  // Before:
  if (i === 0) {
      await (sendPromptNow as any)(ctx.targetInstId, ctx.repoPath, rewrittenPrompt, ctx.convId, rerunMedia.length > 0 ? rerunMedia : undefined);
  } else {
      await (enqueuePrompt as any)(ctx.targetInstId, ctx.repoPath, rewrittenPrompt, ctx.convId, undefined, rerunMedia.length > 0 ? rerunMedia : undefined);
  }

  // After: RERUN ALWAYS ENQUEUES
  const formattedPrompt = formatPromptWithAttachments(rewrittenPrompt, rerunMedia, ctx.repoPath);
  await enqueuePrompt(
      ctx.targetInstId,
      ctx.repoPath,
      formattedPrompt,
      ctx.convId,
      undefined,
      rerunMedia.length > 0 ? rerunMedia : undefined
  );
  ```
  Ensure all `as any` casts are removed.

---

### Phase 2: Refactor `DetailHeader.tsx`

- [ ] **Step 2.1: Import `formatPromptWithAttachments`**
  ```typescript
  import { formatPromptWithAttachments } from '../../../utils/promptAttachment';
  ```

- [ ] **Step 2.2: Refactor `handleRerun` formatting logic**
  Replace legacy concatenation logic:
  ```typescript
  const rerunMediaList: string[] = Array.from(new Set([
      ...existingMedia,
      ...(attachedImage ? [attachedImage.dataUri, attachedImage.path] : []),
  ])).filter((p): p is string => Boolean(p && typeof p === 'string' && p.trim().length > 0));

  const promptWithAttachments = formatPromptWithAttachments(
      originalPrompt,
      rerunMediaList,
      effectiveRepoPath
  );
  ```
  Pass `promptWithAttachments` into `dispatchRerunMessages`.

- [ ] **Step 2.3: Update flash notification and UX text**
  ```typescript
  // Before:
  setRerunFlash(`Sent ×${result.dispatched}`);
  setActionMsg?.(`Rerun ×${result.dispatched} dispatched successfully!${previewMsg}`);

  // After:
  setRerunFlash(`Enqueued ×${result.dispatched}`);
  setActionMsg?.(`Rerun ×${result.dispatched} enqueued in prompt queue!${previewMsg}`);
  ```
  Update button `aria-label` to:
  ```typescript
  aria-label={`Rerun ×${rerunCount} Now: Enqueues ${rerunMessageOption} completion or release nudge into FIFO queue`}
  ```

---

### Phase 3: Refactor `PromptInstructionView.tsx`

- [ ] **Step 3.1: Update `executeDirectRerun` to enqueue**
  Replace `sendPromptNow` with `enqueuePrompt`:
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

- [ ] **Step 3.2: Update `extractImagesFromPrompt` for bracketed paths**
  Add parsing support for `@[...]` bracketed image attachments:
  ```typescript
  const bracketedRegex = /@\[([^\]]+)\]/g;
  let bracketMatch: RegExpExecArray | null;
  while ((bracketMatch = bracketedRegex.exec(text)) !== null) {
      const rawPath = bracketMatch[1]?.trim();
      if (!rawPath) continue;

      const isImage = /\.(png|jpe?g|webp|gif|svg|bmp|ico)$/i.test(rawPath) ||
          rawPath.includes('.user_uploaded') ||
          rawPath.startsWith('data:image/');

      if (isImage) {
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

### Phase 4: Verification & Quality Gates

- [ ] **Step 4.1: Run targeted rerun unit tests**
  ```bash
  npm test -- rerunActions
  ```
  Verify that all rerun rounds invoke `enqueue_prompt` and 0 calls to `send_prompt_now` occur.

- [ ] **Step 4.2: Frontend build and type safety check**
  ```bash
  npm run build
  ```
  Ensure zero TypeScript compiler errors and zero lint regressions.

- [ ] **Step 4.3: Verify repository rules**
  - Positive boolean naming verified.
  - Zero `any` in updated files.
  - Strictly relative paths in all docs and comments.
