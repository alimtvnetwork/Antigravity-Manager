# Component & Process Specification: Git Tag and GitHub Release Page Management

> **/goal** Define the component lifecycle, pruning algorithms, verification matrix, and native GitMap command parity for purging excess Git tags and GitHub Release entries while preserving historical releases and a strict 4-version whitelist.
> **/learn** Read this specification alongside `02-spec/21-app/git-tag-and-release-page-management/01-architecture-and-api-spec.md` before executing local tag pruning, verification audits, or GitMap task cache registrations.

**Version:** 1.0.0  
**Updated:** 2026-10-18  
**Status:** Active  
**AI Confidence:** Production-Ready (100%)  
**Ambiguity:** None (0%)  

---

## 1. Overview & Operational Scope

This specification defines the components, verification pipelines, and future CLI command architecture for managing Git tags and GitHub Releases across the `alimtvnetwork/Antigravity-Manager` repository.

Over rapid iteration cycles, numerous preview and intermediate tags (spanning versions v4.161.0 through v4.226.0) were generated and published to remote GitHub Releases. To restore repository hygiene, optimize clone speeds, and ensure an uncluttered GitHub Releases page for downstream users and package managers, this specification establishes:

1. **Cutoff Boundary**: All tags and releases created at or below version `v4.160.0` (SemVer `<= 4.160.0`) are permanently preserved and remain untouched.
2. **Strict Whitelist**: Exactly four post-4.160 versions are designated as long-term retention anchors:
   - `v4.190.0`
   - `v4.206.0`
   - `v4.225.0`
   - `v4.226.0`
3. **Local Tag Pruning Engine**: Systematic deletion of candidate local tags without polluting remote tracking or modifying branch history.
4. **Three-Tier Verification Matrix**: Multi-source validation across local Git repository storage, remote Git references via `git ls-remote`, and the GitHub REST API via GitHub CLI (`gh release list`).
5. **GitMap Future Parity Architecture**: Standardized design for native `gitmap release prune` and `gitmap tag prune` CLI commands driven by task definitions stored in `.gitmap/cache/tasks/`.

---

## 2. Retention Logic & Version Classification

The version classification engine operates on SemVer strings parsed with prefix normalization.

### 2.1 Version Classification Rules

```
                      ┌───────────────────────────┐
                      │    Incoming Tag/Release   │
                      └─────────────┬─────────────┘
                                    │
                         Parse SemVer (vX.Y.Z)
                                    │
                                    ▼
                      ┌───────────────────────────┐
                      │  Is Version <= 4.160.0?   ├───────► [PRESERVE: HISTORICAL]
                      └─────────────┬─────────────┘         (Permanent Freeze)
                                    │ No
                                    ▼
                      ┌───────────────────────────┐
                      │   Is Version in Strict    ├───────► [PRESERVE: WHITELIST]
                      │ Whitelist (4 Versions)?   │         (v4.190.0, v4.206.0,
                      └─────────────┬─────────────┘          v4.225.0, v4.226.0)
                                    │ No
                                    ▼
                      ┌───────────────────────────┐
                      │    [PRUNE CANDIDATE]      │
                      │ Delete Remote & Local Tag │
                      └───────────────────────────┘
```

### 2.2 Boolean Invariants & Field Standards

Per coding guidelines, all evaluation logic uses positive booleans exclusively:

| Property Name | Type | Description |
|---|---|---|
| `is_historical_preserve` | `bool` | Evaluates to `true` when version is less than or equal to `4.160.0`. Protected from all prune routines. |
| `is_whitelisted` | `bool` | Evaluates to `true` when version exactly matches one of `v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`. |
| `is_above_cutoff` | `bool` | Evaluates to `true` when version is strictly greater than `4.160.0`. |
| `is_prune_eligible` | `bool` | Evaluates to `true` when `is_above_cutoff` is `true` AND `is_whitelisted` is `false`. |
| `is_retained` | `bool` | Evaluates to `true` when either `is_historical_preserve` or `is_whitelisted` is `true`. |
| `is_verified_absent` | `bool` | Evaluates to `true` when tag/release has been confirmed absent from the target query output. |

