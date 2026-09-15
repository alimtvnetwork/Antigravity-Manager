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

export interface AutoProfileSwitcherConfig {
    is_enabled: boolean;
    check_interval_seconds: number;
    low_quota_threshold_percent: number;
    target_model: string;
    has_auto_resume: boolean;
    cooldown_seconds: number;
}

export interface AutoSwitcherStatus {
    is_running: boolean;
    active_instance_id: string;
    active_account_email?: string;
    current_quota_percent?: number;
    last_check_timestamp: number;
    last_switch_timestamp?: number;
    last_switch_reason?: string;
}

export async function getAutoSwitcherStatus(): Promise<AutoSwitcherStatus> {
    return await invoke('get_auto_switcher_status');
}

export async function updateAutoSwitcherConfig(config: AutoProfileSwitcherConfig): Promise<void> {
    return await invoke('update_auto_switcher_config', { config });
}

export async function triggerManualProfileRotation(): Promise<string> {
    return await invoke('trigger_manual_profile_rotation');
}
