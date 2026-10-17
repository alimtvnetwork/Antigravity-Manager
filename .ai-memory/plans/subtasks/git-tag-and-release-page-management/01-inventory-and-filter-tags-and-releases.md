---
id: subtask-01-inventory-and-filter-tags-and-releases
task_id: Task-01
owner: Worker 01 / Discovery Lead
status: READY FOR EXECUTION
files_owned:
  - .ai-memory/plans/subtasks/git-tag-and-release-page-management/01-inventory-and-filter-tags-and-releases.md
---

# Subtask 01: Inventory, Parse SemVer, and Filter Tags and Releases

## 1. Context & Objective
The goal of this subtask is to comprehensively inventory all remote Git tags and GitHub Release entities for repository `alimtvnetwork/Antigravity-Manager`, parse their semantic versions, apply boundary and whitelist filters, and export a validated `purge-manifest.json` file.

This subtask performs **read-only discovery and manifest generation**. No destructive deletions are executed during Subtask 01.

---

## 2. Technical Scope & Implementation Steps

### Step 1: Remote Tags and GitHub Releases Discovery
1. Discover all remote tags pointing to `origin` using the authenticated GitHub API or CLI:
   ```bash
   gh api repos/alimtvnetwork/Antigravity-Manager/git/matching-refs/tags --paginate --jq '.[].ref' | sed 's|refs/tags/||'
   ```
2. Discover all GitHub Releases:
   ```bash
   gh release list --repo alimtvnetwork/Antigravity-Manager --limit 1000 --json tagName,name,isPrerelease,isDraft,createdAt
   ```
3. Collate unique tag identifiers across both sources into a unified inventory list.

### Step 2: SemVer Parsing & Normalization
1. Strip leading `v` or `V` prefix from the tag name.
2. Parse version into numerical components: `(major, minor, patch, prerelease)`.
3. Validate that the string conforms to valid SemVer format (`is_valid_semver = true`).

### Step 3: Partitioning & Filter Evaluation
Apply the core evaluation rules to every discovered tag:

1. **Floor Boundary Check:**
   - If `major < 4` or `(major == 4 and minor <= 160)`:
     - `is_historical = true`
     - `is_post_cutoff = false`
     - `is_whitelisted = false`
     - `is_purge_target = false`
     - Category: `PRESERVED_HISTORICAL` (Preserve permanently)

2. **Whitelist Protection Check:**
   - Whitelist set: `{"v4.190.0", "v4.206.0", "v4.225.0", "v4.226.0"}`
   - If tag name or normalized version matches the whitelist:
     - `is_historical = false`
     - `is_post_cutoff = true`
     - `is_whitelisted = true`
     - `is_purge_target = false`
     - Category: `PRESERVED_WHITELIST` (Preserve permanently)

3. **Purge Candidate Selection:**
   - If `is_post_cutoff == true` and `is_whitelisted == false`:
     - `is_historical = false`
     - `is_post_cutoff = true`
     - `is_whitelisted = false`
     - `is_purge_target = true`
     - Category: `PURGE_TARGET` (Scheduled for deletion in Subtask 02)

### Step 4: Manifest Generation (`purge-manifest.json`)
Serialize the partitioned dataset into `.ai-memory/reports/git-tag-and-release-management/purge-manifest.json` conforming to the schema defined in `02-spec/21-app/git-tag-and-release-page-management/01-architecture-spec.md`.

---

## 3. Positive Boolean Invariant Validation

Ensure all properties and code variables adhere strictly to positive boolean conventions:
- `is_post_cutoff`: True if tag version > `4.160.0`.
- `is_whitelisted`: True if tag is one of `v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`.
- `is_purge_target`: True if and only if `is_post_cutoff && !is_whitelisted`.
- `is_historical`: True if tag version <= `4.160.0`.
- `has_github_release`: True if release entity exists on GitHub.
- `is_remote_tag_present`: True if ref exists on `origin`.
- `is_local_tag_present`: True if ref exists locally.

---

## 4. Verification & Quality Gates

Before declaring Subtask 01 complete, verify:
1. **Zero Historical Leaks:** Confirm that no tag with `major < 4` or `(major == 4 and minor <= 160)` is listed under `purge_targets`.
2. **Zero Whitelist Leaks:** Confirm that `v4.190.0`, `v4.206.0`, `v4.225.0`, and `v4.226.0` are absent from `purge_targets` and present in `preserved_whitelist`.
3. **Deterministic Counts:** Validate that `total_discovered == preserved_historical_count + preserved_whitelist_count + purge_target_count`.
4. **Path Compliance:** Confirm that all generated logs and manifests use strictly relative Git paths.
