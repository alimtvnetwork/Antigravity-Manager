/**
 * Backward-compatible re-export for InstanceAuditTrailModal.
 *
 * The implementation has been split into focused modules under
 * `./InstanceAuditTrail/` (each ≤500 lines). This file preserves the original
 * public API so existing imports keep working.
 */
export { default } from './InstanceAuditTrail/InstanceAuditTrailModal';
export type { InstanceAuditTrailModalProps } from './InstanceAuditTrail/types';
