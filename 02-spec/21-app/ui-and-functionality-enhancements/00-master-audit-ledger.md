# Master Audit Ledger: UI and Functionality Enhancements

- **Slug**: `ui-and-functionality-enhancements`
- **Specification Path**: `02-spec/21-app/ui-and-functionality-enhancements/`
- **Master Plan**: `.ai-memory/plans/ui-and-functionality-enhancements.md`
- **Branch Benchmark**: `backup/ui-and-functionality-enhancements-benchmark`
- **Active Working Branch**: `main`
- **Release Channel**: `main` (Stable minor release)
- **Status**: `[IN PROGRESS]`
- **Target Version**: `v4.207.0` (Minor bump from `v4.206.0`)

---

## User Request (Verbatim)

```text
# UI and Functionality Enhancements: high priority instruction, non-negotiable task

I appreciate the recent changes. The recent changes are good. However, there are certain things like the glow. It is having too much glow right now. And also, if it gets cut off in the screen, if I double-click on the header, it does not make a full screen. That's one issue. The second thing is that the progress bar is not in the consistent shape with the placement of the circles. So the circles look nice, but these are not consistent. So try to reduce the width of the progress bar, not the height. Height is in correct shape how it is right now. The too much glow is bad, so I think we can reduce the glow a bit and try to have the consistency in the circle section. If required, you can have four, let's say, checkpoints. That's absolutely all right if the file is taking more of your issue, but I think five should be easily you can put it to other places. Oh, I think you try to put the green together so that the color is same. That's a nice understanding. So if that is actually causing you the problem, so let's have four with the same equal distance. Currently, it looks really nice, the color combination and everything I really appreciate. We can reduce the width to, let's say, 10% for both of these and just put it a little bit compact so that the text has a little bit of space in the email section. And email section should grow a little bit. There should be a minimum and maximum. And also the user can actually tweak the column, reduce and make it bigger. That option needs to be there. Do you understand? So these are important and high priority instruction that you need to follow immediately. Once we have this, then we can start looking. Also, please confirm the instance testing are already done. That means every instance can send the prompts. And the log and idle situation, I think you can compact these two buttons so that you have less button. So in the instance section, let me try to send. Sending is working now. That's extremely good. Sending is working. Let me try the enqueue. Enqueue button is not working. So send is working. Enqueue is also working. That's a very good thing. So you don't need to verify these two facts. These are excellent work so far. You should write the root cause why it was not working before, how it is working now, and if possible, try to compact these three buttons in the right-hand side. I'm just selecting, so you can put it into the right-hand side is a compact connect together, and send and enqueue should be single button in a drop-down. That is not improved yet. So you should improve that and make a minor bump and do a release. Do you understand? Is it clear so far? I do really appreciate all the recent changes. Also, let me check the Telegram. If we could send some prompt from Telegram to any instance, and you can check it. In the instance section, remote fleet, we don't have the option yet. I think I could run a second instance for your checking. I'm opening two instances. So using `gitmap` nodes, you will find them, connect with them, and you can run commands. And try to check whether in future, you can actually connect with the fleet information. That means all these machines, whatever that they are using, using the Telegram email or the Supabase database, either one of them is connected. It would send the information as a JSON. So the other party will receive from the Telegram or email, any one of them as a JSON, pure JSON, the communication understanding what is the how many instances are there, what are the email addresses that has been picked, and we can also send a signal using Telegram or email, either one, if that is connected to change account, switch fast forwarding, any one of them. So fast forwarding algorithm would be a little bit more efficient because it would know which machine is also using what. I think the fleet remote information is not yet done. We need to work on it. Other than that, it's a great victory, great work so far. Also in the prompt tree section, try to add a new conversation send option. That means usually the prompt tree sends to the current conversation. But also at the same time, we should have an option to send it to the new conversation. We can give a name to the new conversation. So that would be inside the compact button where the copy all, send, and empty would be together. So in between there would be button, the current conversation. We can drop down and say new and then type a name and then send to that conversation by creating newly on that project. Do you understand this? It is very important for you to understand and write as an AI spec so that any AI can understand and then you can verify and do the work. Is it clear? I do really appreciate that. We should take a backup of this branch as well. That would be an engineering benchmark for now, and then start working on anything on the main branch, and then we release bump and proceed forward. Does this make sense?
```

---

## Visual Evidence & UI Artifacts

- **Image 1**: Account Table & Quota Progress Bars (`assets/screenshots/ui-and-functionality-enhancements-01.png`)
  - Shows current 5 uneven checkpoints, neon glow effect, email column constraint, and top header with IDLE and Log buttons.
- **Image 2**: Prompt Tree View Toolbar (`assets/screenshots/ui-and-functionality-enhancements-02.png`)
  - Highlights `[ Copy All v ] [ Send v ] [ Enqueue v ]` disconnected loose capsules needing compaction, unified Send/Enqueue dropdown, and New Conversation flow.
- **Image 3**: Remote Fleet Detection Empty State (`assets/screenshots/ui-and-functionality-enhancements-03.png`)
  - Displays "No Remote Fleet Machines Detected" awaiting Telegram / Email / Supabase JSON telemetry connection.

---

## Traceable Discrete Tasks

| Task ID | Component / Area | Description | Status |
|---|---|---|---|
| **Task-01** | Git Branching | Preserve engineering baseline in `backup/ui-and-functionality-enhancements-benchmark` | `DONE` |
| **Task-02** | Spec & Planning | Comprehensive spec and subtask ledger in `02-spec/` and `.ai-memory/plans/` | `DONE` |
| **Task-03** | Window TitleBar | Double-click header toggles maximize/unmaximize without drag-event swallow | `DONE` |
| **Task-04** | QuotaProgressBar | Reduce glow, shrink width by 10%, 4 equal-distance checkpoints, unified color | `DONE` |
| **Task-05** | AccountTable | Resizable Email column with interactive handle, `[220px, 550px]` bounds, persistence | `DONE` |
| **Task-06** | Status Pill Capsule | Compact `IDLE` status badge and `Log` button into single unified 28px pill | `DONE` |
| **Task-07** | PromptTreeView Toolbar | Compact connected button group, unified Send/Enqueue dropdown, New Conversation dialog | `DONE` |
| **Task-08** | RCA Documentation | Root cause analysis for historical prompt send & enqueue failure resolution | `DONE` |
| **Task-09** | Fleet Sync Engine | Pure JSON telemetry across Telegram, Email, and Supabase for remote fleet discovery | `DONE` |
| **Task-10** | Release Ceremony | Minor version bump (`v4.207.0`), changelog, README sync, and atomic GitMap push | `DONE` |
