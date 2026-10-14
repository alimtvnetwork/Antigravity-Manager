/**
 * Audit recorder — public surface.
 *
 * Mounting (lead wires these two call sites):
 * 1. `useAuditRecorder()` — call once in `src/components/layout/Layout.tsx`
 *    next to `useFastForwardShortcut()` (line ~18).
 * 2. `<RecordingIndicator />` + `<AuditPanel />` — render inside Layout
 *    (indicator: navbar/toolbar area; panel: near ToastContainer).
 */
export { useAuditRecorder } from '../../hooks/useAuditRecorder';
export { RecordingIndicator } from './RecordingIndicator';
export { AuditPanel } from './AuditPanel';
export { useAuditStore } from './audit-store';
export {
    auditStart,
    auditStop,
    auditIsRecording,
    auditRecord,
    auditList,
    auditClear,
    auditExport,
} from './audit-client';
export {
    AUDIT_COMMANDS,
    AUDIT_ATTR_METHOD,
    AUDIT_ATTR_COMPONENT,
} from './audit-types';
export type {
    AuditActor,
    AuditTrigger,
    AuditResult,
    AuditEntry,
    AuditRecordPayload,
} from './audit-types';
