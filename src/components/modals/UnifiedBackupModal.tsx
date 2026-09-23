import { useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import {
    ShieldCheck,
    Download,
    Upload,
    Lock,
    KeyRound,
    FileText,
    Mail,
    CheckCircle2,
    AlertTriangle,
    Loader2,
    X,
    Eye,
    EyeOff,
} from 'lucide-react';
import { request as invoke } from '../../utils/request';
import { isTauri } from '../../utils/env';
import { showToast } from '../common/ToastContainer';
import { exportAccounts, addAccount } from '../../services/accountService';
import { useAccountStore } from '../../stores/useAccountStore';
import { useConfigStore } from '../../stores/useConfigStore';
import { useInstanceStore } from '../../stores/useInstanceStore';
import {
    createBackupEnvelope,
    isEncryptedBackup,
    parseBackupEnvelope,
} from '../../utils/cryptoBackup';

interface UnifiedBackupModalProps {
    isOpen: boolean;
    onClose: () => void;
    initialTab?: 'export' | 'import';
}

export function UnifiedBackupModal({ isOpen, onClose, initialTab = 'export' }: UnifiedBackupModalProps) {
    const { t } = useTranslation();
    const [activeTab, setActiveTab] = useState<'export' | 'import'>(initialTab);

    // Export State
    const [exportScope, setExportScope] = useState<'accounts' | 'full'>('accounts');
    const [usePassword, setUsePassword] = useState(false);
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [showPassword, setShowPassword] = useState(false);
    const [isExporting, setIsExporting] = useState(false);

    // Import State
    const [importFileContent, setImportFileContent] = useState<string | null>(null);
    const [importFileName, setImportFileName] = useState<string | null>(null);
    const [isImportEncrypted, setIsImportEncrypted] = useState(false);
    const [decryptPassword, setDecryptPassword] = useState('');
    const [isImporting, setIsImporting] = useState(false);
    const [parsedDataPreview, setParsedDataPreview] = useState<{
        backupType: 'accounts' | 'full';
        accountsCount: number;
        hasConfig: boolean;
        instancesCount: number;
        rawPayload: any;
    } | null>(null);

    const fileInputRef = useRef<HTMLInputElement>(null);
    const { accounts, fetchAccounts } = useAccountStore();
    const { config, loadConfig } = useConfigStore();
    const { instances, fetchInstances } = useInstanceStore();

    if (!isOpen) return null;

    // ---------------------------------------------------------------------------
    // Export Handlers
    // ---------------------------------------------------------------------------

    const buildExportPayload = async () => {
        const accountIds = accounts.map((a) => a.id);
        const exportRes = await exportAccounts(accountIds);
        const accountsData = exportRes.accounts || [];

        if (exportScope === 'accounts') {
            return accountsData;
        }

        // Full backup payload
        let emailSettings = null;
        let emailAccounts = null;
        try {
            emailSettings = await invoke('get_email_settings');
            emailAccounts = await invoke('list_email_accounts');
        } catch {
            // Standby or not configured
        }

        return {
            accounts: accountsData,
            config: config || {},
            instances: instances.map((i) => i.config),
            email_settings: emailSettings,
            email_accounts: emailAccounts,
        };
    };

    const handleSaveToFile = async () => {
        if (usePassword) {
            if (!password || password.length < 6) {
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
            const ext = usePassword ? 'agmbackup' : 'json';
            const fileName = `agm_${exportScope}_backup_${timestamp}.${ext}`;

            if (isTauri()) {
                const { save } = await import('@tauri-apps/plugin-dialog');
                const defaultDir = config?.default_export_path;
                let savePath = defaultDir ? `${defaultDir}/${fileName}` : fileName;

                const selected = await save({
                    defaultPath: savePath,
                    filters: [
                        {
                            name: usePassword ? 'AGM Encrypted Backup' : 'JSON Backup',
                            extensions: [ext, 'json'],
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
        } catch (err: any) {
            showToast(`${t('common.error', 'Error')}: ${err?.message || err}`, 'error');
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
            const accountsList = ((await invoke('list_email_accounts').catch(() => [])) as any[]) || [];
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
        } catch (err: any) {
            showToast(`${t('common.error', 'Error')}: ${err?.message || err}`, 'error');
        } finally {
            setIsExporting(false);
        }
    };

    // ---------------------------------------------------------------------------
    // Import Handlers
    // ---------------------------------------------------------------------------

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

    const handleFileInputChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
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
            } catch (err) {
                showToast(t('backup.invalid_json', 'Selected file contains invalid JSON.'), 'error');
            }
        } else {
            setParsedDataPreview(null);
        }
    };

    const buildImportPreview = (parsed: any) => {
        let accountsCount = 0;
        let hasConfig = false;
        let instancesCount = 0;
        let backupType: 'accounts' | 'full' = 'accounts';

        const data = parsed.data || parsed;
        if (Array.isArray(data)) {
            accountsCount = data.length;
        } else if (data.accounts && Array.isArray(data.accounts)) {
            accountsCount = data.accounts.length;
            if (data.config) hasConfig = true;
            if (data.instances && Array.isArray(data.instances)) {
                instancesCount = data.instances.length;
            }
            if (hasConfig || instancesCount > 0) {
                backupType = 'full';
            }
        }

        setParsedDataPreview({
            backupType,
            accountsCount,
            hasConfig,
            instancesCount,
            rawPayload: data,
        });
    };

    const handleDecryptPreview = async () => {
        if (!importFileContent) return;
        try {
            const result = await parseBackupEnvelope(importFileContent, decryptPassword);
            buildImportPreview(result.data);
            showToast(t('backup.decrypted_success', 'Backup decrypted successfully! Review below.'), 'success');
        } catch (err: any) {
            showToast(err.message || 'Decryption failed', 'error');
        }
    };

    const handleExecuteRestore = async () => {
        if (!parsedDataPreview) return;
        setIsImporting(true);

        let successAccounts = 0;
        let failAccounts = 0;

        try {
            const payload = parsedDataPreview.rawPayload;
            const accountsList: any[] = Array.isArray(payload) ? payload : payload.accounts || [];

            // Restore Accounts
            for (const item of accountsList) {
                if (item.refresh_token && item.refresh_token.startsWith('1//')) {
                    try {
                        await addAccount(item.email || '', item.refresh_token);
                        successAccounts++;
                    } catch {
                        failAccounts++;
                    }
                    await new Promise((r) => setTimeout(r, 60));
                }
            }

            // Restore Config if full backup
            if (payload.config && typeof payload.config === 'object') {
                try {
                    await invoke('save_app_config', { config: payload.config });
                } catch (e) {
                    console.warn('Config restore non-fatal warning:', e);
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
        } catch (err: any) {
            showToast(`${t('common.error', 'Error')}: ${err?.message || err}`, 'error');
        } finally {
            setIsImporting(false);
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs animate-in fade-in duration-200">
            <div className="relative w-full max-w-xl bg-white dark:bg-slate-900 rounded-2xl shadow-2xl border border-gray-200 dark:border-slate-800 overflow-hidden">
                {/* Hidden browser input */}
                <input
                    ref={fileInputRef}
                    type="file"
                    accept=".json,.agmbackup,application/json"
                    style={{ display: 'none' }}
                    onChange={handleFileInputChange}
                />

                {/* Modal Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100 dark:border-slate-800 bg-gray-50/50 dark:bg-slate-800/50">
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-xl bg-purple-50 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400">
                            <ShieldCheck className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-base font-semibold text-gray-900 dark:text-gray-100">
                                {t('backup.title', 'Unified Backup & Encrypted Vault')}
                            </h2>
                            <p className="text-xs text-gray-500 dark:text-gray-400">
                                {t('backup.subtitle', 'Export accounts, configurations, or restore encrypted archives')}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Tab Switcher */}
                <div className="flex border-b border-gray-100 dark:border-slate-800 px-6 pt-3 bg-gray-50/30 dark:bg-slate-800/30">
                    <button
                        type="button"
                        onClick={() => setActiveTab('export')}
                        className={`flex items-center gap-2 pb-2.5 px-3 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                            activeTab === 'export'
                                ? 'border-purple-600 text-purple-600 dark:text-purple-400'
                                : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                        }`}
                    >
                        <Download className="w-3.5 h-3.5" />
                        <span>{t('backup.export_tab', 'Export Backup')}</span>
                    </button>
                    <button
                        type="button"
                        onClick={() => setActiveTab('import')}
                        className={`flex items-center gap-2 pb-2.5 px-3 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                            activeTab === 'import'
                                ? 'border-purple-600 text-purple-600 dark:text-purple-400'
                                : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                        }`}
                    >
                        <Upload className="w-3.5 h-3.5" />
                        <span>{t('backup.import_tab', 'Restore Backup')}</span>
                    </button>
                </div>

                {/* Modal Body */}
                <div className="p-6 space-y-5 max-h-[75vh] overflow-y-auto">
                    {activeTab === 'export' ? (
                        <>
                            {/* Backup Scope Selection */}
                            <div className="space-y-2">
                                <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">
                                    {t('backup.scope_label', 'Backup Scope')}
                                </label>
                                <div className="grid grid-cols-2 gap-3">
                                    <div
                                        onClick={() => setExportScope('accounts')}
                                        className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                                            exportScope === 'accounts'
                                                ? 'border-purple-500 bg-purple-50/50 dark:bg-purple-950/20'
                                                : 'border-gray-200 dark:border-slate-800 hover:bg-gray-50 dark:hover:bg-slate-800/50'
                                        }`}
                                    >
                                        <div className="flex items-center gap-2">
                                            <FileText className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                                            <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                                                {t('backup.scope_accounts', 'Accounts & Tokens')}
                                            </span>
                                        </div>
                                        <p className="mt-1 text-[11px] text-gray-500 dark:text-gray-400">
                                            {t(
                                                'backup.scope_accounts_desc',
                                                'Exports account credentials and refresh tokens only.'
                                            )}
                                        </p>
                                    </div>

                                    <div
                                        onClick={() => setExportScope('full')}
                                        className={`p-3.5 rounded-xl border cursor-pointer transition-all ${
                                            exportScope === 'full'
                                                ? 'border-purple-500 bg-purple-50/50 dark:bg-purple-950/20'
                                                : 'border-gray-200 dark:border-slate-800 hover:bg-gray-50 dark:hover:bg-slate-800/50'
                                        }`}
                                    >
                                        <div className="flex items-center gap-2">
                                            <ShieldCheck className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                                            <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                                                {t('backup.scope_full', 'Full System')}
                                            </span>
                                        </div>
                                        <p className="mt-1 text-[11px] text-gray-500 dark:text-gray-400">
                                            {t(
                                                'backup.scope_full_desc',
                                                'Accounts, configurations, instances, proxies & email vault.'
                                            )}
                                        </p>
                                    </div>
                                </div>
                            </div>

                            {/* Password Protection Toggle */}
                            <div className="p-3.5 bg-gray-50 dark:bg-slate-800/60 rounded-xl border border-gray-200 dark:border-slate-800 space-y-3">
                                <label className="flex items-center justify-between cursor-pointer">
                                    <div className="flex items-center gap-2">
                                        <Lock className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                                        <span className="text-xs font-semibold text-gray-900 dark:text-gray-100">
                                            {t('backup.encrypt_toggle', 'Encrypt with Password (AES-256-GCM)')}
                                        </span>
                                    </div>
                                    <input
                                        type="checkbox"
                                        checked={usePassword}
                                        onChange={(e) => setUsePassword(e.target.checked)}
                                        className="rounded border-gray-300 text-purple-600 focus:ring-purple-500 w-4 h-4"
                                    />
                                </label>

                                {usePassword && (
                                    <div className="pt-2 space-y-2.5 animate-in fade-in duration-150">
                                        <div className="relative">
                                            <input
                                                type={showPassword ? 'text' : 'password'}
                                                placeholder={t('backup.password_placeholder', 'Enter strong encryption password')}
                                                value={password}
                                                onChange={(e) => setPassword(e.target.value)}
                                                className="w-full px-3 py-1.5 pr-8 text-xs border border-gray-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100 focus:outline-hidden focus:ring-2 focus:ring-purple-500"
                                            />
                                            <button
                                                type="button"
                                                onClick={() => setShowPassword(!showPassword)}
                                                className="absolute right-2.5 top-2 text-gray-400 hover:text-gray-600"
                                            >
                                                {showPassword ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                                            </button>
                                        </div>

                                        <input
                                            type="password"
                                            placeholder={t('backup.confirm_password_placeholder', 'Confirm encryption password')}
                                            value={confirmPassword}
                                            onChange={(e) => setConfirmPassword(e.target.value)}
                                            className="w-full px-3 py-1.5 text-xs border border-gray-300 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100 focus:outline-hidden focus:ring-2 focus:ring-purple-500"
                                        />

                                        <div className="p-2.5 bg-amber-50 dark:bg-amber-950/20 border border-amber-200/50 dark:border-amber-900/30 rounded-lg flex items-start gap-1.5 text-[11px] text-amber-800 dark:text-amber-300">
                                            <AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                                            <span>
                                                {t(
                                                    'backup.password_warning',
                                                    'Encryption uses native PBKDF2 (100k rounds) + AES-GCM. Passwords cannot be recovered if forgotten.'
                                                )}
                                            </span>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </>
                    ) : (
                        <>
                            {/* Import File Selection */}
                            <div className="space-y-3">
                                <div
                                    onClick={handleSelectImportFile}
                                    className="p-6 border-2 border-dashed border-gray-300 dark:border-slate-700 rounded-2xl text-center hover:border-purple-500 dark:hover:border-purple-400 cursor-pointer transition-all bg-gray-50/50 dark:bg-slate-800/40 group"
                                >
                                    <div className="mx-auto w-10 h-10 rounded-xl bg-purple-50 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400 flex items-center justify-center group-hover:scale-110 transition-transform">
                                        <Upload className="w-5 h-5" />
                                    </div>
                                    <div className="mt-2 text-xs font-semibold text-gray-800 dark:text-gray-200">
                                        {importFileName || t('backup.click_to_select', 'Click to choose backup file (.json or .agmbackup)')}
                                    </div>
                                    <p className="mt-1 text-[11px] text-gray-500 dark:text-gray-400">
                                        {t('backup.drop_support', 'Supports plain JSON account lists and password-encrypted AGM backups.')}
                                    </p>
                                </div>

                                {/* Password prompt if encrypted */}
                                {isImportEncrypted && !parsedDataPreview && (
                                    <div className="p-4 bg-purple-50/50 dark:bg-purple-950/20 border border-purple-200 dark:border-purple-800/40 rounded-xl space-y-3 animate-in fade-in">
                                        <div className="flex items-center gap-2 text-xs font-semibold text-purple-900 dark:text-purple-300">
                                            <KeyRound className="w-4 h-4" />
                                            <span>{t('backup.enter_decrypt_password', 'Encrypted Archive: Enter Password')}</span>
                                        </div>
                                        <div className="flex gap-2">
                                            <input
                                                type="password"
                                                placeholder={t('backup.password', 'Password')}
                                                value={decryptPassword}
                                                onChange={(e) => setDecryptPassword(e.target.value)}
                                                className="grow px-3 py-1.5 text-xs border border-purple-300 dark:border-purple-800 rounded-lg bg-white dark:bg-slate-900 text-gray-900 dark:text-gray-100"
                                            />
                                            <button
                                                type="button"
                                                onClick={handleDecryptPreview}
                                                disabled={!decryptPassword}
                                                className="px-3 py-1.5 text-xs font-medium text-white bg-purple-600 hover:bg-purple-500 disabled:opacity-50 rounded-lg cursor-pointer"
                                            >
                                                {t('backup.decrypt_btn', 'Decrypt')}
                                            </button>
                                        </div>
                                    </div>
                                )}

                                {/* Decrypted Preview Summary */}
                                {parsedDataPreview && (
                                    <div className="p-4 bg-emerald-50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-800/40 rounded-xl space-y-2 text-xs text-emerald-900 dark:text-emerald-300 animate-in fade-in">
                                        <div className="flex items-center gap-2 font-semibold">
                                            <CheckCircle2 className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                                            <span>{t('backup.ready_to_restore', 'Backup Verified & Ready to Restore')}</span>
                                        </div>
                                        <div className="grid grid-cols-2 gap-2 pt-1 text-[11px] text-gray-700 dark:text-gray-300">
                                            <div>
                                                • Accounts detected: <strong>{parsedDataPreview.accountsCount}</strong>
                                            </div>
                                            <div>
                                                • Scope: <strong>{parsedDataPreview.backupType.toUpperCase()}</strong>
                                            </div>
                                            <div>
                                                • Config present: <strong>{parsedDataPreview.hasConfig ? 'Yes' : 'No'}</strong>
                                            </div>
                                            <div>
                                                • Instances: <strong>{parsedDataPreview.instancesCount}</strong>
                                            </div>
                                        </div>
                                    </div>
                                )}
                            </div>
                        </>
                    )}
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-between px-6 py-4 border-t border-gray-100 dark:border-slate-800 bg-gray-50/50 dark:bg-slate-800/50">
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-2 text-xs font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 rounded-xl transition-colors cursor-pointer"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>

                    {activeTab === 'export' ? (
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={handleSendToEmail}
                                disabled={isExporting}
                                className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-900/30 border border-purple-200 dark:border-purple-800 hover:bg-purple-100 dark:hover:bg-purple-900/50 rounded-xl disabled:opacity-50 transition-all cursor-pointer"
                                title="Send backup directly to your outbound mailbox"
                            >
                                <Mail className="w-3.5 h-3.5" />
                                <span>{t('backup.send_to_email', 'Send to Mailbox')}</span>
                            </button>
                            <button
                                type="button"
                                onClick={handleSaveToFile}
                                disabled={isExporting}
                                className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium text-white bg-purple-600 hover:bg-purple-500 rounded-xl shadow-xs disabled:opacity-50 transition-all cursor-pointer"
                            >
                                {isExporting ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Download className="w-3.5 h-3.5" />}
                                <span>{t('backup.save_file_btn', 'Save Backup File')}</span>
                            </button>
                        </div>
                    ) : (
                        <button
                            type="button"
                            onClick={handleExecuteRestore}
                            disabled={!parsedDataPreview || isImporting}
                            className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium text-white bg-purple-600 hover:bg-purple-500 rounded-xl shadow-xs disabled:opacity-50 transition-all cursor-pointer"
                        >
                            {isImporting ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Upload className="w-3.5 h-3.5" />}
                            <span>{t('backup.restore_now_btn', 'Execute Restore')}</span>
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
}
