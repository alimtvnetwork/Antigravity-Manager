import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../../stores/error-store';

export interface AutoProfileSwitcherConfig {
    is_enabled: boolean;
    check_interval_seconds: number;
    low_quota_threshold_percent: number;
    target_model: string;
    has_auto_resume: boolean;
    cooldown_seconds: number;
    caution_interval_seconds?: number;
    critical_interval_seconds?: number;
    critical_threshold_percent?: number;
    auto_fast_forward_on_critical?: boolean;
    auto_resume_recent_prompts?: boolean;
    auto_focus_window?: boolean;
    auto_reopen_on_switch?: boolean; // [NEW] Auto reopen active workspace window on profile rotation
    watchdog_interval_seconds?: number;
    prompt_recency_threshold_seconds?: number;
    fast_forward_shortcut?: string;
    stale_binding_timeout_hours?: number;
    account_lockout_window_minutes?: number;
}

export function isInstanceBindingStale(
    inst: InstanceStatus,
    staleTimeoutHours: number = 6,
    nowSec: number = Math.floor(Date.now() / 1000)
): boolean {
    if (inst.is_running) return false;
    const lastUsed = inst.config.last_used || 0;
    if (lastUsed <= 0) return false;
    const timeoutSecs = Math.max(1, staleTimeoutHours) * 3600;
    return nowSec - lastUsed > timeoutSecs;
}

export interface AutoResumePromptInfo {
    project_id: string;
    repo_path: string;
    prompt_preview: string;
    has_image: boolean;
}

export interface AutoResumeResult {
    instance_id: string;
    account_email: string;
    resumed_project_count: number;
    skipped_project_count: number;
    resumed_prompts: AutoResumePromptInfo[];
}

export async function resumeRecentProjectPrompts(
    instanceId?: string,
    maxAgeSeconds?: number
): Promise<AutoResumeResult> {
    return await invoke('resume_recent_project_prompts', {
        instanceId: instanceId || null,
        maxAgeSeconds: maxAgeSeconds || null,
    });
}

export interface InstanceQuotaSummary {
    instance_id: string;
    instance_name: string;
    bound_email?: string;
    quota_percent?: number;
    reset_time_iso?: string;
    seconds_until_reset?: number;
    is_running: boolean;
    is_depleted_before_finish: boolean;
}

export interface AutoSwitcherStatus {
    is_running: boolean;
    active_instance_id: string;
    active_account_email?: string;
    current_quota_percent?: number;
    last_check_timestamp: number;
    last_switch_timestamp?: number;
    last_switch_reason?: string;
    monitored_instance_count?: number;
    monitored_instances?: InstanceQuotaSummary[];
}

export async function getAutoSwitcherStatus(): Promise<AutoSwitcherStatus> {
    return await invoke('get_auto_switcher_status');
}

export async function getAutoSwitcherConfig(): Promise<AutoProfileSwitcherConfig> {
    return await invoke('get_auto_switcher_config');
}

export async function toggleAutoSwitcher(): Promise<boolean> {
    return await invoke('toggle_auto_switcher');
}

export async function updateAutoSwitcherConfig(config: AutoProfileSwitcherConfig): Promise<void> {
    return await invoke('update_auto_switcher_config', { config });
}

export async function triggerManualProfileRotation(instanceId?: string): Promise<string> {
    return await invoke('trigger_manual_profile_rotation', { instanceId: instanceId || null });
}

export interface AutoSwitcherDaemonStatus {
    is_daemon_running: boolean;
    last_evaluated_at: number;
    next_check_timestamp: number;
    next_check_in_seconds: number;
    check_interval_seconds: number;
    current_stage: string;
    active_account_email?: string;
    current_quota_percent: number;
    monitored_instance_count: number;
}

export async function getAutoSwitcherDaemonStatus(): Promise<AutoSwitcherDaemonStatus> {
    try {
        return await invoke('get_auto_switcher_daemon_status');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.getAutoSwitcherDaemonStatus',
            endpoint: 'get_auto_switcher_daemon_status',
            triggerAction: 'get_auto_switcher_daemon_status',
        });
        throw e;
    }
}
