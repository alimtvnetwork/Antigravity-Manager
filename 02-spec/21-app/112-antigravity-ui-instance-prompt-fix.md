# Specification: Antigravity UI, Themes, Instance Cards & Prompts Polish

> **Spec ID:** `112-antigravity-ui-instance-prompt-fix`  
> **Status:** APPROVED & IN PROGRESS  
> **Scope:** UI Quota Progress Bars, Theme System & Contrast, Instance Cards Alignment & Hover, Prompt Tree View & Filtering, IDE Focus/Relaunch, Cross-Platform Switching Diagnostics  

---

## 1. Executive Summary

This specification addresses a cohesive set of frontend UI enhancements, theming fixes, instance card layout polish, prompt management streamlining, IDE window control, and cross-platform profile switching diagnostics for Antigravity Manager.

Key Objectives:
1. **Quota Progress Bar Polish**: Restrict glowing drop-shadow strictly to the first active bubble. Implement multi-step gradient transitions (deep green -> orangey yellow -> orange -> red after completion). Split 4H and Weekly quotas into two distinct columns, remove redundant "4h" repetitions, group percentage and ETA together on the right side, add the Gemini icon, and expand the progress bar into reclaimed date whitespace.
2. **Theme System & Light Contrast Repair**: Fix top-level theme switching so themes properly propagate styles across all components. Overhaul Light theme contrast (eliminate pitch-black rows on white backgrounds, adjust cyan text to high-contrast colors, fix dot visibility). Remove excessive pink/purple colors in favor of VS Code / Antigravity slate, blue, cyan, and mint tokens.
3. **Instance Cards Parity & Hover Polish**: Display both Data directory and Executable path for the Default instance matching cloned cards. Implement CSS3 hover transitions with darker background styling and dark email badge with yellow text highlight. Unify alignment across all cards by standardizing the header height and "Set Default" button layout.
4. **Prompt Tree View & Search Streamlining**: Eliminate synthetic "turn" nodes; display only user-submitted prompts with newest entries sorted to the top. Streamline menu filters into a segmented capsule (`Running`, `Latest`, `Pinned`) with remaining filters in a dropdown. Add a project-by-project dropdown on top of the search input.
5. **Prompt Details & Preview**: Move noisy IDs, churn, idle state, and timestamps into a clean "Details" modal. Fix markdown preview whitespace and newlines (`\n`), headers, and provide an expandable "Detail / More" full-text view.
6. **IDE Focus & Instance Switch Relaunch**: Add a "Focus IDE" / "Open IDE" action when enqueuing or resending prompts. Ensure instance switching acts as a full relaunch of the Antigravity IDE with target profile credentials.
7. **Ubuntu & macOS Switching Diagnostics**: In-depth analysis of LaunchServices / AppImage PID discrepancies and permission barriers on Linux and macOS; add comprehensive logging, explicit exception handling, and error modal reporting.

---

## 2. User Request (Verbatim)

