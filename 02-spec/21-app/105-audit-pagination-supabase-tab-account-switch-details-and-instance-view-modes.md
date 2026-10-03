# Specification: 105 - Audit Pagination, Supabase Top-Level Tab, Account Switch Notification & Instance View Modes

## User Request (Verbatim)
```text
# High Priority Instruction

Okay. So far I understand that you made these CLI commands. That's all right. Can you please test in your machine then? First of all, I want you to take the super base credential from the repo secrets and include in your system. That's the first thing. And again, I wanted the super base in the same level as the email and audit, which I don't see. And also, every time the email or account is switched, it should mention a switched account, okay? And it should actually show an array from where to where it actually switched. The information is missing, and also in the details section, the missed information is very less. Okay? Check the email how we send it. It should be something like this when we go into the details section. Okay. That's the first observation in the status, in audit section. An audit section should have a pagination, right? So by default, it would show 200 or 100 items. So it would paginate and show the item from the SQLite databases, and it will also cache the first 100 data to cache DB so that it can be viewed very quickly. When in case goes more than that, it would bring the data from these hit DBs. Okay, if you do not understand, let me know. Now, the coming part, if I double-click on a row, it should automatically showcase the detail. Okay, that's the first thing. Also, clicking on the email should make the email visible. The another thing is the from and to, that should be the first thing. Okay? And then the action, then the status, then the time. Now, the time format, it should be double D, that means date, hyphen, triple M, that means the months two-digit format, in uppercase, and then finally the year, and then the time in terms of the minute and second. Okay. Yeah, second after second, whatever that is there, it's correct. Okay. Now, the fast forward or switch account needs to have more information in the details section, like which instance, which IDE, which IDC, okay? Where the path of the IDE. Okay, why it switched. So all kinds of detailed information needed in the details section. If we double-click on the details, we should be able to see that. The reasoning is most important. Now, if we go to the account section, okay, there are a few buttons that we saw, right? The buttons are focus, plus, refresh, and then warm, and then show all quotas, okay? I want that show all quotas to be taking out the masking of the email or taking out the masking or hiding the email with masking. So also at the same time, if I double-click on email row in the accounts tab, that would be, I mean, showcase the email. But if I take out my mouse, then it would be again masking details. Okay? Do you understand what I just said? So this is one of the vital information. Okay, and do you have any other component that could showcase the progress bar in a nicer, more nicer way or more, let's say, colorful way? Do you have any component? Can you please suggest that we can review it? Okay. Also, you have to confirm that when we do the cloning of the instance, the text box that shows up here, it does not have proper padding. Please confirm if you have fixed the padding. Okay. So, you have created these commands. I appreciate that. Have you tested the commands? You need to test these commands in this machine with writing some E2E tests, but also these E2E tests will be only local and only for this instance. That means this will not run through CI/CD or not ever run in the locally unless someone asks this to. Okay? That will be exceptional end-to-end test cases, so I want you to run this and verify each cases. And also in the settings, we should have a default path where we can enable rewrite on default path. Okay? Also, we can enable, I mean, by default, we can enable all URLs to be visited. Okay? We can also have a exclude URL section where we can comma-separated pass the URL that should be disallowed. Also, at the same time, all of these items needs to have a LS option, help option. Okay? LS help is very important and mandatory. Make sure we have it. And when we copy, if we do not provide a full path, but relative path or just the name, which we wanted to clone to, then it will automatically copy the, let's say, IDE instance or executable on the same directory we just The relative path or relative name. Remember that. And when you copy prompt, it should always keep the old settings and all the project information, and also it will try to bloom all the, let's say, conversation. Okay, so all conversation needs to be visible on those instances. So make sure of that. Okay. And also, from the UI level, when we go to the instance section, the instance section UI needs to have a compact view. For example, this type of card view needs to be compact, less, where we can also search for the email address, search for the, let's say, instances. And what do you mean by active? All the instances, if the instances are running, so it will check in every 10 minutes or five minutes, depending on what the setup is. If that is there, then it will also check the, let's say, credits. If the credits are out of sync or less than the amount on that email address or on that account for that instance, it will immediately do fast-forwarding on there. So you are not respecting that, which I have seen. Okay, so what do you mean by active? Okay. I don't understand. And also, in this instance profile section, there are a lot of buttons on top. Okay? This does not have proper padding, UI. It looks very bad. Try to make it clear. And also, every one of the items should have a view, like you could switch the view. One is card view, another is the list view. The list view would be same as the accounts view, something similar. Okay? Now it would have more options, more buttons. Now we should be able to see how many prompts or projects are running. We can just click on it, it will open a dialogue for that instance, and we can see how many, let's say, projects as enqueued prompts, and how many of those are running. Okay? Now what we could do is we can also do backup. The backup button should be there, a backup of this prompt, and we can also restore this prompt anytime we want. And again, I'm repeating, the restore functionality after the account actually restores or fast-forwards, you need to wait for 5, 10 seconds to restore the previous running prompt. Okay? I think that is not respected. You need to respect that, and you need to make sure that the running prompts is reinjected once and also send it now to that conversation, specific conversation. Remember that. So when we see the prompts or conversation on that instance, we should only see whatever was running or last 5 to 10 conversation based on the projects. So there should be a tree view left side of the project name and the conversations, and if we click on it, we will see how many, let's say, conversations or prompts are there as a queued item. So it should show as queue items. Okay? So we should be able to click and view the detail of the prompt. If we double-click, that would actually show up into another model or another view, full screen. That should also contain the images, if it contains images. So remember that. Images are very important. And also, if there is any file, it would showcase that file as well. So these are on top of my head instructions, and these should also be done all through the CLI as well. Should be able to see that, get the information as a JSON. So most of the commands that we see, like instance, instance copy, all we can also get the information using JSON flag or, yeah, JSON flag. So if you do that, then the instruction or communication will be in terms of JSON. Now, also I want you to include the Supabase into the database section and also test the Supabase things so that the things are all right. Okay? I also do not see the Supabase is connected yet, so make sure of that. And also, in the setting section, I think we should have a full backup. So test that backup, that backup actually gets to the backup state and can be restored fully. Okay? And also in the backup, we can choose to have the audits or you can skip the audits as well. Also, I want you to test the ADM update with zip capture, something like this. We've discussed. I don't think that you have tested this. This needs to be tested. Okay, and also the restore prompt. That is something that I'm requesting several times now. It's not respectful that you do not fix it. Okay? You need to take care of all of the instances when they are running, okay, because it runs from the tool, so you already know the PID, where that is, how that is. So you can close it, you can take a backup of that, all kinds of things. Okay? And from the instance section, when you do the LS, it should show all these things, PID, file path, what is the name of that sequence, all this thing. Then we can also see it in the JSON format as well. Remember that. Do you understand what I have said? Do you have any question and confusion? Can you please complete this, finally make a release bump, minor release bump, and release it? And also create a prompt at the end, AI verification prompt, which way I could verify all these tasks. Can you please do that for now?
```

