# Architecture Spec: 125-macos-dmg-install-quota-ui-and-instance-prompt-resend-fix

## User Request (Verbatim)

```text
Hi there. In the latest version, most of the things are not working. First of all, I wanted to focus on the macOS. The macOS DMG version is not installing properly. That is a very big issue. Second is that you need to make sure that the... A couple of serious issues. First of all, the UI issues I already mentioned, right? So you need to work on the front and the UI issues. That is kind of must. We need to fix those. But then again, the other issues are very important as well. For example, when we are in the instance mode, the instance mode needs to have the weekly capture or weekly progress bar displayed as well, which is missing. And the progress bar, actually, after the last one, it should be a little bit of orange to red. So I'm just selecting the section so that you understand where you have to correct. So in the instance section I'm showing, but the same thing needs to be applied in the account section. Now in the account page, there is a serious UI glitch. Let me explain the UI glitch. So you have four-hour quota, which is very big slides or the progress bar. But at the same time, weekly quota looks very small, and there are a lot of space in the right-hand side after the date or the action buttons, which can be extended to the right-hand side, which you didn't do it. So for the weekly time and for the weekly and four-hour model, the hour and percentage, those should be in the same column, actually. So make the progress bar a little bit fatty, you could say, or add a height to the progress bar to make it a little bit more resilient. And keep the hour and percentage in same column so that you can reduce the space. So that it looks nice. Or I think you can do that so it would look nice and look more professional. Also, the weekly section needs to be bigger, the color combination and everything. And right after the last item, the color needs to be red, actually. From orange to red right after that. And the weekly quota needs to be bigger, like similar width, both of them. If you put the remaining time and percentage together on the same column, then I think you will have more space to make it better. The UI should be professional, like same width. So for four-hour model, reduce the width a little bit. And for weekly quota, increase the width so that it matches with the four-hour quota. Try to make sure that this matches together. So this needs to be fixed. And then again, in the instance mode, the prompt running, injecting prompt, resending prompt, these are still does not work, which makes the system very bad. Actually, I've been requesting this several times. And also for the macOS, when we are running the shell script, make sure that it fixes the signature issue and makes the app to be installed automatically. Make sure of that. So these are on top of my head that you need to work on. The 50%, 30% quota on weekly, it looks very small in terms of how it is displayed right now. So make sure that this gets fixed, actually. Do you have any question and concern? Let me know.

Okay. So here, if you look into this, the UI does not look okay. So first of all, the line gap, you don't have the line gap. How you get the text, you don't have the line gap. Fix the line gap in terms of display and everywhere. Okay? That's the first. Second, the issue here, because the line gap, you need to fix it with a BR tag only. That's the first thing. Okay. So apply that, and then format it for the markdown. Okay. Then when I click on dot dot, it does not expand to the full text. That is a problem, and also N key, or also Send Now does not work at all. So you need to test it live here, then it goes there, and you can trace back. And also the activity, like where the project is running or things like that. That is also incorrect. For example, if I give you the screenshot of the defaults, the default instance does not have anything but Integrab be running. But it says what presentation is running. This is absolutely wrong. So you have to find the root cause to see why it is happening. And if we go into the gitmap section. You want to get into the gitmap section, the Git maps prompt. If we open, the problem here is that it still has prompts which has untitled conversation and has no prompt content. I asked you not to show this in the list while you are seeing this. And also, in this UI, the prompt view, I need to see a little bit of prompts sequence, prompts ending text, and also the name of the instance of the Id. So again, Id sequence, Id exe name, and also Id instance name. These three things I need to see here in the UI nicely. You need to move one prompt. You need to check the running prompts and also the running prompts property, sending the running prompts, entering the prompt. You need to check all this. It's typically working in the past, but in your case it is not working. Lots of untitled conversations. I asked you to merge these untitled conversations which has no prompt. Try to merge this. Yet understood
```

---

## 1. Architecture Scope & Problem Breakdown

This architecture specification unifies the remediation of six core functional and aesthetic defects across the Antigravity Manager desktop and CLI ecosystem:

### 1.1 Prompt Tree UI: Vertical `<br />` Line Gaps, Dot-Dot Ellipsis Expansion, and Header Metadata Trio
- **Cramped Text & Missing Line Gaps**: Raw and Markdown Preview prompt instructions collapse multiline text and paragraphs without vertical spacing. Enforce explicit `<br />` tags in `RichMarkdownRenderer`, `parseInlineMarkdown`, and the Raw Text View.
- **Click-to-Expand Truncation**: When prompt previews are truncated with `...`, clicking the ellipsis must dynamically toggle full text expansion. Provide a companion `[Expand (Full Text)]` / `[Collapse]` toggle button in the Prompt Instruction toolbar.
- **Header Metadata Trio & Tail Excerpt**: The prompt view header and prompt instruction toolbar must present:
  - Prompt sequence badge (`#P001` / `C001`)
  - Concluding prompt tail snippet (`“… ending with: '<snippet>'”`)
  - Instance Identity Trio: `[#seq · exeName · instanceName]` (e.g. `[#1 · Antigravity.exe · default]`).

