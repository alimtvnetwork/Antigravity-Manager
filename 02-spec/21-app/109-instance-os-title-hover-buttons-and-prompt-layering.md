# Specification: Task 109 - Instance OS Window Title Format, Card Grid Alignment, Hover Animation & Prompt Tree Layering Overhaul

## 1. Executive Summary & Problem Classification
- **Domain**: Instance Window Titles, Card Grid Alignment, CSS3 Hover Animation, and Prompt Tree View Overhaul.
- **Problem Statement**:
  1. **OS Instance Window Title**: When an instance window opens or appears in the OS taskbar/thumbnail, it displays generic text (e.g. `AGM - Standalone - Antigr...`) rather than the required standardized 3-part identity: `#{sequence} {instance_name} - {ide_ending_sequence}` (e.g. `#1 Default - Antigravity`, `#2 8136 - antigravity-8136`).
  2. **Instance Card Button Grid Alignment**: On `#1 Default`, Row 2 of buttons has 5 buttons (`[Prompts] [Settings] [Clone] [Executable] [Wipe]`), while non-default instances have 6 buttons (`... [Delete]`). Because the row used `flex-1`, buttons on `#1 Default` shifted horizontally, misaligning `[Clone]` and `[Executable]` relative to adjacent cards.
  3. **Instance Card Header Styling & CSS3 Hover Animation**: Top accent bar was overly harsh and busy. Need to relocate the color accent to the bottom level, soften borders, and make the glow/color smoothly animate on hover using CSS3 transitions.
  4. **Prompt Tree View Modal Heading & Full-Screen Clipping**: When opened, the modal title and header buttons were clipped by the sticky Navbar (`z-index: 100`). Full-screen mode did not take over the screen cleanly.
  5. **Project-Level Refresh & Pinning Mechanism**: Users could not refresh prompts for an individual project directly from the tree, nor pin important projects to the top of the tree.
  6. **Prompt Layering & Prioritization**: Projects and conversations lacked intelligent sorting (Pinned first -> Actively Running -> Recent Activity -> Alphabetical).
  7. **Prompt Search Filter Pills**: Need dedicated filter pills: `All`, `Running Only`, `Latest Conv`, `Latest Prompt`, `Pinned`.
  8. **Prompt Resending, Image Handling & Confirmation Suffix Dropdown**: Need ability to copy prompt text only, copy prompt with images, save images to disk, and choose confirmation suffixes ("Is it done?", "Is it released?", "Are you sure about it?", etc.) before resending.
  9. **Professional Running Indicator**: Replace basic green ping dot with a professional animated pulse badge displaying `ACTIVE RUNNING`, instance PID, and real-time elapsed ticker.

---

## 2. User Request (Verbatim)
```text
When we create an instance, make sure the instance sequence with the hash should be the first thing. The second would be the instance name. The third would be the ending sequence of the IDE. These are the three things we want in the title for when the instance actually pops up. Remember to change this for the OS. Okay? If you have any understanding issue or things like that, you can let me know. Now, one more thing we want is actually in the instance section. Instance section, the coloring does not look very good. So you have a heading, like tapping. Okay? So this can be changed to a little bit different coloring.

Here, the coloring that you have on top level, let's have it in the bottom level, but also a little bit less bordering. But also this will show up when we hover over. Not always, but when we hover over, they will come as a CSS3 animation. Then the instance section would look very professional. In some cases, the buttons are actually in different position. Could you please make sure the buttons are in the same position in the instance section? I'm just giving you two buttons highlighting, so that you understand the sequence is a bit different here. So try to follow through. That's important. Then I want you to fix the prompt layering in the instance section. That is very crucial. The prompt layering, when it shows up, it should actually show up from caching. That's all right. But also in the conversation, whichever has the running conversation, that should be always on top. And running project also be in the top. Okay? And you should have a pinning mechanism, like I could pin that project in that instance and see it. And also the headings and parts of the prompt model is actually broken. It does not look nice. You need to fix it. And also the full screen mode also does not get into full screen, and it gets cut off by the above header. So need to fix that as well. And we need to have a bunch of more options, like filtering the running prompt, searching the running prompt. So all kinds of small filters, like which has the latest conversation, started by conversation, started by prompt. This needs to be there, and also at the same time, we should have the option to resend the prompt, copy the old prompt, including the image. The images are very important. If it has image, then image needs to be copied as a separate item or to copy all these images together as a one image. Okay? So both sorts of options are needed. Or also, the images can be saved into the file system and referred as a file. So do it whatever that you think is the easiest so that we can resend the prompt again, so that the AI can work on it. And also with each one of the prompt section, we should have another button that would say, "Is it released? Is it done? Are you sure about it?" So this type of things, that should have a dropdown. We can choose and resend the prompt using any one piece. So that would be added and we resend. Okay? So currently the running prompts, looking into those, it shows a green icon, something like this, but it's not very professional. We do not know which one is going on, which one is running. And also there needs to be a Refresh button close to the project. Close to the project, there should be a Refresh button that will tell us whatever the running prompt is. And we should be able to back up the running prompts, enqueued prompts, very important. And also able to resend any single prompt, running prompt or enqueued prompts easily from here. That needs to be done. Okay? Can you please ensure this? So these type of buttons also try to keep in mind the UI/UX concept so that it looks nice. Is it clear?
```

---

## 3. Architecture & Technical Design

