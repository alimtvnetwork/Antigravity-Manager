# Spec 66: Account Switch 98% Simulation E2E, Running Prompts Parallel Backup/Restore, Multi-VM Collision Shielding & Sandbox Lifecycle Verification

## 1. User Request (Verbatim)

```text
I think for testing purpose, you need to test it, end-to-end test, and confirm that it is working, and also make sure that all the commands that we have crafted for the CLI tool, that has nice help text, okay, with examples. Okay. So let's discuss about the account switch. Okay, you have done the account switch before. But then again, we are trying to do the re-verification one more time, okay? So what I want you to do is... Okay, the way that this is going to work is that first you're going to put the threshold for the account switch at 98%. Okay? And then you use some token or use some credits, add some test prompts to somewhere, okay, so that it starts running. Okay. So once the credit goes less than 98%, it switches. Okay? But before it switches, you know the algorithm. The algorithm is just, once we found the right fit, the top one, we check the refresh. We do a refresh on the account, and if the refresh says the same credit remains, same number follows, then we select that. Okay? But again, we do not do anything yet. The first thing we should do is take a backup of that instance to SQLite database using the AGM. Take the prompts backup, the running prompts, actually. Running prompts only. So you took a backup, let's say. And then, so you need to probably add a running prompts command, like running prompts export or backup and restore, something like this. So it does the restoring, I mean, backuping first. So it should be very small, one to one prompts that will be saved to the database, parallely. So you should try to do it parallely if multiple projects are running. Multiple projects, that single prompt will be saved to the database along with the picture information, picture path, and most of the things that these are attached, it needs to be there, synced. And then when we found the account we want to switch, we do the fast-forward button click. That means dedicate that to fast-forward button. Fast-forward button usually just switches the account, right? So it will go to that account, click on the appropriate button to delegate the call to that appropriate button to refresh the IDE with the new account. Up to this part should be very clear. We have discussed this before. Now, it should restore the running prompt and reinject and see like it's running even using another AGM command, it will see commands are running or not. If not, it will notify in the Telegram. Telegram also should let us know the steps that we mentioned. It should also reveal that. That means it's going to take a backup. The backup means we mentioned like taking a backup of running prompts. So it would show like five projects or six projects, whatever the number is. Projects are getting into backup. Projects name can be there listed. No IDs, but the names of the projects. So once it's saved into the backup using the command, it will again restore the running prompts, and it would reinject, and this status also needs to be in the Telegram, also in the email altogether. Do you understand? Okay, write into the spec so that you never forget And also at the end, once all this testing is done. So first you code, because if you switch it, then the code running would be stopped. Okay? Make sure that you have a process that makes sure that the prompts are running. Okay? That is very, very important. Now, at the end, you would have some count that would give you all this information and finalize the process. So at the end, what you do after the testing and everything is done, you reduce the threshold to, let's say, 15%. Okay? Keep it under 15%, and then you make a minor bump and release, and check the CI/CD using GitLab EE licenT. That is all green. So that's it. That's what I'm expecting for. So you need to have this testing. Now, the same thing, you want to test it, end-to-end test in the instance mode. Or you could try to create a new instance and try to test this, that does this work on instance mode or not. And then finally you remove that instance. Okay? So you do the end-to-end testing here, no problem in the machine. You can try to do anything that you like. If anything goes wrong, I can revert this back. I have a snapshot. So you don't have to worry about this. Do you understand? Can you please follow through all this and then finally make a release? That means before you release, you also change the default threshold to 15%. Okay? And make sure that you have the proper email sending and also email checking. You need to check the email. If the email is connected, you need to check in the email if this user profile or the account is already selected by some other DM. If it is, then you need to skip and move to the next one. Remember that, that's a very crucial step I forgot to remind you. Also, if we have the super base, then it is very easy. Every one of the system will check who is not using the account and has the highest value by the algorithm you have to specify. Then it will just switch to this account. That is the standard process. Is it clear? Do you have any question and confusion?
```

## 2. Visual Asset Reference

![Fast-Forward Button & Account Switch](assets/screenshots/account-switch-fast-forward-e2e.png)

## 3. Architecture & Functional Requirements

