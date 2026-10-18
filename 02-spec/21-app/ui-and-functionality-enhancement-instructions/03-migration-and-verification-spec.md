# Migration, Updater & Verification Specification

## 1. Updater Version Detection & Pinned Version Execution

### 1.1 Live Version Extraction
- In `install.ps1`, update `Get-InstalledVersion` to query running process binaries and PE file headers (`[System.Diagnostics.FileVersionInfo]::GetVersionInfo($exePath).ProductVersion`) before falling back to registry values.
- In `Sync-InstalledAppsRegistryBranding`, set `DisplayVersion` to `$currentVer` alongside `DisplayName`.

### 1.2 Pinned Version Tag URL Resolution
- In `install.ps1` (`Resolve-PinnedVersion`), expand the regex matcher to match GitHub release tag URLs:
  `'(?i)(?:releases/tag/|releases/download/|raw\.githubusercontent\.com/...)(?:v)?([0-9]+\.[0-9]+(?:\.[0-9]+)?(?:-[a-zA-Z0-9.]+)?(?:#.*)?)'`
- Allow downgrade installation when `$isPinned` is `$true` by bypassing the `-le` version abort guard.

---

## 2. Standalone JSON Import / Export Specifications

- Standard envelope schema v2.0 (`ExportEnvelope`):
  - `embed_images: true`: Reads images from disk, converts to base64, and embeds in `embedded_images` map. Portable across any machine or environment.
  - `embed_images: false`: Normalizes file paths with `${workDir}` and `${repoDir}` relative anchors for lean disk storage.

---

## 3. Coding Guidelines & Verification Strategy

- Standardize positive boolean flags (`is`, `has`, `should`, `can`).
- Eliminate double negatives in conditional chains (`!a && !b` -> helper function or inverted positive guard).
- Encapsulate magic constants in enums or constants modules.
- Verification via targeted syntax and unit testing per module.
