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
            set({ instances, activeInstanceId: activeId, isLoading: false, error: null });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to fetch instances' });
        }
    },

    fetchSwitcherStatus: async () => {
        try {
            const status = await instanceService.getAutoSwitcherStatus();
            set({ switcherStatus: status });
        } catch (err: any) {
            console.error('Failed to fetch switcher status:', err);
        }
    },

    updateSwitcherConfig: async (config: AutoProfileSwitcherConfig) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.updateAutoSwitcherConfig(config);
            await get().fetchSwitcherStatus();
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to update switcher config' });
            throw err;
        }
    },

    triggerManualRotation: async () => {
        set({ isLoading: true, error: null });
        try {
            const msg = await instanceService.triggerManualProfileRotation();
            await Promise.all([get().fetchInstances(), get().fetchSwitcherStatus()]);
            set({ isLoading: false });
            return msg;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to trigger manual rotation' });
            throw err;
        }
    },

    createInstance: async (name: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.createInstance(name);
            await get().fetchInstances();
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to create instance' });
            throw err;
        }
    },

    copyInstance: async (sourceId: string, targetName: string, cloneMode?: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.copyInstance(sourceId, targetName, cloneMode);
            await get().fetchInstances();
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to copy instance' });
            throw err;
        }
    },

    renameInstance: async (instanceId: string, newName: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.renameInstance(instanceId, newName);
            await get().fetchInstances();
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to rename instance' });
            throw err;
        }
    },

    deleteInstance: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.deleteInstance(instanceId);
            await get().fetchInstances();
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to delete instance' });
            throw err;
        }
    },

    wipeSession: async (instanceId: string) => {
        try {
            await instanceService.wipeInstanceSession(instanceId);
            await get().fetchInstances();
        } catch (err: any) {
            set({ error: err?.toString() || 'Failed to wipe session' });
            throw err;
        }
    },

    launchInstance: async (instanceId: string) => {
        try {
            await instanceService.launchInstance(instanceId);
            await get().fetchInstances();
        } catch (err: any) {
            set({ error: err?.toString() || 'Failed to launch instance' });
            useErrorStore.getState().captureError(err, { source: 'instance', triggerAction: 'launchInstance' });
            throw err;
        }
    },

    cloneInstanceExecutable: async (instanceId: string) => {
        set({ isLoading: true, error: null });
        try {
            const clonedPath = await instanceService.cloneInstanceExecutable(instanceId);
            await get().fetchInstances();
            set({ isLoading: false });
            return clonedPath;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to clone executable' });
            useErrorStore.getState().captureError(err, { source: 'instance', triggerAction: 'cloneInstanceExecutable' });
            throw err;
        }
    },

    setInstanceExecutable: async (instanceId: string, executablePath?: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.setInstanceExecutable(instanceId, executablePath);
            await get().fetchInstances();
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to set executable path' });
            throw err;
        }
    },

    closeInstance: async (instanceId: string) => {
        try {
            await instanceService.closeInstance(instanceId);
            await get().fetchInstances();
        } catch (err: any) {
            set({ error: err?.toString() || 'Failed to close instance' });
            throw err;
        }
    },

    setActiveInstance: async (instanceId: string) => {
        try {
            await instanceService.setActiveInstance(instanceId);
            set({ activeInstanceId: instanceId });
        } catch (err: any) {
            set({ error: err?.toString() || 'Failed to set active instance' });
            throw err;
        }
    },

    switchAccountToInstance: async (accountId: string, instanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            await instanceService.switchAccountToInstance(accountId, instanceId);
            await get().fetchInstances();
            set({ isLoading: false });
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to switch account' });
            useErrorStore.getState().captureError(err, { source: 'instance', triggerAction: 'switchAccountToInstance' });
            throw err;
        }
    },

    exportInstancesJson: async () => {
        return await instanceService.exportInstancesJson();
    },

    importInstancesJson: async (jsonContent: string) => {
        set({ isLoading: true, error: null });
        try {
            const configs = await instanceService.importInstancesJson(jsonContent);
            await get().fetchInstances();
            set({ isLoading: false });
            return configs;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to import instances JSON' });
            throw err;
        }
    },

    smartPlayInstance: async (targetInstanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            const instId = targetInstanceId || get().activeInstanceId || 'default';
            const { useAccountStore } = await import('./useAccountStore');
            let accounts = useAccountStore.getState().accounts;
            const hasAccounts = accounts.length > 0;
            if (!hasAccounts) {
                await useAccountStore.getState().fetchAccounts();
                accounts = useAccountStore.getState().accounts;
            }
            const bestAccount = instanceService.findBestSmartPlayAccount(accounts);

            const hasBest = Boolean(bestAccount);
            if (!hasBest) {
                await instanceService.launchInstance(instId);
                await get().fetchInstances(true);
                set({ isLoading: false });
                const cur = get().instances.find(i => i.config.id === instId);
                return {
                    accountEmail: 'None',
                    instanceName: cur?.config.name || 'Instance',
                };
            }

            await instanceService.switchAccountToInstance(bestAccount!.id, instId);
            await Promise.all([
                get().fetchInstances(true),
                useAccountStore.getState().fetchCurrentAccount(),
            ]);

            set({ isLoading: false });
            const cur = get().instances.find(i => i.config.id === instId);
            return {
                accountEmail: bestAccount!.email,
                instanceName: cur?.config.name || 'Instance',
            };
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to smart play instance' });
            throw err;
        }
    },

    rotateToNextBestProfile: async (sourceInstanceId?: string) => {
        set({ isLoading: true, error: null });
        try {
            await get().fetchInstances(true);
            const list = get().instances;
            const currentId = sourceInstanceId || get().activeInstanceId;
            const best = instanceService.findBestRotationProfile(list, currentId);
            if (!best) {
                throw new Error('No candidate profile available for rotation');
            }

            // Set active and launch candidate profile
            await instanceService.setActiveInstance(best.config.id);
            await instanceService.launchInstance(best.config.id);
            await get().fetchInstances(true);
            set({ activeInstanceId: best.config.id, isLoading: false });
            return best;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to rotate profile' });
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.rotateToNextBestProfile' });
            throw err;
        }
    },
}));