### A. OS Instance Window Title Formatting
- **File**: `src-tauri/src/modules/instance.rs`
- **Helpers**:
  - `compute_ide_ending_sequence(inst: &InstanceConfig) -> String`:
    - Returns `"Antigravity"` if `inst.is_default || inst.id == "default"`.
    - Otherwise returns `"antigravity-{clean_slug}"`.
  - `compute_instance_window_title(inst: &InstanceConfig) -> String`:
    - Template: `format!("#{seq} {name} - {suffix}${{separator}}${{dirty}}${{activeEditorShort}}${{separator}}${{rootName}}")`.
- **Target Injections**:
  - `create_instance_with_account`: injects `window.title` on creation.
  - `copy_instance_with_options`: updates `window.title` for copied instances.
  - `rename_instance`: re-evaluates `window.title` across all `get_instance_settings_targets`.
  - `launch_instance_inner`: ensures `window.title` is freshly injected before process launch.
  - `load_registry`: one-time startup backfill for all existing instances.

### B. Instance Card Button Grid Standardization (`grid-cols-6`)
- **File**: `src/pages/Instances.tsx`
- **Grid Layout**: Replace `flex items-center gap-1 w-full` with `grid grid-cols-6 gap-1 w-full`:
  - **Slot 1**: `[Prompts]` (`Layers` icon)
  - **Slot 2**: `[Settings]` (`SlidersHorizontal` icon)
  - **Slot 3**: `[Clone Profile]` (`Copy` icon)
  - **Slot 4**: `[Clone Binary / Executable]` (`Cpu` icon)
  - **Slot 5**: `[Wipe Credentials]` (`RotateCcw` icon)
  - **Slot 6**: `[Delete]` (`Trash2` icon) for custom instances, or `<div className="w-full invisible" aria-hidden="true" />` for Default instance.
- **Result**: `[Clone]` is strictly at 33.3% and `[Executable]` is strictly at 50% across every single card in the application.

### C. Bottom Level Color Accent & CSS3 Hover Animation
- **File**: `src/pages/Instances.tsx`
- Remove top accent bar (`h-1.5 w-full bg-gradient-to-r`).
- Soften outer borders: `border-gray-200/50 dark:border-[#15334d]/60 hover:border-gray-300/80 dark:hover:border-blue-500/30`.
- Add bottom accent line at the base of the card:
  ```tsx
  <div className="relative w-full h-[3px] overflow-hidden rounded-b-xl">
      <div className={cn(
          "absolute inset-0 bg-gradient-to-r transition-all duration-300 ease-out transform",
          theme.accentBar,
          isActive
              ? "opacity-60 scale-x-100 group-hover:opacity-100 group-hover:shadow-[0_0_8px_rgba(59,130,246,0.5)]"
              : "opacity-0 scale-x-95 group-hover:opacity-100 group-hover:scale-x-100 group-hover:shadow-[0_0_8px_rgba(59,130,246,0.3)]"
      )} />
  </div>
  ```

### D. Prompt Tree View Modal Portal & Full Screen Fix
- **File**: `src/components/instances/PromptTreeViewModal.tsx`
- Mount via `createPortal` to `document.body`.
- Elevate z-index to `fixed inset-0 z-[300]`.
- Remove manual `mt-12 sm:mt-14` margin that was clipping the title.
- Implement `isFullscreen` state with dedicated `[Full]` / `[Exit]` toggle button. When active, expands modal to `h-full w-full rounded-none border-0`.

### E. Project Inline Refresh & Pinning
- **Inline Controls on Project Rows**:
  - `RotateCw` icon button: triggers `handleRefreshSingleProject(projectId)` using `get_project_conversation_tree` with `force: true`.
  - `Pin` icon button: toggles project ID in `pinnedProjectIds` list.
- **Persistence**: Saved in `localStorage` under `agm_pinned_projects_{instance_id || 'default'}`.

### F. Multi-Tier Prompt Layering & Prioritization
- **Project Sorting (Layer 1)**:
  1. Pinned projects (`pinnedProjectIds.includes(p.project_id)`).
  2. Actively running projects (`p.is_running || p.conversations.some(c => c.is_running)`).
  3. Most recent conversation activity (`last_modified`).
  4. Alphabetical name.
- **Conversation Sorting (Layer 2)**:
  1. Actively running conversations (`c.is_running === true`).
  2. Descending by `last_modified`.

### G. Search & Filter Pills
- Filter pills directly under Search Bar:
  - `[All]` (total project count)
  - `[Running]` (active running count with pulse dot)
  - `[Latest Conv]` (most recent conversation per project)
  - `[Latest Prompt]` (sorted by latest prompt turn)
  - `[Pinned]` (pinned project count)

### H. Prompt Resend, Image Handling & Confirmation Suffix Dropdown
- **Image Extraction & Actions**:
  - `handleCopyTextOnly()`: strips image markdown and data URLs.
  - `handleCopyWithImages()`: copies raw verbatim prompt.
  - `handleSaveImages()`: extracts images and prompts user to save them locally.
- **Confirmation Suffix Dropdown**:
  - Presets:
    - `None (Send as is)`
    - `Is it done?`
    - `Is it released?`
    - `Are you sure about it?`
    - `Double check all edge cases`
    - `Verify build and tests`
  - When clicking "Resend", appends suffix if chosen, and writes to `.antigravity_resume_task.json` or dispatches via IPC.
- **Professional Running Indicator**:
  - Animated pulse badge in conversation tree item.
  - Top header ticker displaying `ACTIVE RUNNING`, matched `PID: {instancePid}`, and real-time elapsed time.
