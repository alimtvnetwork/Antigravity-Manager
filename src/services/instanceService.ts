import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';
import type { Account } from '../types/account';
import type { InstanceSwitchHistoryResponse } from '../types/audit';
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
    seq_num?: number;
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

export async function stopInstance(instanceId: string): Promise<void> {
    return await invoke('stop_instance', { instanceId });
}

export async function fastForwardInstance(instanceId: string): Promise<string> {
    return await invoke('fast_forward_instance', { instanceId });
}

export async function cleanAndRestartWorkspace(): Promise<string> {
    try {
        return await invoke('clean_and_restart_workspace');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.cleanAndRestartWorkspace',
            endpoint: 'clean_and_restart_workspace',
            triggerAction: 'clean_and_restart_workspace',
        });
        throw e;
    }
}

export async function getActiveInstance(): Promise<string> {
    return await invoke('get_active_instance');
}

export async function setActiveInstance(instanceId: string): Promise<void> {
    return await invoke('set_active_instance', { instanceId });
}

export async function setDefaultInstance(instanceId: string): Promise<void> {
    return await invoke('set_default_instance', { instanceId });
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

    const scored = available.map(acc => {
        const res = calculateMultiplicativeScore(acc, [], undefined, targetModel);
        return { account: acc, score: res.score };
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

export interface MultiplicativeCandidateResult {
    account: Account;
    score: number;
    activeFactor: number;
    tierMultiplier: number;
    weeklyQuotaPercent: number;
    fourHourQuotaPercent: number;
    daysUntilRefill: number;
    quotaPercentage: number;
    idleHours: number;
    isVerified?: boolean;
}

export type SmartCandidateResult = MultiplicativeCandidateResult;

export function getSubscriptionTierMultiplier(tierName?: string): number {
    const lower = (tierName || '').toLowerCase();
    const isUltra = lower.includes('ultra');
    if (isUltra) return 5;
    const isPro = lower.includes('pro');
    if (isPro) return 3;
    return 1;
}

export function extractWeeklyQuotaPercent(acc: Account): number {
    const quotaGroups = acc.quota?.quota_groups;
    if (quotaGroups) {
        if (quotaGroups.length > 0) {
            const weeklyValues: number[] = [];
            for (const group of quotaGroups) {
                const buckets = group.buckets || [];
                for (const b of buckets) {
                    const win = (b.window || '').toLowerCase();
                    const id = (b.bucket_id || '').toLowerCase();
                    const isWeekly = win.includes('week') || id.includes('week');
                    if (isWeekly) {
                        if (typeof b.remaining_fraction === 'number') {
                            weeklyValues.push(Math.round(b.remaining_fraction * 100));
                        }
                    }
                }
            }
            const hasWeekly = weeklyValues.length > 0;
            if (hasWeekly) {
                // Bottleneck rule: take the minimum weekly remaining quota across groups
                return Math.min(...weeklyValues);
            }
        }
    }

    const models = acc.quota?.models || [];
    const validModels = models.filter(m => {
        return typeof m.percentage === 'number';
    });

    const hasValid = validModels.length > 0;
    if (!hasValid) return 100;

    let total = 0;
    for (const m of validModels) {
        total += m.percentage;
    }
    return Math.round(total / validModels.length);
}

/**
 * Specifically extract the 4-hour / 5-hour rolling window quota percentage (0-100).
 * Accounts MUST have 100% in this window to be prioritized for fast-forward rotation.
 */
export function extract4hWindowQuotaPercent(acc: Account, _targetModel?: string): number {
    if (!acc.quota) return 0;
    if (acc.quota.is_forbidden) return 0;
    if (acc.disabled || acc.proxy_disabled || acc.validation_blocked) return 0;

    const minValues: number[] = [];

    // 1. Check all short-window buckets from quota_groups
    const quotaGroups = acc.quota.quota_groups;
    if (quotaGroups && quotaGroups.length > 0) {
        for (const group of quotaGroups) {
            const buckets = group.buckets || [];
            for (const b of buckets) {
                const win = (b.window || '').toLowerCase();
                const id = (b.bucket_id || '').toLowerCase();
                const isShort =
                    win.includes('4h') ||
                    win.includes('5h') ||
                    id.includes('4h') ||
                    id.includes('5h') ||
                    (!win.includes('week') && !win.includes('7d') && !id.includes('week') && !id.includes('7d'));
                if (isShort && typeof b.remaining_fraction === 'number') {
                    minValues.push(Math.round(b.remaining_fraction * 100));
                }
            }
        }
    }

    // 2. Check all active non-banned models (ignore 3.0, 3.1)
    const models = acc.quota.models || [];
    if (models.length > 0) {
        for (const m of models) {
            const name = (m.name || '').toLowerCase();
            if (name.includes('3.0') || name.includes('3.1')) continue;
            if (typeof m.percentage === 'number') {
                minValues.push(m.percentage);
            }
        }
    }

    if (minValues.length > 0) {
        return Math.min(...minValues);
    }

    return 0;
}

function calculateAccountRefillDays(acc: Account, nowMs: number): number {
    let maxDays = 0;
    const models = acc.quota?.models || [];
    for (const m of models) {
        if (!m.reset_time) continue;
        const resetDate = parseFlexibleDate(m.reset_time);
        if (!resetDate) continue;
        const diffMs = resetDate.getTime() - nowMs;
        if (diffMs > 0) {
            const days = diffMs / 86400000;
            if (days > maxDays) maxDays = days;
        }
    }
    return Math.round(maxDays * 10) / 10;
}

/**
 * Normalized Multiplicative Candidate Scoring Algorithm (divided by 1000 for minimal compact numbers):
 * - Anyone with < 100% 4h quota gets a score of `0`.
 * - Otherwise: Score = (S_active * M_tier * Q_weekly) / 1000
 *   where S_active = 1 if unused (0 if in use), M_tier = {Ultra: 5, Pro: 3, Free: 1}, Q_weekly = 0..100.
 *   Example scores: Ultra 100% = 0.5, Pro 100% = 0.3, Free 100% = 0.1, < 100% 4h quota = 0.
 */
export function calculateMultiplicativeScore(
    acc: Account,
    activeInUseAccountIds: string[] = [],
    currentAccountId?: string,
    targetModel?: string
): MultiplicativeCandidateResult {
    const isInUse = activeInUseAccountIds.includes(acc.id);
    const isCurrent = Boolean(currentAccountId && acc.id === currentAccountId);
    const activeFactor = isInUse || isCurrent ? 0 : 1;

    const tierMultiplier = getSubscriptionTierMultiplier(acc.quota?.subscription_tier);
    const weeklyQuotaPercent = extractWeeklyQuotaPercent(acc);
    const fourHourQuotaPercent = extract4hWindowQuotaPercent(acc, targetModel);

    const nowMs = Date.now();
    const hasResetTimePassed = (acc.quota?.models || []).some(m => {
        if (!m.reset_time) return false;
        const rDate = parseFlexibleDate(m.reset_time);
        return rDate ? rDate.getTime() <= nowMs : false;
    });

    const isFull4h = fourHourQuotaPercent >= 100 || hasResetTimePassed;
    if (!isFull4h || activeFactor === 0) {
        return {
            account: acc,
            score: 0,
            activeFactor: 0,
            tierMultiplier,
            weeklyQuotaPercent,
            fourHourQuotaPercent,
            daysUntilRefill: 0,
            quotaPercentage: 0,
            idleHours: 999,
        };
    }

    const rawScore = activeFactor * tierMultiplier * weeklyQuotaPercent;
    const score = Number((rawScore / 1000).toFixed(4));

    const daysUntilRefill = calculateAccountRefillDays(acc, nowMs);
    const nowSec = Math.floor(nowMs / 1000);
    const idleSec = acc.last_used ? Math.max(0, nowSec - acc.last_used) : 0;
    const idleHours = acc.last_used ? Math.round((idleSec / 3600) * 10) / 10 : 999;

    return {
        account: acc,
        score,
        activeFactor,
        tierMultiplier,
        weeklyQuotaPercent,
        fourHourQuotaPercent,
        daysUntilRefill,
        quotaPercentage: fourHourQuotaPercent,
        idleHours,
    };
}

export function rankSmartCandidates(
    accounts: Account[],
    activeInUseAccountIds: string[] = [],
    currentAccountId?: string
): MultiplicativeCandidateResult[] {
    const eligible = accounts.filter(acc => {
        // Never allow the current account or active in-use accounts to be considered as candidates to rotate into
        if (currentAccountId && acc.id === currentAccountId) return false;
        if (activeInUseAccountIds.includes(acc.id)) return false;

        const isDisabled = Boolean(acc.disabled) || Boolean(acc.proxy_disabled);
        if (isDisabled) return false;
        const isForbidden = Boolean(acc.quota?.is_forbidden);
        if (isForbidden) return false;
        const isBlocked = Boolean(acc.validation_blocked);
        if (isBlocked) return false;

        const q4h = extract4hWindowQuotaPercent(acc);
        const nowMs = Date.now();
        const hasResetTimePassed = (acc.quota?.models || []).some(m => {
            if (!m.reset_time) return false;
            const rDate = parseFlexibleDate(m.reset_time);
            return rDate ? rDate.getTime() <= nowMs : false;
        });
        if (q4h < 100 && !hasResetTimePassed) return false;
        return true;
    });

    const hasEligible = eligible.length > 0;
    if (!hasEligible) return [];

    const scored = eligible
        .map(acc => {
            return calculateMultiplicativeScore(
                acc,
                activeInUseAccountIds,
                currentAccountId
            );
        })
        .filter(item => item.score > 0 && item.activeFactor > 0);

    scored.sort((a, b) => {
        // 1. Highest normalized score first (M_tier * Q_weekly / 1000)
        if (b.score !== a.score) {
            return b.score - a.score;
        }

        // 2. 4-hour quota comparison
        if (b.fourHourQuotaPercent !== a.fourHourQuotaPercent) {
            return b.fourHourQuotaPercent - a.fourHourQuotaPercent;
        }

        // 3. Weekly quota comparison
        if (b.weeklyQuotaPercent !== a.weeklyQuotaPercent) {
            return b.weeklyQuotaPercent - a.weeklyQuotaPercent;
        }

        // 4. Deterministic tie-breaker
        const emailA = (a.account.email || '').toLowerCase();
        const emailB = (b.account.email || '').toLowerCase();
        return emailA.localeCompare(emailB);
    });

    return scored;
}

export function findSmartRotationAccount(
    accounts: Account[],
    currentAccountId?: string,
    activeInUseAccountIds: string[] = []
): MultiplicativeCandidateResult | null {
    const ranked = rankSmartCandidates(
        accounts,
        activeInUseAccountIds,
        currentAccountId
    );
    const hasRanked = ranked.length > 0;
    if (!hasRanked) return null;
    return ranked[0];
}

export function pickBestCandidateAccount(
    accounts: Account[],
    activeInUseAccountIds: string[] = [],
    currentAccountId?: string
): Account | null {
    const smart = findSmartRotationAccount(accounts, currentAccountId, activeInUseAccountIds);
    if (smart?.account && (smart.fourHourQuotaPercent ?? 0) >= 100) {
        return smart.account;
    }
    return null;
}

export const selectNextBestProfile = findBestRotationProfile;

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
