import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../../stores/error-store';
import type { InstanceConfig } from './types';
import type { InstanceSwitchHistoryResponse } from '../../types/audit';

export interface RunningProject {
    id: string;
    instance_id: string;
    repo_name: string;
    repo_path: string;
    workspace_storage_path?: string;
    is_running: boolean;
    last_detected_at: number;
}

export interface ActivePrompt {
    id: string;
    project_id: string;
    instance_id: string;
    repo_path: string;
    prompt_content: string;
    model?: string;
    session_id?: string;
    status: string;
    created_at: number;
    updated_at: number;
}

export async function listRunningProjects(): Promise<RunningProject[]> {
    return await invoke('list_running_projects');
}

export async function listBackedUpPrompts(): Promise<ActivePrompt[]> {
    return await invoke('list_backed_up_prompts');
}

export async function exportInstancesJson(): Promise<string> {
    return await invoke('export_instances_json');
}

export async function importInstancesJson(jsonContent: string): Promise<InstanceConfig[]> {
    return await invoke('import_instances_json', { jsonContent });
}

export async function getInstanceSwitchHistory(
    instanceId: string,
    limit: number = 5
): Promise<InstanceSwitchHistoryResponse> {
    try {
        return await invoke('get_instance_switch_history', { instanceId, limit });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.getInstanceSwitchHistory',
            endpoint: 'get_instance_switch_history',
            triggerAction: 'get_instance_switch_history',
            context: { instanceId, limit },
        });
        throw e;
    }
}

export async function getInstanceAuditTrail(
    instanceId: string,
    limit: number = 50
): Promise<any[]> {
    try {
        return await invoke('get_instance_audit_trail', { instanceId, limit });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.getInstanceAuditTrail',
            endpoint: 'get_instance_audit_trail',
            triggerAction: 'get_instance_audit_trail',
            context: { instanceId, limit },
        });
        throw e;
    }
}
