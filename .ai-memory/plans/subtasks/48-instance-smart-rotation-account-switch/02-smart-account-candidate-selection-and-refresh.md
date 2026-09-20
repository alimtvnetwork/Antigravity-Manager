# Subtask 02: Smart Account Candidate Selection & Refresh

## Status: Completed

## Parent Plan
[Plan 48: Instance Smart Rotation, Process Termination, Account Selection Algorithm & Minor Release v4.37.0](.ai-memory/plans/completed/48-instance-smart-rotation-account-switch.md)

## Goal
Implement candidate account scoring, selection, and live quota refresh verification based on 4-hour idle recency, refill runway (e.g. 6 days until reset), and headroom.

## Key Actions
1. In `src/services/instanceService.ts`:
   - Implement `findSmartRotationAccount(accounts: Account[], currentAccountId?: string): Account | null`:
     - Filter out disabled, forbidden, and validation blocked accounts.
     - Exclude the profile's current bound account if other eligible accounts exist.
     - Factor A (4-Hour Inactivity): If `last_used` was > 4 hours ago or never used, grant major priority (+100,000 pts).
     - Factor B (Refill Runway / Most Room): Calculate days until reset from `reset_time` / weekly bucket (`daysUntilReset >= 6` receives +50,000 pts).
     - Factor C (Quota Headroom): Minimum/average remaining percentage across models (100% full receives highest headroom score).
     - Factor D (Tier): Bonus for Ultra (+3,000 pts) and Pro (+2,000 pts).
2. Implement candidate quota refresh verification:
   - Before switching, refresh the candidate's quota to ensure active session and live validity.
   - If verification passes, proceed to switch.

## Constraints
- No explicit boolean checks (`if isReady == true`).
- Implicit booleans only.
- Strict relative paths.