---

## 3. Component Architecture

### 3.1 Component 1: Local Tag Pruning Engine

#### Responsibilities:
- Enumerates local tags stored under `.git/refs/tags/` and `.git/packed-refs`.
- Computes the intersection of local tags with the prune candidate list.
- Executes local tag deletion via `git tag -d <tag>` in ordered batches.
- Ensures zero modification to Git commits, branches (`main`, `beta`), or stash states.

#### Execution Algorithm:
1. Ingest all local tags matching `v*` via `git tag -l "v*"`.
2. Normalize tag names by stripping optional `v` prefixes for comparison.
3. Partition tags into:
   - `retained_historical_tags`: major < 4 OR (major == 4 AND minor <= 160).
   - `retained_whitelist_tags`: exact match with `v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`.
   - `prune_local_candidates`: all other tags where major == 4 AND minor > 160.
4. For each tag in `prune_local_candidates`:
   - Invoke `git tag -d <tag>`.
   - Log deletion outcome: `[LOCAL PRUNE] Deleted tag <tag> (ref: refs/tags/<tag>)`.

---

### 3.2 Component 2: Multi-Source Retention Verification Engine

#### Responsibilities:
Provides comprehensive audit verification across three distinct tiers to guarantee that prune operations succeeded without unintended data loss.

#### Verification Tiers:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   Multi-Source Verification Matrix                     │
├────────────────────┬────────────────────┬──────────────────────────────┤
│ 1. Local Storage   │ 2. Remote Origin   │ 3. GitHub API Release Store  │
│    (git tag -l)    │ (git ls-remote)    │    (gh release list)         │
├────────────────────┼────────────────────┼──────────────────────────────┤
│ • Strict 4 Whitelist│ • Strict 4 Whitelist│ • Strict 4 Whitelist        │
│ • Full <= 4.160.0  │ • Full <= 4.160.0  │ • Full <= 4.160.0            │
│ • Zero Pruned Tags │ • Zero Pruned Tags │ • Zero Pruned Releases       │
└────────────────────┴────────────────────┴──────────────────────────────┘
```

#### Acceptance Criteria & Verification Logic:
1. **Local Tag Audit (`git tag -l`)**:
   - `v4.190.0`: Must exist locally.
   - `v4.206.0`: Must exist locally.
   - `v4.225.0`: Must exist locally.
   - `v4.226.0`: Must exist locally.
   - Historical anchor sample (`v4.160.0`, `v4.150.0`, `v4.140.0`): Must exist locally.
   - Pruned sample (`v4.161.0`, `v4.205.0`, `v4.224.0`): Must NOT exist locally.

2. **Remote Ref Audit (`git ls-remote --tags origin`)**:
   - Every tag reported by `git ls-remote --tags origin refs/tags/v*` must either have version `<= 4.160.0` or be within the 4 whitelisted tags.
   - Any remote ref for a pruned version (e.g. `refs/tags/v4.205.0`) triggers an immediate audit failure.

3. **GitHub Release Audit (`gh release list --repo alimtvnetwork/Antigravity-Manager`)**:
   - Total releases with version `> 4.160.0` must equal exactly 4.
   - Release titles and tag names must match `v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`.
   - Releases `<= 4.160.0` must remain intact with their existing assets, timestamps, and release notes.

---

### 3.3 Component 3: GitMap Future Parity & Native Command Specification

To prevent recurrent manual pruning across multiple repositories and automate release hygiene in CI/CD pipelines, GitMap will natively implement dedicated release and tag pruning commands.

#### Command Syntax:

```bash
# Prune GitHub Releases based on cutoff boundary and whitelist:
gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160

