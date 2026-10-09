# Subtask 04: UI Tag Compaction, Header Badge Streamlining & Bracket Noise Stripping

- **Subtask ID**: `149-04`
- **Parent Task**: `149-smart-instance-process-cache-and-prompt-enqueue-fix`
- **Target Files**:
  - `src/components/instances/PromptTreeViewModal.tsx`
  - `src/components/instances/InstanceTable.tsx`
  - `src-tauri/src/modules/repo_db.rs`
- **Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)
- **Status**: `[READY_FOR_EXECUTION]`

---

## 1. Objective

Declutter the visual presentation across both GUI and headless interfaces:
1. **Prompt Tree View Modal (`src/components/instances/PromptTreeViewModal.tsx`)**:
   - Strip all bracket wrappers from sequence badges in tree nodes.
   - Consolidate the 6 adjacent pills in the preview header (Row 1) into 2 clean indicators: Sequence Badge and Status Badge.
   - Eliminate the redundant slim in-flight banner below the header, recovering 60px of vertical height.
   - Clean up button labels and keyboard shortcut hint badges (`Ctrl+↵`).
2. **Instance Table (`src/components/instances/InstanceTable.tsx`)**:
   - Compact the merged Profile & Account column and Status & PID column.
3. **Headless Formatters (`src-tauri/src/modules/repo_db.rs`)**:
   - Streamline `format_tree_view_cli` and `format_tree_view_telegram_html` to eliminate `[AGM:...]` and `[GM:...]` bracket noise.

---

## 2. Context & Root Cause Analysis

### 2.1 Cluttered Header Row 1 in `PromptTreeViewModal.tsx`
- **Location**: `src/components/instances/PromptTreeViewModal.tsx` (~L2915–2989).
- **Flaw**: Up to 7 individual pills are mounted side-by-side:
  1. `selectedConversation.seq_code` badge (`P001` or `C025`).
  2. `classifyPromptTier` badge (`User Prompt` / `Subagent` / `System` / `Tool Output`).
  3. `RUNNING` status badge (with PID and elapsed timer).
  4. `QUEUED` status badge.
  5. Conversation title (`<h3>`).
  6. Repeat count badge (`x2 runs`).
  7. Instance identity trio (`#1 · exe · name`).
- **Effect**: Excessive horizontal width causes awkward wrapping onto 2–3 rows on laptop screens, pushing the prompt text off-screen.
- **Fix**: Consolidate into 2 clear indicators:
  - Indicator 1: Clean Sequence Code (`C025` / `P006`) with an integrated compact origin icon.
  - Indicator 2: Unified Execution Status Capsule (`RUNNING · 01:23 (PID 1234)`, `QUEUED`, or `IDLE`).
  - Render instance context as a subtle muted inline breadcrumb without border boxes.

### 2.2 Redundant Slim In-Flight Banner
- **Location**: `src/components/instances/PromptTreeViewModal.tsx` (~L3153–3200).
- **Flaw**: Directly underneath the header row, another banner displays:
  `In-Flight Execution · PID: 1234 · 01:23 · Task actively executing...`
- **Effect**: Repeats the exact same status, PID, and duration shown in Header Row 1.
- **Fix**: Delete lines 3153–3200. The prompt text immediately begins under the header, recovering 60px of vertical space. Detailed execution steps remain visible in Tab 2 ("AI Results & Outputs").

### 2.3 Bracket Noise in Tree Badges & Headless Formatters
- **Locations**:
  - `src/components/instances/PromptTreeViewModal.tsx` (~L2145, ~L2381).
  - `src-tauri/src/modules/repo_db.rs` (~L6389–6435, ~L6491–6530).
- **Flaw**: Rendering `[AGM:P001 | GM:#1]`, `[AGM:C025]`, and `[ProjID: ...]` creates noisy visual friction.
- **Fix**: Use clean `#1`, `P001`, and `C025` identifiers without brackets.

---

## 3. Detailed Implementation Steps

### Step 1: Consolidate Header Row 1 in `PromptTreeViewModal.tsx`

