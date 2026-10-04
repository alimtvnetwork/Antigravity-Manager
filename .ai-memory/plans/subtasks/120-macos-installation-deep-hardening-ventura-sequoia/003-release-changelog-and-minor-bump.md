# Subtask 003 — Release Ceremony: Changelog & Minor Bump to v4.144.0

**Plan:** 120 — macOS Installation Deep Hardening
**Status:** pending

> [!IMPORTANT]
> This subtask is executed by the LEAD agent only. Spec Subagents must NOT
> run git commands, gitmap commits, or version bump scripts.

---

## Scope

- Bump version from `4.143.0` → `4.144.0`
- Update `CHANGELOG.md`
- Update `CHANGELOG_EN.md`
- Update `README.md` (section `## 📝 更新日志`)
- Update `README_EN.md` (section `## 📝 Changelog`)
- Commit via `gitmap cpf`
- Tag `v4.144.0` and push

---

## Step-by-Step

### Step 1 — Atomic Version Sync

```bash
npm run bump minor
```

Verify that all of the following are updated atomically:
- `package.json` → `"version": "4.144.0"`
- `src-tauri/Cargo.toml` → `version = "4.144.0"`
- `src-tauri/tauri.conf.json` → `"version": "4.144.0"`

### Step 2 — Populate `CHANGELOG.md`

Insert the following block **at the top** of the changelog body (below the
existing header), filling in the exact date of release:

```markdown
## v4.144.0 — YYYY-MM-DD

### 🍎 macOS Installation Deep Hardening (Ventura / Sequoia)

- **`install.sh`**: replaced fragile `grep|sed` mount-point extraction with
  robust `awk`; added macOS version detection; multi-step recursive quarantine
  clearing (`xattr -cr` + `xattr -d` + `find … xattr -d`); `spctl --add` on
  macOS 13+; skip `codesign --sign -` on macOS 15+; improved ERR trap with
  command, exit code, line, and FUNCNAME stack; added `detect_ide_path()` via
  `mdfind` and standard path fallback (Thanks to @aukgit)
- **`scripts/Fix_Damaged.command`**: recursive `find … xattr -d` quarantine
  clearing; `spctl --add` on macOS 13+; bilingual status messages
  (Thanks to @aukgit)
- **`scripts/package_dmg.sh`**: ensure `Fix_Damaged.command` is `chmod +x`
  and bundled; strip quarantine from produced DMG (Thanks to @aukgit)
- **`src-tauri/src/modules/process.rs`**: structured JSON IDE-discovery log at
  `data_local_dir/antigravity/ide-discovery.log`; `$HOME/Applications/`
  candidate path added (Thanks to @aukgit)
- **`src-tauri/src/modules/instance.rs`**: `$HOME/Applications/*.app/Contents/MacOS/`
  candidate path; diagnostic JSON on launch failure; `RUST_BACKTRACE=1` on
  child spawn (Thanks to @aukgit)
```

### Step 3 — Populate `CHANGELOG_EN.md`

Mirror the same entry as Step 2 in `CHANGELOG_EN.md` (English only, same
format).

### Step 4 — Update `README.md`

Under `## 📝 更新日志`, add a concise entry for `v4.144.0`:

```markdown
### v4.144.0 — YYYY-MM-DD
- macOS Installation Deep Hardening: robust mount-point extraction, recursive
  quarantine clearing, Ventura/Sequoia `spctl` + `xattr` hardening, bilingual
  Fix_Damaged.command, Rust IDE-discovery logging with `$HOME/Applications`
  parity.
```

### Step 5 — Update `README_EN.md`

Under `## 📝 Changelog`, mirror the Step 4 entry in English.

### Step 6 — Pre-flight Checks

```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
npm run build
```

All three must pass with zero errors before proceeding.

### Step 7 — Commit

```bash
gitmap cpf
```

Commit message (the `cpf` template will prompt or autofill):
```
feat(macos): deep hardening for Ventura/Sequoia install, Fix_Damaged, DMG packaging, and Rust IDE-discovery log parity
```

### Step 8 — Tag and Push

```bash
git tag v4.144.0
git push origin v4.144.0
```

Confirm the tag is on `main` branch only (stable release channel).

---

## Non-Negotiables

- Attribution in changelogs: `(Thanks to @aukgit)` only — no other GitHub handles.
- README files (`README.md`, `README_EN.md`) **must** be updated for stable releases.
- The `CHANGELOG.md` heading must be exactly `## v4.144.0` (character-for-character match with the git tag `v4.144.0`).
- Spec Subagents must not execute Steps 6–8; those are Lead-agent-only.

---

## Acceptance Criteria

- [ ] `npm run bump minor` executed and all three manifests show `4.144.0`
- [ ] `CHANGELOG.md` has `## v4.144.0` heading with all five bullet items
- [ ] `CHANGELOG_EN.md` mirrors the English changelog entry
- [ ] `README.md` `## 📝 更新日志` updated with v4.144.0 summary
- [ ] `README_EN.md` `## 📝 Changelog` updated with v4.144.0 summary
- [ ] Pre-flight checks pass: `cargo fmt`, `cargo clippy`, `npm run build`
- [ ] `gitmap cpf` commit created with descriptive message
- [ ] `git tag v4.144.0` created and pushed to `origin`
- [ ] Tag lands on `main` branch only (not `beta`)
