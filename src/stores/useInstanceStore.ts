import { create } from 'zustand';
import * as instanceService from '../services/instanceService';
import * as accountService from '../services/accountService';
import { useErrorStore } from './error-store';
import { useConfigStore } from './useConfigStore';
import { showToast } from '../components/common/ToastContainer';
import type { Account } from '../types/account';
import type {
    InstanceConfig,
    InstanceStatus,
    AutoProfileSwitcherConfig,
    AutoSwitcherStatus,
} from '../services/instanceService';

interface InstanceState {
    instances: InstanceStatus[];
    activeInstanceId: string;
    switcherStatus: AutoSwitcherStatus | null;
    isLoading: boolean;
    error: string | null;

    fetchInstances: (silent?: boolean) => Promise<void>;
    fetchSwitcherStatus: () => Promise<void>;
    updateSwitcherConfig: (config: AutoProfileSwitcherConfig) => Promise<void>;
    triggerManualRotation: () => Promise<string>;
    createInstance: (name: string, boundAccountId?: string, fromInstanceId?: string) => Promise<InstanceConfig>;
    copyInstance: (sourceId: string, targetName: string, cloneMode?: string, copyProjects?: boolean) => Promise<InstanceConfig>;
    renameInstance: (instanceId: string, newName: string) => Promise<InstanceConfig>;
    deleteInstance: (instanceId: string) => Promise<void>;
    wipeSession: (instanceId: string) => Promise<void>;
    launchInstance: (instanceId: string) => Promise<void>;
    cloneInstanceExecutable: (instanceId: string) => Promise<string>;
    setInstanceExecutable: (instanceId: string, executablePath?: string) => Promise<void>;
    closeInstance: (instanceId: string) => Promise<void>;
    stopInstance: (instanceId: string) => Promise<void>;
    restartInstance: (instanceId: string) => Promise<InstanceStatus>;
    fastForwardInstance: (instanceId: string) => Promise<string>;
    toggleAutoSwitcher: () => Promise<void>;
    setActiveInstance: (instanceId: string) => Promise<void>;
    setDefaultInstance: (instanceId: string) => Promise<void>;
    switchAccountToInstance: (accountId: string, instanceId?: string) => Promise<void>;
    exportInstancesJson: () => Promise<string>;
    importInstancesJson: (jsonContent: string) => Promise<InstanceConfig[]>;
    smartPlayInstance: (instanceId?: string) => Promise<{ accountEmail: string; instanceName: string }>;
    rotateToNextBestProfile: (sourceInstanceId?: string) => Promise<InstanceStatus>;
    smartRotateProfileAccount: (targetInstanceId?: string) => Promise<{
        accountEmail: string;
        instanceName: string;
        daysUntilRefill: number;
        resumedProjectsCount?: number;
        skippedProjectsCount?: number;
    }>;
    cleanAndRestartWorkspace: () => Promise<string>;
    resumeRecentProjectPrompts: (instanceId?: string) => Promise<instanceService.AutoResumeResult>;
    syncInstance: (instanceId: string) => Promise<void>;
    syncAllInstances: () => Promise<void>;
}

let instanceSelectionEpoch = 0;