```text
# High Priority Instruction

Okay. A couple of issues. Let's discuss one problem at a time. First, let's start with the UI. I really appreciate the progress bar you have made, but the progress bar is too much color right now. And also it needs to be the glowing a little bit reduced. Only the glowing can be on the first ball or bubble, not every one of them. Rest of them will have the round shape, but you can reduce the round shape a little bit less, and the glowing would be a little bit less. So as it goes from first color to the second, the color will be a little bit deep green, and then the third one would be a little bit of orangey yellow, and then last one would be a little bit of orange, and after the last circle, it will be red. So this is how the color of the progress bar should go. And for the, let's say, hourly quota and weekly quota, you try to put the hour there, so the space reduced a lot. It does not look good. So what you have to do is the numbers, like when this is going to pre-fill or things like that, you have to probably keep a space in one side. Let's say right-hand side, you can just put one space where you could just put this text, how much percentage and how much time that it would take to full fill. And you don't put, let's say, four hour in two places. It doesn't make any sense. You don't need four hour, right? So four hour is four hour. We understand. Same with the weekly. So you could actually divide this into two column because it's a two column, right? The first column will say four hour, and the next column will say weekly. So the weekly section, we know it's already weekly. And you can just put the, let's say, you don't need to put any Gemini or things like that because by default it already says that we're selecting Gemini. But you can put an icon besides Gemini so we know it's already Gemini. So you can space, reduce, and space. Now the coloring needs to be improved, and you can also spread the progress bar a little bit right-hand side where the date is because date is fixed and there are a lot of white space in the date section, so which you can reduce in the right-hand side. So you can just look into the picture and try to understand how you improvise the UI. The coloring and everything, I really do appreciate and I liked it. So you follow through the same color, but in a little bit, in a different way. That's the first thing. Second, when we change the theme in the top level, the theme has no effect. It does not change any coloring. So make sure that the theme is actually in effect. It can change the stuff. Currently, there is no effect. Only the white theme light if we do. It looks terrible. It looks really, really terrible, which you need to fix. So themes needs to be better. That's the first observation. Try to observe this one. The root cause fix the theming issue. Now coming to the instance part. I do like what you have included so far. So one more observation is that usually the projects which are cloned, they have two sections. One is the data, another is the exe. So same thing should happen for the default one as well. Default data also is important. Show that where that is so that everything is consistent. I like the hover effect which you have done to the cards. And also, if possible, you could create a color difference. For example, when we hover over it could, the card color can be switched to a little bit darker if we are in dark mode. For example, the VS code dark mode color. And it will again go away when we hover to another one, and there will be a little bit CSS3 transition that will look really nice, amazing. All of that. And also when we hover over, make sure the emails section actually becomes dark and email text becomes yellow so that we understand the highlight section. And make sure the alignment for both or all the controls are in the same level. Currently, if you see the one control has a different alignment than others, have a set default button where the set default button should be on top. If you need to combine the buttons, make sure that things are in one place. If you need to put the set default to somewhere, we could put it either in the bottom or somewhere same place so that it does not change the places. It's very, very important. Respect that, then the things will be very much easier. That's the next observation. The third observation is that when we go inside the prompt, we double-click and go into the prompt and see where the project is running. There are several bugs. First of all, I really do like the tree view, which you have done. What is this turn? I don't understand. If something is turn- I think you can skip that from keeping it in the display. Only the prompts which is given by the user. Terms are, I think, is done by the IDE, not by us. Checkpoint of the conversation. If that is the case, then I request you to take out the terms. Only display the prompts which you find as a text, you display that. And this is how you proceed further. There are a lot of issues in the menu section. For example, if we start with the filter, the filter has too many filters, which I think you should put running and latest. Running, latest, and pinned as the filter. And rest of these, you should have a drop-down button where we could create this or do these filters. So this is one of the observation. And also there could be a project-by-project search. That means there could be a drop-down on top of the search. By default, it will do all projects, but I could select a specific project and search into that project for the conversation as well. So this is another issue that you need to fix. Now, coming to the right-hand side of the prompt section. The full screen is fixed, I really appreciate that. So in this section, right-hand side, I have highlighted. This section is too much clumsy. Why we have queued, idle, churn? It does not make any sense. Too much ``Id``. The IDs can be as a button or another pop-up, like details. We click on it, we see more details if we wanted to. But unless we wanted to, we don't see this ``Id``, modified time. This will be waste. Does this make sense? And the latest one, we always want to have the top in the tree view of the tree view. Does this make sense? Now, coming to the point. Let's say we enqueue a prompt or resend a prompt. That resend prompt or enqueue the prompt, it should also have a button to open or focus the IDE. So I should click on focus IDE, and that should open the IDE. If the IDE is not open, it should open the IDE, and I should see that the prompts are running. So this is must, I wanted to have. Now, the next problem is actually the visualization of the prompts that is running. So now I'm giving you a section that the prompt is running. Prompt is running, but the preview is not really very high quality, and it's broken. So in your preview section, you do not respect the new line concept. So you need to respect the new line concept. And based on that, you need to preview in the markdown. So here what you see, it is already actually put as, let's say, markdown. And there was a new line after the text. And the new line starts with the hash, and that should have another new line where the OK starts, and you don't have it. It looks really poor. So make sure the new lines are respected and the preview is nicely displayed, which is not happening at the moment. And user should have a detail button or more button. You can click on it. All the text would be visible here. So these are my concerns. And also the another concern, I want you to do very much detailed analysis why in Ubuntu and macOS, the Antigravity manager does not switch the profile properly for the Antigravity. So for this reason, I want you to do all types of logging. And if anything that you're slightly concerned with, make sure you throw an exception and use the error model to capture those errors so that we can fix the error later on. Is it understood? Can you please follow through? And all this selection color in the markdown, make it much more colorful, more bright. Use a different color tactics. And I really don't like this pink color that you have everywhere. Try to have a VS Code type theme, Antigravity current theme that we have, similar type of color, not bad. There are lots of IDE colors. Try to follow this color concept, that will look very nice. Make sure the themes works properly. You should have a perfect color theme, other color themes, which you can get from the coding guideline. Coding guideline has a design system inside the spec folder. You can look into this deep and find the root cause and improve your design system, add more theme coloring transition so that it looks really handy. Do you understand? So in most cases, I have given you all these images, all these things. Now it's your part to make sure that we win together. Is it clear? the switch button should act as a relaunch of the IDE make sure of it. please
```

