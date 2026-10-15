# Plan 151 — AGM rebrand to alimtvnetwork + uninstaller + cask automation

## Context
User: "agm should be my one not from the other guy" — the bundle identifier
`com.lbjlaq.antigravity-tools` (upstream author's ID) must become the user's
own: `com.alimtvnetwork.antigravity-tools`. Plus: build the Linux
uninstaller and automate the Homebrew cask version bump (user said "yes
please" to both).

## Task 1: Rebrand bundle identifier
`com.lbjlaq.antigravity-tools` → `com.alimtvnetwork.antigravity-tools`

Files to change:
- `src-tauri/tauri.conf.json`: `"identifier"`
- `install.sh`: `APP_ID=`, all `com.lbjlaq.antigravity-tools.desktop` refs
  (keep old names in the legacy-cleanup lists so old installs migrate)
- `Casks/antigravity-tools.rb`: zap trash paths
- `scripts/Install_Or_Repair.command`, `scripts/Fix_Damaged.command`,
  `scripts/fix_app.sh`, `scripts/pkg-scripts/postinstall`,
  `scripts/pkg-scripts/preinstall`: `BUNDLE_ID=`
- The awk lsregister patterns `/com\.lbjlaq\.antigravity-tools|.../`:
  match BOTH old and new IDs (migration safety)

Do NOT touch: `.ai-memory/`, `02-spec/` (historical records),
`scripts/close_integrated_prs.sh` (PR history references).

## Task 2: Linux uninstaller (`uninstall.sh`)
Mirror install.sh conventions (colors, sudo handling, DRY_RUN):
- `dpkg -r antigravity-manager-tools` (apt) / rpm remove (dnf/yum)
- Remove `~/.local/share/applications/agm-alim.desktop` + legacy .desktop files
- Remove `~/.local/bin/agm` / `agm-alim` symlinks (if pointing at our app)
- Optional `--purge` flag: also remove `~/.config/antigravity-manager-tools`,
  `~/.local/share/antigravity-manager-tools` (user data)
- `--help`, `--dry-run` flags

## Task 3: Cask version automation
`Casks/antigravity-tools.rb` pins `version "4.200.1"`. Create
`scripts/bump-cask-version.sh` that:
- Reads version from `package.json`
- Updates the `version "..."` line in the cask
- Verifies the new DMG/.deb URLs return HTTP 200
- Prints clear success/failure

## Verification
- `bash -n` on all shell scripts
- `grep -r "com.lbjlaq" src-tauri/tauri.conf.json install.sh Casks/ scripts/` → empty (except intentional legacy-cleanup entries)
- `uninstall.sh --help` and `--dry-run` output sane

## Out of scope
- Rebuilding/releasing (bundle ID change needs a new release to take effect;
  flag this to the user)
- Data migration for existing users (old `~/Library/Application Support/com.lbjlaq.*`
  stays; new ID uses fresh paths)
