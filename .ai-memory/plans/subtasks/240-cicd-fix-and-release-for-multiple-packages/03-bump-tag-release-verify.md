# Subtask 03 - Bump, tag, release, verify

Date: 31-Oct-26
Status: in progress (bump done, changelog/readme notes written, commit pending)
Owned files (exclusive):
- version manifests touched by `npm run bump minor` (package.json,
  package-lock.json, version.json, src-tauri/Cargo.toml,
  src-tauri/tauri.conf.json, src-tauri/Cargo.lock)
- changelog.md, readme.md, readme_en.md (bump-script sections only)
- .ai-memory/plans/subtasks/240-cicd-fix-and-release-for-multiple-packages/03-bump-tag-release-verify.md
Work:
- `npm run bump minor` (4.283.0 -> 4.284.0) without the script's
  --commit flag; commit via the gitmap suite instead.
- Tag v4.284.0 on main, push the tag, watch the Release workflow.
- Verify: tag on remote via `git ls-remote`, Release run green,
  release published (not draft, not prerelease) with assets recorded,
  tree clean, local main equal to origin/main.
Evidence: bump commit SHA, tag SHA, Release run ID/URL, release URL,
asset count.
