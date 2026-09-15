import { create } from 'zustand';
import * as instanceService from '../services/instanceService';
import type { InstanceConfig, InstanceStatus } from '../services/instanceService';

interface InstanceState {
    instances: InstanceStatus[];
    activeInstanceId: string;
    isLoading: boolean;
    error: string | null;

    fetchInstances: () => Promise<void>;
    createInstance: (name: string) => Promise<InstanceConfig>;
    copyInstance: (sourceId: string, targetName: string) => Promise<InstanceConfig>;
    deleteInstance: (instanceId: string) => Promise<void>;
    wipeSession: (instanceId: string) => Promise<void>;
    launchInstance: (instanceId: string) => Promise<void>;
    closeInstance: (instanceId: string) => Promise<void>;
    setActiveInstance: (instanceId: string) => Promise<void>;
    switchAccountToInstance: (accountId: string, instanceId?: string) => Promise<void>;
}

export const useInstanceStore = create<InstanceState>((set, get) => ({
    instances: [],
    activeInstanceId: 'default',
    isLoading: false,
    error: null,

    fetchInstances: async () => {
        try {
            const [instances, activeId] = await Promise.all([
                instanceService.listInstances(),
                instanceService.getActiveInstance(),
            ]);
            set({ instances, activeInstanceId: activeId, error: null });
        } catch (err: any) {
            set({ error: err?.toString() || 'Failed to fetch instances' });
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

    copyInstance: async (sourceId: string, targetName: string) => {
        set({ isLoading: true, error: null });
        try {
            const config = await instanceService.copyInstance(sourceId, targetName);
            await get().fetchInstances();
            set({ isLoading: false });
            return config;
        } catch (err: any) {
            set({ isLoading: false, error: err?.toString() || 'Failed to copy instance' });
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
            throw err;
        }
    },
}));