---

## 3. Architecture & Root Cause Analyses

### 3.1 Quota Progress Bar & Quota Layout
- **Glow Restriction**: In `WaterDrainProgressBar.tsx`, checkpoint bubbles currently all render `shadow-[0_0_8px_rgba(26,241,141,0.6)]`. Only the first bubble (`idx === 0`) must have the glow effect. Subsequent bubbles have clean borders without drop-shadow glow.
- **Color Progression**: Checkpoints `[100, 75, 50, 25]` map sequentially:
  - 1st bubble (100%): Vibrant Green (`#1af18d`) with glow
  - 2nd bubble (75%): Deep Green (`#059669`)
  - 3rd bubble (50%): Orangey Yellow (`#eab308` / `#f59e0b`)
  - 4th bubble (25%): Orange (`#f97316`)
  - Beyond last circle (< 25% / exhausted): Red (`#ef4444`)
- **Two-Column Quota Division**:
  - `AccountTable.tsx` header splits into `4H Quota` and `Weekly Quota`.
  - Account rows split the quota `<td>` into two separate cells: Cell 1 for 4H Quota, Cell 2 for Weekly Quota.
  - Eliminate duplicate "4h" string in rows; column header identifies the window. Row cells display model icon + progress bar + metrics.
  - Metrics (Percentage & ETA) are grouped together in a right-aligned container (`ml-auto shrink-0`).
  - Add `Gemini.Color` icon next to Gemini header tab.
  - Date column width is reduced from `150px` to `95px`, allowing progress bars to expand horizontally into reclaimed space.

### 3.2 Theme System & Contrast Repair
- **Root Cause**: `ThemeManager.tsx` sets CSS variables (`--bg`, `--surface`, `--primary`), but UI components (`AccountTable.tsx`, `Instances.tsx`, `QuotaItem.tsx`) hardcode arbitrary Tailwind classes (e.g. `dark:bg-[#071a27]`, `bg-slate-900/90`). In Light mode, `bg-slate-900/90` renders pitch-black rows on white surfaces.
- **Fix**:
  1. Map theme CSS variables to core container classes and establish true light mode tokens: surface `bg-white`, border `border-slate-200`, text `text-slate-800`, active row highlight `bg-blue-50/70` with border `border-blue-200`.
  2. In `QuotaItem.tsx`, update Light mode text colors so cyan/blue have >= 4.5:1 contrast ratio (`text-cyan-800` / `text-emerald-700` in light mode).
  3. Purge unwanted pink accents (`from-purple-600 to-pink-600`, `rose-500`) and replace with Antigravity / VS Code slate, blue, cyan, and mint design tokens.

### 3.3 Instance Cards Parity, Hover & Alignment
- **Default Card Data & Exe**: Default instance config currently has `executable_path: null`. Provide fallback `config?.default_executable_path || 'antigravity.exe (Default)'` so both Data and Exe badges render consistently across all cards.
- **Card Hover Transition**: Add `hover:bg-slate-50 dark:hover:bg-[#061421] transition-all duration-200` to non-active cards.
- **Email Section Hover**: On card hover (`group-hover`), email container transitions to dark background (`dark:group-hover:bg-[#050f18] group-hover:border-amber-400/50`) and email text highlights in yellow (`group-hover:text-amber-300`).
- **Alignment Unification**: Eliminate vertical jumping between cards caused by `flex-wrap` and differing "DEFAULT" vs "Set Default" button widths. Lock header height to `h-6`, standardize button sizing, and align toolbar actions.

