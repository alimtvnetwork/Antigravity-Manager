import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export interface ImprovementBatch {
    index: number;
    status: string;
    title: string;
    prompt_rel: string;
    item_start: number;
    item_end: number;
}

export interface FlywheelState {
    wave: number;
    kind: string;
    total: number;
    batch_size: number;
    next_batch: number;
    completed_batches: number;
    last_action: string;
    updated_at: number;
    ledger_rel: string;
    dispatched_prompt_id: string | null;
    batches: ImprovementBatch[];
    workspace_root?: string;
}

export type ImprovementKind = 'ui' | 'code' | 'system' | 'ideas';

export const improvementsService = {
    async plan(
        kind: ImprovementKind,
        dryRun: boolean,
        instanceId?: string,
    ): Promise<FlywheelState> {
        try {
            return await invoke<FlywheelState>('improvements_plan', {
                kind,
                dryRun,
                instanceId: instanceId ?? 'default',
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'ImprovementsService.plan',
                triggerAction: 'improvements_plan',
            });
            throw error;
        }
    },

    async next(dryRun: boolean, instanceId?: string): Promise<FlywheelState> {
        try {
            return await invoke<FlywheelState>('improvements_next', {
                dryRun,
                instanceId: instanceId ?? 'default',
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'ImprovementsService.next',
                triggerAction: 'improvements_next',
            });
            throw error;
        }
    },

    async status(instanceId?: string): Promise<FlywheelState> {
        try {
            return await invoke<FlywheelState>('improvements_status', {
                instanceId: instanceId ?? 'default',
            });
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'ImprovementsService.status',
                triggerAction: 'improvements_status',
            });
            throw error;
        }
    },

    async check(instanceId?: string): Promise<FlywheelState> {
        try {
            return await invoke<FlywheelState>('improvements_check', {
                instanceId: instanceId ?? 'default',
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'ImprovementsService.check',
                triggerAction: 'improvements_check',
            });
            throw error;
        }
    },

    async detail(dryRun: boolean, instanceId?: string): Promise<FlywheelState> {
        try {
            return await invoke<FlywheelState>('improvements_detail', {
                dryRun,
                instanceId: instanceId ?? 'default',
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'ImprovementsService.detail',
                triggerAction: 'improvements_detail',
            });
            throw error;
        }
    },
};
