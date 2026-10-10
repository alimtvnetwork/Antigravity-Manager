import { useState, useRef } from 'react';
import type { ChangeEvent, RefObject } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../../utils/request';
import { isTauri } from '../../../utils/env';
import { showToast } from '../../common/ToastContainer';
import { addAccount } from '../../../services/accountService';
import { useAccountStore } from '../../../stores/useAccountStore';
import { useConfigStore } from '../../../stores/useConfigStore';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { useErrorStore } from '../../../stores/error-store';
import { isEncryptedBackup, parseBackupEnvelope } from '../../../utils/cryptoBackup';
import {
    REFRESH_TOKEN_PREFIX,
    RESTORE_THROTTLE_MS,
    type ImportPreviewData,
} from './types';

export interface UseBackupImportOptions {
    onClose: () => void;
}

export function useBackupImport({ onClose }: UseBackupImportOptions) {
    const { t } = useTranslation();

    // Import State
    const [importFileContent, setImportFileContent] = useState<string | null>(null);
    const [importFileName, setImportFileName] = useState<string | null>(null);
    const [isImportEncrypted, setIsImportEncrypted] = useState(false);
    const [decryptPassword, setDecryptPassword] = useState('');
    const [isImporting, setIsImporting] = useState(false);
    const [parsedDataPreview, setParsedDataPreview] = useState<ImportPreviewData | null>(null);

    const fileInputRef = useRef<HTMLInputElement>(null);
    const { fetchAccounts } = useAccountStore();
    const { loadConfig } = useConfigStore();
    const { fetchInstances } = useInstanceStore();

    // ---------------------------------------------------------------------------
    // Import Handlers
    // ---------------------------------------------------------------------------

    const handleRawFileContent = (content: string, fileName: string) => {
        setImportFileContent(content);
        setImportFileName(fileName);
        const encrypted = isEncryptedBackup(content);
        setIsImportEncrypted(encrypted);

        if (!encrypted) {
            // Parse plain preview immediately
            try {
                const parsed = JSON.parse(content);
                buildImportPreview(parsed);
            } catch {
                showToast(t('backup.invalid_json', 'Selected file contains invalid JSON.'), 'error');
            }
        } else {
            setParsedDataPreview(null);
        }
    };

    const handleSelectImportFile = async () => {
        if (isTauri()) {
            try {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const selected = await open({
                    multiple: false,
                    filters: [
                        {
                            name: 'AGM Backup or JSON',
                            extensions: ['agmbackup', 'json'],
                        },
                    ],
                });

                if (selected && typeof selected === 'string') {
                    const content: string = await invoke('read_text_file', { path: selected });
                    handleRawFileContent(content, selected.split(/[\\/]/).pop() || 'backup');
                }
            } catch (err) {
                showToast(`${t('common.error', 'Error')}: ${err}`, 'error');
            }
        } else {
            fileInputRef.current?.click();
        }
    };

    const handleFileInputChange = async (e: ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        try {
            const content = await file.text();
            handleRawFileContent(content, file.name);
        } catch (err) {
            showToast(`${t('common.error', 'Error reading file')}: ${err}`, 'error');
        } finally {
            e.target.value = '';
        }
    };

    const buildImportPreview = (parsed: unknown) => {
        let accountsCount = 0;
        let hasConfig = false;
        let instancesCount = 0;
        let auditHistoryCount = 0;
        let backupType: 'accounts' | 'full' = 'accounts';

        const data = (parsed as { data?: unknown }).data || parsed;
        if (Array.isArray(data)) {
            accountsCount = data.length;
        } else if (data !== null && typeof data === 'object') {
            const obj = data as {
                accounts?: unknown;
                config?: unknown;
                instances?: unknown;
                audit_history?: unknown;
            };
            if (Array.isArray(obj.accounts)) {
                accountsCount = obj.accounts.length;
                if (obj.config) hasConfig = true;
                if (Array.isArray(obj.instances)) {
                    instancesCount = obj.instances.length;
                }
                if (Array.isArray(obj.audit_history)) {
                    auditHistoryCount = obj.audit_history.length;
                }
                if (hasConfig || instancesCount > 0) {
                    backupType = 'full';
                }
            }
        }

        setParsedDataPreview({
            backupType,
            accountsCount,
            hasConfig,
            instancesCount,
            auditHistoryCount,
            rawPayload: data,
        });
    };

    const handleDecryptPreview = async () => {
        if (!importFileContent) return;
        try {
            const result = await parseBackupEnvelope(importFileContent, decryptPassword);
            buildImportPreview(result.data);
            showToast(t('backup.decrypted_success', 'Backup decrypted successfully! Review below.'), 'success');
        } catch (err) {
            showToast(err instanceof Error && err.message ? err.message : 'Decryption failed', 'error');
        }
    };

    const handleExecuteRestore = async () => {
        if (!parsedDataPreview) return;
        setIsImporting(true);

        let successAccounts = 0;
        let failAccounts = 0;

        try {
            const payload = parsedDataPreview.rawPayload;
            const payloadRecord = payload as { accounts?: unknown; config?: unknown };
            const accountsList: { email?: string; refresh_token?: string }[] = Array.isArray(payload)
                ? (payload as { email?: string; refresh_token?: string }[])
                : (payloadRecord.accounts as { email?: string; refresh_token?: string }[] | undefined) || [];

            // Restore Accounts
            for (const item of accountsList) {
                if (item.refresh_token && item.refresh_token.startsWith(REFRESH_TOKEN_PREFIX)) {
                    try {
                        await addAccount(item.email || '', item.refresh_token);
                        successAccounts++;
                    } catch (e) {
                        useErrorStore.getState().trackWarning(e, {
                            source: 'UnifiedBackupModal.import',
                            triggerAction: 'addAccount',
                        });
                        failAccounts++;
                    }
                    await new Promise((r) => setTimeout(r, RESTORE_THROTTLE_MS));
                }
            }

            // Restore Config if full backup
            const restoreConfig = payloadRecord.config;
            if (restoreConfig && typeof restoreConfig === 'object') {
                try {
                    await invoke('save_app_config', { config: restoreConfig });
                } catch (e) {
                    console.warn('Config restore non-fatal warning:', e);
                    // Tracked in the error module; config section skipped, remaining backup sections still restore.
                    useErrorStore.getState().trackWarning(e, {
                        source: 'UnifiedBackupModal.restoreBackup',
                        endpoint: 'save_app_config',
                        triggerAction: 'restore_config_section',
                    });
                }
            }

            // Refresh frontend stores
            await fetchAccounts();
            await loadConfig();
            await fetchInstances();

            showToast(
                t(
                    'backup.restore_complete',
                    `Restore complete: ${successAccounts} account(s) restored (${failAccounts} skipped).`
                ),
                'success'
            );
            onClose();
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            showToast(`${t('common.error', 'Error')}: ${message}`, 'error');
        } finally {
            setIsImporting(false);
        }
    };

    const importApi: {
        fileInputRef: RefObject<HTMLInputElement | null>;
        importFileName: string | null;
        isImportEncrypted: boolean;
        decryptPassword: string;
        setDecryptPassword: (v: string) => void;
        isImporting: boolean;
        parsedDataPreview: ImportPreviewData | null;
        handleSelectImportFile: () => Promise<void>;
        handleFileInputChange: (e: ChangeEvent<HTMLInputElement>) => Promise<void>;
        handleDecryptPreview: () => Promise<void>;
        handleExecuteRestore: () => Promise<void>;
    } = {
        fileInputRef,
        importFileName,
        isImportEncrypted,
        decryptPassword,
        setDecryptPassword,
        isImporting,
        parsedDataPreview,
        handleSelectImportFile,
        handleFileInputChange,
        handleDecryptPreview,
        handleExecuteRestore,
    };

    return importApi;
}