### 3.1 High-Threshold 98% Simulation & Algorithm Validation
- **Simulation Threshold**: Configure `low_quota_threshold_percent = 98.0` via CLI flag `-t 98` or configuration override.
- **Candidate Evaluation & Freshness Probe**:
  1. The scoring algorithm ranks candidate accounts by 4-hour quota fraction descending, prioritizing full (100%) quota accounts.
  2. Before committing a switch, perform a direct live refresh probe against Google's quota API (`account::fetch_quota_with_retry`).
  3. If quota matches or confirms 100% full capacity, proceed; if depleted, immediately evaluate the next candidate.

### 3.2 Parallel Running Prompts Snapshot to Split SQLite (`backup-prompts.db`)
- **Parallel Scanning**: Use `std::thread::scope` to concurrently scan all active workspace state databases (`state.vscdb`) and resume task files (`.antigravity_resume_task.json`).
- **Human-Friendly Naming**: Extract clean project directory names (e.g. `Antigravity-Manager`, `gitmap`) eliminating all internal UUID hashes or machine ID noise.
- **Asset Synchronization**: Persist attached images, base64 data, and file paths into the split SQLite database `C:\Users\Administrator\.antigravity_tools\backup-prompts\backup-prompts.db`.
- **CLI Commands**:
  - `agm backup` / `agm backup ls` / `agm backup clean`
  - `agm restore` / `agm prompts ls` / `agm which-prompts-running`

### 3.3 Fast-Forward Button Delegation & Profile Rotation
- Delegate rotation to the fast-forward pipeline (`cmd_fast_forward` / `smart_rotate_account`).
- Safely cycle IDE process or re-bind credentials without data loss.
- Post-switch, automatically invoke `agm restore` to re-enqueue running prompts into the active workspace queue.
- Validate running status using `agm prompts ls` to confirm commands are active.

### 3.4 Multi-Stage Telegram & Email Telemetry
- **Step Reporting**: Broadcast explicit status cards showing:
  - Backed up projects (human names listed, e.g. `• Projects: Antigravity-Manager, gitmap`).
  - Total prompt counts secured.
  - Selected replacement profile and refresh probe verification.
  - Re-injected prompt count and live execution status.

### 3.5 Multi-VM Conflict Avoidance
- Evaluate candidate accounts against:
  1. Active Supabase Root DB distributed leases (`workspace_leases` table).
  2. Recent (3600s) IMAP switch broadcast events from sibling nodes.
- If an account is leased or actively claimed by another VM, cleanly skip it and evaluate the next candidate.

### 3.6 Sandbox Instance Lifecycle Testing
- Create an isolated test instance: `agm instances create test-sandbox-e2e`.
- Run rotation and prompt backup/restore inside `test-sandbox-e2e`.
- Verify instance process separation and prompt re-injection.
- Cleanly destroy and remove `test-sandbox-e2e` via `agm instances rm test-sandbox-e2e --force`.

### 3.7 Release Gate & Default Quota Standard (15.0%)
- After all E2E verifications pass, enforce `15.0%` as the production default threshold (`low_quota_threshold_percent: 15.0`, `threshold_percentage: 15`).
- Run pre-flight checks: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`.
- Bump version to minor (`v4.88.0`), update changelogs attributed to `alim, devorg.bd@gmail.com, @aukgit`, push tag, and monitor CI/CD (`gitmap pe -t`).

---

## 4. Acceptance Criteria & Test Matrix

- **AC-66-001 (Visual Asset Ingestion)**: Screenshot saved to `assets/screenshots/account-switch-fast-forward-e2e.png` and referenced.
- **AC-66-002 (98% Quota Simulation)**: `agm switch-if-low-credit -t 98` triggers candidate evaluation and live Google API refresh probe.
- **AC-66-003 (Parallel SQLite Snapshot)**: `agm backup` secures running prompts across workspaces with human names and image payloads.
- **AC-66-004 (Fast-Forward & Prompt Re-injection)**: Fast-forward rotates profile, restores in-flight prompts, and verifies execution via `agm prompts ls`.
- **AC-66-005 (Telemetry Multi-Stage Dispatch)**: Email and Telegram notifications report project names and prompt counts across backup and restoration.
- **AC-66-006 (Multi-VM Collision Shielding)**: Candidate selection skips accounts actively claimed in Supabase leases or IMAP broadcasts.
- **AC-66-007 (Sandbox Lifecycle Isolation)**: `test-sandbox-e2e` created, tested, and cleanly removed with zero residual processes.
- **AC-66-008 (15% Default & v4.88.0 Release)**: Default threshold confirmed at 15.0%, version bumped to `v4.88.0`, pushed, and verified 100% green via `gitmap pe -t`.
