import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';
import type { Account } from '../types/account';
import { parseFlexibleDate } from '../utils/format';

export interface InstanceConfig {
    id: string;
    name: string;
    data_dir: string;
    executable_path?: string;
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

export async function copyInstance(
    sourceId: string,
    targetName: string,
    cloneMode?: string
): Promise<InstanceConfig> {
    return await invoke('copy_instance', {
        sourceId,
        targetName,
        cloneMode: cloneMode || 'full',
    });
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

export async function launchInstance(instanceId: string): Promise<void> {
    try {
        return await invoke('launch_instance', { instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.launchInstance',
            endpoint: 'launch_instance',
            triggerAction: 'launch_instance',
        });
        throw e;
    }
}

export async function cloneInstanceExecutable(instanceId: string): Promise<string> {
    try {
        return await invoke('clone_instance_executable', { instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.cloneInstanceExecutable',
            endpoint: 'clone_instance_executable',
            triggerAction: 'clone_instance_executable',
        });
        throw e;
    }
}

export async function setInstanceExecutable(instanceId: string, executablePath?: string): Promise<void> {
    try {
        return await invoke('set_instance_executable', { instanceId, executablePath });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.setInstanceExecutable',
            endpoint: 'set_instance_executable',
            triggerAction: 'set_instance_executable',
        });
        throw e;
    }
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
    try {
        return await invoke('switch_account_to_instance', { accountId, instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.switchAccountToInstance',
            endpoint: 'switch_account_to_instance',
            triggerAction: 'switch_account_to_instance',
        });
        throw e;
    }
}

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

export async function updateAutoSwitcherConfig(config: AutoProfileSwitcherConfig): Promise<void> {
    return await invoke('update_auto_switcher_config', { config });
}

export async function triggerManualProfileRotation(): Promise<string> {
    return await invoke('trigger_manual_profile_rotation');
}

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

export function findBestSmartPlayAccount(
    accounts: Account[],
    targetModel: string = 'gemini-pro'
): Account | null {
    const available = accounts.filter(acc => {
        const isForbidden = Boolean(acc.quota?.is_forbidden);
        if (isForbidden) return false;
        const isDisabled = Boolean(acc.disabled);
        if (isDisabled) return false;
        return true;
    });

    const hasAvailable = available.length > 0;
    if (!hasAvailable) return null;

    const nowSec = Math.floor(Date.now() / 1000);

    const scored = available.map(acc => {
        let score = 0;

        // 1. Idle time factor (Longest time not used, or never used)
        const hasNeverUsed = !acc.last_used || acc.last_used === 0;
        if (hasNeverUsed) {
            score += 100000;
        } else {
            const idleHours = Math.max(0, (nowSec - acc.last_used) / 3600);
            score += Math.min(50000, idleHours * 1000);
        }

        // 2. 4-Hour quota remaining (lowest credit used = highest percentage remaining)
        const models = acc.quota?.models || [];
        let lowestRemaining = 100;
        for (const m of models) {
            if (typeof m.percentage === 'number') {
                if (m.percentage < lowestRemaining) {
                    lowestRemaining = m.percentage;
                }
            }
        }
        score += lowestRemaining * 200;

        // 3. Target model bonus
        const target = models.find(m => m.name.toLowerCase().includes(targetModel.toLowerCase()));
        if (target && typeof target.percentage === 'number') {
            score += target.percentage * 100;
        }

        // 4. Tier bonus
        const tier = (acc.quota?.subscription_tier || '').toLowerCase();
        if (tier.includes('ultra')) {
            score += 3000;
        } else if (tier.includes('pro')) {
            score += 2000;
        }

        return { account: acc, score };
    });

    scored.sort((a, b) => b.score - a.score);
    return scored[0]?.account || null;
}

/**
 * Smart Profile Rotation Algorithm
 * Prioritizes profiles used longer ago (e.g. 15 hours ago > 30 minutes ago).
 * Evaluates candidate profiles in batches of 3, picking the best idle profile.
 */
export function findBestRotationProfile(
    instances: InstanceStatus[],
    currentInstanceId?: string
): InstanceStatus | null {
    if (instances.length === 0) return null;

    // Filter candidates: prefer those not currently running
    const nonRunning = instances.filter(i => {
        if (i.is_running) return false;
        if (instances.length > 1 && i.config.id === currentInstanceId) return false;
        return true;
    });

    const pool = nonRunning.length > 0 ? nonRunning : instances.filter(i => i.config.id !== currentInstanceId);
    if (pool.length === 0) return instances[0] || null;

    // Sort by longest delay since last used (ascending timestamp, where 0 or lowest = oldest / longest ago)
    const sorted = [...pool].sort((a, b) => {
        const lastA = a.config.last_used || 0;
        const lastB = b.config.last_used || 0;
        return lastA - lastB;
    });

    // Batched evaluation: step through chunks of 3 and pick the first available
    for (let i = 0; i < sorted.length; i += 3) {
        const batch = sorted.slice(i, i + 3);
        const match = batch.find(item => !item.is_running);
        if (match) {
            return match;
        }
    }

    return sorted[0] || null;
}

