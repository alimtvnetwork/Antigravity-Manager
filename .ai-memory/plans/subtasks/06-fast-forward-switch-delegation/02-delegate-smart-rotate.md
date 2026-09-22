# Subtask 02: Delegate Smart Rotate to Proven switchAccount Pipeline

## Objective
Refactor `smartRotateProfileAccount` and `smartPlayInstance` in `src/stores/useInstanceStore.ts` and ensure button handlers in `src/components/navbar/InstanceSelector.tsx` delegate the account switch execution to `useAccountStore.getState().switchAccount`.

## Detailed Requirements
1. In `src/stores/useInstanceStore.ts`:
   - Keep logical workspace/instance selection and multiplicative candidate ranking.
   - When candidate account is verified, resolve `targetIdeParam`:
     `const targetIdeParam = (instId && instId !== 'default') ? ('instance:' + instId) : undefined;`
   - Call `await useAccountStore.getState().switchAccount(verified.account.id, targetIdeParam);`.
   - In `smartPlayInstance`, also use `await useAccountStore.getState().switchAccount(candidate.id, targetIdeParam);`.
   - Maintain post-switch synchronizations (`fetchInstances`, `fetchAccounts`, auto-resume recent prompts).
2. In `src/components/navbar/InstanceSelector.tsx`:
   - Ensure `onClick={(e) => { e.stopPropagation(); handleSmartRotate(activeInstance?.config.id); }}` properly blocks bubbling.

## Status
- [x] completed
