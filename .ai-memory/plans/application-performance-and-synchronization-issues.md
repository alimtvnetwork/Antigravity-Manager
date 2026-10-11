# Execution Plan: Application Performance & Synchronization Issues

**Slug**: `application-performance-and-synchronization-issues`  
**Parent Task ID**: 1  
**Budget**: 300 steps  
**Status**: ACTIVE  

---

## User Request (Verbatim)

```text
# Application Performance and Synchronization Issues: high priority instruction, non-negotiable task

This list view, make sure we have the exe path that is visible in that column. I mean, the ending path will be enough, exe path. And also, make sure that there is no code that actually swallows the error. And there are some glitches, like when we try to open, let's say IDE or Switch, it takes a long time. And this is a process-to-process communication, that is all right. But sometimes the application gets stuck. So can you please check that part, actually, why the application gets stuck? That's one thing. Another is that your account needs to be synchronized, even if we don't have the super base, using the email or the Telegram. So make sure that you pass something as a JSON, as an instruction, so that we understand which machine is using which, let's say, instance, which account, which Id, which workspace, things like that. So using the internal database concept and other options, you would understand what is happening in each machine, so that as a summary, you can know it from the top level.

## slug: application-performance-and-synchronization-issues

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/application-performance-and-synchronization-issues/ and enqueue plan task in .ai-memory/plans/application-performance-and-synchronization-issues.md (subtasks in .ai-memory/plans/subtasks/application-performance-and-synchronization-issues/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Ensure the exe path is visible in the list view column and check for any code that swallows errors.
7. Investigate and resolve application glitches and performance issues, particularly when opening IDE or Switch.
8. Synchronize accounts using email or Telegram, passing JSON instructions to track machine usage, instances, accounts, IDs, and workspaces.
9. Utilize internal database concepts to monitor and summarize machine activities at a top-level view.
```

---

## Technical Strategy

1. **Executable Path in Instance Table**:
   - Update `src/components/instances/InstanceTable.tsx` and `src/components/instances/table/InstanceRowCells.tsx` to display the executable path cleanly alongside data path or as a dedicated column/badge.
2. **Error Swallowing Audit**:
   - Replace bare `let _ =`, `.ok()`, `.unwrap_or_default()`, and silent frontend catches with `AppError` logging (`record_ignored` with justification) and `useErrorStore.getState().trackWarning`.
3. **Application Stalls / Glitches**:
   - Refactor `wait_for_instance_prompt_channel` to avoid blocking synchronous thread sleeps (6000ms delay) during IDE launch.
   - Guard against synchronous mutex starvation in process table caching.
4. **Decentralized Machine Synchronization**:
   - Implement `FleetSync` JSON messaging protocol (`agm-fleet-sync-v1.0`) over Email and Telegram.
   - Maintain internal SQLite `fleet_sync.db` recording remote machines, instances, accounts, and workspace prompts for top-level cluster visibility.
