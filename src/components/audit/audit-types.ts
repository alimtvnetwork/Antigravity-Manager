/**
 * Audit recorder — shared types and Tauri command names.
 *
 * Backend contract (Task-01, `src-tauri/src/modules/audit/`): the backend
 * stores one entry per interaction with the fields below.
 */

/** Tauri command names exposed by the audit backend module. */
export const AUDIT_COMMANDS = {
    start: 'start_ui_audit_recording',
    stop: 'stop_ui_audit_recording',
    clear: 'clear_ui_audit_recording',
    export: 'export_ui_audit_recording',
    isRecording: 'is_ui_audit_recording',
    list: 'list_ui_audit_interactions',
    record: 'record_ui_interaction',
} as const;

/** Who performed the interaction. */
export type AuditActor = 'user' | 'cli' | 'telegram' | 'system';

/** What kind of interaction triggered the entry. */
export type AuditTrigger =
    | 'click'
    | 'keyboard'
    | 'programmatic'
    | 'navigation'
    | 'input'
    | 'scroll';

/** Outcome recorded for the interaction. */
export type AuditResult = 'ok' | 'error';

/** One frame of a captured stack trace: function name + source location. */
export interface AuditStackFrame {
    function: string;
    location: string;
}

/**
 * One recorded UI interaction.
 * `uiTarget` follows the convention `route | component | element`.
 */
export interface AuditEntry {
    id?: number | string;
    timestamp: number | string;
    actor: AuditActor;
    methodName: string;
    method_name?: string;
    uiTarget: string;
    ui_target?: string;
    trigger: AuditTrigger;
    /** JSON string or already-parsed object, depending on backend serialization. */
    params: string | Record<string, unknown>;
    result: AuditResult;
    /** Human-readable message when `result === 'error'`. */
    message?: string;
    /**
     * Dedicated stack-trace column: a JSON array of `AuditStackFrame`, or the
     * raw JSON string when not yet decoded. Absent/null for entries recorded
     * before the trace column existed.
     */
    stackTrace?: string | AuditStackFrame[] | null;
}

/** Payload sent to the `audit_record` command for one interaction. */
export interface AuditRecordPayload {
    actor: AuditActor;
    methodName: string;
    method_name?: string;
    uiTarget: string;
    ui_target?: string;
    /**
     * Backend-required top-level route (the `RecordInteractionInput.route`
     * field). Injected by `report()` in useAuditRecorder.ts when absent —
     * without it the backend rejects the payload (missing-field error).
     */
    route?: string;
    component?: string;
    elementId?: string;
    trigger: AuditTrigger;
    params: Record<string, unknown>;
    /**
     * Caller-captured stack frames. The backend promotes these into the
     * dedicated trace column; when absent it captures its own backtrace.
     */
    stackTrace?: AuditStackFrame[];
    result: AuditResult;
    message?: string;
}

/**
 * Opt-in DOM attributes components MAY set to give the recorder precise
 * names instead of inferred ones (no changes required elsewhere):
 * - `data-audit-method`: the handler/method name invoked (e.g. "handleSave").
 * - `data-audit-component`: the logical component name (e.g. "AccountTable").
 */
export const AUDIT_ATTR_METHOD = 'data-audit-method';
export const AUDIT_ATTR_COMPONENT = 'data-audit-component';
