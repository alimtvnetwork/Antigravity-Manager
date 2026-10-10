import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../../utils/request';
import { isTauri } from '../../../utils/env';
import { showToast } from '../../common/ToastContainer';
import {
    exportAccounts,
    deployAccountsToFleet,
    checkGitmapAvailable,
    type FleetDeployResult,
} from '../../../services/accountService';
import { useAccountStore } from '../../../stores/useAccountStore';
import { useConfigStore } from '../../../stores/useConfigStore';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { useErrorStore } from '../../../stores/error-store';
import { createBackupEnvelope } from '../../../utils/cryptoBackup';
import {
    AUDIT_HISTORY_LIMIT,
    ENCRYPTED_BACKUP_EXT,
    MIN_EXPORT_PASSWORD_LENGTH,
    PLAIN_BACKUP_EXT,
    type ExportPayload,
    type ExportScope,
    type FullBackupPayload,
} from './types';

export interface UseBackupExportOptions {
    isOpen: boolean;
    onClose: () => void;
}

export function useBackupExport({ isOpen, onClose }: UseBackupExportOptions) {
    const { t } = useTranslation();

    // Export State
    const [exportScope, setExportScope] = useState<ExportScope>('accounts');
    const [includeAuditHistory, setIncludeAuditHistory] = useState(true);
    const [usePassword, setUsePassword] = useState(false);
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [showPassword, setShowPassword] = useState(false);
    const [isExporting, setIsExporting] = useState(false);

    // Fleet Deployment State
    const [includeMainNode, setIncludeMainNode] = useState(false);
    const [isDeployingFleet, setIsDeployingFleet] = useState(false);
    const [gitmapAvailable, setGitmapAvailable] = useState<boolean | null>(null);
    const [fleetDeployResult, setFleetDeployResult] = useState<FleetDeployResult | null>(null);

    const { accounts } = useAccountStore();
    const { config } = useConfigStore();
    const { instances } = useInstanceStore();

    useEffect(() => {
        if (isOpen) {
            checkGitmapAvailable()
                .then((avail) => setGitmapAvailable(avail))
                .catch((err) => {
                    // GitMap CLI missing or unreachable — fleet deploy will be disabled,
                    // tracked for visibility in the error module.
                    useErrorStore.getState().trackWarning(err, {
                        source: 'UnifiedBackupModal.fleet',
                        triggerAction: 'checkGitmapAvailable',
                    });
                    setGitmapAvailable(false);
                });
        }
    }, [isOpen]);

    // ---------------------------------------------------------------------------
    // Export Handlers
    // ---------------------------------------------------------------------------

    const buildExportPayload = async (): Promise<ExportPayload> => {
        const accountIds = accounts.map((a) => a.id);
        const exportRes = await exportAccounts(accountIds);
        const accountsData = exportRes.accounts || [];

        if (exportScope === 'accounts') {
            return accountsData;
        }

        // Full backup payload
        let emailSettings: unknown = null;
        let emailAccounts: unknown = null;
        let auditHistory: unknown = null;
        try {
            emailSettings = await invoke('get_email_settings');
            emailAccounts = await invoke('list_email_accounts');
        } catch (e) {
            // Standby or not configured — benign, tracked for visibility
            useErrorStore.getState().trackWarning(e, {
                source: 'UnifiedBackupModal.export',
                triggerAction: 'get_email_settings',
            });
        }

        if (includeAuditHistory) {
            try {
                auditHistory = await invoke('list_task_history', { offset: 0, limit: AUDIT_HISTORY_LIMIT });
            } catch (e) {
                // Standby or not configured — benign, tracked for visibility
                useErrorStore.getState().trackWarning(e, {
                    source: 'UnifiedBackupModal.export',
                    triggerAction: 'list_task_history',
                });
            }
        }

        const fullPayload: FullBackupPayload = {
            accounts: accountsData,
            config: (config || {}) as Record<string, unknown>,
            instances: instances.map((i: { config: unknown }) => i.config),
            email_settings: emailSettings,
            email_accounts: emailAccounts,
            audit_history: auditHistory,
        };
        return fullPayload;
    };

    const handleSaveToFile = async () => {
        if (usePassword) {
            if (!password || password.length < MIN_EXPORT_PASSWORD_LENGTH) {
                showToast(t('backup.password_too_short', 'Password must be at least 6 characters.'), 'warning');
                return;
            }
            if (password !== confirmPassword) {
                showToast(t('backup.passwords_do_not_match', 'Passwords do not match.'), 'error');
                return;
            }
        }

        setIsExporting(true);
        try {
            const rawPayload = await buildExportPayload();
            const envelopeJson = await createBackupEnvelope(
                rawPayload,
                exportScope,
                usePassword ? password : undefined
            );

            const timestamp = new Date().toISOString().split('T')[0];
            const ext = usePassword ? ENCRYPTED_BACKUP_EXT : PLAIN_BACKUP_EXT;
            const fileName = `agm_${exportScope}_backup_${timestamp}.${ext}`;

            if (isTauri()) {
                const { save } = await import('@tauri-apps/plugin-dialog');
                const defaultDir = config?.default_export_path;
                const savePath = defaultDir ? `${defaultDir}/${fileName}` : fileName;

                const selected = await save({
                    defaultPath: savePath,
                    filters: [
                        {
                            name: usePassword ? 'AGM Encrypted Backup' : 'JSON Backup',
                            extensions: [ext, PLAIN_BACKUP_EXT],
                        },
                    ],
                });

                if (selected) {
                    await invoke('save_text_file', { path: selected, content: envelopeJson });
                    showToast(t('backup.export_success', `Saved backup to ${selected}`), 'success');
                    onClose();
                }
            } else {
                // Browser download fallback
                const blob = new Blob([envelopeJson], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = fileName;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                showToast(t('backup.export_success', `Downloaded ${fileName}`), 'success');
                onClose();
            }
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            showToast(`${t('common.error', 'Error')}: ${message}`, 'error');
        } finally {
            setIsExporting(false);
        }
    };

    const handleSendToEmail = async () => {
        setIsExporting(true);
        try {
            const rawPayload = await buildExportPayload();
            const envelopeJson = await createBackupEnvelope(
                rawPayload,
                exportScope,
                usePassword ? password : undefined
            );

            const timestamp = new Date().toISOString();
            const subject = `[AGM Backup] ${exportScope.toUpperCase()} Backup (${timestamp})`;

            // Query configured outbound email account
            const accountsList = (await invoke('list_email_accounts')
                .catch((e) => {
                    // No outbound account or backend unreachable — fallback to empty
                    // list so the user gets the "no email account" hint below.
                    useErrorStore.getState().trackWarning(e, {
                        source: 'UnifiedBackupModal.export',
                        triggerAction: 'list_email_accounts',
                    });
                    return [];
                })) as { is_default?: boolean; email: string }[];
            if (!accountsList || accountsList.length === 0) {
                showToast(
                    t('backup.no_email_account', 'No outbound email accounts configured in Email settings.'),
                    'warning'
                );
                return;
            }

            const targetAccount = accountsList.find((a) => a.is_default) || accountsList[0];
            await invoke('send_outbound_email_message', {
                subject,
                body: envelopeJson,
                recipient: targetAccount.email,
            });

            showToast(t('backup.email_sent_success', `Backup emailed to ${targetAccount.email}`), 'success');
            onClose();
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : String(err);
            showToast(`${t('common.error', 'Error')}: ${message}`, 'error');
        } finally {
            setIsExporting(false);
        }
    };

    const handleDeployToFleet = async () => {
        setIsDeployingFleet(true);
        setFleetDeployResult(null);
        try {
            const result = await deployAccountsToFleet(includeMainNode);
            setFleetDeployResult(result);
            if (result.success) {
                showToast(
                    result.message || t('backup.fleet_deploy_success', 'Fleet accounts deployed successfully!'),
                    'success'
                );
            } else {
                showToast(
                    result.message || t('backup.fleet_deploy_failed', 'Fleet deployment failed.'),
                    'error'
                );
            }
        } catch (err: unknown) {
            const errMsg = err instanceof Error ? err.message : String(err);
            const failResult: FleetDeployResult = {
                success: false,
                message: errMsg,
                raw_output: errMsg,
            };
            setFleetDeployResult(failResult);
            showToast(`${t('common.error', 'Error')}: ${errMsg}`, 'error');
        } finally {
            setIsDeployingFleet(false);
        }
    };

    return {
        exportScope,
        setExportScope,
        includeAuditHistory,
        setIncludeAuditHistory,
        usePassword,
        setUsePassword,
        password,
        setPassword,
        confirmPassword,
        setConfirmPassword,
        showPassword,
        setShowPassword,
        isExporting,
        includeMainNode,
        setIncludeMainNode,
        isDeployingFleet,
        gitmapAvailable,
        fleetDeployResult,
        handleSaveToFile,
        handleSendToEmail,
        handleDeployToFleet,
    };
}
