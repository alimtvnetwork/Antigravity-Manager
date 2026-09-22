# Issue 29: TypeScript Instance Service Missing Exports and Rustfmt Drift

## Why It Happened
During previous feature refactoring on multi-instance profile management and watchdog background tasks, `useInstanceStore.ts` was updated to invoke `wipeSession`, `pickBestCandidateAccount`, and `selectNextBestProfile`. However, in `src/services/instanceService.ts`, the functions were named `wipeInstanceSession`, `findSmartRotationAccount`/`findBestSmartPlayAccount`, and `findBestRotationProfile`. Consequently, TypeScript strict compilation failed with `error TS2339`. Additionally, recent backend code edits introduced slight whitespace/formatting differences causing `cargo fmt --check` to exit non-zero.

## How It Happened
1. Feature work on instance auto-switching introduced calls to `instanceService.pickBestCandidateAccount`, `instanceService.selectNextBestProfile`, and `instanceService.wipeSession`.
2. The TypeScript compiler (`tsc --noEmit`) flagged these missing exported identifiers on `instanceService`.
3. Concurrently, manual edits to Rust files across `src-tauri` diverged slightly from rustfmt standard line breaking rules.

## Root Cause
- Missing backward-compatible alias exports and wrapper implementation in `src/services/instanceService.ts`.
- Unformatted Rust source files in `src-tauri`.

## Code Fix
1. In `src/services/instanceService.ts`:
   - Export alias: `export const wipeSession = wipeInstanceSession;`.
   - Export alias: `export const selectNextBestProfile = findBestRotationProfile;`.
   - Implement and export:
     ```typescript
     export function pickBestCandidateAccount(
         accounts: Account[],
         activeInUseAccountIds: string[] = [],
         currentAccountId?: string
     ): Account | null {
         const smart = findSmartRotationAccount(accounts, currentAccountId, activeInUseAccountIds);
         if (smart?.account) {
             return smart.account;
         }
         return findBestSmartPlayAccount(accounts);
     }
     ```
2. In `src-tauri`:
   - Execute `cargo fmt --manifest-path src-tauri/Cargo.toml`.
