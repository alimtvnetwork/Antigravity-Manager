# Subtask 01: Prompt Tree Selection Stability, Anti-Flapping & Media Preview

## 1. Subtask Metadata
- **ID**: `Task-03`
- **Slug**: `root-cause-analysis-and-end-to-end-testing`
- **Owner**: Worker 01
- **Status**: Ready for Execution
- **Exclusive Owned Files**:
  - `src/components/instances/PromptTreeView/usePromptTreeState.ts`
  - `src/components/instances/PromptTreeView/usePromptTreeData.ts`
  - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/instances/PromptTreeView/clipboard.ts`
  - `src/components/instances/PromptTreeView/markdown.tsx`

---

## 2. Action Items
1. **Memoize Action Handlers in `usePromptTreeState.ts`**:
   - Wrap state setters (`setShowAllWords`, `setIsExpandedFull`, `setIsDetailsOpen`, `setIsCopyMenuOpen`, `setIsSendMenuOpen`, `setIsRerunMenuOpen`, `setIsExportMenuOpen`, `setSearchQuery`, `setActiveTab`, `setSelectedFilter`) with `useCallback` to prevent spurious hook dependency triggers.
2. **Eliminate 30s Polling Glitch in `usePromptTreeData.ts`**:
   - Use `hasInitializedRef` so `performAutoSelection()` runs only once on initial mount/open.
   - When `loadTree()` fetches updated conversations, match by `currentConv.conversation_id` and retain `selected_turn_number` and active turn prompt text without resetting to root.
3. **Turn-Scoped Image Deduplication in `PromptInstructionView.tsx`**:
   - Implement `normalizeMediaUri` (strip `file:///`, clean leading slashes, decode URI).
   - Only extract images belonging to the active turn when a turn is selected.
   - Prevent double-counting between `media_paths` and markdown `![alt](url)` matches.
4. **Fix Image Path Resolution & Prevent Silent Hiding**:
   - In `clipboard.ts::resolveImageSrc`, fix `file:///` slice offset from 7 to 8 for Windows drive paths.
   - Support relative `.user_uploaded/...` paths using instance brain directory or `convertFileSrc`.
   - In `PromptInstructionView.tsx` and `markdown.tsx`, replace `display: none` with a visible placeholder fallback card so images are never silently swallowed.
5. **Align Turn Count in Headers**:
   - In `DetailHeader.tsx:471` and `PromptInstructionView.tsx:158`, compute turns as `selectedConversation.prompts?.length || 1` instead of `step_count`.

---

## 3. Verification Criteria
- [ ] No selection jumping or title flickering during background 30s sync.
- [ ] Attached images count matches exact prompt attachments (4, not 6).
- [ ] Image thumbnails render cleanly, and broken links show visible fallback badges instead of disappearing.
- [ ] Turn count in header says 12 turns, matching sidebar T1-T12.
