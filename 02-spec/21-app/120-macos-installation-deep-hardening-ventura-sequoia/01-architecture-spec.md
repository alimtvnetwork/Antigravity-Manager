# Plan 120: macOS Installation Deep Hardening — Ventura / Sequoia

**Status:** ACTIVE  
**Spec Path:** `02-spec/21-app/120-macos-installation-deep-hardening-ventura-sequoia/`

---

## 1. Root Cause Analysis

### RCA-01 — Gatekeeper Quarantine Clearing Broken on macOS 13+

macOS 13 Ventura introduced a behavior change in Gatekeeper where `xattr -cr <bundle>` no
longer reliably removes the `com.apple.quarantine` attribute from every file inside a `.app`
bundle. The command operates at the bundle root but does not guarantee recursive per-file
removal in all cases (particularly when the bundle contains nested frameworks or plug-ins).

**Required fix:** A two-phase quarantine removal:

1. `xattr -cr "$target_app"` — bulk clear at bundle root (fast path, covers most cases)
2. A recursive per-file loop using `find "$target_app" -exec xattr -d com.apple.quarantine {} \; 2>/dev/null`
   to remove the attribute from every individual file that survived phase 1.

Additionally, `spctl --assess --type exec "$target_app"` must be run after clearing to confirm
Gatekeeper no longer objects before the user launches the app.

---

### RCA-02 — Ad-hoc `codesign --sign -` Rejected on macOS 15 Sequoia

`codesign --force --deep --sign -` (ad-hoc signing) has been progressively restricted.
On macOS 15 Sequoia, Gatekeeper rejects ad-hoc signatures for third-party `.app` bundles
distributed outside the Mac App Store; the signature is applied but the Gatekeeper assessment
still fails, giving a false sense of security while the "damaged" dialog still appears.

**Required fix:**

- On **macOS < 13**: ad-hoc signing may be attempted as a best-effort fallback.
- On **macOS 13–14**: skip `codesign --sign -`; rely entirely on quarantine clearing + `spctl --add`.
- On **macOS 15+**: skip `codesign --sign -` entirely; it has no beneficial effect and can
  interfere with legitimate developer signatures embedded by Tauri's build pipeline.

---

### RCA-03 — Fragile `hdiutil` Mount Point Detection

The install script currently detects the DMG mount point with:

```bash
mount_point=$(hdiutil attach "$dmg_path" | grep -o '/Volumes/.*')
```

On macOS 13+ the `hdiutil attach` output changed: it now emits trailing tabs and extra
whitespace after the mount path, and occasionally wraps across columns differently. The
`grep -o '/Volumes/.*'` pattern captures the trailing whitespace as part of the path, causing
`cp` or `rsync` to fail with "no such file or directory" errors.

**Required fix:** Replace the fragile `grep -o` with `awk` column extraction:

```bash
mount_point=$(hdiutil attach "$dmg_path" | awk '/\/Volumes\//{print $NF}')
```

`$NF` always selects the last whitespace-separated token on the matching line, which is
the clean path regardless of leading/trailing whitespace or tab formatting.

---

### RCA-04 — Missing `spctl --add` Step After Install

After quarantine clearing, Gatekeeper's exception list is not updated. Even with quarantine
attributes removed, a first-launch security assessment can still prompt the "damaged" dialog
on Ventura+ because the app was not registered with the per-user Gatekeeper exception database.

**Required fix:** After a successful install on macOS 13+, run:

```bash
spctl --add "$target_app"
```

This registers the bundle path in Gatekeeper's exception list so the first-launch assessment
succeeds silently.

---

### RCA-05 — Missing macOS Version Detection

The install script applies the same code path for all macOS versions, making it impossible to
branch behavior for version-specific Gatekeeper restrictions.

**Required fix:** Detect the macOS major version at script startup:

```bash
macos_major=$(sw_vers -productVersion | awk -F'.' '{print $1}')
```

Use `$macos_major` throughout the script to gate version-specific logic branches.

---

## 2. Comparison: Windows vs. macOS Installer

| Dimension | Windows (NSIS) | macOS (bash + DMG) |
|---|---|---|
| Code signing | Proper Authenticode certificate via CI/CD pipeline | Ad-hoc `codesign --sign -` (rejected on Sequoia) |
| Distribution trust | SmartScreen respects Authenticode | Gatekeeper requires Apple Developer ID or exception list |
| Quarantine clearing | Not applicable (NTFS, no quarantine attribute) | `xattr -d com.apple.quarantine` required recursively |
| Mount point detection | N/A | `hdiutil attach` output; format changed on macOS 13+ |
| Exception registration | N/A | `spctl --add` required on macOS 13+ |
| Stack trace on failure | NSIS log + error code | `trap` with `FUNCNAME` array (currently missing) |

The fundamental gap is that Gatekeeper's trust model has grown stricter with each macOS release
since Ventura while the install script has remained unchanged.

---

## 3. Architecture of the Fix

### 3.1 Robust `hdiutil` Mount Point Parsing

