/**
 * Canonical task execution status emitted by the task history database.
 * - 'queued': Task is scheduled and awaiting execution slot.
 * - 'running': Task is actively executing.
 * - 'ok': Task finished successfully.
 * - 'fail': Task terminated with an error or unhandled panic.
 */
export type AuditTaskStatus = 'queued' | 'running' | 'ok' | 'fail';

export interface TaskRecord {
    id: string;
    action_code: number;
    action: string;
    action_label: string;
    status: AuditTaskStatus;
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

export interface SwitchBackupStep {
    prompt_count: number;
    project_names: string[];
    project_paths: string[];
    backup_batch_id?: string;
    success: boolean;
}

export interface SwitchResetStep {
    terminated_pids: number[];
    auth_swapped: boolean;
    credentials_injected: boolean;
    success: boolean;
}

export interface SwitchRestoreStep {
    method: string;
    restored_count: number;
    dispatched_count: number;
    resent_count: number;
    prompt_channel_waited: boolean;
    success: boolean;
}

export interface SwitchVerificationStep {
    verified: boolean;
    active_worker_pids: number[];
    resume_file_verified: boolean;
    verified_projects: string[];
    message: string;
}

export interface SwitchAuditSteps {
    backup: SwitchBackupStep;
    reset: SwitchResetStep;
    restore: SwitchRestoreStep;
    verification: SwitchVerificationStep;
}

export interface InstanceSwitchRecord {
    id: string;
    from_email: string;
    to_email: string;
    created_at: number;
    finished_at?: number | null;
    status: AuditTaskStatus | string;
    switch_reason: string;
    how: string;
    steps?: SwitchAuditSteps;
    raw_payload?: string;
}

export interface InstanceSwitchHistoryResponse {
    instance_id: string;
    total_switches: number;
    recent_switches: InstanceSwitchRecord[];
}

/** Check if a task status represents a completed terminal state. */
export function isTaskTerminal(status: AuditTaskStatus | string): boolean {
    return status === 'ok' || status === 'fail';
}

/** Check if a task status represents an in-flight active state. */
export function isTaskInFlight(status: AuditTaskStatus | string): boolean {
    return status === 'queued' || status === 'running';
}

