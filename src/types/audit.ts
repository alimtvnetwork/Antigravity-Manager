export interface TaskRecord {
    id: string;
    action_code: number;
    action: string;
    action_label: string;
    status: string;
    subject: string;
    detail: string;
    instance_id: string;
    split_path: string;
    from_email?: string;
    to_email?: string;
    created_at: number;
    finished_at?: number | null;
}

export type TaskHistoryItem = TaskRecord;

export interface TaskDetail extends TaskRecord {
    payload_json: string;
}

export interface SplitInfo {
    id: string;
    file_path: string;
    row_count: number;
    is_current: boolean;
}

export interface TaskHistoryPage {
    total: number;
    offset: number;
    limit: number;
    items: TaskRecord[];
    splits: SplitInfo[];
}

export interface SwitchPayload {
    from_email?: string;
    to_email?: string;
    reason?: string;
    how?: string;
    prompt_id?: string;
    prompt_text?: string;
    conversation_id?: string;
    prompt_reinjected?: boolean;
    moved_at?: number;
    switch_ok?: boolean;
    instance_id?: string;
    ide_type?: string;
    idc_machine_alias?: string;
    ide_path?: string;
    switch_reason?: string;
}

export interface SchedulerPayload {
    scheduler_run_id?: string;
    project_name?: string;
    repo_path?: string;
    instance_id?: string;
    prompt_id?: string;
    prompt_preview?: string;
    prompt_text?: string;
    conversation_id?: string;
    action_taken?: string;
    reason?: string;
    idle_check_passed?: boolean;
    timestamp?: number;
}

export type TaskPayload = SwitchPayload & SchedulerPayload & Record<string, unknown>;

export type AuditFilterType = 'all' | 'switch' | 'scheduler';
