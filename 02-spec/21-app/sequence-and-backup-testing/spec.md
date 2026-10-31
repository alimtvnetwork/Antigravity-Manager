# Sequence and Backup Testing Specification

## 1. Metadata & Identification
- **Title**: Sequence and Backup Testing: Prompt Tree Sorting, Standalone Image Pipeline, Backup/Restore Verification, and Multi-Machine SSH Instance-Account Mapping
- **Slug**: sequence-and-backup-testing
- **Author**: @aukgit
- **Created**: 2026-10-19
- **Status**: Draft / Active Verification
- **Target Subsystems**:
  - `src/components/instances/PromptTreeView/` (Tree hierarchy, sorting, context menu, image clipboard)
  - `src-tauri/src/modules/repo_db/` (Active prompts, backup database, restore transactions)
  - `src-tauri/src/modules/ssh_manager/` (Multi-machine SSH mesh, instance detection, account discovery)
  - `src/components/settings/EmailNotifications/SshGitmapSection.tsx` (Shared cluster instance & account UI)

---

## 2. User Request (Verbatim)

```markdown
Can you please confirm that you can sequence the proper tree view with the most recent prompt and the queued prompts and then the others, and also the image from the prompts can be copied, imported, exported as a standalone item? Please confirm that. This is something we are struggling for quite some time. And also the restoring prompt, which I have asked several times, backing up and restoring, which does not work. So you need to test it very carefully. You need to queue some tested prompt. You need to back up. So when you have the tested prompt, try to take a screenshot of the ``Id`` and try to add it there so that you can check if the screenshot can be get back and not. So do this end-to-end testing and also two machines are there. Try to make sure the SSH config and everything can be connected to the SSH. I should know how many instances are running, each one of the instances which email is connected there. Please make sure that you fix it properly.

## task: Sequence and Backup Testing
## slug: sequence-and-backup-testing

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work — first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/sequence-and-backup-testing/ and enqueue plan task in .ai-memory/plans/sequence-and-backup-testing.md (subtasks in .ai-memory/plans/subtasks/sequence-and-backup-testing/) first
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Confirm the sequence of the proper tree view with the most recent prompt and queued prompts.
6. Ensure images from prompts can be copied, imported, and exported as standalone items.
7. Test the backup and restore functionality thoroughly.
8. Queue some tested prompts and take a screenshot of the ``Id`` to verify if the screenshot can be retrieved.
9. Conduct end-to-end testing on two machines, ensuring SSH configuration and connectivity.
10. Verify the number of running instances and the associated email for each instance.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing the work to reduce the credits.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Only add the relative paths, never add the absolute path during your work; this should be respected on the release page and in release notes as well.
```

---

## 3. Architecture & Functional Requirements

### 3.1 Prompt Tree View Sequencing Invariant
The prompt tree view MUST maintain a deterministic 3-tier ordering hierarchy across both projects, conversations, and individual prompt turns:
1. **Tier 1 (Execution Primacy - Top)**: `RUNNING` prompts. Any conversation with an active execution step or in-flight prompt turn MUST be pinned at the absolute top of the tree view.
2. **Tier 2 (Queued Work - Second)**: `QUEUED` / `PENDING` prompts. Conversations with queued turns must display an amber `QUEUED` badge with clock icon and appear immediately below running items.
3. **Tier 3 (Historical Records - Descending)**: Completed, idle, or cancelled conversations must be sorted strictly by `last_modified` / `timestamp` in descending order (`ORDER BY last_modified DESC`).

### 3.2 Standalone Image Lifecycle (Copy, Export, Import)
- **Direct Base64 Decoded Blobs**: Images attached to prompts must be read via `read_image_as_base64` in `src-tauri/src/commands/cmd_files.rs` and decoded into binary PNG Blobs in `src/components/instances/PromptTreeView/clipboard.ts`, bypassing WebView2 canvas CORS tainting.
- **Standalone Export**: Prompt JSON exports must use the v2.0 JSON envelope format with portable base64 embedded image payloads and repository-relative paths (`${workDir}`).
- **Standalone Import**: Importing a standalone prompt bundle must reconstitute images into the target workspace cache without path breakage.

### 3.3 Prompt Backup & Restoration Subsystem
- **Queue Seeding**: Prompts enqueued via CLI (`agm prompt queue <text>`) or UI must insert into `active_prompts` with status `queued`.
- **Backup Verification**: `agm prompt backup` must write the snapshot into `prompt_backups` with complete metadata, including image references.
- **Restore Verification**: `agm prompt restore` must recover backed-up prompts in strict FIFO order (`ORDER BY created_at ASC, id ASC`), marking them as ready for dispatch and re-populating the Prompt Tree View.

### 3.4 Multi-Machine SSH Fleet & Account-to-Instance Mapping
- **2-Machine Cluster Configuration**: Validate SSH connectivity between Machine 1 (`w1`, local) and Machine 2 (`w2`, remote) using native SSH config (`~/.ssh/config`) and GitMap vaulted credentials.
- **Instance Inventory**: Query real active instances and process vitality on both nodes via `get_cluster_instances_and_balances`.
- **Account Binding**: Accurately display each running instance along with its bound Google account email (or explicit `unbound` tag), eliminating duplicate or misattributed local entries.

---

## 4. Verification & Testing Matrix

| Requirement | Test Method | Expected Outcome | Pass/Fail Gate |
|---|---|---|---|
| **Tree Sequencing** | Component / Unit test in `conversationGuards.ts` & `ProjectNode.tsx` | Running on top, Queued second, timestamp desc third | Strict equality check |
| **Image Copy** | Browser / Node unit test on `base64ToBlob` | Binary Blob generated with image/png MIME | Non-zero Blob bytes |
| **Image Export/Import** | JSON roundtrip test on v2.0 envelope schema | Image base64 matches original hash | Bit-for-bit parity |
| **Backup & Restore** | End-to-end PowerShell test script | Queued prompt backed up & restored cleanly | DB row exists with identical prompt & screenshot ID |
| **SSH 2-Machine Fleet** | Integration test on `get_cluster_instances_and_balances` | Reports instance count + email for `w1` and `w2` | Connected status with valid telemetry |