## User Uploaded Media References
- `assets/screenshots/105-01-instances-card-running.png`: Instance Card with running PID, path, and action bar.
- `assets/screenshots/105-02-instances-toolbar-padding.png`: Toolbar buttons (`Running: 4/4`, `Refresh`, `Clean & Restart`, `Auto-Switch: ON`, `Eval Quota`, `+ New Instance`) needing padding and clean layout.
- `assets/screenshots/105-03-duplicate-modal-padding.png`: Duplicate / Clone Profile modal with input text box padding issues highlighted in red.
- `assets/screenshots/105-04-cli-command-summary.png`: CLI commands reference list (#1 to #6).

---

## 1. Executive Architecture Summary

### 1.1 Supabase Integration & Top-Level Tab Placement
- **Credentials Ingestion**: Ingest from `scripts/supabase-endpoints.json` into `%APPDATA%\antigravity-manager\supabase_config.json` and `~/.antigravity_tools/supabase_config.json`.
- **Top-Level Navigation**: Add `Supabase` alongside `Email` and `Audit` as a first-class navigation view in the main sidebar / top navigation bar, exposing sync status, nodes, leases, command queues, and connectivity health probes.

### 1.2 Audit Trail Overhaul: Pagination, Column Ordering, Time Formatting & Detail Modal
- **Pagination & Caching**: Implement default 100/200 item pagination from SQLite databases (`repo_db`). First 100 items cached in memory/cache DB for instant zero-latency rendering.
- **Column Order**:
  1. `From` & `To` (e.g. `[previous_account, target_account]`)
  2. `Action`
  3. `Status`
  4. `Time` (Formatted strictly as `DD-MMM-YYYY HH:MM:SS`, e.g., `03-OCT-2026 16:02:37`)
- **Interactive UX**:
  - Double-click on any row opens the full-screen / expanded detail modal.
  - Clicking on an email reveals/unmasks the address.
- **Enriched Rotation Details**: Full breakdown of `instance_id`, `ide_type`, `idc_machine_alias`, `ide_path`, `switch_reason`, credit thresholds, and prompt preservation state matching email notification format.

### 1.3 Accounts View Modernization & Email Masking/Unmasking
- **Toolbar Buttons**: Segmented, padded button group: `Focus`, `+` (Add Account), `Refresh`, `Warm`, and `Show All Quotas` (unmasks emails across all rows).
- **Hover / Double-Click Unmasking**: Double-clicking an email temporarily unmasks it; `onMouseLeave` immediately reverts to masked display (`maru***@gmail.com`).
- **Colorful Progress Bar Component**: Provide dynamic multi-stop gradient progress bars (iridescent emerald-to-cyan-to-indigo, glowing animated pulse).

### 1.4 Instances Page: Compact Mode, Card vs. List View Toggle & Search
- **View Modes**: Switchable between Card View and List View (similar to Accounts tabular view).
- **Search**: Real-time filtering by instance name, bound email, PID, and status.
- **Toolbar Styling**: Generous padding (`px-4 py-2.5`), segmented pill grouping for `Running: X/Y`, `Refresh`, `Clean & Restart`, `Auto-Switch`, `Eval Quota`, and `+ New Instance`.
- **Clone Modal Input Padding**: Fix cramped input box in `CloneProfileModal` / `Instances.tsx` duplicate dialog (`px-4 py-3`, rounded-xl, font-medium).
- **Telemetry Display**: Explicitly show PID, executable path, profile directory, and sequence order across both card and list views.

### 1.5 Prompt Preservation & Instance Tree View Drawer
- **5-10 Second Stabilization Delay**: Enforce strict `tokio::time::sleep(Duration::from_secs(7))` delay before injecting `.antigravity_resume_task.json` post-rotation to ensure the IDE WebSocket channel is ready.
- **Project Tree View Dialog**:
  - Left panel: Tree view of project folders and recent conversations.
  - Right panel: Queued prompts and running prompts.
  - Double-click prompt: Opens rich inspection modal rendering text, file attachments, and image previews.

### 1.6 Settings & Proxy Enhancements
- **Default Path Rewrite**: Configurable default path rewrite toggle.
- **URL Whitelisting / Blacklisting**: Enable all URLs by default; comma-separated excluded/disallowed URLs list.
- **Full Backup & Restore**: Full system backup archive (`gui_config.json`, instances, accounts, with optional toggle to include or skip audit logs).

### 1.7 CLI Enhancements & Local E2E Test Suite
- **Global `--json` Flag**: Ensure all `agm instance`, `agm accounts`, and `agm audit` commands output JSON.
- **Mandatory `ls` and `--help` Coverage**: Comprehensive help tables and command alias coverage.
- **Local-Only E2E Test Runner**: Dedicated test script `scripts/e2e-instance-commands-test.ps1` that exercises all commands on the machine without touching CI/CD.

---

## 2. Deliverables & Verification Checklist
- [x] Supabase credentials loaded into system config.
- [ ] Top-level Supabase tab added to navigation.
- [ ] Audit pagination (100/200 items) + cache DB.
- [ ] Audit column order: From/To, Action, Status, Time (`DD-MMM-YYYY HH:MM:SS`).
- [ ] Audit row double-click modal & enriched switch metadata.
- [ ] Accounts email hover unmasking & toolbar buttons (`Focus`, `+`, `Refresh`, `Warm`, `Show All Quotas`).
- [ ] Colorful progress bar options.
- [ ] Duplicate modal input text box padding fix.
- [ ] Instances Card View vs. List View toggle & search filter.
- [ ] Instance prompt tree view drawer with queued/running status & image previews.
- [ ] 5-10s prompt restoration delay guarantee.
- [ ] Settings default path rewrite & excluded URLs.
- [ ] Backup & restore with optional audit inclusion.
- [ ] Local E2E test script for CLI commands.
- [ ] Minor release bump & AI verification prompt.