### 1.2 Prompt Dispatch Engine: 'N' Key Hotkey & Live IDE Injection
- **Hotkeys**: Register a modal-level `keydown` listener intercepting `N` and `n` keys to dispatch the active prompt, with guards preventing interference with input fields, textareas, selects, or contentEditable nodes.
- **IPC Dispatch & Resend**: Wire active IPC dispatch writing `.antigravity_resume_task.json` into the target project workspace, calling `resume_recent_project_prompts`, focusing the running IDE instance, and providing instant visual toast feedback.

### 1.3 Project Activity Scoping & Empty "Untitled Conversation" Merge/Filter
- **False-Positive Running State (`white-presentation-v1`)**: In `src-tauri/src/modules/repo_db.rs`, project liveness checks previously relied on loose prefix matching and stale database records. Enforce a strict 120-second recency gate (`now - updated_at <= 120`) and process liveness verification. Inactive projects remain strictly IDLE.
- **Ghost Conversation Purge**: Filter out 0-word empty conversation nodes (titled "Untitled Conversation" or empty with no prompt content) across backend `compute_project_conversation_tree` and frontend tree rendering.

### 1.4 Quota Progress Bar UI Overhaul
- **Width Imbalance**: The 4H Model Quota progress bar was disproportionately wide while Weekly Quota was tiny and compressed. Balance both bars to have equal, professional widths.
- **Vertical Stacking of Time & Percentage**: Combine remaining time (e.g. `3d 11h`) and percentage (e.g. `33%`) into a single column (`flex flex-col items-end`), eliminating horizontal space waste and extending bars into available right-hand space.
- **Resilient "Fatty" Height**: Increase progress bar height (`h-2.5` / `h-3`) for robust visibility.
- **Orange-to-Red Color Progression**: The progress bar's final threshold must transition from orange/amber to vibrant red.
- **Enlarged Typography**: Enlarge 30% and 50% weekly quota badges and labels for prominent readability.

### 1.5 Weekly Quota Display in Instance Mode
- Display weekly quota progress bars alongside 4-hour model bars inside Instance cards (`Instances.tsx`) and Instance table rows (`InstanceTable.tsx`).

### 1.6 macOS DMG Installation & Shell Script Signature Hardening
- Harden `install.sh`, `scripts/Fix_Damaged.command`, and `scripts/package_dmg.sh` to ensure recursive quarantine stripping (`xattr -d com.apple.quarantine`) and ad-hoc code signing (`codesign` / `spctl`) execute non-interactively without "damaged, move to trash" errors.

---

## 2. Technical Blueprint & Component Specifications

### 2.1 QuotaProgressBar.tsx Redesign
- Change bar height from `h-1.5` / `h-2` to `h-2.5` / `h-3` (`rounded-full`).
- End color: Set the final segment background and gradient stops to transition to `from-amber-500 to-rose-600` / `bg-rose-500`.
- Remaining time and percentage rendered as a unified column:
  ```tsx
  <div className="flex flex-col items-end shrink-0 leading-tight">
      <span className="text-[11px] font-bold text-slate-700 dark:text-slate-200">{timeText}</span>
      <span className={cn("text-[10px] font-black", percentColor)}>{percentage}%</span>
  </div>
  ```

### 2.2 Accounts.tsx & AccountRow.tsx Grid Realignment
- Adjust column template:
  `grid-cols-[28px_20px_minmax(180px,1.2fr)_minmax(240px,2fr)_minmax(240px,2fr)_100px_110px]`
  allocating equal `2fr` width to both 4H Quota and Weekly Quota.
- Ensure the right-hand action column does not leave dead space.

### 2.3 Instances.tsx & InstanceTable.tsx Weekly Quota Addition
- Inside instance cards and table rows, read `boundAccount?.quota?.weekly` alongside `geminiModel`.
- Render the weekly quota progress bar directly below the 4-hour model bar.

### 2.4 PromptTreeViewModal.tsx `<br />` Line Gaps & Full Text Expansion
- `RichMarkdownRenderer`:
  - When line is empty: emit `<br key={`br-${i}`} className="my-2" />`.
  - When rendering paragraph: `<p key={`p-${i}`} className="text-xs text-slate-800 dark:text-slate-200 my-2 leading-relaxed whitespace-pre-wrap break-words">{parseInlineMarkdown(line)}</p>`.
- Raw View:
  - Map `activePromptText.split('\n')` with explicit `<br className="my-1.5" />` tags.
- Trailing `...`:
  - Interactive clickable badge toggling `setShowAllWords(!showAllWords)`.

### 2.5 repo_db.rs Strict Recency & Liveness Gates
- All conversation summary turns and active prompt checks require `(now - last_detected_at) <= 120`.
- If `!is_inst_alive`, `is_running` is strictly `false`.
- Skip empty untitled conversation nodes where `word_count == 0`.
