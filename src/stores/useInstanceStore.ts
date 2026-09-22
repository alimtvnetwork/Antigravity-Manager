import { create } from 'zustand';
import * as instanceService from '../services/instanceService';
import { useErrorStore } from './error-store';
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
    switchAccountToInstance: (accountId: string, instanceId?: string) => Promise<void>;
    exportInstancesJson: () => Promise<string>;
    importInstancesJson: (jsonContent: string) => Promise<InstanceConfig[]>;
    smartPlayInstance: (instanceId?: string) => Promise<{ accountEmail: string; instanceName: string }>;
    rotateToNextBestProfile: (sourceInstanceId?: string) => Promise<InstanceStatus>;
    smartRotateProfileAccount: (targetInstanceId?: string) => Promise<{ accountEmail: string; instanceName: string; daysUntilRefill: number }>;
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
            const result = await instanceService.triggerManualProfileRotation();
            await Promise.all([
                get().fetchInstances(true),
                get().fetchSwitcherStatus(),
            ]);
            set({ isLoading: false });
            return result;
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

    switchAccountToInstance: async (accountId: string, instanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.switchAccountToInstance(accountId, instanceId);
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

            await instanceService.switchAccountToInstance(candidate.id, instId);

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

            // 1. Close running processes from target profile directory first
            await instanceService.closeInstance(instId);

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

            // 4. Rank candidate accounts using multiplicative formula
            const ranked = instanceService.rankSmartCandidates(
                accounts,
                activeInUseAccountIds,
                currentAccountId
            );
            const hasRanked = ranked.length > 0;
            if (!hasRanked) {
                set({ isLoading: false });
                throw new Error('No candidate accounts found for rotation');
            }

            // 5. Pre-activation verification and demotion loop
            let verified: instanceService.MultiplicativeCandidateResult | null = null;
            const queue = [...ranked];

            while (queue.length > 0) {
                const pick = queue.shift();
                if (!pick) continue;
                if (pick.score <= 0) continue;

                try {
                    await useAccountStore.getState().refreshQuota(pick.account.id);
                } catch {
                    continue;
                }

                const freshAccounts = useAccountStore.getState().accounts;
                const freshAccount = freshAccounts.find(a => a.id === pick.account.id);
                if (!freshAccount) continue;

                const freshScore = instanceService.calculateMultiplicativeScore(
                    freshAccount,
                    activeInUseAccountIds,
                    currentAccountId
                );

                const isUnused = freshScore.activeFactor === 1;
                const hasQuota = freshScore.weeklyQuotaPercent >= 10;
                if (isUnused) {
                    if (hasQuota) {
                        verified = { ...freshScore, isVerified: true };
                        break;
                    }
                }
            }

            if (!verified) {
                set({ isLoading: false });
                throw new Error('All candidate accounts were either depleted (<10% quota) or currently in use');
            }

            // 6. Inject verified account tokens into target profile state.vscdb and launch
            await instanceService.switchAccountToInstance(verified.account.id, instId);

            // 6.5 Auto-resume recent active prompts (<1h) if enabled
            try {
                await instanceService.resumeRecentProjectPrompts(instId);
            } catch (resumeErr) {
                console.warn('[useInstanceStore] Auto-resume recent prompts notice:', resumeErr);
            }

            // 7. Synchronize UI state
            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchCurrentAccount(),
                useAccountStore.getState().fetchAccounts(),
            ]);

            set({ activeInstanceId: instId, isLoading: false });

            return {
                accountEmail: verified.account.email,
                instanceName,
                daysUntilRefill: verified.daysUntilRefill,
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
