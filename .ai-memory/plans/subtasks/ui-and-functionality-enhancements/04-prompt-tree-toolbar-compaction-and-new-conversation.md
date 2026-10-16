---
plan: ui-and-functionality-enhancements
subtask: "04"
title: PromptTreeView Toolbar Compaction, Unified Send/Enqueue Split Dropdown, and New Conversation Creation Flow
domain: frontend-ui
depends_on: 03-resizable-email-column-ui.md
citations:
  architecture_spec: ../../../../02-spec/21-app/ui-and-functionality-enhancements/01-architecture-and-ui-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/components/instances/PromptTreeView/DetailHeader.tsx
  - src/components/instances/PromptTreeView/usePromptTreeState.ts
  - src/components/instances/PromptTreeView/usePromptTreePromptActions.ts
  - src/components/instances/PromptTreeView/PromptTreeViewModal.tsx
status: pending
---

# Subtask 04: PromptTreeView Toolbar Compaction, Unified Send/Enqueue Split Dropdown, and New Conversation Creation Flow

## 1. Context & Motivation
In `PromptTreeView`, the action toolbar currently renders three disconnected floating capsules (`[Copy All v]`, `[Send N v]`, `[Enqueue v]`), violating the house design rule that adjacent toolbar controls must be wrapped into contiguous segmented pill capsules.
Furthermore:
1. `Send` and `Enqueue` represent two dispatch variations of the same operational intent (immediate vs background FIFO queue). Having two large separate capsules wastes valuable header space. They must be merged into a single primary split button with a comprehensive options dropdown.
2. Users frequently need to send a prompt to a **brand new conversation** rather than appending to an existing thread, with the ability to assign a meaningful conversation title before dispatching. Currently, only a basic toggle exists without a title input, leading to unlabeled conversations in the IDE.

---

## 2. Target Files & Symbols Breakdown

| Target File | Symbols / Components | Specific Modifications |
| :--- | :--- | :--- |
| `src/components/instances/PromptTreeView/usePromptTreeState.ts` | `usePromptTreeState`, `PromptTreeState` | Add `newConversationTitle: string` and `setNewConversationTitle: (title: string) => void`. |
| `src/components/instances/PromptTreeView/DetailHeader.tsx` | `DetailHeader`, `DetailHeaderProps` | 1. Accept `newConversationTitle` and `setNewConversationTitle` props.<br/>2. Merge 3 separate capsules into 1 contiguous segmented dark-glass group.<br/>3. Combine Send and Enqueue into a primary split button with dropdown flyout.<br/>4. Render custom title input when `conversationTargetMode === 'new'`. |
| `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` | `usePromptTreePromptActions`, `handleResendPrompt`, `handleEnqueuePrompt` | Support `newConversationTitle` during prompt dispatch, pass `targetConvId = undefined` for new conversations, format title in status message. |
| `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` | `PromptTreeViewModal` | Wire `newConversationTitle` and `setNewConversationTitle` from state into `<DetailHeader />`. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Add New Conversation Title State in `usePromptTreeState.ts`
1. In `usePromptTreeState.ts`, add the state variable:
   ```tsx
   const [newConversationTitle, setNewConversationTitle] = useState<string>('');
   ```
2. Export `newConversationTitle` and `setNewConversationTitle` in the returned state object:
   ```tsx
   return {
       ...
       newConversationTitle,
       setNewConversationTitle,
   };
   ```
3. Update `PromptTreeState` interface to include:
   ```tsx
   newConversationTitle: string;
   setNewConversationTitle: (title: string) => void;
   ```

### 3.2 Step 2: Update `DetailHeaderProps` and Toolbar Layout in `DetailHeader.tsx`
1. In `DetailHeader.tsx`, add to `DetailHeaderProps`:
   ```tsx
   newConversationTitle?: string;
   setNewConversationTitle?: (title: string) => void;
   ```
2. Replace the three disconnected floating capsules with a single contiguous segmented dark-glass capsule:
   ```tsx
   {/* Right: Consolidated Segmented Dark-Glass Action Capsule */}
   <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0">
       {/* Segment 1: Copy All Split Button */}
       <button
           type="button"
           onClick={handleCopyAll}
           className="flex items-center gap-1.5 px-3 py-1 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
           title="Copy full prompt text and images"
       >
           {isCopiedAllActive ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3 text-slate-500 dark:text-slate-400" />}
           <span>{isCopiedAllActive ? 'Copied!' : 'Copy All'}</span>
       </button>
       <button
           type="button"
           onClick={() => setIsCopyMenuOpen((prev) => !prev)}
           className="flex items-center justify-center px-1.5 py-1 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all cursor-pointer"
           title="Copy options"
       >
           <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform", isCopyMenuOpen && "rotate-180")} />
       </button>

       {/* Segment 2: Unified Primary Send / Enqueue Split Button */}
       <button
           type="button"
           onClick={handleSendPrimary}
           disabled={isResending}
           className="flex items-center gap-1.5 px-3 py-1 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100/80 dark:hover:bg-[#40285e] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50"
           title="Immediately inject prompt to running instance (Hotkey: N)"
       >
           <RotateCw className={cn('w-3 h-3 text-emerald-500', isResending && 'animate-spin')} />
           <span>Send</span>
           <kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[8.5px] font-mono font-bold">N</kbd>
       </button>
       <button
           type="button"
           onClick={() => setIsSendMenuOpen((prev) => !prev)}
           className="flex items-center justify-center px-1.5 py-1 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-[11px] transition-all cursor-pointer"
           title="Dispatch & queue options"
       >
           <ChevronDown className={cn("w-3.5 h-3.5 text-slate-400 transition-transform", isSendMenuOpen && "rotate-180")} />
       </button>
   </div>
   ```

