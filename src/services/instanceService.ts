import { invoke } from '@tauri-apps/api/core';

export interface InstanceConfig {
    id: string;
    name: string;
    data_dir: string;
    extensions_dir?: string;
    bound_account_id?: string;
    bound_email?: string;
    created_at: number;
    last_used: number;
    is_default: boolean;
}

export interface InstanceStatus {
    config: InstanceConfig;
    is_running: boolean;
    pid?: number;
    memory_mb?: number;
}

export async function listInstances(): Promise<InstanceStatus[]> {
    return await invoke('list_instances');
}

export async function createInstance(name: string): Promise<InstanceConfig> {
    return await invoke('create_instance', { name });
}

export async function copyInstance(sourceId: string, targetName: string): Promise<InstanceConfig> {
    return await invoke('copy_instance', { sourceId, targetName });
}

export async function deleteInstance(instanceId: string): Promise<void> {
    return await invoke('delete_instance', { instanceId });
}

export async function wipeInstanceSession(instanceId: string): Promise<void> {
    return await invoke('wipe_instance_session', { instanceId });
}

export async function launchInstance(instanceId: string): Promise<void> {
    return await invoke('launch_instance', { instanceId });
}

export async function closeInstance(instanceId: string): Promise<void> {
    return await invoke('close_instance', { instanceId });
}

export async function getActiveInstance(): Promise<string> {
    return await invoke('get_active_instance');
}

export async function setActiveInstance(instanceId: string): Promise<void> {
    return await invoke('set_active_instance', { instanceId });
}

export async function switchAccountToInstance(accountId: string, instanceId?: string): Promise<void> {
    return await invoke('switch_account_to_instance', { accountId, instanceId });
}
