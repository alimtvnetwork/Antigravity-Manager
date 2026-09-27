# Spec 65: Update-All JSON & Fleet Sync, CLI/UI Help Polish, and Live Prompt Backup/Restore Verification

> **/goal** Provide automated fleet update orchestration via `agm update all` / `update-all` / `ua` with zero-noise structured JSON communication for remote automation, upgrade CLI Part 2 and UI help typography to enterprise standards, verify live traceable prompt injection across active workspaces, and prove end-to-end prompt backup, restore, and re-injection with minor release v4.87.0.
> **/learn** When automating fleet updates across remote machines over SSH or machine agents, standard stdout must remain strictly parseable JSON without ASCII banners or ANSI noise. Visual terminal modes must provide high-contrast structured status cards. Prompt backup must snapshot all active projects in parallel to split SQLite DB, allowing clean re-injection upon restoration.

**Version:** 1.0.0
**Updated:** 2026-09-28
**Author:** MD ALIM UL KARIM (alim, devorg.bd@gmail.com, @aukgit, @alimtvnetwork)
**Status:** Approved & Grounded

---

## 1. User Request (Verbatim)

```text
is it done properly???



Okay. Currently, most of the things are working fine. I think it's good, but I think I do have some requests that you could modify. First of all, the gitmap update all. It could be hyphen all or space all or UA. This would do the same thing. But now coming to the point how it's going to do it, the update is a delicate situation. So if we are running from the external machine, we should actually use JSON as a communication so that the help text does not get included in it. And currently, the way that you are doing the formatting or displaying it looks very poor. Okay? So I think this is where we need to work on. We need to make sure the help and things, these are really high quality. Okay? So currently, the output of the part two, how you're doing it, it's quite poor. Okay? And this is where we need to work on. And also make sure that, at the end, you bump the minor version and make a release and check the Gitmap PE. Also now I wanted you to check, so you can even run a test command on, let's say, scripts picture I or something like this. And you check by the running prompts LS that this project is actually running and other projects, but this project needs to be there. Or you could do other stuff like the own project. Let's say the status, alien status sample, just put a hi there and check if this comes up in the running prompt when it is running or put some weight. Okay? Like sleep and then say hi. You can do that so you can trace back. So that's one thing. Another is backing up the running prompts. So each one of the prompts from all the projects, the running prompts would be backed up. Then you can restore the prompt, running prompt, that should inject and run the prompt that we have taken the backup. So make sure that you do the end-to-end testing so that you can confirm it, that it's working very well and there is no confusion. Okay? It's been several times that we are trying that. I hope there should be no issues, right? If possible, you can also test out with some file the Gitmap SC or deploy command, sync left, sync right command. Okay. And also update the help text and also UI help text. These are very important. Do not miss it. Is it clear? Can you please do that? At the end, do a minor bump and a release and do the Gitmap PE to check the errors. Okay.
```

---

## 2. Technical Requirements

### 2.1 Unified `agm update [all]` / `update-all` / `ua` Command
- In `src-tauri/src/bin/agm.rs`:
  - Route `"update" | "update-all" | "ua" => cmd_update(&cmd_args)`.
  - Handle flags:
    - `--json` / `-j`: Output ONLY pure parseable JSON without any banners, headers, or conversational stdout noise.
    - `all` / `--all` / `-a`: Run full fleet update (AGM binary check + local git pull if applicable + local instances sync).
    - `--check` / `-c`: Dry-run query to inspect release without modifying files.
    - `--force` / `-f`: Force reinstallation even if already latest.
  - Interactive Terminal Mode:
    - Render a formatted, high-contrast status card detailing binary target, current version, latest release, update status, workspace repo status, and fleet node info.

### 2.2 CLI Part Two (`print_help`) & UI Help Polish
- In `src-tauri/src/bin/agm.rs`:
  - Restructure `print_help()` into clean visual command cards with distinct boxed headers, aligned syntax columns, and clear descriptions.
  - Support `agm help --json` / `agm --help --json` returning structured command schema for automation tools.
- In `src/pages/Settings.tsx` & frontend UI:
  - Add contextual help panels and tooltips for CLI commands, auto-switch thresholds (15% production standard vs 98% simulation), and instance isolation.

### 2.3 Live Prompt Injection & Active Tracking
- Dispatch a live test prompt (e.g. `agm prompt "Status probe: sleep 2 && echo hi"`) with project tracking.
- Verify that `agm prompts ls` / `agm running-prompts ls` reflects the active project and prompt snippet in real time.

### 2.4 Multi-Project Parallel Prompt Backup & Restoration
- Snapshot running prompts across all active workspaces via `agm backup`.
- Verify storage in split SQLite DB (`backup-prompts.db`) with human-friendly project directory names (`Antigravity-Manager`, `gitmap`) and zero UUID hashes.
- Restore and re-enqueue via `agm restore`, confirming active execution and resume task file updates.

### 2.5 GitMap SC / Fleet Synchronization
- Verify GitMap commands (`gitmap sc`, `gitmap sync`, cluster node status).

### 2.6 Minor Version Bump (v4.87.0) & CI/CD Verification
- Bump minor version to `v4.87.0` across all 14 project manifests.
- Pre-flight checks (`cargo fmt`, `cargo clippy`, `npm run build`).
- Atomically commit with contributor attribution: `alim, devorg.bd@gmail.com, @aukgit`.
- Push tag `v4.87.0` and monitor CI via `gitmap pe -t` to 100% green.

---

## 3. Acceptance Criteria & Test Matrix

- **AC-65-001 (Update-All JSON & Interactive Mode)**: `agm update all --json` emits pure parseable JSON; `agm update all` renders a high-quality visual status card.
- **AC-65-002 (CLI Part 2 & UI Help Polish)**: `print_help()` displays aligned boxed sections; UI includes contextual help guide.
- **AC-65-003 (Live Prompt Tracking)**: Injected prompt appears immediately in `agm prompts ls` under the current project.
- **AC-65-004 (Prompt Backup & Restore Verification)**: All running prompts are backed up to SQLite and successfully restored/re-injected.
- **AC-65-005 (GitMap SC / Fleet Verification)**: `gitmap sc` and sync commands operate without errors.
- **AC-65-006 (Minor Release v4.87.0 & CI Green)**: Manifests bumped to `4.87.0`, pre-flights pass, tag `v4.87.0` pushed, and CI passes 100% green.
