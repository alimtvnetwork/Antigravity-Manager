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

            const tierMultipliers = instanceService.tierMultipliersFromSettings(
                useConfigStore.getState().config?.auto_profile_switcher
            );
            let candidate = instanceService.pickBestCandidateAccount(
                accounts,
                activeInUseAccountIds,
                cur?.config.bound_account_id,
                tierMultipliers
            );

            if (!candidate) {
                set({ isLoading: false });
                throw new Error('No eligible accounts with remaining quota available to play this profile.');
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
            //    Primary pass ranks >= 100% 4h quota; fallback pass ranks > 0% 4h quota proportionally
            const tierMultipliers = instanceService.tierMultipliersFromSettings(
                useConfigStore.getState().config?.auto_profile_switcher
            );
            let candidatePool = instanceService.rankSmartCandidates(
                eligibleAccounts,
                activeInUseAccountIds,
                currentAccountId,
                tierMultipliers
            );

            const hasCandidates = candidatePool.length > 0;
            if (!hasCandidates) {
                set({ isLoading: false });
                throw new Error('No eligible candidates available for rotation');
            }

            // 5. Pre-activation Live Quota Refresh Verification Loop:
            //    Step 1/2: Prefer candidates verified at 100% 4-hour rolling window quota.
            //    Step 3 (Proportional Sub-100% Fallback): Track the highest-scoring candidate with > 0% 4h quota
            //    when all probed candidates are below 100%.
            let verifiedCandidate: Account | null = null;
            let bestFallbackCandidate: { account: Account; score: number } | null = null;
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

                    // Primary pass: confirm 100% for 4-hour window
                    if (!isForbidden && !isBlocked && fresh4hQuota >= 100 && isWeeklyHealthy) {
                        console.log(
                            `[useInstanceStore] Candidate ${updatedAccount.email} confirmed with 100% 4h window quota. Selected for switch!`
                        );
                        verifiedCandidate = updatedAccount;
                        break;
                    }

                    // Step 3 Sub-100% Proportional Fallback tracking
                    if (!isForbidden && !isBlocked && fresh4hQuota > 0 && freshWeeklyQuota >= 8) {
                        const fallbackRes = instanceService.calculateFallbackMultiplicativeScore(
                            updatedAccount,
                            activeInUseAccountIds,
                            currentAccountId,
                            undefined,
                            tierMultipliers
                        );
                        if (
                            fallbackRes.score > 0 &&
                            (!bestFallbackCandidate || fallbackRes.score > bestFallbackCandidate.score)
                        ) {
                            bestFallbackCandidate = {
                                account: updatedAccount,
                                score: fallbackRes.score,
                            };
                        }
                    }

                    console.warn(
                        `[useInstanceStore] Candidate ${updatedAccount.email} live 4h window quota is ${fresh4hQuota}% (< 100%). Checking next candidate...`
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

            // Step 3 (Proportional Sub-100% Fallback): use best live-probed or refreshed sub-100% account
            if (!verifiedCandidate && bestFallbackCandidate) {
                console.log(
                    `[useInstanceStore] Using Step 3 proportional fallback candidate ${bestFallbackCandidate.account.email} (score: ${bestFallbackCandidate.score}).`
                );
                verifiedCandidate = bestFallbackCandidate.account;
            }

            if (!verifiedCandidate) {
                const refreshedAccounts = useAccountStore.getState().accounts;
                verifiedCandidate = instanceService.pickBestCandidateAccount(
                    refreshedAccounts.length > 0 ? refreshedAccounts : eligibleAccounts,
                    activeInUseAccountIds,
                    currentAccountId,
                    tierMultipliers
                );
            }

            if (!verifiedCandidate) {
                set({ isLoading: false });
                throw new Error('No eligible accounts with remaining 4-hour and weekly quota available in pool.');
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

            // Refresh instance state FIRST before checking is_running to avoid duplicate launch after switchAccount relaunched
            await get().fetchInstances(true);

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
