# Master Plan: Instance Restart Split Button, Distinct Sync Icons & Running Projects Detection Fix

## Task Summary
- **Slug**: `132-instance-restart-split-sync-icon-and-running-projects-fix`
- **Goal**: Implement instance restart on current account, contiguous split Play/Stop/Restart button, distinct sync icons, and root-cause fix for buggy running projects detection.

## Subtasks
1. `01-backend-instance-restart-command.md`: Implement `restart_instance` in `instance.rs` and `commands/instance.rs`.
2. `02-running-projects-detection-root-cause-fix.md`: Fix SQLite queries, 0-word untitled filter, and recency guards in `repo_db.rs`.
3. `03-frontend-split-restart-button-and-distinct-icons.md`: Add split Stop/Restart capsule, distinct `Cpu` sync icons, and preserve Switch button.
4. `04-verification-and-ci-cd-minor-release.md`: Run preflight checks, bump minor version, commit, tag, and verify green CI via `gitmap pe -t`.
