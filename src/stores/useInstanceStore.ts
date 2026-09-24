import { create } from 'zustand';
import * as instanceService from '../services/instanceService';
import * as accountService from '../services/accountService';
import { useErrorStore } from './error-store';
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
    createInstance: (name: string) => Promise<InstanceConfig>;
    copyInstance: (sourceId: string, targetName: string, cloneMode?: string) => Promise<InstanceConfig>;
    renameInstance: (instanceId: string, newName: string) => Promise<InstanceConfig>;
    deleteInstance: (instanceId: string) => Promise<void>;
    wipeSession: (instanceId: string) => Promise<void>;
    launchInstance: (instanceId: string) => Promise<void>;
    cloneInstanceExecutable: (instanceId: string) => Promise<string>;
    setInstanceExecutable: (instanceId: string, executablePath?: string) => Promise<void>;
    closeInstance: (instanceId: string) => Promise<void>;
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
}

export const useInstanceStore = create<InstanceState>((set, get) => ({
    instances: [],
    activeInstanceId: 'default',
    switcherStatus: null,
    isLoading: false,
    error: null,

    fetchInstances: async (silent: boolean = false) => {
        if (!silent) {
            set({ isLoading: true, error: null });
        }
        try {
            const [instances, activeId] = await Promise.all([
                instanceService.listInstances(),
                instanceService.getActiveInstance(),
            ]);
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

    createInstance: async (name: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.createInstance(name);
            await get().fetchInstances(true);
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to create instance' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.createInstance' });
            throw err;
        }
    },

    copyInstance: async (sourceId: string, targetName: string, cloneMode?: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.copyInstance(sourceId, targetName, cloneMode);
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
        set({ isLoading: true, error: null });
        try {
            await instanceService.deleteInstance(instanceId);
            await get().fetchInstances(true);
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to delete instance' });
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

    setActiveInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.setActiveInstance(instanceId);
            set({ activeInstanceId: instanceId, isLoading: false });
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
            let targetIde: string | undefined;
            if (instanceId) {
                if (instanceId !== 'default') {
                    targetIde = `instance:${instanceId}`;
                }
            }
            const { useAccountStore } = await import('./useAccountStore');
            await useAccountStore.getState().switchAccount(accountId, targetIde);
            await get().fetchInstances(true);
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
                cur?.config.bound_account_id
            );

            if (!candidate) {
                candidate = accounts.find(a => !activeInUseAccountIds.includes(a.id)) || accounts[0];
            }

            if (!candidate) {
                set({ isLoading: false });
                throw new Error('No available account found to play this instance');
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
            const currentAccountId = cur?.config.bound_account_id;

            // 1. Close running processes from target profile directory first (for isolated sandboxes)
            if (instId !== 'default') {
                try {
                    await instanceService.closeInstance(instId);
                } catch (closeErr) {
                    console.warn('[useInstanceStore] Non-fatal close instance notice:', closeErr);
                }
            }

            // 2. Discover active running instances
            const runningInstances = get().instances.filter(i => i.is_running && i.config.bound_account_id);
            const activeInUseAccountIds = runningInstances
                .map(i => i.config.bound_account_id as string)
                .filter(id => id !== currentAccountId);

            // 3. Discover accounts
            const { useAccountStore } = await import('./useAccountStore');
            let accounts = useAccountStore.getState().accounts;
            const hasAccounts = accounts.length > 0;
            if (!hasAccounts) {
                await useAccountStore.getState().fetchAccounts();
                accounts = useAccountStore.getState().accounts;
            }

            // 4. Filter eligible accounts (not disabled, not forbidden, not blocked)
            const eligibleAccounts = accounts.filter(acc => {
                const isDisabled = Boolean(acc.disabled);
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
                throw new Error('No eligible accounts available for transfer');
            }

            // 5. Rank candidate accounts based on Multiplicative Scoring:
            // Score = S_active * M_tier * Q_weekly
            let candidatePool = instanceService.rankSmartCandidates(
                eligibleAccounts,
                activeInUseAccountIds,
                currentAccountId
            );

            const hasCandidates = candidatePool.length > 0;
            if (!hasCandidates) {
                set({ isLoading: false });
                throw new Error('No eligible candidates available for rotation');
            }

            // 6. Pre-activation Live Quota Refresh Verification Loop:
            // Probe the top candidate with live quota refresh.
            // If quota degraded or account is invalid, demote and try next best candidate.
            let verifiedCandidate: Account | null = null;
            const triedAccountIds = new Set<string>();

            while (candidatePool.length > 0) {
                const topCandidate = candidatePool[0];
                if (!topCandidate) {
                    break;
                }
                const alreadyTried = triedAccountIds.has(topCandidate.account.id);
                if (alreadyTried) {
                    break;
                }
                triedAccountIds.add(topCandidate.account.id);

                try {
                    // Live quota refresh probe
                    const freshQuota = await accountService.fetchAccountQuota(topCandidate.account.id);
                    const updatedAccount: Account = {
                        ...topCandidate.account,
                        quota: freshQuota,
                    };

                    // Re-evaluate score with live quota
                    const reScored = instanceService.calculateMultiplicativeScore(
                        updatedAccount,
                        activeInUseAccountIds,
                        currentAccountId
                    );

                    const isForbidden = Boolean(freshQuota.is_forbidden);
                    const isBlocked = Boolean(updatedAccount.validation_blocked);
                    const isDepleted = reScored.weeklyQuotaPercent <= 5;
                    const isZeroScore = reScored.score <= 0;

                    const isInvalid = isForbidden || isBlocked || isDepleted || isZeroScore;
                    if (!isInvalid) {
                        verifiedCandidate = updatedAccount;
                        break;
                    }

                    // Demote candidate to bottom of pool and re-sort
                    candidatePool = candidatePool
                        .filter(c => c.account.id !== topCandidate.account.id)
                        .concat({
                            ...topCandidate,
                            account: updatedAccount,
                            score: 0,
                            weeklyQuotaPercent: reScored.weeklyQuotaPercent,
                        });
                } catch (probeErr) {
                    console.warn(`[useInstanceStore] Live quota refresh probe failed for ${topCandidate.account.id}, demoting:`, probeErr);
                    candidatePool = candidatePool
                        .filter(c => c.account.id !== topCandidate.account.id)
                        .concat({
                            ...topCandidate,
                            score: 0,
                        });
                }
            }

            const targetCandidate = verifiedCandidate || candidatePool[0]?.account || eligibleAccounts[0];

            // 7. Delegate execution directly to proven switchAccount command (Button 2 delegation)
            let targetIdeParam: string | undefined;
            if (instId) {
                if (instId !== 'default') {
                    targetIdeParam = `instance:${instId}`;
                }
            }
            await useAccountStore.getState().switchAccount(targetCandidate.id, targetIdeParam);

            // 8. Auto-resume recent active prompts (<1h) if enabled
            let resumeResult: instanceService.AutoResumeResult | null = null;
            try {
                resumeResult = await instanceService.resumeRecentProjectPrompts(instId);
            } catch (resumeErr) {
                console.warn('[useInstanceStore] Auto-resume recent prompts notice:', resumeErr);
            }

            // 9. Synchronize UI state
            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchCurrentAccount(),
                useAccountStore.getState().fetchAccounts(),
            ]);

            set({ activeInstanceId: instId, isLoading: false });

            return {
                accountEmail: targetCandidate.email,
                instanceName,
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
}));
