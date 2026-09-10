---
name: Version Source of Truth Standard
description: version.json is the single source of truth for repository versioning. Supports backend, frontend, and sub-package sections with 'inherit'. All codebases import version.json.
type: standard
---

# Version Source of Truth Standard (`version.json`)

**Canonical spec:** `02-spec/01-spec-authoring-guide/14-version-schema.md`
**Rule:** Hard Rule 19 in `02-spec/17-consolidated-guidelines/34-compiled-simple-coding-guidelines.md`
**Reading Queue:** Enqueued in `.lovable/what-to-read.md` and `.lovable/memory/01-index.md`

## 1. Single Source of Truth at Repository Root

Every repository in this ecosystem MUST maintain a single, canonical `version.json` file at its root directory.
This file is the authoritative source for:
- Repository Version (`Version`) — the global root version (or initial starting version, e.g. `1.0.0` or `0.1.0`)
- Repository Identity (`Title`, `RepoSlug`, `RepoUrl`, `LastCommitSha`)
- Author Attribution (`Authors[]`)
- Sub-component & Monorepo tracks (`backend`, `frontend`, `cli`, `linters`, `services`, `packages`)
- Integrated Specs & Prompt Architect version tracking (`codingGuideline`, `promptArchitectByRiseupAsia`)

## 2. Multi-Section Versioning and the `"inherit"` Keyword

In polyglot codebases, monorepos, and client-server architectures, sub-components may share the repository version or maintain dedicated release tracks. `version.json` governs all sub-package tracks in one central file:

```json
{
  "_purpose": "version.json is the single canonical source of truth for repository versioning, releases, metadata, and cross-repo dependencies. All tools, packages, scripts, CI/CD pipelines, manifests, and AI agents must read this file for version information. To bump or change the version in this repository, update the 'Version' field (or sub-package sections) in this file and run the project sync script (e.g. 'npm run sync' or 'npm run sync:version').",
  "_instructions": {
    "versionSourceOfTruth": "Every codebase must use version.json at the repository root as the single source of truth for version numbers.",
    "howToChangeVersion": "To change the version of the repository, edit the 'Version' field in version.json, then run 'npm run sync' (or the project's sync command) to propagate it across all files, docs, badges, and manifests.",
    "subPackageSections": "Supports sub-component sections (e.g. backend, frontend, cli, linters). Set to 'inherit' to use the global root Version, or specify an explicit version string for independent version tracks.",
    "inheritanceRule": "When a section is set to 'inherit', all tooling and codebases resolving that sub-package must inherit the root global Version.",
    "prohibition": "Never hardcode version numbers in code, scripts, or documentation. Always read from or sync with version.json."
  },
  "Version": "6.32.0",
  "Title": "Coding Guidelines",
  "RepoSlug": "coding-guidelines-v24",
  "RepoUrl": "https://github.com/alimtvnetwork/coding-guidelines-v24",
  "backend": "inherit",
  "frontend": "inherit",
  "linters": "3.79.0"
}
```

### Inheritance Rules:

1. **`"inherit"`**: When a section (e.g., `"backend": "inherit"`, `"frontend": "inherit"`) specifies `"inherit"`, all build systems, Dockerfiles, and runtime modules resolving that sub-package MUST automatically resolve to the root `Version` string.
2. **Explicit Track** (e.g., `"linters": "3.79.0"`): Used when a sub-system maintains an independent release cadence while still keeping version authority centralized in root `version.json`.
3. **Starting Version**: Repositories can define whatever starting version they want (e.g., `1.0.0`, `0.1.0`) in the root `Version` field.

## 3. Mandatory Codebase Import Protocol

**Every codebase in the repository (Go, TypeScript, Python, PHP, C#, Rust, shell scripts) MUST import or read root `version.json` directly:**

- **TypeScript / Frontend**:
  ```typescript
  import versionData from "../../version.json";
  export const APP_VERSION = versionData.frontend === "inherit" ? versionData.Version : versionData.frontend;
  ```
- **Go / Backend**:
  Embed root `version.json` via `//go:embed` or read at initialization:
  ```go
  //go:embed version.json
  var versionJSON []byte
  ```
- **Python**:
  ```python
  import json, pathlib
  version_data = json.loads(pathlib.Path("version.json").read_text(encoding="utf-8"))
  APP_VERSION = version_data["Version"] if version_data.get("backend") == "inherit" else version_data.get("backend", version_data["Version"])
  ```
- **PHP**:
  ```php
  $versionData = json_decode(file_get_contents(__DIR__ . '/version.json'), true);
  define('APP_VERSION', $versionData['backend'] === 'inherit' ? $versionData['Version'] : $versionData['backend']);
  ```
- **Shell / CI/CD**:
  ```bash
  APP_VERSION=$(python3 -c "import json; v=json.load(open('version.json')); print(v.get('Version'))")
  ```

## 4. How to Change or Bump the Version

To release or change the version across ANY repository:
1. Edit the `"Version"` (and any applicable sub-package version) in `version.json`.
2. Run the repository's synchronization script:
   ```bash
   npm run sync
   # or node scripts/sync-version.mjs
   ```
3. The sync script automatically propagates the new version to `package.json`, manifests, documentation, stamps, and health badges.
4. Commit and tag:
   ```bash
   git commit -m "release: vX.Y.Z"
   git tag vX.Y.Z
   git push && git push --tags
   ```

## 5. Strict Prohibitions (Non-Negotiable)

- ❌ **NEVER** hardcode version numbers across random source files, constants, or markdown bodies.
- ❌ **NEVER** create competing version files (e.g. `frontend/version.json`, `backend/version.json`).
- ❌ **NEVER** allow version drift across files; `version.json` always wins.

## 6. Enqueueing in `what-to-read.md` and Bundle Transfer

1. **`what-to-read.md`**: `version.json` and this standard are explicitly enqueued at the top of `.lovable/what-to-read.md` as mandatory reading before any task or code changes.
2. **Bundle Install & Update**: During bundle install and update phases, all installation scripts transfer `.lovable/memory/` and `.lovable/what-to-read.md` to destination repositories, ensuring that all downstream repositories automatically inherit this versioning standard and execution queue.
