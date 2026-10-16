/**
 * Audit recorder — thin Tauri IPC client.
 *
 * Every failure is routed into the error module via `trackWarning`
 * (No Silent Errors house rule) and surfaced to callers as `null`,
 * so callers can fall back without ever swallowing the error.
 */
import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../../stores/error-store';
import {
    AUDIT_COMMANDS,
    type AuditEntry,
    type AuditRecordPayload,
} from './audit-types';

const SOURCE = 'audit/audit-client';

function warn(error: unknown, triggerAction: string, context?: Record<string, unknown>): void {
    useErrorStore.getState().trackWarning(error, {
        source: SOURCE,
        triggerComponent: 'AuditClient',
        triggerAction,
        endpoint: triggerAction,
        context,
    });
}

/** Start a new recording session. Returns true when the backend confirms. */
export async function auditStart(): Promise<boolean> {
    try {
        await invoke(AUDIT_COMMANDS.start);
        return true;
    } catch (error) {
        warn(error, AUDIT_COMMANDS.start);
        return false;
    }
}

/** Stop the active recording session. */
export async function auditStop(): Promise<boolean> {
    try {
        await invoke(AUDIT_COMMANDS.stop);
        return true;
    } catch (error) {
        warn(error, AUDIT_COMMANDS.stop);
        return false;
    }
}

/** Whether the backend currently has an active recording session. */
export async function auditIsRecording(): Promise<boolean> {
    try {
        return await invoke<boolean>(AUDIT_COMMANDS.isRecording);
    } catch (error) {
        warn(error, AUDIT_COMMANDS.isRecording);
        return false;
    }
}

/**
 * Report one captured interaction to the backend.
 * Best-effort by design: a failed report must never break the UI the
 * user is interacting with — the warning lands in the error module.
 */
export async function auditRecord(payload: AuditRecordPayload): Promise<boolean> {
    try {
        await invoke(AUDIT_COMMANDS.record, { input: payload });
        return true;
    } catch (error) {
        warn(error, AUDIT_COMMANDS.record, { trigger: payload.trigger, methodName: payload.methodName });
        return false;
    }
}

/** Fetch recorded entries (newest first, capped by backend default). */
export async function auditList(limit = 500): Promise<AuditEntry[] | null> {
    try {
        const entries = await invoke<AuditEntry[]>(AUDIT_COMMANDS.list, { limit });
        return Array.isArray(entries) ? entries : [];
    } catch (error) {
        warn(error, AUDIT_COMMANDS.list, { limit });
        return null;
    }
}

/** Delete all recorded entries. */
export async function auditClear(): Promise<boolean> {
    try {
        await invoke(AUDIT_COMMANDS.clear);
        return true;
    } catch (error) {
        warn(error, AUDIT_COMMANDS.clear);
        return false;
    }
}

/**
 * Ask the backend to persist its export (headless/CLI parity path).
 * The panel's user-facing JSON/JSONL download is generated client-side
 * from `audit_list` entries; this command is the server-side counterpart.
 */
export async function auditExport(format: 'json' | 'jsonl'): Promise<string | null> {
    try {
        const result = await invoke<string | { path?: string }>(AUDIT_COMMANDS.export, { format });
        if (typeof result === 'string') return result;
        return result?.path ?? null;
    } catch (error) {
        warn(error, AUDIT_COMMANDS.export, { format });
        return null;
    }
}
