# Plan: Superbase Information Removal & Git History Purging
- **Task Slug:** `superbase-information-removal`
- **Parent Task ID:** `1`
- **Database:** `.ai-memory/temp-agents/02-superbase-information-removal/agent-task.db`
- **Status:** `IN_PROGRESS`
- **Total Budget:** 300 steps
- **Lead Orchestrator:** Antigravity AI Agent
---
## User Request (Verbatim)
# Execution Plan: Superbase Information Removal
**Slug:** `superbase-information-removal`  
**Parent Task ID:** 1  
**Status:** ACTIVE  
**Step Budget:** 300  
**Created:** 2026-10-13  
## 1. User Request (Verbatim)

```text
# Superbase Information Removal: high priority instruction, non-negotiable task

Can you please check the recent few commits and anywhere if we have any Superbase information that is committed? If there is, then make sure this is actually fixed and correctly removed. And also at the same time, the previous files would be replaced with the current files in the Git history, so that the history never remains there. They cannot find the trace of the Superbase credential. Do you understand? All the credentials must be saved inside the repo secrets properly with a single liner to import and things like that. Is it clear?

## slug: superbase-information-removal

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/superbase-information-removal/ and enqueue plan task in .ai-memory/plans/superbase-information-removal.md (subtasks in .ai-memory/plans/subtasks/superbase-information-removal/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Check recent commits for any Superbase information and ensure it is removed
7. Replace previous files with current files in Git history to remove traces of Superbase credentials
8. Save all credentials inside repo secrets with a single liner for import

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing the work to reduce the credits.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Only add the relative paths, never add the absolute path during your work; this should be respected on the release page and in release notes as well.
```

---

## Core Rules & Invariants Cited
- **R1 Zero Builds or Test Suites:** CI executes full builds.
- **R3 Evidence or It Did Not Happen:** Every completion verified with command exit codes and diffstat.
- **R5 Mandatory Subagents:** Spawned Worker 01 and Worker 02 via `invoke_subagent`.
- **R6 One Owner Per File:** Disjoint file assignment per worker.
- **R8/R9 Atomic Commit & Push via GitMap:** Finish with single `gitmap cpf` call using hyphen format.
- **R11 Strict Relative Git Paths:** No absolute paths, no `file:///` URIs.
- **R16 Zero Secrets in Standard Repos:** Remove all sensitive API keys and URLs; offload to `repo-secrets`.
---
## Task Breakdown
### Task-01: Author Application Specification & Master Plan Ledger
- **Status:** `IN_PROGRESS`
- **Assigned:** Lead Orchestrator
- **Scope:** Author architecture spec `02-spec/21-app/superbase-information-removal/01-architecture-spec.md`, component spec `02-spec/21-app/superbase-information-removal/02-component-spec.md`, master plan ledger `.ai-memory/plans/superbase-information-removal.md`, and modular subtasks under `.ai-memory/plans/subtasks/superbase-information-removal/`.
### Task-02: Codebase & Working Tree Credential Sanitization
- **Status:** `PENDING`
- **Assigned:** Worker 01
- **Scope:** Replace hardcoded credentials in `scripts/supabase-endpoints.json`, `scripts/supabase-accounts.json`, `scripts/supabase-setup.ps1`, `scripts/setup-supabase.ps1`, documentation, and tests with sanitized placeholders.
### Task-03: Git Commit History Audit & Secret Purging
- **Assigned:** Worker 02 / Lead
- **Scope:** Audit commits touching Supabase credentials (`f33a0212`, `33000649`, etc.) and rewrite history so previous files are replaced with current sanitized files with zero trace remaining in Git history.
### Task-04: Repo Secrets Ingestion & Single-Liner Import Provisioning
- **Scope:** Save actual credentials inside `repo-secrets` and create `scripts/import-supabase-secrets.ps1` with a clean one-liner command to import credentials at runtime without committing secrets.
### Task-05: Verification, Index Registration, and Atomic Push via GitMap
- **Scope:** Run secrets gate (`gitmap aum search`), update index files (`02-spec/21-app/readme.md`, `.ai-memory/plans/readme.md`), and execute atomic push via `gitmap cpf "security - sanitize supabase credentials and purge git history"`.
## 2. Identified Credentials to Eradicate
All occurrences of the following sensitive patterns must be eradicated from tracked files and Git history:
1. `sb_publishable_[REDACTED_LOVABLE_KEY]`
2. `sb_publishable_[REDACTED_PERSONAL_KEY]`
3. `https://[REDACTED_LOVABLE_REF].supabase.co`
4. `https://[REDACTED_PERSONAL_REF].supabase.co`
5. Associated project reference IDs
## 3. Subsystem Breakdown & Subtasks
### Subtask 01: Repo-Secrets Vault Storage & Single-Liner Import Protocol
- **File:** `.ai-memory/plans/subtasks/superbase-information-removal/01-repo-secrets-vault-and-oneliner.md`
- Persist sanitized credentials in `d:/work/repo-secrets/` under `02-antigravity-manager/vault/` and `03-supabase/`.
- Provide zero-hardcoding PowerShell one-liner (`agm supabase load-secrets; agm supabase test`) and companion script `scripts/import-supabase-credentials.ps1`.
### Subtask 02: Source Code, Scripts & Test Fixtures Sanitization
- **File:** `.ai-memory/plans/subtasks/superbase-information-removal/02-source-and-scripts-sanitization.md`
- Sanitize `src-tauri/src/modules/supabase_sync/config_a.rs`, `src-tauri/src/modules/json_envelope/tests.rs`.
- Sanitize `scripts/verify-supabase-and-ui-ux.ps1`, `scripts/setup-supabase.ps1`, `scripts/supabase-setup.ps1`, `scripts/supabase-endpoints.json`, `scripts/supabase-accounts.json`.
### Subtask 03: Specs, Memory & Changelogs Sanitization
- **File:** `.ai-memory/plans/subtasks/superbase-information-removal/03-specs-memory-and-changelogs-sanitization.md`
- Sanitize `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-architecture-spec.md`.
- Sanitize `.ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md` and related subtasks.
- Sanitize `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`, `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`, `changelog.md`, and `changelog_en.md`.
### Subtask 04: Git History Rewriting & Trace Eradication
- **File:** `.ai-memory/plans/subtasks/superbase-information-removal/04-git-history-rewriting.md`
- Amend commit `089d3fa2` to replace all leaked credentials with clean sanitized code.
- Verify that `git log -p` across recent commits contains 0 occurrences of secret strings.
### Subtask 05: Zero-Secret Verification Gate & Force-Push
- **File:** `.ai-memory/plans/subtasks/superbase-information-removal/05-zero-secret-verification-and-push.md`
- Execute repository-wide `gitmap aum search` secret scans.
- Run `scripts/verify-supabase-and-ui-ux.ps1` to ensure functionality remains green.
- Force push sanitized commit chain to remote tracking `origin/main`.
