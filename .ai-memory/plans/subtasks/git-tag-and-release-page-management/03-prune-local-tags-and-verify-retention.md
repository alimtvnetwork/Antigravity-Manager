# Subtask 03: Prune Local Git Tags and Verify Complete Retention Matrix

**Owner:** Worker 03 / QA Engineer  
**Task Group:** `git-tag-and-release-page-management`  
**Target Repository:** `alimtvnetwork/Antigravity-Manager`  
**Referenced Specifications:**  
- `02-spec/21-app/git-tag-and-release-page-management/01-architecture-and-api-spec.md`  
- `02-spec/21-app/git-tag-and-release-page-management/02-component-and-ui-spec.md`  

---

## 1. Context & Objectives

Following the remote cleanup of GitHub Releases and remote Git tags (Subtasks 01 and 02), the local repository clone retains tags stored in `.git/refs/tags/` and `.git/packed-refs`. If these local tags are not pruned, developers running subsequent commands like `git push --tags` or automated release pipelines could inadvertently re-publish pruned tags back to GitHub.

This subtask defines the exact operational workflow to:
1. Identify all local tags that fall within the prune window (version `> 4.160.0` and NOT in the strict 4-version whitelist).
2. Safely execute local deletion using `git tag -d <tag>`.
3. Verify retention across three authoritative sources:
   - Local Git tags (`git tag -l`)
   - Remote Git tags (`git ls-remote --tags origin`)
   - GitHub Releases (`gh release list --repo alimtvnetwork/Antigravity-Manager`)
4. Confirm zero disruption to historical tags (`<= 4.160.0`) and perfect preservation of the 4 whitelist anchors.

---

## 2. Retention Rules & Boundary Matrix

| Category | SemVer Boundary | Whitelist Filter | Action | Examples |
|---|---|---|---|---|
| **Historical Preserve** | SemVer `<= 4.160.0` | N/A (Rule of Origin) | **PRESERVE** | `v4.160.0`, `v4.159.0`, `v4.150.0`, `v4.140.0`, ..., `v0.1.0` |
| **Whitelisted Anchors** | SemVer `> 4.160.0` | Strict 4 Versions | **PRESERVE** | `v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0` |
| **Prune Candidates** | SemVer `> 4.160.0` | NOT in Whitelist | **PRUNE** (Local Delete) | `v4.161.0`, `v4.170.0`, `v4.185.0`, `v4.205.0`, `v4.217.0`, `v4.224.0`, etc. |

---

## 3. Step-by-Step Execution Procedure

### Step 1: Enumerate Local Tags
Query all local tags and format them for evaluation:

```bash
git tag -l "v*"
```

Parse and extract the SemVer components `(major, minor, patch)` from each tag string.

### Step 2: Compute Local Prune Candidates
Filter the local tag list applying positive boolean checks:

```python
# Reference classification algorithm:
def classify_tag(tag_name: str) -> str:
    version = parse_semver(tag_name)
    is_historical_preserve = version.major < 4 or (version.major == 4 and version.minor <= 160)
    if is_historical_preserve:
        return "PRESERVE_HISTORICAL"
    
    whitelist = {"v4.190.0", "v4.206.0", "v4.225.0", "v4.226.0"}
    is_whitelisted = tag_name in whitelist
    if is_whitelisted:
        return "PRESERVE_WHITELIST"
    
    is_prune_eligible = version.major == 4 and version.minor > 160
    if is_prune_eligible:
        return "PRUNE_CANDIDATE"
    
    return "UNKNOWN"
```

Generate the explicit list of tags marked as `PRUNE_CANDIDATE`.

### Step 3: Execute Local Tag Deletion
Iterate through the computed candidate list and execute local tag deletion:

```bash
# Example sequential deletion:
git tag -d v4.161.0
git tag -d v4.162.0
# ... for each prune candidate tag
```

Or execute in batch via PowerShell / bash array iteration without touching git branches or commits:

```powershell
# PowerShell batch execution pattern:
$pruneCandidates | ForEach-Object {
    git tag -d $_
}
```

Verify that each deletion reports:
`Deleted tag '<tag>' (was <commit-hash>)`

---

## 4. Multi-Source Verification Gates

The executor must run three comprehensive verification gates to confirm retention integrity.

### Verification Gate 1: Local Tag Inventory
Run:

```bash
git tag -l
```

#### Verification Assertions:
- [ ] Whitelist tag `v4.190.0` is present.
- [ ] Whitelist tag `v4.206.0` is present.
- [ ] Whitelist tag `v4.225.0` is present.
- [ ] Whitelist tag `v4.226.0` is present.
- [ ] Historical anchor `v4.160.0` is present.
- [ ] Sample intermediate tags (`v4.161.0`, `v4.205.0`, `v4.217.0`, `v4.224.0`) are CONFIRMED ABSENT.
- [ ] Total count of local tags with `minor > 160` equals exactly 4.

### Verification Gate 2: Remote Origin Tag References
Query the remote Git repository directly to ensure that remote tag references reflect the clean state:

```bash
git ls-remote --tags origin "refs/tags/v*"
```

#### Verification Assertions:
- [ ] Remote refs for `refs/tags/v4.190.0` and `refs/tags/v4.190.0^{}` are present.
- [ ] Remote refs for `refs/tags/v4.206.0` and `refs/tags/v4.206.0^{}` are present.
- [ ] Remote refs for `refs/tags/v4.225.0` and `refs/tags/v4.225.0^{}` are present.
- [ ] Remote refs for `refs/tags/v4.226.0` and `refs/tags/v4.226.0^{}` are present.
- [ ] Remote refs for historical tags (e.g. `refs/tags/v4.160.0`) are present.
- [ ] Zero remote refs exist for pruned tags (`refs/tags/v4.161.0`, `refs/tags/v4.205.0`, etc.).

### Verification Gate 3: GitHub Release Page Inventory
Query GitHub Releases via GitHub CLI:

```bash
gh release list --repo alimtvnetwork/Antigravity-Manager --limit 100
```

#### Verification Assertions:
- [ ] Releases corresponding to the 4 whitelisted tags (`v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`) are published and accessible.
- [ ] All releases with tag `<= v4.160.0` remain accessible with historical release notes and assets.
- [ ] Zero releases exist for pruned tags above `v4.160.0`.
- [ ] Total count of releases with version `> 4.160.0` equals exactly 4.

---

## 5. Acceptance Criteria Checklist

- [ ] All local tags with version `> 4.160.0` (except the 4 whitelisted versions) deleted via `git tag -d`.
- [ ] Local tag whitelist intact (`v4.190.0`, `v4.206.0`, `v4.225.0`, `v4.226.0`).
- [ ] Local historical tags intact (all tags `<= 4.160.0`).
- [ ] Remote tags verified via `git ls-remote --tags origin`.
- [ ] Remote releases verified via `gh release list`.
- [ ] Commit history, active branches (`main`, `beta`), and working tree unmodified.
- [ ] Positive boolean naming conventions strictly adhered to in all validation logic.