export function formatTimeAgo(timestampSec?: number): string {
    if (!timestampSec || timestampSec <= 0) return 'Never used';
    const nowSec = Math.floor(Date.now() / 1000);
    const diffSec = Math.max(0, nowSec - timestampSec);
    if (diffSec < 60) return 'Just now';
    const mins = Math.floor(diffSec / 60);
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
}

export interface SmartCandidateResult {
    account: Account;
    score: number;
    idleHours: number;
    daysUntilRefill: number;
    quotaPercentage: number;
}

/**
 * Smart Candidate Account Scoring & Selection
 * 1. Filter healthy accounts (not disabled, not forbidden, not validation blocked).
 * 2. Exclude current profile's account if alternative accounts exist.
 * 3. 4-Hour Inactivity Factor: Accounts not used in past 4h (or never used) receive highest priority.
 * 4. Refill Runway: Longest runway until quota reset (e.g. 6 days) receives top priority.
 * 5. Quota Headroom: Highest percentage remaining across models (100% full headroom).
 */
export function findSmartRotationAccount(
    accounts: Account[],
    currentAccountId?: string
): SmartCandidateResult | null {
    const eligible = accounts.filter(acc => {
        const isDisabled = Boolean(acc.disabled);
        if (isDisabled) return false;
        const isForbidden = Boolean(acc.quota?.is_forbidden);
        if (isForbidden) return false;
        const isBlocked = Boolean(acc.validation_blocked);
        if (isBlocked) return false;
        return true;
    });

    const hasEligible = eligible.length > 0;
    if (!hasEligible) return null;

    let pool = eligible;
    const hasMultiple = eligible.length > 1;
    if (hasMultiple) {
        if (currentAccountId) {
            const others = eligible.filter(a => a.id !== currentAccountId);
            const hasOthers = others.length > 0;
            if (hasOthers) {
                pool = others;
            }
        }
    }

    const nowMs = Date.now();
    const nowSec = Math.floor(nowMs / 1000);

    const scored: SmartCandidateResult[] = pool.map(acc => {
        let score = 0;

        // Factor A: 4-Hour Inactivity Recency (Top priority)
        let idleHours = 999;
        const hasNoLastUsed = !acc.last_used;
        const isLastUsedZero = acc.last_used === 0;
        if (hasNoLastUsed) {
            idleHours = 999;
            score += 100000;
        } else if (isLastUsedZero) {
            idleHours = 999;
            score += 100000;
        } else {
            const idleSec = Math.max(0, nowSec - acc.last_used);
            idleHours = idleSec / 3600;
            const isFourHoursIdle = idleSec >= 4 * 3600;
            if (isFourHoursIdle) {
                score += 100000;
                score += Math.min(20000, idleHours * 500);
            } else {
                score += Math.max(0, idleHours * 1000);
            }
        }

        // Factor B: Refill Runway (Longest time until reset / weekly refill, e.g. 6 days)
        let maxRefillDays = 0;
        const models = acc.quota?.models || [];
        for (const m of models) {
            if (m.reset_time) {
                const resetDate = parseFlexibleDate(m.reset_time);
                if (resetDate) {
                    const diffMs = resetDate.getTime() - nowMs;
                    if (diffMs > 0) {
                        const days = diffMs / (1000 * 60 * 60 * 24);
                        if (days > maxRefillDays) {
                            maxRefillDays = days;
                        }
                    }
                }
            }
        }

        const groups = acc.quota?.quota_groups || [];
        for (const g of groups) {
            for (const b of g.buckets || []) {
                if (b.reset_time) {
                    const resetDate = parseFlexibleDate(b.reset_time);
                    if (resetDate) {
                        const diffMs = resetDate.getTime() - nowMs;
                        if (diffMs > 0) {
                            const days = diffMs / (1000 * 60 * 60 * 24);
                            if (days > maxRefillDays) {
                                maxRefillDays = days;
                            }
                        }
                    }
                }
            }
        }

        const hasSixDaysRunway = maxRefillDays >= 6;
        if (hasSixDaysRunway) {
            score += 50000;
        }
        score += Math.min(50000, Math.floor(maxRefillDays * 8000));

        // Factor C: Quota Headroom (Remaining percentage across models)
        let avgPercentage = 100;
        const hasModels = models.length > 0;
        if (hasModels) {
            let totalPct = 0;
            let count = 0;
            for (const m of models) {
                if (typeof m.percentage === 'number') {
                    totalPct += m.percentage;
                    count += 1;
                }
            }
            const hasCount = count > 0;
            if (hasCount) {
                avgPercentage = Math.round(totalPct / count);
            }
        }
        score += avgPercentage * 200;

        // Factor D: Subscription Tier bonus
        const tier = (acc.quota?.subscription_tier || '').toLowerCase();
        if (tier.includes('ultra')) {
            score += 3000;
        } else if (tier.includes('pro')) {
            score += 2000;
        }

        return {
            account: acc,
            score,
            idleHours: Math.round(idleHours * 10) / 10,
            daysUntilRefill: Math.round(maxRefillDays * 10) / 10,
            quotaPercentage: avgPercentage,
        };
    });

    scored.sort((a, b) => b.score - a.score);
    return scored[0] || null;
}