1. Open `src/components/instances/PromptTreeViewModal.tsx` at line 2915.
2. Refactor the left container of Row 1:
   ```tsx
   {/* Row 1: Identity, Badges & Actions */}
   <div className="flex items-center justify-between gap-2.5 flex-wrap">
       {/* Left: Indicator 1 (Sequence + Tier), Indicator 2 (Status Capsule), Title, Instance Context */}
       <div className="flex items-center gap-2 flex-wrap min-w-0">
           {/* Indicator 1: Clean Sequence Code with compact Origin Tier Icon */}
           {(() => {
               const rawSeq = selectedConversation.seq_code || 'P001';
               const cleanSeq = rawSeq.replace(/^(AGM:|GM:)/i, '').replace(/[\[\]]/g, '').trim();
               const seqDisplay = cleanSeq.startsWith('#') || cleanSeq.startsWith('P') || cleanSeq.startsWith('C') ? cleanSeq : `#${cleanSeq}`;
               const tierInfo = classifyPromptTier(activePromptText, selectedConversation.title);
               return (
                   <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-mono font-bold bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20" title={`${tierInfo.tier}`}>
                       {tierInfo.tier === 'USER_PROMPT' && <User className="w-2.5 h-2.5 text-sky-500 shrink-0" />}
                       {tierInfo.tier === 'SUBAGENT_INSTRUCTION' && <Bot className="w-2.5 h-2.5 text-purple-500 shrink-0" />}
                       {tierInfo.tier === 'SYSTEM_MESSAGE' && <Terminal className="w-2.5 h-2.5 text-slate-500 shrink-0" />}
                       {tierInfo.tier === 'TOOL_OUTPUT' && <Wrench className="w-2.5 h-2.5 text-amber-500 shrink-0" />}
                       <span>{seqDisplay}</span>
                   </span>
               );
           })()}

           {/* Indicator 2: Unified Status Capsule */}
           {Boolean(selectedConversation.is_running) && !isGhostConversation(selectedConversation) ? (
               <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[9.5px] font-bold font-mono bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40 shadow-2xs animate-pulse">
                   <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                   <span>RUNNING</span>
                   {instancePid ? <span className="opacity-80">PID: {instancePid}</span> : null}
                   <span className="border-l border-emerald-400/40 pl-1">{formatDuration(elapsedSeconds)}</span>
               </span>
           ) : Boolean(selectedConversation.is_queued) ? (
               <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9.5px] font-bold font-mono bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 shadow-2xs">
                   <Clock className="w-2.5 h-2.5 text-amber-500" />
                   <span>QUEUED</span>
               </span>
           ) : (
               <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[9px] font-mono text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800/60 border border-slate-200 dark:border-slate-700">
                   IDLE
               </span>
           )}

           {/* Title */}
           <h3 className="text-xs font-bold text-slate-900 dark:text-white truncate max-w-[200px] md:max-w-xs" title={selectedConversation.title || selectedConversation.short_id}>
               {selectedConversation.title || selectedConversation.short_id}
           </h3>

           {/* Subtle Instance Context (Breadcrumb style) */}
           <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 truncate" title={`Instance #${instanceSeqNum} · ${instanceExeName} (${instanceNameDisplay})`}>
               #{instanceSeqNum} · {instanceNameDisplay}
           </span>
       </div>
       ...
   ```

### Step 2: Remove Redundant Slim In-Flight Banner in `PromptTreeViewModal.tsx`

1. Locate lines 3153–3200 in `src/components/instances/PromptTreeViewModal.tsx`.
2. Safely remove the entire `{/* In-Flight Execution or Queued Slim Banner (if active) */}` JSX block.
3. Verify that the prompt editor / viewer renders directly below the tab navigation without visual jumps.

### Step 3: Streamline Action Buttons & Keyboard Hints

1. Update the Send Prompt button capsule in `PromptTreeViewModal.tsx`:
   - Replace label with `"Send"` and keyboard badge with `<kbd className="...">Ctrl+↵</kbd>`.
   - Strip any literal bracket text (`[...]`).

### Step 4: Compact Profile & Status Columns in `src/components/instances/InstanceTable.tsx`

1. Open `src/components/instances/InstanceTable.tsx` at line 210.
2. Ensure `#seq` badge is tightly styled:
   ```tsx
   <span className="px-1.5 py-0.5 rounded-[4px] text-[10px] font-mono font-bold bg-slate-100 dark:bg-[#071a27] text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-[#15334d]">
       #{seq}
   </span>
   ```
3. In Status & PID column (~L333):
   - Render `Running (${inst.pid})` in a single unified pill.
   - For idle status, use `text-[10px] px-1.5 py-0.5 rounded-[4px]`.

### Step 5: Clean CLI & Telegram Formatters in `src-tauri/src/modules/repo_db.rs`

1. Open `src-tauri/src/modules/repo_db.rs` at line 6389 (`format_tree_view_cli`):
   - Replace:
     ```rust
     "📁 [AGM:{} | {}] [ProjID: {}] {} ({}) — {} [Instance: {}{}{}]\n"
     ```
     With:
     ```rust
     "📁 #{} ({}) · {} ({}) — {} [Instance: {}{}{}]\n",
     proj.seq_code,
     proj.gitmap_seq_code,
     label,
     proj_badge,
     proj.repo_path,
     inst_seq_str,
     proj.instance_name,
     email_str
     ```
2. In `format_tree_view_telegram_html` (~L6491):
   - Replace bracketed `<code>[AGM:{} | {}]</code>` with clean `<b>#{}</b> (<code>{}</code>)`.

---

## 4. Invariants & Constraints

- **Minimalist & Contextual UI**: Follow the project maintenance guidelines — wrap controls into contiguous segmented capsules, eliminate disjointed loose buttons.
- **Transparent Select Styling**: Inline dropdowns inside segmented capsules must remain `bg-transparent border-0 focus:ring-0 focus:outline-none`.
- **Zero Content Truncation**: Preserve multiline prompt text and clipboard content integrity.

---

## 5. Verification & Pre-flight Checklist

```bash
# Verify frontend build
npm run build

# Verify Rust code compilation and format
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
```

---

## 6. Done When

- [ ] Sequence badges across tree nodes and headers in `PromptTreeViewModal.tsx` render without bracket noise (`[...]`).
- [ ] Header Row 1 is consolidated into 2 clean indicators: Sequence Badge and Status Capsule.
- [ ] Duplicate slim in-flight banner (~L3153–3200) is removed, recovering vertical space.
- [ ] Action buttons render clean labels with `<kbd>Ctrl+↵</kbd>`.
- [ ] `InstanceTable.tsx` profile and status columns are compacted.
- [ ] Headless CLI and Telegram formatters in `src-tauri/src/modules/repo_db.rs` output clean unbracketed trees.
- [ ] `npm run build` and `cargo clippy` execute with zero errors.