### 3.4 Prompt Tree View & Filtering
- **Strip Synthetic Turns**: Remove Layer 3 synthetic turn expansion (`[Turn #1] Checkpoint prompt...`) in `PromptTreeViewModal.tsx`. Display real user-submitted prompt text directly from conversations.
- **Descending Sort**: Ensure conversations and prompts sort strictly descending by `last_modified` so latest prompts remain at the top.
- **Filter Bar Capsules**: Convert filter buttons to a clean segmented pill with `[Running]`, `[Latest]`, and `[Pinned]`. Place remaining filters (`All`, `Archived`) in a dropdown `<select>`.
- **Project Dropdown Search**: Add a project selector dropdown beside the search box to scope search to "All Projects" or a specific project repository.

### 3.5 Prompt Details Modal & Markdown Preview
- **Details Modal**: Extract right pane metadata (Conversation UUID, PID, Churn, Idle state, Modified timestamp) from the header into an inspectable "Details" popover modal, leaving the header clean and focused.
- **Markdown Newline Preservation**: Fix `RichMarkdownRenderer` which currently drops empty lines (`if (!trimmed) continue;`). Retain spacer elements for blank lines, apply `whitespace-pre-wrap`, and ensure headers (`#`) on newlines render properly. Provide a prominent "Detail / More" button for full inspection.

### 3.6 IDE Focus & Instance Switch Relaunch
- **Focus / Launch IDE**: Implement Tauri command `focus_or_launch_instance` that checks if the instance PID is alive. If alive, brings the window to the foreground (Win32 `SetForegroundWindow`, macOS `osascript activate`, Linux `xdotool/wmctrl`). If not alive, launches the instance. Connect this action to "Focus IDE" buttons in prompt enqueue and resend dialogs.
- **Instance Switch Acts as Relaunch**: In `switch_account_to_instance`, terminate the old instance process and immediately launch the newly activated instance with injected profile credentials.

### 3.7 Cross-Platform (Ubuntu / macOS) Switching Diagnostics
- **macOS Issue**: `open -n -a ...` exits immediately; child PID is transient, causing dead PID tracking. Solution: resolve actual bundle executable binary path or query PID via `pgrep -f Antigravity`.
- **Linux Issue**: Sandboxing (Snap/Flatpak) and AppImage environment variable conflicts. Solution: sanitize environment variables and check directory write permissions before launch.
- **Logging & Error Capture**: Add structured logging (`INSTANCE_SWITCH`) across all execution steps. Propagate detailed OS error messages to `useErrorStore` so errors trigger the user-facing `ErrorModal`.

---

## 4. Verification Criteria

- [ ] Checkpoint bubbles have glow ONLY on the first active bubble; color transition goes deep green -> orangey yellow -> orange -> red.
- [ ] 4H and Weekly quotas appear in two distinct columns without duplicate "4h" text; percentage and ETA are aligned to the right.
- [ ] Theme switching alters visual appearance across all views; Light theme maintains strong contrast and high readability.
- [ ] Default instance card displays both Data dir and Executable path; card hover shows darker background; email badge highlights yellow on dark.
- [ ] Instance cards maintain uniform height and control alignment.
- [ ] Prompt tree view excludes synthetic "turn" nodes and shows user prompts sorted newest first.
- [ ] Filter toolbar features `Running`, `Latest`, `Pinned` with dropdown for others; project search dropdown allows filtering by project.
- [ ] Right-hand prompt header is decluttered with metadata housed in a "Details" modal.
- [ ] Markdown preview preserves newlines and formatting with an expandable "Detail" view.
- [ ] Enqueue and resend actions include "Focus IDE" capability; instance switch relaunches target IDE instance.
- [ ] Ubuntu and macOS profile switching includes robust logging and error modal captures.
