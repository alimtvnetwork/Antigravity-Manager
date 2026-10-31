# Component Specification: Prompt Tree Stability, Image Previews & Turn Calculation

## 1. Scope & Objective
This specification governs the concrete remediation of:
1. `src/components/instances/PromptTreeView/usePromptTreeState.ts`
2. `src/components/instances/PromptTreeView/usePromptTreeData.ts`
3. `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
4. `src/components/instances/PromptTreeView/DetailHeader.tsx`
5. `src/components/instances/PromptTreeView/clipboard.ts`
6. `src/components/instances/PromptTreeView/markdown.tsx`

---

## 2. Selection Stability & Anti-Flapping Engine

### 2.1 Problem Formulation
When `Sync: 30s` fires, `usePromptTreeData.ts` runs `loadTree()`.
Because `usePromptTreeState.ts` exposed inline function references, the modal open `useEffect` had an unstable dependency array, triggering `performAutoSelection()` repeatedly.
Furthermore, `loadTree()` merged fresh conversations by setting `activePromptText = updatedConv.prompt_text`, which overwrote the user's selected turn with the root conversation prompt and reset `selected_turn_number`.

### 2.2 Detailed Remediation Protocol
1. **Memoize State Actions**:
   In `usePromptTreeState.ts`, wrap all setter handlers in `useCallback` with stable dependencies:
   - `setShowAllWords`, `setIsExpandedFull`, `setIsDetailsOpen`, `setIsCopyMenuOpen`, `setIsSendMenuOpen`, `setIsRerunMenuOpen`, `setIsExportMenuOpen`, `setSearchQuery`, `setActiveTab`, `setSelectedFilter`.
2. **Stable Selection Retention**:
   In `usePromptTreeData.ts`, retain the active turn state across sync cycles:
   ```typescript
   if (selectedConversationRef.current) {
     const currentId = selectedConversationRef.current.conversation_id;
     const currentTurn = selectedConversationRef.current.selected_turn_number;
     const updated = newProjects.flatMap(p => p.conversations).find(c => c.conversation_id === currentId);
     if (updated) {
       if (currentTurn != null && updated.prompts?.length) {
         const matchingPrompt = updated.prompts.find(p => p.turn_number === currentTurn);
         if (matchingPrompt) {
           updated.selected_turn_number = currentTurn;
           updated.activePromptText = matchingPrompt.prompt_text;
         }
       }
       setSelectedConversation(updated);
     }
   }
   ```
3. **Decouple Initial Auto-Selection from Sync**:
   Use `hasInitializedRef = useRef(false)` so `performAutoSelection()` executes ONLY on the first mount or when the modal opens, never on background interval updates.

---

## 3. Image Extraction, Deduplication & Rendering Pipeline

### 3.1 Problem Formulation
- The chat prompt has 4 attached images.
- Prompt Tree View displayed `Attached Images (6)` because:
  - It combined `media_paths` across all turns when viewing the conversation.
  - Deduping failed because raw file paths (e.g. `C:\...\image.png`) did not match markdown URLs (`file:///C:/...` or `.user_uploaded/...`).
  - Images failed to display because `clipboard.ts::resolveImageSrc` sliced 7 characters instead of 8 for `file:///` on Windows, leaving `/C:/...` which corrupted Tauri asset URLs.
  - `<img onError>` set `style.display = 'none'`, completely hiding the image without showing the user what went wrong.

### 3.2 Detailed Remediation Protocol
1. **Path Normalization Helper**:
   Define `normalizeMediaUri(uri: string): string`:
   - Strip leading `file:///` or `file://`.
   - On Windows, strip any leading slash before the drive letter (e.g. `/C:/` -> `C:/`).
   - Decode percent-encoding (`decodeURIComponent`).
   - Replace backslashes with forward slashes for unified key matching.
2. **Turn-Scoped Image Extraction**:
   In `PromptInstructionView.tsx::extractImagesFromPrompt`:
   - If `conv.selected_turn_number != null`, ONLY extract images belonging to that specific turn (`prompt.media_paths`).
   - If viewing root conversation, extract images only from the primary/first user turn.
   - Use `normalizeMediaUri` for `seen.has(key)` to prevent duplicate markdown + media_path entries.
3. **Interactive Thumbnail Cards with Visible Fallback**:
   In `PromptInstructionView.tsx`:
   - Replace the plain text line with a thumbnail card grid.
   - For each image, render an interactive thumbnail with `convertFileSrc(resolvedPath)`.
   - On `onError`: DO NOT hide the element. Render an explicit dark-glass fallback card:
     `[📷 Image: <basename> (Click to open / inspect)]`
   - On click, open the full-resolution preview modal or launch default system image viewer.

---

## 4. Turn Count Alignment & Timestamp Disambiguation

### 4.1 Turn Count Alignment
- In `InstanceCardProjects.tsx`, `DetailHeader.tsx`, and `PromptInstructionView.tsx`:
  - Replace `selectedConversation.step_count` fallback with:
    `const totalTurns = selectedConversation.prompts?.length || 1;`
  - Ensure the sidebar count matches the header pill badge (`12 turns` matches `T1..T12`).

### 4.2 Timestamp & Duration Disambiguation
- In `PromptInstructionView.tsx`:
  - Render dual badges with crystal-clear semantics:
    - **Session Span**: `16h 18m (Turn 1 @ 01:23 AM -> Turn 12 @ 05:41 PM)`
    - **Current Turn Active Runtime**: `PID: 32 (45s elapsed)`
