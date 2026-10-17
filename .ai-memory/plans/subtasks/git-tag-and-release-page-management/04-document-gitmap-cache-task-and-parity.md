# Subtask 04: Document GitMap Cache Task and CLI Parity Specification

**Owner:** Worker 04 / GitMap Integration Lead  
**Task Group:** `git-tag-and-release-page-management`  
**Target Repositories:** `alimtvnetwork/Antigravity-Manager` & `GitMap`  
**Referenced Specifications:**  
- `02-spec/21-app/git-tag-and-release-page-management/01-architecture-and-api-spec.md`  
- `02-spec/21-app/git-tag-and-release-page-management/02-component-and-ui-spec.md`  
- Target Cache Location: `.gitmap/cache/tasks/git-tag-and-release-management.json`  

---

## 1. Context & Objectives

Executing manual release deletion and tag pruning via bespoke scripts or ad-hoc `gh` CLI invocations carries human error risk, lacks unified telemetry, and cannot be cleanly delegated across multi-machine GitMap fleet clusters.

This subtask formalizes:
1. Recording the complete operational recipe as a versioned task definition within `.gitmap/cache/tasks/` (or via `gitmap rc`) for reproducibility.
2. Defining the exact syntax, flags, and behavioral requirements for native GitMap command parity:
   - `gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160`
   - `gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160`
3. Outlining the Go implementation architecture for GitMap's upcoming release engine (`cmd_release.go` and `cmd_tag.go`).

---

## 2. GitMap Task Cache Specification

GitMap standardizes reusable maintenance playbooks under `.gitmap/cache/tasks/`. The task file records the retention boundaries, repository identification, and execution rules.

### Task Definition: `.gitmap/cache/tasks/git-tag-and-release-management.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "task_id": "agm-git-tag-and-release-management",
  "name": "Antigravity Manager Tag and Release Pruning",
  "version": "1.0.0",
  "target_repository": "alimtvnetwork/Antigravity-Manager",
  "created_at": "2026-10-18T00:00:00Z",
  "policy": {
    "cutoff_rule": "greater_than",
    "cutoff_version": "4.160.0",
    "historical_preserve_boundary": "4.160.0",
    "whitelisted_versions": [
      "v4.190.0",
      "v4.206.0",
      "v4.225.0",
      "v4.226.0"
    ]
  },
  "cli_commands": {
    "release_prune": "gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160",
    "tag_prune": "gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160"
  },
  "execution_sequence": [
    {
      "step": 1,
      "id": "inventory_and_dry_run",
      "description": "Preview prune candidates without side effects",
      "command": "gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160 --dry-run"
    },
    {
      "step": 2,
      "id": "remote_release_prune",
      "description": "Delete remote GitHub Releases exceeding cutoff excluding whitelist",
      "command": "gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160 --confirm"
    },
    {
      "step": 3,
      "id": "remote_tag_prune",
      "description": "Delete remote Git tags matching pruned releases",
      "command": "gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160 --remote-only --confirm"
    },
    {
      "step": 4,
      "id": "local_tag_prune",
      "description": "Delete local Git tags to align local repo refs",
      "command": "gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160 --local-only --confirm"
    },
    {
      "step": 5,
      "id": "verification_audit",
      "description": "Verify retention across local tags, remote tags, and releases",
      "command": "gitmap release verify --keep 4.190,4.206,4.225,4.226 --after 4.160"
    }
  ]
}
```

---

## 3. GitMap Native CLI Parity Specification

Future versions of the GitMap CLI binary will implement native `release` and `tag` management commands directly in Go, removing dependency on external shell scripts.

### 3.1 Primary Commands & Syntax

```bash
# 1. Prune GitHub Releases:
gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160

# 2. Prune Git Tags (both remote and local):
gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160

