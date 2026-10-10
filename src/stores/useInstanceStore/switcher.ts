import * as instanceService from '../../services/instanceService';
import { useErrorStore } from '../error-store';
import type { AutoProfileSwitcherConfig } from '../../services/instanceService';
import type { InstanceState } from './types';

type SetState = (p: Partial<InstanceState> | ((s: InstanceState) => Partial<InstanceState>)) => void;
type GetState = () => InstanceState;

export function createSwitcherSlice(set: SetState, get: GetState): Pick<InstanceState,
    'fetchSwitcherStatus' | 'updateSwitcherConfig' | 'triggerManualRotation' | 'toggleAutoSwitcher'> {
    return {
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

    toggleAutoSwitcher: async () => {
        try {
            await instanceService.toggleAutoSwitcher();
            await get().fetchSwitcherStatus();
        } catch (err: any) {
            useErrorStore.getState().captureError(err, { source: 'useInstanceStore.toggleAutoSwitcher' });
            throw err;
        }
    },
    };
}
