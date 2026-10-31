# Subtask 03: Audit Candidate Account Selection Synchronization

- **Task**: Instance-Based Prompt and UI Functionality Verification
- **Subtask**: 03 of 06
- **Status**: Completed
- **Path**: `.ai-memory/plans/subtasks/241-instance-based-prompt-and-ui-functionality-verification/03-subtask-account-selection-synchronization.md`

## Audit Outcomes
- Verified `src-tauri/src/modules/auto_switcher/selection.rs`:
  - `select_candidate_profiles` automatically evaluates candidate accounts without requiring pre-selection.
  - Urgent expiration tier prioritizes accounts expiring in <= 24h with weekly quota >= 15% (`extract_urgent_expiration_hours`), sorted ascending by remaining hours so expiring quota is used first.
  - Cluster-wide active lease exclusion (`get_remote_cluster_active_accounts`, `is_account_or_email_leased_by_other`) eliminates remote account collision.
  - Frontend selection rationale exposed transparently via 3 tiers in `src/stores/useInstanceStore/smart.ts`.
