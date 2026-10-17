/**
 * instanceService — facade
 * 实际实现拆分至 ./instanceService/ 子模块，此处统一 re-export 以保持导入路径不变。
 */
export * from './instanceService/types';
export * from './instanceService/crud';
export * from './instanceService/lifecycle';
export * from './instanceService/autoSwitcher';
export * from './instanceService/projects';
export * from './instanceService/scoring';
export * from './instanceService/sync';

import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export interface PendingPromptPreview {
    id: string;
    project_id: string;
    repo_name: string;
    repo_path: string;
    prompt_summary: string;
    full_prompt: string;
    created_at: number;
    has_image: boolean;
}

export interface ScheduledResumeEntry {
    instance_id: string;
    scheduled_at_ms: number;
    fire_at_ms: number;
    remaining_seconds: number;
    total_delay_seconds: number;
    prompt_count: number;
    prompts: PendingPromptPreview[];
}

export async function getPendingPostSwitchResumes(): Promise<ScheduledResumeEntry[]> {
    try {
        return await invoke<ScheduledResumeEntry[]>('get_pending_post_switch_resumes');
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'instanceService.getPendingPostSwitchResumes',
            triggerAction: 'get_pending_post_switch_resumes',
        });
        return [];
    }
}

export async function getInstancePendingResume(instanceId: string): Promise<ScheduledResumeEntry | null> {
    try {
        return await invoke<ScheduledResumeEntry | null>('get_instance_pending_resume', { instanceId });
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'instanceService.getInstancePendingResume',
            triggerAction: 'get_instance_pending_resume',
        });
        return null;
    }
}

export async function cancelPostSwitchResume(instanceId: string): Promise<boolean> {
    try {
        return await invoke<boolean>('cancel_post_switch_resume', { instanceId });
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'instanceService.cancelPostSwitchResume',
            triggerAction: 'cancel_post_switch_resume',
        });
        return false;
    }
}

export async function forceDispatchPendingResume(instanceId: string): Promise<void> {
    try {
        await invoke('force_dispatch_pending_resume', { instanceId });
    } catch (err) {
        useErrorStore.getState().trackWarning(err, {
            source: 'instanceService.forceDispatchPendingResume',
            triggerAction: 'force_dispatch_pending_resume',
        });
    }
}

