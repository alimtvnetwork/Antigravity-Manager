import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../../stores/error-store';
import type { InstanceConfig, InstanceStatus } from './types';

export async function listInstances(): Promise<InstanceStatus[]> {
    return await invoke('list_instances');
}

export async function createInstance(
    name: string,
    boundAccountId?: string,
    fromInstanceId?: string
): Promise<InstanceConfig> {
    return await invoke('create_instance', {
        name,
        boundAccountId: boundAccountId || null,
        fromInstanceId: fromInstanceId || null,
    });
}

export async function copyInstance(
    sourceId: string,
    targetName: string,
    cloneMode?: string,
    copyProjects?: boolean
): Promise<InstanceConfig> {
    return await invoke('copy_instance', {
        sourceId,
        targetName,
        cloneMode: cloneMode || 'full',
        copyProjects: copyProjects ?? true,
    });
}

export async function copyInstanceProjects(fromId: string, toId: string): Promise<number> {
    try {
        return await invoke('copy_instance_projects', { fromId, toId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.copyInstanceProjects',
            endpoint: 'copy_instance_projects',
            triggerAction: 'copy_instance_projects',
            context: { fromId, toId },
        });
        throw e;
    }
}

export async function copyInstanceSettings(fromId: string, toId: string): Promise<void> {
    try {
        return await invoke('copy_instance_settings', { fromId, toId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.copyInstanceSettings',
            endpoint: 'copy_instance_settings',
            triggerAction: 'copy_instance_settings',
            context: { fromId, toId },
        });
        throw e;
    }
}

export async function copyInstanceBoth(fromId: string, toId: string): Promise<{ projectsCount: number }> {
    try {
        await copyInstanceSettings(fromId, toId);
        const projectsCount = await copyInstanceProjects(fromId, toId);
        return { projectsCount };
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.copyInstanceBoth',
            endpoint: 'copy_instance_both',
            triggerAction: 'copy_instance_both',
            context: { fromId, toId },
        });
        throw e;
    }
}

export async function enforceDefaultSettings(targetInstance?: string): Promise<number> {
    try {
        return await invoke('enforce_default_settings', {
            targetInstance: targetInstance || null,
        });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.enforceDefaultSettings',
            endpoint: 'enforce_default_settings',
            triggerAction: 'enforce_default_settings',
            context: { targetInstance },
        });
        throw e;
    }
}

export async function setInstanceTurboMode(targetInstance?: string, enabled: boolean = true): Promise<number> {
    try {
        return await invoke('set_instance_turbo_mode', {
            targetInstance: targetInstance || null,
            enabled,
        });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.setInstanceTurboMode',
            endpoint: 'set_instance_turbo_mode',
            triggerAction: 'set_instance_turbo_mode',
            context: { targetInstance, enabled },
        });
        throw e;
    }
}

export async function setInstancePlanReview(targetInstance?: string, alwaysProceed: boolean = true): Promise<number> {
    try {
        return await invoke('set_instance_plan_review', {
            targetInstance: targetInstance || null,
            alwaysProceed,
        });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.setInstancePlanReview',
            endpoint: 'set_instance_plan_review',
            triggerAction: 'set_instance_plan_review',
            context: { targetInstance, alwaysProceed },
        });
        throw e;
    }
}

export async function exportInstanceSettings(instanceId: string): Promise<string> {
    try {
        return await invoke('export_instance_settings', { instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.exportInstanceSettings',
            endpoint: 'export_instance_settings',
            triggerAction: 'export_instance_settings',
            context: { instanceId },
        });
        throw e;
    }
}

export async function importInstanceSettings(targetInstance?: string, jsonStr: string = ''): Promise<number> {
    try {
        return await invoke('import_instance_settings', {
            targetInstance: targetInstance || null,
            jsonStr,
        });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.importInstanceSettings',
            endpoint: 'import_instance_settings',
            triggerAction: 'import_instance_settings',
            context: { targetInstance },
        });
        throw e;
    }
}

export async function renameInstance(instanceId: string, newName: string): Promise<InstanceConfig> {
    return await invoke('rename_instance', { instanceId, newName });
}

export async function deleteInstance(instanceId: string): Promise<void> {
    return await invoke('delete_instance', { instanceId });
}

export async function wipeInstanceSession(instanceId: string): Promise<void> {
    return await invoke('wipe_instance_session', { instanceId });
}

export const wipeSession = wipeInstanceSession;
