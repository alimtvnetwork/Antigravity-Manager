# Subtask 001 — Installer Hardening (Ventura / Sequoia)

**Plan:** 120 — macOS Installation Deep Hardening
**Status:** pending

## Scope

Files to modify:
- `install.sh`
- `scripts/Fix_Damaged.command`
- `scripts/package_dmg.sh`

---

## 1. `install.sh` — `install_macos()` Rewrites

### 1.1 Robust Mount-Point Extraction

Replace fragile `grep -o '/Volumes/.*' | head -n1 | sed` with `awk`:

```bash
mount_point=$(echo "$mount_output" | awk '/\/Volumes\// { for(i=1;i<=NF;i++) if($i ~ /^\/Volumes\//) { print $i; exit } }')
```

### 1.2 macOS Version Detection

Add at the top of `install_macos()`:

```bash
local macos_major
macos_major=$(sw_vers -productVersion | cut -d. -f1)
```

### 1.3 Multi-Step Recursive Quarantine Clearing

Replace single-step `xattr -cr` with:

```bash
xattr -cr "$target_app" 2>/dev/null || true
xattr -d com.apple.quarantine "$target_app" 2>/dev/null || true
# Recursively clear from all nested files
find "$target_app" -exec xattr -d com.apple.quarantine {} \; 2>/dev/null || true
```

### 1.4 spctl Registration on macOS 13+ (Ventura+)

After `codesign`:

```bash
if [[ "$macos_major" -ge 13 ]]; then
    spctl --add "$target_app" 2>/dev/null || true
fi
```

### 1.5 Sequoia (macOS 15+) Codesign Skip

On macOS 15+, skip `codesign --sign -` (no longer effective for quarantine bypass).
Instead rely exclusively on `spctl --add` and recursive `xattr -d`:

```bash
if [[ "$macos_major" -lt 15 ]]; then
    codesign --force --deep --sign - "$target_app" 2>/dev/null || true
fi
```

### 1.6 Improved ERR Trap

Replace existing ERR trap with one that prints:
- Exact failing command
- Exit code
- Line number
- Full `FUNCNAME` stack

```bash
trap 'echo "[ERROR] Command: ${BASH_COMMAND}" \
      "| Exit: $?" \
      "| Line: ${LINENO}" \
      "| Stack: ${FUNCNAME[*]}" >&2' ERR
```

### 1.7 IDE Discovery at End of Install

Add a call at the end of `install_macos()`:

```bash
detect_ide_path
```

### 1.8 New `detect_ide_path()` Function

Add a standalone helper:

```bash
detect_ide_path() {
    local ide_path=""

    # Try mdfind (Spotlight)
    ide_path=$(mdfind "kMDItemCFBundleIdentifier == 'com.antigravity.ide'" 2>/dev/null | head -n1)

    # Fallback: standard paths
    if [[ -z "$ide_path" ]]; then
        local candidates=(
            "/Applications/Antigravity.app"
            "$HOME/Applications/Antigravity.app"
        )
        for candidate in "${candidates[@]}"; do
            if [[ -d "$candidate" ]]; then
                ide_path="$candidate"
                break
            fi
        done
    fi

    if [[ -n "$ide_path" ]]; then
        echo "[INFO] Antigravity IDE detected at: $ide_path"
    else
        echo "[WARN] Antigravity IDE path could not be determined."
    fi
}
```

---

## 2. `scripts/Fix_Damaged.command` Rewrites

### 2.1 macOS Version Detection

```bash
macos_major=$(sw_vers -productVersion | cut -d. -f1)
```

### 2.2 Recursive xattr Pattern

Replace `xattr -cr` only with:

```bash
echo "🔧 Clearing quarantine attributes... / 正在清除隔离属性..."
xattr -cr "$app_path" 2>/dev/null || true
xattr -d com.apple.quarantine "$app_path" 2>/dev/null || true
find "$app_path" -exec xattr -d com.apple.quarantine {} \; 2>/dev/null || true
echo "✅ Quarantine cleared. / 隔离属性已清除。"
```

### 2.3 spctl Registration on macOS 13+

```bash
if [[ "$macos_major" -ge 13 ]]; then
    echo "🔐 Registering app with Gatekeeper... / 正在向看门人注册应用..."
    spctl --add "$app_path" 2>/dev/null || true
    echo "✅ Gatekeeper registration done. / 看门人注册完成。"
fi
```

### 2.4 Bilingual Status Messages

Each step must print an English and Chinese message (using `echo`) before and after execution, consistent with the patterns above.

---

## 3. `scripts/package_dmg.sh`

### 3.1 Ensure Fix_Damaged.command Is Executable and Included

Before creating the DMG, add:

```bash
chmod +x scripts/Fix_Damaged.command
cp scripts/Fix_Damaged.command "$dmg_staging_dir/"
```

### 3.2 Strip Quarantine from DMG Itself

After the DMG is created:

```bash
xattr -d com.apple.quarantine "$output_dmg" 2>/dev/null || true
```

---

## Acceptance Criteria

- [ ] `install_macos()` uses `awk` for mount-point extraction
- [ ] macOS version detection present and used for branching
- [ ] Multi-step recursive quarantine clearing applied
- [ ] `spctl --add` called on macOS 13+
- [ ] `codesign --sign -` skipped on macOS 15+
- [ ] ERR trap prints command, exit code, line, and full FUNCNAME stack
- [ ] `detect_ide_path()` function exists and is called at end of install
- [ ] `Fix_Damaged.command` uses recursive `find ... xattr -d` pattern
- [ ] `Fix_Damaged.command` calls `spctl --add` on macOS 13+
- [ ] `Fix_Damaged.command` has bilingual status messages
- [ ] `package_dmg.sh` `chmod +x`s and copies `Fix_Damaged.command`
- [ ] `package_dmg.sh` strips quarantine from produced DMG
