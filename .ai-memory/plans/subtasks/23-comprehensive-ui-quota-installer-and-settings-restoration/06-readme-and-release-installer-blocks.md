# Subtask 06: Root README Overhaul & Release Page Installer Split Blocks
Traceability ID: Task-11, Task-12
Spec Reference: [02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md](../../../02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md)
Target Files: readme.md, README_EN.md, 03-ai-scripts/29-release-orchestrator.py, .github/workflows/release.yml
Action:
- Overhaul `readme.md`: remove outdated v2.0.0 about screen and legacy Chinese screenshots.
- Replace with current English UI assets (`assets/screenshots/prnt-0b5O014ZQS37.png`, modern layout).
- Split Direct Latest and Pinned Version one-liner commands into two distinct code blocks in `readme.md` and release templates.
- Ensure pinned version PowerShell command works cleanly: e.g. `& ([scriptblock]::Create((irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1))) -Version <version>`.
- In `release.yml` and release assets, rename Windows installer asset to include version: `agm-alim_4.65.3_x64-setup.exe` (or `Antigravity_Manager_Tools_4.65.3_x64-setup.exe`).
Acceptance Criteria:
- Root README has clean English visuals and modern branding.
- One-liner commands copy with 1 click without mixed comments.
Status: Completed
Targeted Verification: Replaced outdated about-dark screenshot with dashboard-modern.png, updated readme.md, README_EN.md, and release orchestrator to use separated code blocks with verified & ([scriptblock]::Create(...)) syntax, and removed unversioned setup.exe copy in release.yml.
