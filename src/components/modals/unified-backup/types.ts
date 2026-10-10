export type BackupTab = 'export' | 'import';

export type ExportScope = 'accounts' | 'full';

export interface UnifiedBackupModalProps {
    isOpen: boolean;
    onClose: () => void;
    initialTab?: BackupTab;
}

export interface ImportPreviewData {
    backupType: 'accounts' | 'full';
    accountsCount: number;
    hasConfig: boolean;
    instancesCount: number;
    auditHistoryCount?: number;
    rawPayload: unknown;
}

export interface FullBackupPayload {
    accounts: unknown[];
    config: Record<string, unknown>;
    instances: unknown[];
    email_settings: unknown;
    email_accounts: unknown;
    audit_history: unknown;
}

export type ExportPayload = unknown[] | FullBackupPayload;

// --- Named constants (replacing magic literals) ---
export const MIN_EXPORT_PASSWORD_LENGTH = 6;
export const AUDIT_HISTORY_LIMIT = 500;
export const ENCRYPTED_BACKUP_EXT = 'agmbackup';
export const PLAIN_BACKUP_EXT = 'json';
export const RESTORE_THROTTLE_MS = 60;
export const REFRESH_TOKEN_PREFIX = '1//';