Replace all `grep -o '/Volumes/.*'` occurrences in `install.sh` and `package_dmg.sh` with:

```bash
mount_point=$(hdiutil attach "$dmg_path" 2>&1 | awk '/\/Volumes\//{print $NF}')
```

Add a guard to abort early if `$mount_point` is empty:

```bash
if [[ -z "$mount_point" ]]; then
    echo "ERROR: failed to detect DMG mount point" >&2
    exit 1
fi
```

---

### 3.2 Multi-Step Quarantine Clearing

```bash
quarantine_clear() {
    local target="$1"
    # Phase 1 — bulk clear
    xattr -cr "$target" 2>/dev/null || true
    # Phase 2 — recursive per-file removal
    find "$target" -exec xattr -d com.apple.quarantine {} \; 2>/dev/null || true
    echo "Quarantine attributes cleared from: $target"
}
```

---

### 3.3 macOS Version-Gated Signing and Registration

```bash
macos_major=$(sw_vers -productVersion | awk -F'.' '{print $1}')

sign_and_register() {
    local target="$1"
    if [[ "$macos_major" -lt 13 ]]; then
        codesign --force --deep --sign - "$target" 2>/dev/null || true
    fi
    if [[ "$macos_major" -ge 13 ]]; then
        spctl --add "$target" 2>/dev/null || true
    fi
}
```

On macOS 15+ (`$macos_major -ge 15`), the `codesign --sign -` branch is skipped entirely.
On macOS 13–14, it is skipped and replaced with `spctl --add` only.

---

### 3.4 Stack Trace Capture in `install.sh`

Replace the current minimal `trap ERR` with a full stack trace capture using the `FUNCNAME`
array:

```bash
_stack_trace() {
    echo "=== INSTALL FAILED ===" >&2
    echo "Exit code: $?" >&2
    local frame=0
    while caller $frame; do
        ((frame++))
    done
    echo "FUNCNAME stack: ${FUNCNAME[*]}" >&2
}
trap '_stack_trace' ERR
set -euo pipefail
```

`caller $frame` emits `<line> <function> <file>` for each frame, giving a full call chain.

---

### 3.5 IDE Discovery via `mdfind`

In `process.rs`, `discover_and_persist_initial_ide_info` must use `mdfind` as the primary
discovery strategy on macOS, with standard path fallbacks:

**Discovery order:**

1. `mdfind 'kMDItemCFBundleIdentifier == "com.lbjlaq.antigravity-tools"'` — Spotlight index
2. `/Applications/AntigravityTools.app`
3. `$HOME/Applications/AntigravityTools.app`
4. `/Applications/Antigravity Tools.app` (name with space variant)

All discovery results must be written as structured JSON diagnostics to:
`~/.local/share/antigravity/ide-discovery.log`

Example log entry:

```json
{
  "timestamp": "<ISO8601>",
  "strategy": "mdfind",
  "found": true,
  "path": "/Applications/AntigravityTools.app",
  "macos_version": "15.0"
}
```

---

### 3.6 `instance.rs` — `$HOME/Applications/` Path Parity

The macOS launch path check in `instance.rs` currently only tests `/Applications/`. It must
also check `$HOME/Applications/` (the per-user Applications folder, writable without admin):

```rust
let candidate_paths = vec![
    PathBuf::from("/Applications").join(&app_name),
    home_dir().unwrap_or_default().join("Applications").join(&app_name),
];
```

Iterate over `candidate_paths` in order; use the first path that exists.

---

## 4. Affected Files

| File | Change Type |
|---|---|
| `scripts/install.sh` | hdiutil fix, quarantine clearing, version gate, stack trace |
| `scripts/Fix_Damaged.command` | quarantine clearing, spctl --add |
| `scripts/package_dmg.sh` | hdiutil mount point fix |
| `src-tauri/src/process.rs` | mdfind IDE discovery + JSON diagnostic log |
| `src-tauri/src/instance.rs` | $HOME/Applications/ fallback |

---

## 5. Acceptance Criteria

| ID | Criterion | Verification |
|---|---|---|
| AC1 | Fresh install on macOS 13 Ventura — no "damaged, move to trash" dialog | Manual test on Ventura VM |
| AC2 | Fresh install on macOS 14 Sonoma — no "damaged" dialog | Manual test on Sonoma VM |
| AC3 | Fresh install on macOS 15 Sequoia — no "damaged" dialog | Manual test on Sequoia hardware |
| AC4 | `agm` CLI symlink created and functional immediately after install | `which agm && agm --version` |
| AC5 | IDE path detected and written to `~/.local/share/antigravity/ide-discovery.log` on first Rust startup | `cat ~/.local/share/antigravity/ide-discovery.log` |
| AC6 | IDE switching between instances works on macOS | UI smoke test: switch instance, verify IDE relaunches |
| AC7 | Install script outputs full call stack (`caller` + `FUNCNAME`) on any failure | Inject deliberate failure, inspect stderr |
| AC8 | `$HOME/Applications/` fallback used when `/Applications/` is not writable | Run as non-admin user; verify install path |
