import * as instanceService from '../../services/instanceService';
import * as accountService from '../../services/accountService';
import { useErrorStore } from '../error-store';
import { useConfigStore } from '../useConfigStore';
import type { Account } from '../../types/account';
import type { InstanceState } from './types';

type SetState = (p: Partial<InstanceState> | ((s: InstanceState) => Partial<InstanceState>)) => void;
type GetState = () => InstanceState;

export function createSmartSlice(set: SetState, get: GetState): Pick<InstanceState,
    'smartPlayInstance' | 'rotateToNextBestProfile' | 'smartRotateProfileAccount' |
    'cleanAndRestartWorkspace' | 'resumeRecentProjectPrompts'> {
    return {
    smartPlayInstance: async (instanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            const instId = instanceId || get().activeInstanceId || 'default';
            const cur = get().instances.find(i => i.config.id === instId);
            const instanceName = cur?.config.name || instId;

            const isAlreadyRunning = cur?.is_running;
            if (isAlreadyRunning) {
                set({ isLoading: false });
                return {
                    accountEmail: cur?.config.bound_email || 'Already Running',
                    instanceName,
                };
            }

            const activeInUseAccountIds = get()
                .instances.filter(i => i.is_running && i.config.bound_account_id)
                .map(i => i.config.bound_account_id as string);

            const { useAccountStore } = await import('../useAccountStore');
            let accounts = useAccountStore.getState().accounts;
            const hasAccounts = accounts.length > 0;
            if (!hasAccounts) {
                await useAccountStore.getState().fetchAccounts();
                accounts = useAccountStore.getState().accounts;
            }

            let candidate = instanceService.pickBestCandidateAccount(
                accounts,
                activeInUseAccountIds,
                cur?.config.bound_account_id,
                instanceService.tierMultipliersFromSettings(useConfigStore.getState().config?.auto_profile_switcher)
            );

            if (!candidate) {
                set({ isLoading: false });
                throw new Error('No accounts verified with 100% 4-hour quota available to play this profile.');
            }

            let targetIdeParam: string | undefined;
            if (instId) {
                if (instId !== 'default') {
                    targetIdeParam = `instance:${instId}`;
                }
            }
            await useAccountStore.getState().switchAccount(candidate.id, targetIdeParam);

            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchCurrentAccount(),
                useAccountStore.getState().fetchAccounts(),
            ]);

            set({ activeInstanceId: instId, isLoading: false });

            return {
                accountEmail: candidate.email,
                instanceName,
            };
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to play instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.smartPlayInstance' });
            throw err;
        }
    },

    rotateToNextBestProfile: async (sourceInstanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            const currentActiveId = sourceInstanceId || get().activeInstanceId || 'default';
            const instances = get().instances;

            const nextBest = instanceService.selectNextBestProfile(instances, currentActiveId);
            if (!nextBest) {
                set({ isLoading: false });
                throw new Error('No available idle profile found for rotation');
            }

            await instanceService.closeInstance(currentActiveId);
            await instanceService.setActiveInstance(nextBest.config.id);
            await instanceService.launchInstance(nextBest.config.id);

            await get().fetchInstances(true);
            set({ activeInstanceId: nextBest.config.id, isLoading: false });

            return nextBest;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to rotate profile' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.rotateToNextBestProfile' });
            throw err;
        }
    },

    smartRotateProfileAccount: async (targetInstanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            const instId = targetInstanceId || get().activeInstanceId || 'default';
            const cur = get().instances.find(i => i.config.id === instId);
            const instanceName = cur?.config.name || instId;

            // 1. Discover accounts (always fetch fresh from backend to prevent stale state)
            const { useAccountStore } = await import('../useAccountStore');
            await useAccountStore.getState().fetchAccounts();
            const accounts = useAccountStore.getState().accounts;
            const currentAccountId = cur?.config.bound_account_id || useAccountStore.getState().currentAccount?.id;

            // 2. Discover active non-stale instances and exclude in-use accounts with >0% credits
            //    Bindings older than stale_binding_timeout_hours (default 6h) with no running process or 0% credits are treated as inactive
            const activeInUseAccountIds = get()
                .instances.filter(i => {
                    if (!i.config.bound_account_id) return false;
                    if (instanceService.isInstanceBindingStale(i, 6)) return false;
                    const boundAcc = accounts.find(a => a.id === i.config.bound_account_id);
                    if (boundAcc && instanceService.extract4hWindowQuotaPercent(boundAcc) <= 0) {
                        return false;
                    }
                    return i.is_running;
                })
                .map(i => i.config.bound_account_id as string)
                .filter(id => id !== currentAccountId);
            if (currentAccountId && !activeInUseAccountIds.includes(currentAccountId)) {
                activeInUseAccountIds.push(currentAccountId);
            }

            // 3. Filter eligible accounts (not disabled, not proxy_disabled, not forbidden, not blocked)
            const eligibleAccounts = accounts.filter(acc => {
                const isDisabled = Boolean(acc.disabled) || Boolean(acc.proxy_disabled);
                if (isDisabled) return false;
                const isForbidden = Boolean(acc.quota?.is_forbidden);
                if (isForbidden) return false;
                const isBlocked = Boolean(acc.validation_blocked);
                if (isBlocked) return false;
                return true;
            });

            const hasEligible = eligibleAccounts.length > 0;
            if (!hasEligible) {
                set({ isLoading: false });
                throw new Error('No eligible active accounts available for transfer');
            }

            // 4. Rank candidate accounts based on Normalized Multiplicative Scoring (÷ 1000):
            //    < 100% 4h quota = 0; 100% 4h quota = (S_active * M_tier * Q_weekly) / 1000
            let candidatePool = instanceService.rankSmartCandidates(
                eligibleAccounts,
                activeInUseAccountIds,
                currentAccountId,
                instanceService.tierMultipliersFromSettings(useConfigStore.getState().config?.auto_profile_switcher)
            );

            const hasCandidates = candidatePool.length > 0;
            if (!hasCandidates) {
                set({ isLoading: false });
                throw new Error('No eligible candidates available for rotation');
            }

            // 5. Pre-activation Live Quota Refresh Verification Loop:
            //    Confirm candidate actually has 100% quota for the 4-hour rolling window,
            //    then pass the verified candidate directly to the unified 5-Step Switch Button handler!
            let verifiedCandidate: Account | null = null;
            const triedAccountIds = new Set<string>();

            while (candidatePool.length > 0) {
                const topCandidate = candidatePool[0];
                if (!topCandidate) {
                    break;
                }
                const alreadyTried = triedAccountIds.has(topCandidate.account.id);
                if (alreadyTried) {
                    candidatePool = candidatePool.slice(1);
                    continue;
                }
                triedAccountIds.add(topCandidate.account.id);

                try {
                    // Live quota refresh probe directly from Google API
                    const freshQuota = await accountService.fetchAccountQuota(topCandidate.account.id);
                    const updatedAccount: Account = {
                        ...topCandidate.account,
                        quota: freshQuota,
                    };

                    const fresh4hQuota = instanceService.extract4hWindowQuotaPercent(updatedAccount);
                    const freshWeeklyQuota = instanceService.extractWeeklyQuotaPercent(updatedAccount);

                    const isForbidden = Boolean(freshQuota.is_forbidden);
                    const isBlocked = Boolean(updatedAccount.validation_blocked);
                    const isWeeklyHealthy = freshWeeklyQuota > 15;

                    console.log(
                        `[useInstanceStore] Pre-switch live verification: ${updatedAccount.email} -> 4h quota: ${fresh4hQuota}%, weekly: ${freshWeeklyQuota}%`
                    );

                    // User invariant: Must confirm 100% for 4-hour window!
                    if (!isForbidden && !isBlocked && fresh4hQuota >= 100 && isWeeklyHealthy) {
                        console.log(
                            `[useInstanceStore] Candidate ${updatedAccount.email} confirmed with 100% 4h window quota. Selected for switch!`
                        );
                        verifiedCandidate = updatedAccount;
                        break;
                    }

                    console.warn(
                        `[useInstanceStore] Candidate ${updatedAccount.email} live 4h window quota is ${fresh4hQuota}% (< 100%). Strictly rejecting (<100% treated as zero). Checking next candidate...`
                    );

                    // Remove current candidate and advance to next best
                    candidatePool = candidatePool.slice(1);
                } catch (probeErr) {
                    useErrorStore.getState().trackWarning(probeErr, {
                        source: 'useInstanceStore.smartRotateProfileAccount',
                        triggerAction: 'live-quota-probe',
                    });
                    candidatePool = candidatePool.slice(1);
                }
            }

            if (!verifiedCandidate) {
                set({ isLoading: false });
                throw new Error('No accounts verified with 100% 4-hour quota in pool. All available accounts are below 100%.');
            }

            const targetCandidate = verifiedCandidate as Account & { instanceId?: string };

            // Detect if the target candidate belongs to another instance
            const candidateBoundInstance = get().instances.find(
                i => i.config.bound_account_id === targetCandidate.id
            );
            const candidateInstanceId =
                targetCandidate.instanceId || candidateBoundInstance?.config.id;

            if (candidateInstanceId && candidateInstanceId !== instId) {
                // Gracefully close the depleted instance first
                await get().closeInstance(instId);
                await instanceService.setActiveInstance(candidateInstanceId);
            }

            const effectiveInstanceId = candidateInstanceId || instId;

            // 7. Delegate execution directly to proven switchAccount command (Button 2 delegation)
            let targetIdeParam: string | undefined;
            if (effectiveInstanceId) {
                if (effectiveInstanceId !== 'default') {
                    targetIdeParam = `instance:${effectiveInstanceId}`;
                }
            }
            await useAccountStore.getState().switchAccount(targetCandidate.id, targetIdeParam);

            const isTargetRunning = get().instances.find(i => i.config.id === effectiveInstanceId)?.is_running;
            if (!isTargetRunning) {
                await get().launchInstance(effectiveInstanceId);
            }

            // 8. Auto-resume recent active prompts (<1h) if enabled
            let resumeResult: instanceService.AutoResumeResult | null = null;
            try {
                resumeResult = await instanceService.resumeRecentProjectPrompts(effectiveInstanceId);
            } catch (resumeErr) {
                useErrorStore.getState().trackWarning(resumeErr, {
                    source: 'useInstanceStore.smartRotateProfileAccount',
                    triggerAction: 'auto-resume-prompts',
                });
            }

            // 9. Synchronize UI state (Ensure fetchInstances() and fetchAccounts() are called upon rotation completion)
            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchAccounts(),
                useAccountStore.getState().fetchCurrentAccount(),
            ]);

            const finalInst = get().instances.find(i => i.config.id === effectiveInstanceId);
            const resolvedInstanceName = finalInst?.config.name || instanceName;

            set({ activeInstanceId: effectiveInstanceId, isLoading: false });

            return {
                accountEmail: targetCandidate.email,
                instanceName: resolvedInstanceName,
                daysUntilRefill: 0,
                resumedProjectsCount: resumeResult?.resumed_project_count ?? 0,
                skippedProjectsCount: resumeResult?.skipped_project_count ?? 0,
            };
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to smart rotate profile' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.smartRotateProfileAccount' });
            throw err;
        }
    },

    cleanAndRestartWorkspace: async () => {
        set({ isLoading: true, error: null });
        try {
            const msg = await instanceService.cleanAndRestartWorkspace();
            await get().fetchInstances(true);
            set({ isLoading: false });
            return msg;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to clean and restart workspace' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.cleanAndRestartWorkspace' });
            throw err;
        }
    },

    resumeRecentProjectPrompts: async (instanceId?: string) => {
        try {
            const instId = instanceId || get().activeInstanceId || 'default';
            return await instanceService.resumeRecentProjectPrompts(instId);
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.resumeRecentProjectPrompts' });
            throw err;
        }
    },
    };
}