3. Update the dropdown flyout menu (`isSendMenuOpen`) to include:
   - Primary `Send Now` action.
   - Secondary `Enqueue to FIFO Queue` action.
   - Divider line.
   - Destination Conversation mode toggle (`Current Conv` vs `New Conv`).
   - Conditional inline text input for `New Conversation Title`:
     ```tsx
     {conversationTargetMode === 'new' && (
         <div className="px-2.5 py-1.5 space-y-1 bg-slate-50 dark:bg-[#1a1228] rounded-lg border border-slate-200/60 dark:border-purple-500/20">
             <label className="text-[10px] font-medium text-slate-500 dark:text-slate-400 block">
                 New Conversation Title:
             </label>
             <input
                 type="text"
                 value={newConversationTitle || ''}
                 onChange={(e) => setNewConversationTitle?.(e.target.value)}
                 placeholder="Enter title (e.g. Feature X)..."
                 className="w-full px-2 py-1 text-xs rounded bg-white dark:bg-[#090611] border border-slate-300 dark:border-slate-700 text-slate-900 dark:text-slate-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
                 maxLength={60}
             />
         </div>
     )}
     ```
   - Confirmation Suffix dropdown (`None`, `Is it done?`, `Is it released?`, etc.).
   - Focus IDE Window trigger.

### 3.3 Step 3: Handle Title & New Conversation in `usePromptTreePromptActions.ts`
1. In `usePromptTreePromptActions.ts`, destructure `newConversationTitle` from `state`.
2. In `handleResendPrompt`:
   ```tsx
   const isNewConv = conversationTargetMode === 'new';
   const targetConvId = isNewConv ? undefined : conv?.conversation_id;

   let promptContent = (!selectedConversation && conv)
       ? (conv.prompt_preview_200w || '')
       : (editedPromptText.trim() || activePromptTextRef.current || activePromptText || conv?.prompt_preview_200w || '');

   // If creating a new conversation and a custom title was provided, prepend title metadata
   if (isNewConv && newConversationTitle.trim()) {
       promptContent = `<!-- title: ${newConversationTitle.trim()} -->\n${promptContent}`;
   }

   if (confirmationSuffix && confirmationSuffix !== 'None (Send as is)') {
       promptContent = `${promptContent}\n\n${confirmationSuffix}`;
   }
   ```
3. Update the feedback message on success:
   ```tsx
   if (dispatchOk) {
       const msg = isNewConv && newConversationTitle.trim()
           ? `New conversation "${newConversationTitle.trim()}" dispatched to IDE!`
           : "Prompt dispatched to IDE & copied to clipboard!";
       setActionMsg(msg);
       setTimeout(() => setActionMsg(null), 3000);
   }
   ```

### 3.4 Step 4: Pass Props in `PromptTreeViewModal.tsx`
1. Forward `newConversationTitle` and `setNewConversationTitle` from `state` to `<DetailHeader />`.

---

## 4. Visual & DOM Invariants
- The toolbar controls MUST render as a single contiguous pill capsule with `rounded-full` and `divide-x`.
- Floating loose buttons on the right side of the header are strictly forbidden.
- The New Conversation title input MUST only be rendered when `conversationTargetMode === 'new'`.
- All inline dropdowns and inputs MUST follow dark-glass styling conventions (`bg-transparent border-0` or dark card styling).
- Fallbacks for clipboard, Tauri invoke, and state errors must route to `useErrorStore.getState().captureException()` or `trackWarning()`.

---

## 5. Verification Checklist
- [ ] Open Prompt Tree View modal: verify that Copy All, Send, and Enqueue actions are housed inside a single contiguous segmented capsule.
- [ ] Hover over split Chevron on Send: dropdown displays Send Now, Enqueue, Destination mode, Suffix, and Focus IDE.
- [ ] Click Primary "Send": immediately sends prompt to running instance and copies to clipboard.
- [ ] Press hotkey "N": triggers immediate prompt dispatch.
- [ ] Select "New Conv" from Destination dropdown: "New Conversation Title" input field appears.
- [ ] Enter a title "Refactor Architecture" and click "Send Now": prompt is sent with `conversationId = undefined`, creating a new conversation.
- [ ] Verify banner feedback displays: `New conversation "Refactor Architecture" dispatched to IDE!`.
- [ ] Select "Enqueue to FIFO Queue": prompt is added to FIFO queue without interrupting active IDE window.
- [ ] Pre-flight checks: `npm run build` succeeds without TypeScript errors.