# Prune Remote & Local Git Tags based on cutoff boundary and whitelist:
gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160
```

#### Extended CLI Flag Specifications:

| Flag | Short | Default | Description |
|---|---|---|---|
| `--keep` | `-k` | (Required) | Comma-delimited list of SemVer versions to strictly preserve above the cutoff. Supports prefixes `v` or bare numbers. |
| `--after` | `-a` | (Required) | Cutoff SemVer threshold. Only tags/releases with version `> threshold` are considered for pruning. |
| `--remote` | `-r` | `origin` | Target Git remote name. Resolves repository URL automatically. |
| `--dry-run` | `-d` | `false` | When `true`, scans and reports candidates without executing deletions. |
| `--confirm` | `-y` | `false` | When `true`, bypasses terminal interactive confirmation prompt. |
| `--local-only` | `-l` | `false` | Restricts pruning strictly to local Git tags (`.git/refs/tags/`). |
| `--remote-only` | `-m` | `false` | Restricts pruning strictly to remote GitHub Releases / remote tags. |
| `--json` | `-j` | `false` | Emits machine-readable JSON output detailing preserved and pruned items. |

#### Execution Flow in GitMap:
1. **Remote Repository Resolution**: GitMap inspects `.git/config` to resolve `remote.origin.url`, identifying `alimtvnetwork/Antigravity-Manager`.
2. **API Authentication**: Discovers token via `GITHUB_TOKEN`, `GH_TOKEN`, or GitMap's internal encrypted vault (`repo-secrets`).
3. **Candidate Computation**: Fetches remote release page list, builds the candidate deletion queue, and prints a formatted preview table.
4. **Execution Ladder**:
   - Remote GitHub Releases: Deleted via `DELETE /repos/{owner}/{repo}/releases/{release_id}`.
   - Remote Git Tags: Deleted via `DELETE /repos/{owner}/{repo}/git/refs/tags/{tag}` or `git push origin --delete <tag>`.
   - Local Git Tags: Deleted via `git tag -d <tag>`.
5. **Audit Report Emission**: Writes JSON audit record to `.gitmap/cache/reports/release-prune-<timestamp>.json`.

---

### 3.4 Component 4: GitMap Task Cache & Recipe Registration

GitMap maintains reusable maintenance tasks within the `.gitmap/cache/tasks/` directory or manages them via `gitmap rc`.

#### Task Definition Schema (`.gitmap/cache/tasks/git-tag-and-release-management.json`):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "task_id": "agm-git-tag-and-release-management",
  "version": "1.0.0",
  "description": "Prune excess preview releases and tags while preserving 4 strict anchors and historical versions",
  "target_repository": "alimtvnetwork/Antigravity-Manager",
  "cutoff_boundary": {
    "rule": "greater_than",
    "version": "4.160.0"
  },
  "whitelist_versions": [
    "v4.190.0",
    "v4.206.0",
    "v4.225.0",
    "v4.226.0"
  ],
  "preserved_historical_boundary": "4.160.0",
  "execution_steps": [
    {
      "step_order": 1,
      "action": "github_release_prune",
      "command": "gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160 --confirm"
    },
    {
      "step_order": 2,
      "action": "git_remote_tag_prune",
      "command": "gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160 --remote-only --confirm"
    },
    {
      "step_order": 3,
      "action": "git_local_tag_prune",
      "command": "gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160 --local-only --confirm"
    },
    {
      "step_order": 4,
      "action": "verification_audit",
      "command": "gitmap release verify --keep 4.190,4.206,4.225,4.226 --after 4.160"
    }
  ]
}
```

---

### 3.5 Component 5: UI & Terminal Visual Representation

When rendered within GitMap terminal output or the Antigravity Manager GUI interface, status badges and summaries must follow the unified styling conventions:

#### Badge Standards:

