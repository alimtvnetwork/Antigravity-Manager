---
id: subtask-02-delete-github-releases-and-remote-tags
task_id: Task-02
owner: Worker 02 / Operations Lead
status: READY FOR EXECUTION
files_owned:
  - .ai-memory/plans/subtasks/git-tag-and-release-page-management/02-delete-github-releases-and-remote-tags.md
---

# Subtask 02: Delete GitHub Releases and Remote Tags

## 1. Context & Objective
The goal of this subtask is to execute the operational removal of obsolete GitHub Release entities and remote Git tags from repository `alimtvnetwork/Antigravity-Manager`. 

This subtask processes the verified candidates specified in `purge-manifest.json` (produced in Subtask 01), enforcing pre-flight safety locks, the strict two-stage deletion sequence (Release entity first, remote Git tag second), and GitMap CLI fallback ladder protocols.

---

## 2. Tooling Analysis & GitMap Fallback Ladder

GitMap (`v6.563.0`) does not provide native tag or release pruning verbs (such as `gitmap tag delete` or `gitmap release delete`). Therefore, operations must follow the structured CLI fallback ladder:

```
Step 1: Probe GitMap native commands (records unsupported verb state in v6.563.0)
Step 2: Authenticated GitHub CLI Release Removal:
        gh release delete <tag> --yes
Step 3: Authenticated Git Remote Ref Deletion:
        git push origin --delete <tag>
```

---

## 3. Pre-Flight Safety Assertion (Double-Lock Guard)

Before executing any deletion command against a tag, the execution worker MUST assert the following invariant condition:

```python
def verify_safety_assertion(tag: str, semver_str: str) -> None:
    # Immunity Whitelist Guard
    whitelist = {"v4.190.0", "v4.206.0", "v4.225.0", "v4.226.0"}
    is_whitelisted = tag in whitelist or f"v{semver_str}" in whitelist
    if is_whitelisted:
        raise RuntimeError(f"ABORT: Attempted to delete whitelisted tag: {tag}")
    
    # Historical Floor Guard
    parsed = parse_semver(semver_str)
    is_historical = (parsed.major < 4) or (parsed.major == 4 and parsed.minor <= 160)
    if is_historical:
        raise RuntimeError(f"ABORT: Attempted to delete historical tag <= 4.160.0: {tag}")
```

If either check triggers, the entire deletion loop terminates immediately with a non-zero exit code.

---

## 4. Execution Sequence & Protocols

### Phase 1: Ingest & Validate Purge Manifest
1. Load `.ai-memory/reports/git-tag-and-release-management/purge-manifest.json`.
2. Confirm manifest repository targets `alimtvnetwork/Antigravity-Manager`.
3. Assert that `preserved_whitelist_count == 4`.
4. Enumerate `purge_targets` array.

### Phase 2: Tier 1 - Purge GitHub Releases (Release-First Invariant)
For each target in `purge_targets` where `has_github_release == true`:
1. Execute pre-flight assertion check.
2. Delete the GitHub Release entity:
   ```bash
   gh release delete "<tag_name>" --yes --repo alimtvnetwork/Antigravity-Manager
   ```
3. Record deletion status and timestamp into `.ai-memory/reports/git-tag-and-release-management/audit-log.json`.
4. Pause for 200ms to maintain rate limit compliance.

### Phase 3: Tier 2 - Purge Remote Git Tags
For each target in `purge_targets` where `is_remote_tag_present == true`:
1. Execute pre-flight assertion check.
2. Delete the remote reference on `origin`:
   ```bash
   git push origin --delete "<tag_name>"
   ```
3. Record remote ref deletion status and response in `audit-log.json`.
4. Pause for 200ms.

---

## 5. Rate Limiting & Error Recovery

- **GitHub Secondary Rate Limits:** To prevent HTTP 403 / 429 throttling during large batch operations, a fixed sliding pause of 200ms is enforced between successive `gh release delete` calls.
- **Network Interruption Recovery:** Because each deletion step is atomic and idempotently logged in `audit-log.json`, a interrupted run can be resumed immediately without re-processing already-deleted tags.
- **Non-Existent Target Handling:** If a tag ref or release entity is already absent (e.g. HTTP 404 or ref does not exist), the error is recorded as `already_deleted` and execution continues to the next candidate.

---

## 6. Verification & Quality Gates

At the conclusion of Subtask 02:
1. **Whitelist Verification:** Query `gh release view <tag>` and confirm HTTP 200 for all 4 whitelisted versions:
   - `v4.190.0`
   - `v4.206.0`
   - `v4.225.0`
   - `v4.226.0`
2. **Floor Verification:** Confirm that `v4.160.0` remains intact on both remote and GitHub Releases.
3. **Purge Verification:** Query remote tags to confirm that all items in `purge_targets` have been purged from `origin`.
4. **Audit Log Completeness:** Confirm that `.ai-memory/reports/git-tag-and-release-management/audit-log.json` contains verified records for all executed deletions.
