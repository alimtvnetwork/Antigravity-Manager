# Plan: UI and UX Enhancements for Antigravity Manager (ui-and-ux-enhancements-for-antigravity-manager)

## User Request (Verbatim)
```text
# UI and UX Enhancements for Antigravity Manager: high priority instruction, non-negotiable task

So far, good work. I want you to test using creating an end-to-end test instance. Okay, so here in the code or in the machine, you can build the code, you can run the code, and execute each test case from the AI memory and confirm that it works. Also, at the same time, I want you to change the CLI name to `Antigravity Manager (`Antigravity Manager (AGM)`)`-ui.exe, for all the machines, okay? Not the Ali, not the name, but `Antigravity Manager (`Antigravity Manager (AGM)`)`-ui. And also there are some glitches in the UI cross button that you see. It's cut off in places, so you need to make this fix. That's the first thing you should do. Top level, the cut button. But also at the same time, this UI becomes a bit challenging. What I want you to do is, we need to have the HTML/CSS command, which we could run something from the code level that would open something in the browser so that I could verify change the coloring or color theme the way that I want. I could change it, I could save it back, and that would actually save as the JSON instruction for every state. And you can actually follow the builder mode in the flat slides and other places, okay, same similar concept. That is for the UI mode. It is not going to be available in the exe, but that would be the ad hog, the UI changes and UI verification. Because every time you make a change, it takes some time to change your code and then resend back, which is not very healthy. Okay? So if we could have a way using that, if you can change the UI or take a look at the UI, that would be very much user-friendly. And for the settings section, there is no need to have the settings text, just the icon is fine. And, since the settings is a part of the last item, it should be triangle. It shouldn't be the settings, the round shape. It shouldn't be like that. Also, the update section is not available in the dropdown. Oh, in the first part. Okay, but checking the update does not do anything, at least at this point. It could say, "No updates," or something like this that is kind of missing. Yeah. So that is one of the crucial parts. And also at the same time, the glowing effect for every one of the color and the height of the progress bar. Try to make it a little bit less. A little bit. Now the circle ones need to be outside of the progress bar. And I will give you a sample one. You don't have to copy the exact, but the close enough would be lovely. Okay? Also, some of the dialog bars, when these dialogs pop up, the dialogs actually has some issues. So I'll give you the, again, sample progress bar. But you just change the coloring a bit, and you explain this progress bar to the AI in very detailed level as a reference. That should be your first job, then you start with other jobs. And also in some places, when the modal dialog comes, the modal header part goes inside of the top header, which is wrong. The modal should always have the high priority. So make sure that these are fixed. And also at the end, try to create a progress bar example with ``Id`` details so that any blind AI can follow and implement this progress bar in their system. Okay? The color, I really appreciate the green color. That should be the first color in the right-hand side. And then as we go, the last one would be the red, a little bit dark, and in between it would be blended in color, yellow and others. No more than five dots. And the beginning and the end should not have any circle dots. Okay, try to explain it just like this one. So this one has three, we should have five splits so that we can understand more visualization of the color, how it's getting down. Okay? And anytime we actually hover over, the color of this would be a little bit glowing if we hover to that row. If we don't, then no glowing. It'll be a little bit opacity or transparent, a little bit like 90% or 80%. When we hover over, it would be 100%, so it would be bright. So these are my observations. So make sure that you understood first, okay, and then write it, and then also read the previous end-to-end test cases that is in the AI memory and done by other AIs so that you can actually create your other instances and check again the sending, enqueuing prompt, drop-down UI, so many things. Okay? So make sure that you can do that. Is it clear? And also the standalone prompt copying is a must. Please verify that you have this feature. Is it clear so far? So make the UI more professional in terms of the UI UX principle. Is it clear?

## slug: ui-and-ux-enhancements-for-antigravity-manager

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/ui-and-ux-enhancements-for-antigravity-manager/ and enqueue plan task in .ai-memory/plans/ui-and-ux-enhancements-for-antigravity-manager.md (subtasks in .ai-memory/plans/subtasks/ui-and-ux-enhancements-for-antigravity-manager/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Create an end-to-end test instance to build, run, and execute each test case from AI memory
7. Change the CLI name to `Antigravity Manager (`Antigravity Manager (AGM)`)`-ui.exe for all machines
8. Fix UI cross button glitches and ensure the top-level cut button is corrected
9. Implement HTML/CSS command for browser-based UI verification and JSON state saving
10. Update settings section to icon-only and ensure the last item is a triangle
11. Ensure update section in dropdown provides feedback like "No updates"
12. Adjust progress bar height, color, and glowing effect; ensure circle indicators are outside the bar
13. Correct modal dialog header priority and ensure it doesn't overlap with the top header
14. Create a progress bar example with ``Id`` details for AI implementation
15. Ensure standalone prompt copying feature is verified and functional
```

---

## Architecture Boundaries & Deliverables
- **Spec Folder**: `02-spec/21-app/ui-and-ux-enhancements-for-antigravity-manager/`
  - `01-architecture-spec.md`: System-wide UI/UX tokens, binary alignment, modal priority hierarchy, and builder mode.
  - `02-component-spec.md`: Component-level modifications (NavMenu capsule, TitleBar/NavSettings window controls, QuotaProgressBar, Modal z-index).
  - `03-progress-bar-spec.md`: Grounded standalone progress bar blueprint with explicit IDs and CSS tokens for autonomous AI implementation.
- **Subtask Plans**: `.ai-memory/plans/subtasks/ui-and-ux-enhancements-for-antigravity-manager/`
  - `01-ui-components-and-builder.md`: Top-level cut/close button, settings icon/triangle, updater feedback, quota progress bar, modal header priority, browser builder mode.
  - `02-binary-alignment-and-e2e.md`: Align binary name to `Antigravity Manager (AGM)-ui.exe` / `agm-ui`, verify standalone prompt copy, run end-to-end test instance suite, and execute pre-flight checks.