| Tier | Status Label | Style Class / Color | Context |
|---|---|---|---|
| Historical Freeze | `[HISTORICAL]` | Zinc Badge (`bg-zinc-800 text-zinc-300 border-zinc-700`) | Versions `<= 4.160.0` (Untouched) |
| Whitelisted Anchor | `[WHITELISTED]` | Emerald Badge (`bg-emerald-950 text-emerald-400 border-emerald-600`) | The 4 active versions (`v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`) |
| Prune Candidate | `[PRUNE CANDIDATE]` | Amber Badge (`bg-amber-950 text-amber-400 border-amber-600`) | Pending deletion during preview / dry-run |
| Successfully Pruned | `[DELETED]` | Rose Badge (`bg-rose-950 text-rose-400 border-rose-600`) | Confirmed removed from release and tag index |

#### Terminal Output Format Example:

```
┌────────────────────────────────────────────────────────────────────────┐
│ GitMap Release & Tag Pruning Summary: alimtvnetwork/Antigravity-Manager│
├────────────────────────────────────────────────────────────────────────┤
│ Cutoff Threshold : > 4.160.0                                          │
│ Whitelist Anchors: v4.190.0, v4.206.0, v4.225.0, v4.226.0              │
├────────────┬──────────────┬──────────────────┬─────────────────────────┤
│ Tag        │ Target Type  │ Action           │ Status                  │
├────────────┼──────────────┼──────────────────┼─────────────────────────┤
│ v4.160.0   │ Tag/Release  │ Preserve         │ [HISTORICAL]            │
│ v4.161.0   │ Tag/Release  │ Prune            │ [DELETED]               │
│ ...        │ Tag/Release  │ Prune            │ [DELETED]               │
│ v4.190.0   │ Tag/Release  │ Preserve         │ [WHITELISTED]           │
│ ...        │ Tag/Release  │ Prune            │ [DELETED]               │
│ v4.206.0   │ Tag/Release  │ Preserve         │ [WHITELISTED]           │
│ ...        │ Tag/Release  │ Prune            │ [DELETED]               │
│ v4.225.0   │ Tag/Release  │ Preserve         │ [WHITELISTED]           │
│ v4.226.0   │ Tag/Release  │ Preserve         │ [WHITELISTED]           │
└────────────┴──────────────┴──────────────────┴─────────────────────────┘
```

---

## 4. Error Handling & Invariants

1. **Idempotent Deletion**: Prune operations must handle cases where a tag or release is already deleted without throwing fatal errors. Missing resources receive warning records and proceed.
2. **Network Resilience**: Remote API invocations to GitHub REST endpoints must implement exponential backoff retry ladders (1s, 2s, 4s) to tolerate secondary rate limits or transient transport dropouts.
3. **Commit History Protection**: No prune operation shall execute `git filter-branch`, `git reset`, `git rebase`, or force-push branches. Only refs matching `refs/tags/*` and GitHub release objects are targeted.
4. **Positive Booleans**: All validation methods and conditional variables throughout implementation code MUST use positive naming (`is_retained`, `is_valid`, `is_whitelisted`). Negative forms (`is_not_whitelisted`, `unretained`) are strictly forbidden.

---

## 5. Cross-References

| Document | Path | Purpose |
|---|---|---|
| Architecture & API Spec | `02-spec/21-app/git-tag-and-release-page-management/01-architecture-and-api-spec.md` | Core architecture, REST API routes, and rate-limit mitigation |
| Subtask 01 | `.ai-memory/plans/subtasks/git-tag-and-release-page-management/01-inventory-and-filter-candidates.md` | Discovery and candidate filtering |
| Subtask 02 | `.ai-memory/plans/subtasks/git-tag-and-release-page-management/02-delete-remote-releases-and-tags.md` | Remote GitHub Releases and tags deletion |
| Subtask 03 | `.ai-memory/plans/subtasks/git-tag-and-release-page-management/03-prune-local-tags-and-verify-retention.md` | Local Git tag pruning and retention verification |
| Subtask 04 | `.ai-memory/plans/subtasks/git-tag-and-release-page-management/04-document-gitmap-cache-task-and-parity.md` | GitMap cache task registration and CLI parity |
| Spec Authoring Guide | `02-spec/01-spec-authoring-guide/readme.md` | Repository-wide specification authoring rules |
