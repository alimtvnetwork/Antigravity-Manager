# Subtask 05: GitMap Credential Commands, SSH Conversion & Windows Defender Fix

- Remove `vssadmin` shadow copy execution/strings in `cli/cmdchromeprofile/chromeprofile_vss_windows.go`, hidden PowerShell WMI/taskkill in `cli/fspath/lockcheck_windows.go`, and `DefaultPassword` in `cli/cmdos/os_autologin_windows.go`.
- Update `cli/cmdpull/pull_remediation_fallback.go` and `cli/cmdpull/pull_efficient_render.go` to show credential & SSH commands in each failed repo's `Solutions:` subtree and a unified command at the bottom.
- Update `cli/cmdpull/pull_remediation.go` and `cli/cmdfix/fix_cmd.go` so `gitmap fix` and `gitmap fix --all` auto-remediate `wincredman` and convert same-owner (`alimtvnetwork`) HTTPS remotes to SSH (`git@github.com:alimtvnetwork/<repo>.git`).
