import React, { useState, useEffect } from 'react';
import {
    Mail,
    Send,
    Inbox,
    Server,
    Key,
    RefreshCw,
    Plus,
    Trash2,
    CheckCircle2,
    AlertTriangle,
    Download,
    Upload,
    ShieldCheck,
    FileSpreadsheet,
    FileText,
    FileJson,
    Check,
    Cpu,
    Radio,
    Sparkles,
    Database,
} from 'lucide-react';
import {
    EmailAccount,
    EmailAccountInput,
    NotifyRecipient,
    EmailNotificationSettings as ISettings,
    WatcherStatus,
    listEmailAccounts,
    addEmailAccount,
    updateEmailAccount,
    deleteEmailAccount,
    setDefaultEmailAccount,
    listNotifyRecipients,
    addNotifyRecipient,
    deleteNotifyRecipient,
    getEmailSettings,
    saveEmailSettings,
    testSmtpConnection,
    testImapConnection,
    exportEmailData,
    importEmailData,
    backupEmailDb,
    restoreEmailDb,
    getEmailWatcherStatus,
    triggerManualEmailCheck,
} from '../../services/emailService';
import ModalDialog from '../common/ModalDialog';
import { showToast } from '../common/ToastContainer';

export default function EmailNotificationSettings() {
    const [accounts, setAccounts] = useState<EmailAccount[]>([]);
    const [recipients, setRecipients] = useState<NotifyRecipient[]>([]);
    const [settings, setSettings] = useState<ISettings>({
        id: 'global',
        is_enabled: false,
        polling_interval_minutes: 3,
        inbox_check_interval_minutes: 1,
        notify_on_quota_drop: true,
        quota_drop_threshold_percent: 15,
        notify_on_workspace_switch: true,
        notify_on_idle_workspace: true,
        allow_remote_prompt_execution: true,
        allow_remote_cli_execution: true,
        allow_remote_instance_rotation: true,
        local_machine_name: '',
        local_machine_ip: '',
        updated_at: 0,
    });
    const [watcherStatus, setWatcherStatus] = useState<WatcherStatus | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [isSaving, setIsSaving] = useState(false);

    // Account modal state
    const [isAccountModalOpen, setIsAccountModalOpen] = useState(false);
    const [editingAccount, setEditingAccount] = useState<EmailAccountInput>({
        alias: '',
        email: '',
        password: '',
        smtp_host: 'smtp.gmail.com',
        smtp_port: 587,
        imap_host: 'imap.gmail.com',
        imap_port: 993,
        encryption_type: 'TLS',
        is_default: false,
        is_active: true,
    });

    // Recipient state
    const [newRecipientEmail, setNewRecipientEmail] = useState('');
    const [newRecipientGroup, setNewRecipientGroup] = useState('default');

    // Import/Export state
    const [isImportModalOpen, setIsImportModalOpen] = useState(false);
    const [importFormat, setImportFormat] = useState<'json' | 'csv' | 'xlsx'>('json');
    const [importPayload, setImportPayload] = useState('');
    const [testingAccountId, setTestingAccountId] = useState<string | null>(null);
    const fileInputRef = React.useRef<HTMLInputElement>(null);

    const loadAll = async () => {
        setIsLoading(true);
        try {
            const [accs, recs, sets, st] = await Promise.all([
                listEmailAccounts(),
                listNotifyRecipients(),
                getEmailSettings(),
                getEmailWatcherStatus(),
            ]);
            setAccounts(accs);
            setRecipients(recs);
            setSettings(sets);
            setWatcherStatus(st);
        } catch (e: any) {
            console.error('Failed to load email settings:', e);
            showToast('Failed to load email configurations', 'error');
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        loadAll();
    }, []);

    const handleSaveSettings = async () => {
        setIsSaving(true);
        try {
            await saveEmailSettings(settings);
            showToast('Notification settings saved successfully', 'success');
            const st = await getEmailWatcherStatus();
            setWatcherStatus(st);
        } catch (e: any) {
            showToast('Failed to save settings: ' + (e?.message || e), 'error');
        } finally {
            setIsSaving(false);
        }
    };

    const handleOpenAddAccount = () => {
        setEditingAccount({
            alias: '',
            email: '',
            password: '',
            smtp_host: 'smtp.gmail.com',
            smtp_port: 587,
            imap_host: 'imap.gmail.com',
            imap_port: 993,
            encryption_type: 'TLS',
            is_default: accounts.length === 0,
            is_active: true,
        });
        setIsAccountModalOpen(true);
    };

    const handleOpenEditAccount = (acc: EmailAccount) => {
        setEditingAccount({
            id: acc.id,
            alias: acc.alias,
            email: acc.email,
            password: '',
            smtp_host: acc.smtp_host,
            smtp_port: acc.smtp_port,
            imap_host: acc.imap_host,
            imap_port: acc.imap_port,
            encryption_type: acc.encryption_type,
            is_default: acc.is_default,
            is_active: acc.is_active,
        });
        setIsAccountModalOpen(true);
    };

    const handleSaveAccount = async () => {
        if (!editingAccount.email.trim()) {
            showToast('Email address is required', 'error');
            return;
        }

        try {
            if (editingAccount.id) {
                await updateEmailAccount(editingAccount);
                showToast('Mailbox updated', 'success');
            } else {
                await addEmailAccount(editingAccount);
                showToast('Mailbox account added to vault', 'success');
            }
            setIsAccountModalOpen(false);
            const accs = await listEmailAccounts();
            setAccounts(accs);
        } catch (e: any) {
            showToast('Failed to save mailbox: ' + (e?.message || e), 'error');
        }
    };

    const handleDeleteAccount = async (id: string) => {
        if (!window.confirm('Are you sure you want to remove this mailbox?')) return;
        try {
            await deleteEmailAccount(id);
            showToast('Mailbox removed', 'success');
            setAccounts(accounts.filter((a) => a.id !== id));
        } catch (e: any) {
            showToast('Failed to delete account: ' + (e?.message || e), 'error');
        }
    };

    const handleSetDefault = async (id: string) => {
        try {
            await setDefaultEmailAccount(id);
            showToast('Default mailbox updated', 'success');
            const accs = await listEmailAccounts();
            setAccounts(accs);
        } catch (e: any) {
            showToast('Failed to set default: ' + (e?.message || e), 'error');
        }
    };

    const handleTestSmtp = async (id: string) => {
        setTestingAccountId(id);
        try {
            const res = await testSmtpConnection(id);
            showToast(res, 'success');
        } catch (e: any) {
            showToast('SMTP Test Failed: ' + (e?.message || e), 'error');
        } finally {
            setTestingAccountId(null);
        }
    };

    const handleTestImap = async (id: string) => {
        setTestingAccountId(id);
        try {
            const res = await testImapConnection(id);
            showToast(res, 'success');
        } catch (e: any) {
            showToast('IMAP Test Failed: ' + (e?.message || e), 'error');
        } finally {
            setTestingAccountId(null);
        }
    };

    const handleAddRecipient = async () => {
        if (!newRecipientEmail.trim()) {
            showToast('Recipient email is required', 'error');
            return;
        }
        try {
            const added = await addNotifyRecipient({
                email: newRecipientEmail.trim(),
                group_name: newRecipientGroup.trim() || 'default',
                is_active: true,
            });
            setRecipients([...recipients, added]);
            setNewRecipientEmail('');
            showToast('Recipient added', 'success');
        } catch (e: any) {
            showToast('Failed to add recipient: ' + (e?.message || e), 'error');
        }
    };

    const handleDeleteRecipient = async (id: string) => {
        try {
            await deleteNotifyRecipient(id);
            setRecipients(recipients.filter((r) => r.id !== id));
            showToast('Recipient removed', 'success');
        } catch (e: any) {
            showToast('Failed to delete recipient: ' + (e?.message || e), 'error');
        }
    };

    const handleExport = async (format: 'json' | 'csv' | 'xlsx') => {
        try {
            const content = await exportEmailData(format);
            const blob = new Blob([content], {
                type: format === 'json' ? 'application/json' : format === 'csv' ? 'text/csv' : 'application/vnd.ms-excel',
            });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `antigravity-mailboxes-${Date.now()}.${format === 'xlsx' ? 'xls' : format}`;
            a.click();
            URL.revokeObjectURL(url);
            showToast(`Exported ${format.toUpperCase()} successfully`, 'success');
        } catch (e: any) {
            showToast('Export failed: ' + (e?.message || e), 'error');
        }
    };

    const handleImportSubmit = async () => {
        if (!importPayload.trim()) {
            showToast('Payload cannot be empty', 'error');
            return;
        }
        try {
            const summary = await importEmailData(importFormat, importPayload);
            showToast(
                `Imported ${summary.accounts_imported} mailboxes, ${summary.recipients_imported} recipients`,
                'success'
            );
            setIsImportModalOpen(false);
            setImportPayload('');
            await loadAll();
        } catch (e: any) {
            showToast('Import failed: ' + (e?.message || e), 'error');
        }
    };

    const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        const lowerName = file.name.toLowerCase();
        if (lowerName.endsWith('.json')) {
            setImportFormat('json');
        } else if (lowerName.endsWith('.csv')) {
            setImportFormat('csv');
        } else if (lowerName.endsWith('.xml') || lowerName.endsWith('.xlsx') || lowerName.endsWith('.xls')) {
            setImportFormat('xlsx');
        }

        const reader = new FileReader();
        reader.onload = (event) => {
            const text = event.target?.result as string;
            if (text) {
                setImportPayload(text);
                showToast(`Loaded file ${file.name}`, 'info');
            }
        };
        reader.onerror = () => {
            showToast('Failed to read file', 'error');
        };
        reader.readAsText(file);
    };

    const handleBackupDb = async () => {
        try {
            const defaultFilename = `antigravity-email-vault-${Date.now()}.db`;
            const targetPath = window.prompt(
                'Enter backup destination path (e.g. D:\\backups\\email_vault.db):',
                defaultFilename
            );
            if (!targetPath) return;
            const res = await backupEmailDb(targetPath);
            showToast(res, 'success');
        } catch (e: any) {
            showToast('Backup failed: ' + (e?.message || e), 'error');
        }
    };

    const handleRestoreDb = async () => {
        try {
            const sourcePath = window.prompt('Enter path to email_vault.db to restore:');
            if (!sourcePath) return;
            if (!window.confirm('Restoring vault database will overwrite current configuration. Continue?')) return;
            const res = await restoreEmailDb(sourcePath);
            showToast(res, 'success');
            await loadAll();
        } catch (e: any) {
            showToast('Restore failed: ' + (e?.message || e), 'error');
        }
    };

    const handleTriggerManualCheck = async () => {
        try {
            const res = await triggerManualEmailCheck();
            showToast(res, 'success');
        } catch (e: any) {
            showToast('Manual check failed: ' + (e?.message || e), 'error');
        }
    };

    return (
        <div className="space-y-6">
            {/* Header & Machine Telemetry Banner */}
            <div className="bg-gradient-to-r from-slate-900 to-indigo-950 text-white rounded-2xl p-6 shadow-md border border-slate-800">
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div className="space-y-1">
                        <div className="flex items-center gap-2">
                            <Mail className="w-6 h-6 text-sky-400" />
                            <h2 className="text-xl font-bold tracking-tight">Mailbox Remote Automation & Security Vault</h2>
                        </div>
                        <p className="text-sm text-slate-300">
                            Split database encryption, mailbox pool failover swapping, quota sensors, and remote prompt injection.
                        </p>
                    </div>

                    <div className="flex flex-wrap items-center gap-3">
                        <div className="bg-slate-800/80 px-3.5 py-1.5 rounded-lg border border-slate-700 text-xs flex items-center gap-2">
                            <Cpu className="w-4 h-4 text-emerald-400" />
                            <span>Node: <strong className="text-slate-100">{watcherStatus?.machine_name || 'Detecting...'}</strong></span>
                        </div>
                        <div className="bg-slate-800/80 px-3.5 py-1.5 rounded-lg border border-slate-700 text-xs flex items-center gap-2">
                            <Radio className="w-4 h-4 text-sky-400" />
                            <span>Local IP: <strong className="text-slate-100">{watcherStatus?.machine_ip || '127.0.0.1'}</strong></span>
                        </div>
                        <div className={`px-3 py-1 rounded-full text-xs font-semibold flex items-center gap-1.5 ${watcherStatus?.is_running ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-amber-500/20 text-amber-300 border border-amber-500/30'}`}>
                            <span className={`w-2 h-2 rounded-full ${watcherStatus?.is_running ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'}`}></span>
                            {watcherStatus?.is_running ? 'Watcher Active' : 'Watcher Idle'}
                        </div>
                    </div>
                </div>
            </div>

            {/* Mailboxes Card */}
            <div className="bg-white dark:bg-base-100 rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-base-200">
                <div className="flex items-center justify-between mb-4">
                    <div>
                        <h3 className="text-base font-semibold text-gray-900 dark:text-base-content flex items-center gap-2">
                            <Server className="w-4 h-4 text-blue-500" />
                            Email Accounts & Mailbox Pool
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                            Mailbox pool with default sender prioritization and automatic failover swapping.
                        </p>
                    </div>

                    <div className="flex items-center gap-2">
                        <button
                            onClick={() => handleExport('json')}
                            className="px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 flex items-center gap-1.5"
                            title="Export JSON"
                        >
                            <FileJson className="w-3.5 h-3.5 text-amber-500" />
                            JSON
                        </button>
                        <button
                            onClick={() => handleExport('csv')}
                            className="px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 flex items-center gap-1.5"
                            title="Export CSV"
                        >
                            <FileText className="w-3.5 h-3.5 text-blue-500" />
                            CSV
                        </button>
                        <button
                            onClick={() => handleExport('xlsx')}
                            className="px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 flex items-center gap-1.5"
                            title="Export Excel"
                        >
                            <FileSpreadsheet className="w-3.5 h-3.5 text-emerald-500" />
                            Excel
                        </button>
                        <button
                            onClick={() => setIsImportModalOpen(true)}
                            className="px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 flex items-center gap-1.5"
                        >
                            <Upload className="w-3.5 h-3.5 text-indigo-500" />
                            Import
                        </button>
                        <button
                            onClick={handleBackupDb}
                            className="px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 flex items-center gap-1.5"
                            title="Backup SQLite Vault Database"
                        >
                            <Database className="w-3.5 h-3.5 text-purple-500" />
                            Backup DB
                        </button>
                        <button
                            onClick={handleRestoreDb}
                            className="px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 flex items-center gap-1.5"
                            title="Restore SQLite Vault Database"
                        >
                            <RefreshCw className="w-3.5 h-3.5 text-cyan-500" />
                            Restore DB
                        </button>
                        <button
                            onClick={handleOpenAddAccount}
                            className="px-3.5 py-1.5 text-xs font-medium bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center gap-1.5 shadow-sm"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            Add Mailbox
                        </button>
                    </div>
                </div>

                {accounts.length === 0 ? (
                    <div className="py-10 text-center border-2 border-dashed border-gray-200 dark:border-base-300 rounded-xl">
                        <Mail className="w-8 h-8 text-gray-300 dark:text-gray-600 mx-auto mb-2" />
                        <p className="text-sm font-medium text-gray-600 dark:text-gray-300">No mailboxes configured in vault</p>
                        <p className="text-xs text-gray-400 mt-1">Add an SMTP/IMAP account to enable dispatch and remote command execution</p>
                    </div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full text-left text-xs border-collapse">
                            <thead>
                                <tr className="border-b border-gray-200 dark:border-base-300 text-gray-500 dark:text-gray-400">
                                    <th className="py-2.5 px-3 font-semibold">Alias & Email</th>
                                    <th className="py-2.5 px-3 font-semibold">SMTP Host</th>
                                    <th className="py-2.5 px-3 font-semibold">IMAP Host</th>
                                    <th className="py-2.5 px-3 font-semibold">Enc</th>
                                    <th className="py-2.5 px-3 font-semibold">Role</th>
                                    <th className="py-2.5 px-3 font-semibold text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-gray-100 dark:divide-base-200">
                                {accounts.map((acc) => (
                                    <tr key={acc.id} className="hover:bg-gray-50/50 dark:hover:bg-base-200/50 transition-colors">
                                        <td className="py-2.5 px-3">
                                            <div className="font-semibold text-gray-800 dark:text-gray-200">{acc.alias}</div>
                                            <div className="text-gray-500 dark:text-gray-400">{acc.email}</div>
                                        </td>
                                        <td className="py-2.5 px-3 font-mono text-gray-600 dark:text-gray-300">
                                            {acc.smtp_host}:{acc.smtp_port}
                                        </td>
                                        <td className="py-2.5 px-3 font-mono text-gray-600 dark:text-gray-300">
                                            {acc.imap_host}:{acc.imap_port}
                                        </td>
                                        <td className="py-2.5 px-3">
                                            <span className="px-2 py-0.5 rounded bg-gray-100 dark:bg-base-300 text-gray-700 dark:text-gray-300 text-[10px] font-mono">
                                                {acc.encryption_type}
                                            </span>
                                        </td>
                                        <td className="py-2.5 px-3">
                                            {acc.is_default ? (
                                                <span className="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 text-[10px] font-semibold">
                                                    Default Sender
                                                </span>
                                            ) : (
                                                <button
                                                    onClick={() => handleSetDefault(acc.id)}
                                                    className="text-[10px] text-gray-400 hover:text-blue-600 underline"
                                                >
                                                    Set Default
                                                </button>
                                            )}
                                        </td>
                                        <td className="py-2.5 px-3 text-right space-x-1.5">
                                            <button
                                                onClick={() => handleTestSmtp(acc.id)}
                                                disabled={testingAccountId === acc.id}
                                                className="px-2 py-1 rounded bg-sky-50 dark:bg-sky-950/40 text-sky-600 dark:text-sky-400 hover:bg-sky-100 text-[11px] font-medium"
                                                title="Test Outbound SMTP"
                                            >
                                                Test SMTP
                                            </button>
                                            <button
                                                onClick={() => handleTestImap(acc.id)}
                                                disabled={testingAccountId === acc.id}
                                                className="px-2 py-1 rounded bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-100 text-[11px] font-medium"
                                                title="Test Inbound IMAP"
                                            >
                                                Test IMAP
                                            </button>
                                            <button
                                                onClick={() => handleOpenEditAccount(acc)}
                                                className="px-2 py-1 rounded hover:bg-gray-100 dark:hover:bg-base-200 text-gray-600 dark:text-gray-300 text-[11px]"
                                            >
                                                Edit
                                            </button>
                                            <button
                                                onClick={() => handleDeleteAccount(acc.id)}
                                                className="px-2 py-1 rounded hover:bg-red-50 dark:hover:bg-red-950/30 text-red-600 text-[11px]"
                                            >
                                                <Trash2 className="w-3 h-3 inline" />
                                            </button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

            {/* Notification Recipients Card */}
            <div className="bg-white dark:bg-base-100 rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-base-200">
                <h3 className="text-base font-semibold text-gray-900 dark:text-base-content flex items-center gap-2 mb-1">
                    <Send className="w-4 h-4 text-emerald-500" />
                    Notification Recipients
                </h3>
                <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                    External user email addresses and distribution groups that receive sensor alerts and telemetry.
                </p>

                <div className="flex flex-col sm:flex-row gap-2 mb-4">
                    <input
                        type="email"
                        placeholder="Recipient Email (e.g. user@domain.com)"
                        value={newRecipientEmail}
                        onChange={(e) => setNewRecipientEmail(e.target.value)}
                        className="flex-1 px-3 py-2 text-xs border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <input
                        type="text"
                        placeholder="Group (e.g. dev, ops, default)"
                        value={newRecipientGroup}
                        onChange={(e) => setNewRecipientGroup(e.target.value)}
                        className="w-36 px-3 py-2 text-xs border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button
                        onClick={handleAddRecipient}
                        className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-medium transition-colors shadow-sm"
                    >
                        Add Recipient
                    </button>
                </div>

                <div className="flex flex-wrap gap-2">
                    {recipients.map((rec) => (
                        <div
                            key={rec.id}
                            className="bg-gray-50 dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-lg px-3 py-1.5 flex items-center gap-2 text-xs"
                        >
                            <span className="font-medium text-gray-700 dark:text-gray-200">{rec.email}</span>
                            <span className="text-[10px] px-1.5 py-0.5 bg-gray-200 dark:bg-base-300 text-gray-600 dark:text-gray-400 rounded">
                                {rec.group_name}
                            </span>
                            <button
                                onClick={() => handleDeleteRecipient(rec.id)}
                                className="text-gray-400 hover:text-red-500"
                            >
                                <Trash2 className="w-3 h-3" />
                            </button>
                        </div>
                    ))}
                    {recipients.length === 0 && (
                        <span className="text-xs text-gray-400 italic">No recipients registered. Alerts will be skipped.</span>
                    )}
                </div>
            </div>

            {/* Watcher Daemon & Sensors Card */}
            <div className="bg-white dark:bg-base-100 rounded-2xl p-6 shadow-sm border border-gray-100 dark:border-base-200 space-y-5">
                <div className="flex items-center justify-between">
                    <div>
                        <h3 className="text-base font-semibold text-gray-900 dark:text-base-content flex items-center gap-2">
                            <Sparkles className="w-4 h-4 text-purple-500" />
                            Background Watcher Daemon & Multi-Trigger Sensors
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                            Configure automatic quota monitoring, idle workspace detection, and remote mailbox commands.
                        </p>
                    </div>

                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            checked={settings.is_enabled}
                            onChange={(e) => setSettings({ ...settings, is_enabled: e.target.checked })}
                            className="sr-only peer"
                        />
                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
                    </label>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-gray-300 block">
                            Telemetry & Sensor Loop Interval
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={1}
                                max={5}
                                value={settings.polling_interval_minutes}
                                onChange={(e) =>
                                    setSettings({ ...settings, polling_interval_minutes: parseInt(e.target.value) || 3 })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 w-16 text-right">
                                {settings.polling_interval_minutes} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400">
                            Interval for telemetry collection and quota/idle workspace sensor sampling.
                        </p>
                    </div>

                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-gray-300 block">
                            Inbound Mailbox Check Interval
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={1}
                                max={5}
                                value={settings.inbox_check_interval_minutes}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        inbox_check_interval_minutes: parseInt(e.target.value) || 1,
                                    })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 w-16 text-right">
                                {settings.inbox_check_interval_minutes} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400">
                            Frequency of reading the last 5 unread instructions via IMAP.
                        </p>
                    </div>
                </div>

                {/* Sensor Toggles */}
                <div className="space-y-3 border-t border-gray-100 dark:border-base-200 pt-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-gray-400">Sensor Notification Triggers</h4>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <div className="p-3 rounded-lg border border-gray-200 dark:border-base-300 flex items-start gap-2.5">
                            <input
                                type="checkbox"
                                id="chk_quota_drop"
                                checked={settings.notify_on_quota_drop}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_quota_drop: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_quota_drop" className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer">
                                <div className="font-semibold">Quota Drop Alert</div>
                                <div className="text-[11px] text-gray-400">Trigger when credit &lt; 15%</div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-base-300 flex items-start gap-2.5">
                            <input
                                type="checkbox"
                                id="chk_ws_switch"
                                checked={settings.notify_on_workspace_switch}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_workspace_switch: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_ws_switch" className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer">
                                <div className="font-semibold">Workspace Switch Notice</div>
                                <div className="text-[11px] text-gray-400">Email notice before auto-rotation</div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-base-300 flex items-start gap-2.5">
                            <input
                                type="checkbox"
                                id="chk_idle_ws"
                                checked={settings.notify_on_idle_workspace}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_idle_workspace: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_idle_ws" className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer">
                                <div className="font-semibold">Idle Workspace Alert</div>
                                <div className="text-[11px] text-gray-400">Ask for prompt when queue is empty</div>
                            </label>
                        </div>
                    </div>
                </div>

                {/* Remote Execution Toggles */}
                <div className="space-y-3 border-t border-gray-100 dark:border-base-200 pt-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-gray-400">Inbound Mailbox Command Capabilities</h4>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <div className="p-3 rounded-lg border border-gray-200 dark:border-base-300 flex items-start gap-2.5">
                            <input
                                type="checkbox"
                                id="chk_allow_prompt"
                                checked={settings.allow_remote_prompt_execution}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_prompt_execution: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_prompt" className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer">
                                <div className="font-semibold">Prompt Injection</div>
                                <div className="text-[11px] text-gray-400">Match <code>Project: &lt;name&gt;</code></div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-base-300 flex items-start gap-2.5">
                            <input
                                type="checkbox"
                                id="chk_allow_cli"
                                checked={settings.allow_remote_cli_execution}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_cli_execution: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_cli" className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer">
                                <div className="font-semibold">CLI Execution</div>
                                <div className="text-[11px] text-gray-400">Match <code>exec: &lt;ip&gt;</code></div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-base-300 flex items-start gap-2.5">
                            <input
                                type="checkbox"
                                id="chk_allow_instance"
                                checked={settings.allow_remote_instance_rotation}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_instance_rotation: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_instance" className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer">
                                <div className="font-semibold">Instance Launch / Rotate</div>
                                <div className="text-[11px] text-gray-400">Match <code>instance: new</code></div>
                            </label>
                        </div>
                    </div>
                </div>

                <div className="flex items-center justify-between pt-4 border-t border-gray-100 dark:border-base-200">
                    <button
                        onClick={handleTriggerManualCheck}
                        className="px-4 py-2 text-xs font-medium border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 rounded-lg transition-colors flex items-center gap-1.5"
                    >
                        <Send className="w-3.5 h-3.5 text-blue-500" />
                        Dispatch Test Cheat Sheet
                    </button>

                    <button
                        onClick={handleSaveSettings}
                        disabled={isSaving}
                        className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm"
                    >
                        {isSaving ? 'Saving Settings...' : 'Save Watcher Settings'}
                    </button>
                </div>
            </div>

            {/* Account Add/Edit Modal */}
            <ModalDialog
                isOpen={isAccountModalOpen}
                onClose={() => setIsAccountModalOpen(false)}
                title={editingAccount.id ? 'Edit Mailbox Configuration' : 'Add Mailbox to Secure Split Vault'}
            >
                <div className="space-y-4 text-xs">
                    <div>
                        <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">Account Alias</label>
                        <input
                            type="text"
                            placeholder="e.g. Primary Gmail, Alerts Mailer"
                            value={editingAccount.alias}
                            onChange={(e) => setEditingAccount({ ...editingAccount, alias: e.target.value })}
                            className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">Email Address</label>
                        <input
                            type="email"
                            placeholder="user@example.com"
                            value={editingAccount.email}
                            onChange={(e) => setEditingAccount({ ...editingAccount, email: e.target.value })}
                            className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Password / App Password
                            {editingAccount.id && <span className="text-gray-400 ml-1 font-normal">(Leave blank to keep existing)</span>}
                        </label>
                        <div className="relative">
                            <input
                                type="password"
                                placeholder={editingAccount.id ? '••••••••' : 'Application password / secret'}
                                value={editingAccount.password || ''}
                                onChange={(e) => setEditingAccount({ ...editingAccount, password: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                            <Key className="w-3.5 h-3.5 text-gray-400 absolute right-3 top-2.5" />
                        </div>
                        <p className="text-[11px] text-gray-400 mt-1">
                            Stored in isolated split database <code>email_passwords.db</code> with salted SSH RSA identity and machine-bound encryption.
                        </p>
                    </div>

                    <div className="grid grid-cols-3 gap-3">
                        <div className="col-span-2">
                            <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">SMTP Host</label>
                            <input
                                type="text"
                                value={editingAccount.smtp_host}
                                onChange={(e) => setEditingAccount({ ...editingAccount, smtp_host: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200"
                            />
                        </div>
                        <div>
                            <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">SMTP Port</label>
                            <input
                                type="number"
                                value={editingAccount.smtp_port}
                                onChange={(e) =>
                                    setEditingAccount({ ...editingAccount, smtp_port: parseInt(e.target.value) || 587 })
                                }
                                className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200"
                            />
                        </div>
                    </div>

                    <div className="grid grid-cols-3 gap-3">
                        <div className="col-span-2">
                            <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">IMAP Host</label>
                            <input
                                type="text"
                                value={editingAccount.imap_host}
                                onChange={(e) => setEditingAccount({ ...editingAccount, imap_host: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200"
                            />
                        </div>
                        <div>
                            <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">IMAP Port</label>
                            <input
                                type="number"
                                value={editingAccount.imap_port}
                                onChange={(e) =>
                                    setEditingAccount({ ...editingAccount, imap_port: parseInt(e.target.value) || 993 })
                                }
                                className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200"
                            />
                        </div>
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">Encryption Type</label>
                        <select
                            value={editingAccount.encryption_type}
                            onChange={(e) => setEditingAccount({ ...editingAccount, encryption_type: e.target.value })}
                            className="w-full px-3 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200"
                        >
                            <option value="TLS">TLS (Recommended)</option>
                            <option value="STARTTLS">STARTTLS</option>
                            <option value="SSL">SSL</option>
                            <option value="NONE">None / Plain</option>
                        </select>
                    </div>

                    <div className="flex items-center gap-4 pt-2">
                        <label className="flex items-center gap-2 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={editingAccount.is_default}
                                onChange={(e) => setEditingAccount({ ...editingAccount, is_default: e.target.checked })}
                                className="rounded text-blue-600"
                            />
                            <span>Set as Default Sender</span>
                        </label>
                        <label className="flex items-center gap-2 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={editingAccount.is_active}
                                onChange={(e) => setEditingAccount({ ...editingAccount, is_active: e.target.checked })}
                                className="rounded text-blue-600"
                            />
                            <span>Active</span>
                        </label>
                    </div>

                    <div className="flex justify-end gap-2 pt-4 border-t border-gray-100 dark:border-base-200">
                        <button
                            onClick={() => setIsAccountModalOpen(false)}
                            className="px-4 py-2 border border-gray-200 dark:border-base-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={handleSaveAccount}
                            className="px-5 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium shadow-sm"
                        >
                            Save Account
                        </button>
                    </div>
                </div>
            </ModalDialog>

            {/* Import Modal */}
            <ModalDialog
                isOpen={isImportModalOpen}
                onClose={() => setIsImportModalOpen(false)}
                title="Import Email Accounts & Settings"
            >
                <div className="space-y-4 text-xs">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <span className="font-semibold text-gray-700 dark:text-gray-300">Format:</span>
                            <label className="flex items-center gap-1.5 cursor-pointer">
                                <input
                                    type="radio"
                                    name="import_fmt"
                                    value="json"
                                    checked={importFormat === 'json'}
                                    onChange={() => setImportFormat('json')}
                                />
                                <span>JSON</span>
                            </label>
                            <label className="flex items-center gap-1.5 cursor-pointer">
                                <input
                                    type="radio"
                                    name="import_fmt"
                                    value="csv"
                                    checked={importFormat === 'csv'}
                                    onChange={() => setImportFormat('csv')}
                                />
                                <span>CSV</span>
                            </label>
                            <label className="flex items-center gap-1.5 cursor-pointer">
                                <input
                                    type="radio"
                                    name="import_fmt"
                                    value="xlsx"
                                    checked={importFormat === 'xlsx'}
                                    onChange={() => setImportFormat('xlsx')}
                                />
                                <span>Excel (XML / XLSX)</span>
                            </label>
                        </div>

                        <div>
                            <input
                                type="file"
                                ref={fileInputRef}
                                className="hidden"
                                accept=".json,.csv,.xml,.xlsx,.xls"
                                onChange={handleFileUpload}
                            />
                            <button
                                type="button"
                                onClick={() => fileInputRef.current?.click()}
                                className="px-2.5 py-1 text-[11px] font-medium bg-gray-100 dark:bg-base-300 hover:bg-gray-200 dark:hover:bg-base-200 rounded text-gray-700 dark:text-gray-200 flex items-center gap-1.5"
                            >
                                <Upload className="w-3 h-3 text-blue-500" />
                                Browse File
                            </button>
                        </div>
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Paste or Load {importFormat.toUpperCase()} Content
                        </label>
                        <textarea
                            rows={8}
                            value={importPayload}
                            onChange={(e) => setImportPayload(e.target.value)}
                            placeholder={`Paste your ${importFormat.toUpperCase()} here or click Browse File...`}
                            className="w-full px-3 py-2 font-mono text-[11px] border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div className="flex justify-end gap-2 pt-2">
                        <button
                            onClick={() => setIsImportModalOpen(false)}
                            className="px-4 py-2 border border-gray-200 dark:border-base-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={handleImportSubmit}
                            className="px-5 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium shadow-sm"
                        >
                            Execute Import
                        </button>
                    </div>
                </div>
            </ModalDialog>
        </div>
    );
}
