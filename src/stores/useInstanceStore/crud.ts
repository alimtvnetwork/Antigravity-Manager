import * as instanceService from '../../services/instanceService';
import * as accountService from '../../services/accountService';
import { useErrorStore } from '../error-store';
import { useConfigStore } from '../useConfigStore';
import { showToast } from '../../components/common/ToastContainer';
import type { Account } from '../../types/account';
import type {
    InstanceConfig,
    InstanceStatus,
    AutoProfileSwitcherConfig,
    AutoSwitcherStatus,
} from '../../services/instanceService';
import type { InstanceState } from './types';
import { instanceSelectionEpoch } from './shared';

type SetState = (p: Partial<InstanceState> | ((s: InstanceState) => Partial<InstanceState>)) => void;
type GetState = () => InstanceState;

export function createCrudSlice(set: SetState, get: GetState): Partial<InstanceState> {
    return {
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
    };
}
