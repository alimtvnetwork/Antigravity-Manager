import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../../stores/error-store';
import type { InstanceStatus } from './types';

export async function syncInstancePidAndQuota(instanceId: string): Promise<InstanceStatus> {
    try {
        return await invoke('sync_instance_pid_and_quota', { instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.syncInstancePidAndQuota',
            endpoint: 'sync_instance_pid_and_quota',
            triggerAction: 'sync_instance_pid_and_quota',
            context: { instanceId },
        });
        throw e;
    }
}

export async function syncAllInstancesAndQuotas(): Promise<InstanceStatus[]> {
    try {
        return await invoke('sync_all_instances_and_quotas');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.syncAllInstancesAndQuotas',
            endpoint: 'sync_all_instances_and_quotas',
            triggerAction: 'sync_all_instances_and_quotas',
        });
        throw e;
    }
}

export type SyncInterval = '15s' | '30s' | '1m' | '2m' | 'off';

export function getPromptTreeSyncInterval(): SyncInterval {
    try {
        const val = localStorage.getItem('agm_prompt_tree_sync_interval') as SyncInterval;
        if (val === '15s' || val === '30s' || val === '1m' || val === '2m' || val === 'off') {
            return val;
        }
    } catch (e) {
        // Best-effort UI preference read; localStorage may throw in restricted contexts. Tracked, safe default applied.
        useErrorStore.getState().trackWarning(e, {
          source: 'instanceService.getPromptTreeSyncInterval',
          triggerAction: 'read_sync_interval_preference',
        });
    }
    return '30s';
}

export function setPromptTreeSyncInterval(interval: SyncInterval): void {
    try {
        localStorage.setItem('agm_prompt_tree_sync_interval', interval);
    } catch (e) {
        // Best-effort UI preference write; localStorage may throw in restricted contexts. Tracked, preference not persisted.
        useErrorStore.getState().trackWarning(e, {
          source: 'instanceService.setPromptTreeSyncInterval',
          triggerAction: 'write_sync_interval_preference',
        });
    }
}

export function getArchivedProjectsForInstance(instanceId: string): string[] {
    try {
        const key = `agm_archived_projects_${instanceId || 'default'}`;
        const raw = localStorage.getItem(key);
        if (!raw) return [];
        const parsed = JSON.parse(raw);
        return Array.isArray(parsed) ? parsed : [];
    } catch (e) {
        // Best-effort UI state read; corrupt or unavailable storage falls back to empty. Tracked.
        useErrorStore.getState().trackWarning(e, {
          source: 'instanceService.getArchivedProjectsForInstance',
          triggerAction: 'read_archived_projects',
        });
        return [];
    }
}

export function setArchivedProjectsForInstance(instanceId: string, projectIds: string[]): void {
    try {
        const key = `agm_archived_projects_${instanceId || 'default'}`;
        localStorage.setItem(key, JSON.stringify(projectIds));
    } catch (e) {
        // Best-effort UI state write; localStorage may throw in restricted contexts. Tracked, archive state not persisted.
        useErrorStore.getState().trackWarning(e, {
          source: 'instanceService.setArchivedProjectsForInstance',
          triggerAction: 'write_archived_projects',
        });
    }
}

export function getInstanceCardDensity(): 'normal' | 'compact' {
    try {
        const val = localStorage.getItem('agm_instance_card_density');
        if (val === 'compact' || val === 'normal') {
            return val;
        }
    } catch {
        // Best-effort UI preference read; localStorage may throw in restricted contexts. Tracked, safe default applied.
        useErrorStore.getState().trackWarning('localStorage.getItem failed for agm_instance_card_density', {
          source: 'instanceService.getInstanceCardDensity',
          triggerAction: 'read_density_preference',
        });
    }
    return 'normal';
}

export function setInstanceCardDensity(density: 'normal' | 'compact'): void {
    try {
        localStorage.setItem('agm_instance_card_density', density);
    } catch (e) {
        // Best-effort UI preference write; localStorage may throw in restricted contexts. Tracked, preference not persisted.
        useErrorStore.getState().trackWarning(e, {
          source: 'instanceService.setInstanceCardDensity',
          triggerAction: 'write_density_preference',
        });
    }
}