# 3. Dedicated Target Deletion (Single Entity):
gitmap release delete <release-tag> [--remote <origin>] [--confirm]
gitmap tag delete <tag-name> [--remote <origin>] [--confirm]
```

### 3.2 Flag Definitions & Parsing Invariants

| Flag | Type | Default | Description |
|---|---|---|---|
| `--keep` | `string` | (Required for prune) | Comma-separated list of versions to preserve above the cutoff (e.g. `4.190,4.206,4.225,4.226`). Accepts `v` prefix or raw digits. |
| `--after` | `string` | (Required for prune) | Cutoff version. Only items with version `> threshold` are targeted. |
| `--remote` | `string` | `origin` | Remote name to resolve GitHub repository path from `.git/config`. |
| `--dry-run` | `bool` | `false` | Scans and outputs candidate table without executing delete calls. |
| `--confirm` | `bool` | `false` | Bypasses interactive terminal `[y/N]` confirmation prompt. |
| `--remote-only` | `bool` | `false` | Restricts operations to remote Git tags or GitHub Releases. |
| `--local-only` | `bool` | `false` | Restricts operations to local Git repository tags (`.git/refs/tags/`). |
| `--json` | `bool` | `false` | Outputs machine-readable JSON detailing candidates and actions taken. |

### 3.3 Internal Implementation Flow (Go Engine)

```go
// Architectural blueprint for GitMap cmd_release.go:
type PruneOptions struct {
    KeepVersions []string
    AfterVersion string
    IsDryRun     bool
    IsConfirmed  bool
    IsRemoteOnly bool
    IsLocalOnly  bool
}

func ExecuteReleasePrune(repo string, opts PruneOptions) error {
    // 1. Ingest all releases via GitHub API
    releases, err := githubClient.ListReleases(repo)
    if err != nil {
        return fmt.Errorf("failed to list releases: %w", err)
    }

    // 2. Classify candidates
    var candidates []Release
    for _, rel := range releases {
        ver := parseSemver(rel.TagName)
        isAboveCutoff := ver.GreaterThan(opts.AfterVersion)
        isWhitelisted := containsVersion(opts.KeepVersions, ver)

        if isAboveCutoff && !isWhitelisted {
            candidates = append(candidates, rel)
        }
    }

    // 3. Render candidate table
    renderCandidateTable(candidates)

    if opts.IsDryRun {
        log.Info("Dry-run mode active: zero deletions executed.")
        return nil
    }

    // 4. Interactive confirmation if not bypassed
    if !opts.IsConfirmed {
        if !promptUserConfirmation(len(candidates)) {
            log.Warn("Operation aborted by user.")
            return nil
        }
    }

    // 5. Batch deletion with rate-limit backoff ladder
    for _, c := range candidates {
        err := githubClient.DeleteRelease(repo, c.ID)
        if err != nil {
            log.Errorf("Failed to delete release %s: %v", c.TagName, err)
            continue
        }
        log.Infof("[PRUNED] Release %s (ID: %d)", c.TagName, c.ID)
    }

    return nil
}
```

---

## 4. GitMap Cache & Recipe Integration (`gitmap rc`)

The GitMap `rc` (repository cache) command suite allows developers and automated agents to interact with cached tasks and playbooks:

```bash
# Register or inspect task definition in repo cache:
gitmap rc file .gitmap/cache/tasks/git-tag-and-release-management.json

# Execute task directly via GitMap task runner:
gitmap task run agm-git-tag-and-release-management
```

### Safety Guards:
1. **Repository Identity Check**: The runner confirms current Git remote matches `alimtvnetwork/Antigravity-Manager` before proceeding.
2. **Historical Cutoff Invariant**: Hard-coded safety boundary prevents any execution where `--after` is lower than `4.160.0`, protecting historical v1.x - v4.160.x release records.
3. **Whitelist Safeguard**: If any of the 4 designated whitelist versions (`v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`) appear in the computed delete list, the command halts with a fatal invariant assertion.

---

## 5. Acceptance Criteria Checklist

- [ ] Task JSON schema validated and registered under `.gitmap/cache/tasks/git-tag-and-release-management.json`.
- [ ] Command syntax documented with exact parameter specs:
  - `gitmap release prune --keep 4.190,4.206,4.225,4.226 --after 4.160`
  - `gitmap tag prune --keep 4.190,4.206,4.225,4.226 --after 4.160`
- [ ] Positive booleans used exclusively across task options (`is_dry_run`, `is_confirmed`, `is_remote_only`, `is_local_only`).
- [ ] Clear Go implementation blueprint provided for GitMap developers.
- [ ] Strict relative paths used throughout all documentation files.