export const useInstanceStore = create<InstanceState>((set, get) => ({
    instances: [],
    activeInstanceId: 'default',
    switcherStatus: null,
    isLoading: false,
    error: null,

    fetchInstances: async (silent: boolean = false) => {
        const epochAtStart = instanceSelectionEpoch;
        if (!silent) {
            set({ isLoading: true, error: null });
        }
        try {
            const [instances, activeId] = await Promise.all([
                instanceService.listInstances(),
                instanceService.getActiveInstance(),
            ]);
            if (epochAtStart !== instanceSelectionEpoch) {
                set({ instances, isLoading: false });
                return;
            }
            set({ instances, activeInstanceId: activeId, isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to fetch instances' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.fetchInstances' });
        }
    },

    fetchSwitcherStatus: async () => {
        try {
            const status = await instanceService.getAutoSwitcherStatus();
            set({ switcherStatus: status });
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.fetchSwitcherStatus' });
        }
    },

    updateSwitcherConfig: async (config: AutoProfileSwitcherConfig) => {
        try {
            await instanceService.updateAutoSwitcherConfig(config);
            await get().fetchSwitcherStatus();
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.updateSwitcherConfig' });
            throw err;
        }
    },

    triggerManualRotation: async () => {
        set({ isLoading: true, error: null });
        try {
            const targetId = get().activeInstanceId || 'default';
            const result = await get().smartRotateProfileAccount(targetId);
            const resumeNote = (result.resumedProjectsCount ?? 0) > 0
                ? ` (Auto-resumed ${result.resumedProjectsCount} project(s) <1h)`
                : '';
            const msg = `Successfully rotated to profile '${result.instanceName}' with account '${result.accountEmail}'${resumeNote}`;
            await Promise.all([
                get().fetchInstances(true),
                get().fetchSwitcherStatus(),
            ]);
            set({ isLoading: false });
            return msg;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to trigger profile rotation' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.triggerManualRotation' });
            throw err;
        }
    },

    createInstance: async (name: string, boundAccountId?: string, fromInstanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.createInstance(name, boundAccountId, fromInstanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to create instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.createInstance' });
            throw err;
        }
    },

    copyInstance: async (sourceId: string, targetName: string, cloneMode?: string, copyProjects?: boolean) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.copyInstance(sourceId, targetName, cloneMode, copyProjects);
            await get().fetchInstances(true);
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to copy instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.copyInstance' });
            throw err;
        }
    },

    renameInstance: async (instanceId: string, newName: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.renameInstance(instanceId, newName);
            await get().fetchInstances(true);
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to rename instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.renameInstance' });
            throw err;
        }
    },

    deleteInstance: async (instanceId: string) => {
        // Optimistically remove the instance immediately to prevent duplicate UI clicks
        set((state) => ({
            instances: state.instances.filter((i) => i.config.id !== instanceId),
            isLoading: true,
            error: null,
        }));
        try {
            await instanceService.deleteInstance(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            const errStr = err?.toString() || '';
            // If already deleted or not found, reconcile store gracefully without throwing fatal error
            if (errStr.toLowerCase().includes('not found')) {
                await get().fetchInstances(true);
                set({ isLoading: false });
                return;
            }
            set({ isLoading: false, error: errStr || 'Failed to delete instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.deleteInstance' });
            throw err;
        }
    },

    wipeSession: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.wipeSession(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to wipe session' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.wipeSession' });
            throw err;
        }
    },

    launchInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.launchInstance(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to launch instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.launchInstance' });
            throw err;
        }
    },

    cloneInstanceExecutable: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            const path = await instanceService.cloneInstanceExecutable(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
            return path;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to clone instance executable' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.cloneInstanceExecutable' });
            throw err;
        }
    },

    setInstanceExecutable: async (instanceId: string, executablePath?: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.setInstanceExecutable(instanceId, executablePath);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to set instance executable' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.setInstanceExecutable' });
            throw err;
        }
    },

    closeInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.closeInstance(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to close instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.closeInstance' });
            throw err;
        }
    },

    stopInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.stopInstance(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to stop instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.stopInstance' });
            throw err;
        }
    },

    restartInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            const status = await instanceService.restartInstance(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
            return status;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to restart instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.restartInstance' });
            throw err;
        }
    },

    fastForwardInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            const msg = await instanceService.fastForwardInstance(instanceId);
            await Promise.all([
                get().fetchInstances(true),
                get().fetchSwitcherStatus(),
            ]);
            set({ isLoading: false });
            return msg;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to fast forward instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.fastForwardInstance' });
            throw err;
        }
    },

    toggleAutoSwitcher: async () => {
        try {
            await instanceService.toggleAutoSwitcher();
            await get().fetchSwitcherStatus();
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.toggleAutoSwitcher' });
            throw err;
        }
    },

    setActiveInstance: async (instanceId: string) => {
        instanceSelectionEpoch += 1;
        const epoch = instanceSelectionEpoch;
        set({ isLoading: true, error: null, activeInstanceId: instanceId });
        try {
            await instanceService.setActiveInstance(instanceId);
            if (epoch === instanceSelectionEpoch) {
                set({ activeInstanceId: instanceId, isLoading: false });
            }
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to set active instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.setActiveInstance' });
            throw err;
        }
    },

    setDefaultInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.setDefaultInstance(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to set default instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.setDefaultInstance' });
            throw err;
        }
    },

    switchAccountToInstance: async (accountId: string, instanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.switchAccountToInstance(accountId, instanceId);
            const { useAccountStore } = await import('./useAccountStore');
            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchAccounts(),
                useAccountStore.getState().fetchCurrentAccount(),
            ]);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to switch account to instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.switchAccountToInstance' });
            throw err;
        }
    },

    exportInstancesJson: async () => {
        try {
            return await instanceService.exportInstancesJson();
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.exportInstancesJson' });
            throw err;
        }
    },

    importInstancesJson: async (jsonContent: string) => {
        set({ isLoading: true, error: null });
        try {
            const configs = await instanceService.importInstancesJson(jsonContent);
            await get().fetchInstances(true);
            set({ isLoading: false });
            return configs;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to import instances' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.importInstancesJson' });
            throw err;
        }
    },

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

            const { useAccountStore } = await import('./useAccountStore');
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
            const { useAccountStore } = await import('./useAccountStore');
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
                    console.warn(`[useInstanceStore] Live quota refresh probe failed for ${topCandidate.account.id}, skipping to next:`, probeErr);
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
                console.warn('[useInstanceStore] Auto-resume recent prompts notice:', resumeErr);
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

    syncInstance: async (instanceId: string) => {
        try {
            const updated = await instanceService.syncInstancePidAndQuota(instanceId);
            set(state => ({
                instances: state.instances.map(inst =>
                    inst.config.id === instanceId ? updated : inst
                ),
            }));
            const { useAccountStore } = await import('./useAccountStore');
            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchAccounts(),
                useAccountStore.getState().fetchCurrentAccount(),
            ]);
            showToast(`Synchronized PID & quota for instance '${updated?.config?.name || instanceId}'`, 'success');
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.syncInstance' });
            showToast(`Sync failed: ${err?.message || err}`, 'error');
            throw err;
        }
    },

    syncAllInstances: async () => {
        try {
            const updatedList = await instanceService.syncAllInstancesAndQuotas();
            if (updatedList && updatedList.length > 0) {
                set({ instances: updatedList });
            }
            const { useAccountStore } = await import('./useAccountStore');
            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchAccounts(),
                useAccountStore.getState().fetchCurrentAccount(),
            ]);
            showToast(`Synchronized PID & quota across all instances`, 'success');
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.syncAllInstances' });
            showToast(`Sync all failed: ${err?.message || err}`, 'error');
            throw err;
        }
    },
}));

