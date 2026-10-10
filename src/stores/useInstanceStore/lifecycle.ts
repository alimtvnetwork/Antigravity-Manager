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

export function createLifecycleSlice(set: SetState, get: GetState): Partial<InstanceState> {
    return {
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
    };
}
