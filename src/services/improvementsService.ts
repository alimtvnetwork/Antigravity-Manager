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
    task_hints?: string[];
}

export type ImprovementKind = 'ui' | 'code' | 'system' | 'ideas';

export function emptyFlywheelState(batchSize = 10): FlywheelState {
    const total = 100;
    const count = Math.max(1, Math.floor(total / batchSize));
    const batches: ImprovementBatch[] = Array.from({ length: count }, (_, i) => {
        const index = i + 1;
        const item_start = i * batchSize + 1;
        const item_end = index * batchSize;
        return {
            index,
            status: 'pending',
            title: `Slot ${String(index).padStart(2, '0')} — items ${String(item_start).padStart(2, '0')}–${String(item_end).padStart(2, '0')}`,
            prompt_rel: '01-prompts/30-improvements/07-implement-next-10-improvements.md',
            item_start,
            item_end,
        };
    });
    return {
        wave: 1,
        kind: 'system',
        total,
        batch_size: batchSize,
        next_batch: 1,
        completed_batches: 0,
        last_action: 'unplanned',
        updated_at: 0,
        ledger_rel: '.ai-memory/improvements/flywheel.json',
        dispatched_prompt_id: null,
        batches,
        workspace_root: '',
        task_hints: [],
    };
}

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
            // Status is a page-load probe. Never open the E5001 modal when
            // the packaged GUI cwd is the install dir.
            return emptyFlywheelState();
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
