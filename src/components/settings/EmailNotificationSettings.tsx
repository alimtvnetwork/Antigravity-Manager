import React, { useState, useEffect } from 'react';
import {
    Mail,
    Send,
    Server,
    Key,
    RefreshCw,
    Plus,
    Trash2,
    Upload,
    FileSpreadsheet,
    FileText,
    FileJson,
    Sparkles,
    Database,
    ChevronDown,
    SlidersHorizontal,
    Copy,
    CheckCircle2,
    AlertCircle,
    Loader2,
} from 'lucide-react';
import AiSampleTemplatesModal from './ai-sample-templates-modal';
import {
    EmailAccount,
    EmailAccountInput,
    NotifyRecipient,
    EmailNotificationSettings as ISettings,
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
    testDirectEmailConnection,
    exportEmailData,
    importEmailData,
    backupEmailDb,
    restoreEmailDb,
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
    const [isActionsOpen, setIsActionsOpen] = useState(false);
    const [isSampleTemplatesOpen, setIsSampleTemplatesOpen] = useState(false);
    const actionsDropdownRef = React.useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (actionsDropdownRef.current && !actionsDropdownRef.current.contains(event.target as Node)) {
                setIsActionsOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const loadAll = async () => {
        try {
            const [accs, recs, sets] = await Promise.all([
                listEmailAccounts(),
                listNotifyRecipients(),
                getEmailSettings(),
            ]);
            setAccounts(accs);
            setRecipients(recs);
            setSettings(sets);
        } catch (e: any) {
            console.error('Failed to load email settings:', e);
            showToast('Failed to load email configurations', 'error');
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
        } catch (e: any) {
            showToast('Failed to save settings: ' + (e?.message || e), 'error');
        } finally {
            setIsSaving(false);
        }
    };

    const [isTestingDirect, setIsTestingDirect] = useState(false);
    const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

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
        setTestResult(null);
        setIsTestingDirect(false);
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
        setTestResult(null);
        setIsTestingDirect(false);
        setIsAccountModalOpen(true);
    };

    const handleEmailChange = (newEmail: string) => {
        const trimmed = newEmail.trim();
        let updated = { ...editingAccount, email: newEmail };

        if (trimmed.includes('@')) {
            const parts = trimmed.split('@');
            const userPart = parts[0];
            const domain = parts[1]?.toLowerCase();

            if (domain === 'gmail.com' || domain === 'googlemail.com') {
                updated.smtp_host = 'smtp.gmail.com';
                updated.smtp_port = 587;
                updated.imap_host = 'imap.gmail.com';
                updated.imap_port = 993;
                updated.encryption_type = 'TLS';
            } else if (domain && domain.includes('.')) {
                if (domain === 'outlook.com' || domain === 'hotmail.com' || domain === 'live.com' || domain === 'office365.com') {
                    updated.smtp_host = 'smtp.office365.com';
                    updated.smtp_port = 587;
                    updated.imap_host = 'outlook.office365.com';
                    updated.imap_port = 993;
                    updated.encryption_type = 'TLS';
                } else if (domain === 'yahoo.com') {
                    updated.smtp_host = 'smtp.mail.yahoo.com';
                    updated.smtp_port = 465;
                    updated.imap_host = 'imap.mail.yahoo.com';
                    updated.imap_port = 993;
                    updated.encryption_type = 'SSL';
                } else {
                    // Custom Domain auto-discovery heuristics per user requirements:
                    // Outgoing Server: mail.<domain>, Port 465 (SSL)
                    // Incoming Server: mail.<domain>, Port 993 (SSL)
                    updated.smtp_host = `mail.${domain}`;
                    updated.smtp_port = 465;
                    updated.imap_host = `mail.${domain}`;
                    updated.imap_port = 993;
                    updated.encryption_type = 'SSL';
                }

                if (!editingAccount.alias || editingAccount.alias === 'Primary Mailbox') {
                    updated.alias = `${userPart} (${domain})`;
                }
            }
        }

        setEditingAccount(updated);
        if (testResult) {
            setTestResult(null);
        }
    };

    const handleTestDirectConnection = async () => {
        if (!editingAccount.email.trim()) {
            showToast('Email address is required to run connection test', 'error');
            return;
        }
        const isEmailFormatValid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(editingAccount.email.trim());
        if (!isEmailFormatValid) {
            showToast('Please enter a valid email format before testing', 'error');
            return;
        }
        const hasMissingPassword = !editingAccount.password || !editingAccount.password.trim();
        if (!editingAccount.id) {
            if (hasMissingPassword) {
                showToast('Password is required to test mailbox authentication', 'error');
                return;
            }
        }

        setIsTestingDirect(true);
        setTestResult(null);
        try {
            const res = await testDirectEmailConnection(editingAccount);
            setTestResult({ success: true, message: res });
            showToast('Connection verified! Self-test email delivered.', 'success');
        } catch (e: any) {
            const err = e?.message || String(e);
            setTestResult({ success: false, message: err });
            showToast(`Connection Test Failed: ${err}`, 'error');
        } finally {
            setIsTestingDirect(false);
        }
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

    const trimmedEmail = editingAccount.email.trim();
    let emailFormatStatus: 'empty' | 'invalid' | 'valid' = 'empty';
    if (trimmedEmail.length > 0) {
        if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmedEmail)) {
            emailFormatStatus = 'valid';
        } else {
            emailFormatStatus = 'invalid';
        }
    }

    const testResultStatus: 'none' | 'success' | 'failed' = testResult
        ? (testResult.success ? 'success' : 'failed')
        : 'none';

    return (
        <div className="space-y-4">
            {/* Mailboxes Card */}
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
                    <div>
                        <h3 className="text-sm sm:text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Server className="w-4 h-4 text-blue-500" />
                            Email Accounts & Mailbox Pool
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400">
                            Mailbox pool with default sender prioritization and automatic failover swapping.
                        </p>
                    </div>

                    <div className="flex items-center gap-2 self-end sm:self-auto">
                        <button
                            type="button"
                            onClick={() => setIsSampleTemplatesOpen(true)}
                            className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-purple-200 dark:border-purple-900/50 bg-purple-50/50 dark:bg-purple-950/20 text-purple-700 dark:text-purple-300 hover:bg-purple-100 dark:hover:bg-purple-900/40 transition-all flex items-center gap-1.5 shadow-xs cursor-pointer"
                            title="AI Sample Instructions & Template Formats"
                        >
                            <Sparkles className="w-3.5 h-3.5 text-purple-600 dark:text-purple-400" />
                            <span>AI Templates</span>
                        </button>

                        <div className="relative" ref={actionsDropdownRef}>
                            <button
                                type="button"
                                onClick={() => setIsActionsOpen(!isActionsOpen)}
                                className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-800 text-gray-800 dark:text-slate-100 hover:bg-gray-50 dark:hover:bg-slate-700 hover:border-gray-400 dark:hover:border-slate-600 transition-all flex items-center gap-1.5 shadow-xs cursor-pointer"
                                title="Mailbox & Vault Actions"
                            >
                                <SlidersHorizontal className="w-3.5 h-3.5 text-gray-600 dark:text-slate-300" />
                                <span>Actions</span>
                                <ChevronDown className={`w-3.5 h-3.5 text-gray-500 dark:text-slate-400 transition-transform duration-150 ${isActionsOpen ? 'rotate-180' : ''}`} />
                            </button>

                            {isActionsOpen && (
                                <div className="absolute right-0 mt-1.5 w-48 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-lg z-50 py-1 divide-y divide-gray-100 dark:divide-slate-800 animate-in fade-in slide-in-from-top-1">
                                    <div className="py-1">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('json');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <FileJson className="w-3.5 h-3.5 text-amber-500" />
                                            <span>Export JSON</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('csv');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <FileText className="w-3.5 h-3.5 text-blue-500" />
                                            <span>Export CSV</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('xlsx');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <FileSpreadsheet className="w-3.5 h-3.5 text-emerald-500" />
                                            <span>Export Excel</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                setIsImportModalOpen(true);
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <Upload className="w-3.5 h-3.5 text-indigo-500" />
                                            <span>Import Accounts</span>
                                        </button>
                                    </div>
                                    <div className="py-1">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleBackupDb();
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <Database className="w-3.5 h-3.5 text-purple-500" />
                                            <span>Backup Vault DB</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleRestoreDb();
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                        >
                                            <RefreshCw className="w-3.5 h-3.5 text-cyan-500" />
                                            <span>Restore Vault DB</span>
                                        </button>
                                    </div>
                                </div>
                            )}
                        </div>

                        <button
                            type="button"
                            onClick={handleOpenAddAccount}
                            className="px-3 py-1.5 text-xs font-semibold bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white rounded-lg transition-all flex items-center gap-1 shadow-sm hover:shadow hover:scale-[1.02] cursor-pointer"
                            title="Add New Mailbox Account"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            <span>Add Mailbox</span>
                        </button>
                    </div>
                </div>

                {accounts.length === 0 ? (
                    <div className="py-6 px-4 text-center border border-dashed border-gray-200 dark:border-slate-800 rounded-xl flex flex-col items-center justify-center bg-gray-50/50 dark:bg-slate-900/40">
                        <Mail className="w-6 h-6 text-gray-400 dark:text-slate-500 mx-auto mb-1.5" />
                        <p className="text-xs font-medium text-gray-600 dark:text-slate-300">No mailboxes configured in vault</p>
                        <p className="text-[11px] text-gray-400 dark:text-slate-500 mt-0.5 mb-3">Add an SMTP/IMAP account to enable dispatch and remote command execution</p>
                        <button
                            type="button"
                            onClick={handleOpenAddAccount}
                            className="px-3.5 py-1.5 text-xs font-semibold bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center gap-1.5 shadow-sm cursor-pointer"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            <span>Add Mailbox</span>
                        </button>
                    </div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full text-left text-xs border-collapse">
                            <thead>
                                <tr className="border-b border-gray-200 dark:border-slate-800 text-gray-500 dark:text-slate-400">
                                    <th className="py-2 px-2.5 font-semibold">Alias & Email</th>
                                    <th className="py-2 px-2.5 font-semibold">SMTP Host</th>
                                    <th className="py-2 px-2.5 font-semibold">IMAP Host</th>
                                    <th className="py-2 px-2.5 font-semibold">Enc</th>
                                    <th className="py-2 px-2.5 font-semibold">Role</th>
                                    <th className="py-2 px-2.5 font-semibold text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-gray-100 dark:divide-slate-800">
                                {accounts.map((acc) => (
                                    <tr key={acc.id} className="hover:bg-gray-100/70 dark:hover:bg-slate-800/80 transition-colors group">
                                        <td className="py-2 px-2.5">
                                            <div className="font-semibold text-gray-800 dark:text-slate-100 group-hover:text-blue-600 dark:group-hover:text-white leading-tight transition-colors">{acc.alias}</div>
                                            <div className="text-[11px] text-gray-500 dark:text-slate-400 group-hover:text-gray-700 dark:group-hover:text-slate-200 leading-tight transition-colors">{acc.email}</div>
                                        </td>
                                        <td className="py-2 px-2.5 font-mono text-gray-600 dark:text-slate-300 group-hover:text-gray-900 dark:group-hover:text-slate-100 transition-colors">
                                            {acc.smtp_host}:{acc.smtp_port}
                                        </td>
                                        <td className="py-2 px-2.5 font-mono text-gray-600 dark:text-slate-300 group-hover:text-gray-900 dark:group-hover:text-slate-100 transition-colors">
                                            {acc.imap_host}:{acc.imap_port}
                                        </td>
                                        <td className="py-2 px-2.5">
                                            <span className="px-1.5 py-0.5 rounded bg-gray-100 dark:bg-slate-800 border border-gray-200/80 dark:border-slate-700 text-gray-700 dark:text-slate-300 text-[10px] font-mono">
                                                {acc.encryption_type}
                                            </span>
                                        </td>
                                        <td className="py-2 px-2.5">
                                            {acc.is_default ? (
                                                <span className="px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-950/60 text-blue-700 dark:text-blue-300 border border-blue-200/60 dark:border-blue-800/50 text-[10px] font-semibold">
                                                    Default Sender
                                                </span>
                                            ) : (
                                                <button
                                                    onClick={() => handleSetDefault(acc.id)}
                                                    className="text-[10px] text-gray-400 dark:text-slate-400 hover:text-blue-600 dark:hover:text-blue-400 underline cursor-pointer"
                                                >
                                                    Set Default
                                                </button>
                                            )}
                                        </td>
                                        <td className="py-2 px-2.5 text-right space-x-1">
                                            <button
                                                onClick={() => handleTestSmtp(acc.id)}
                                                disabled={testingAccountId === acc.id}
                                                className="px-2 py-0.5 rounded bg-sky-50 dark:bg-sky-950/40 text-sky-600 dark:text-sky-400 hover:bg-sky-100 dark:hover:bg-sky-900/50 border border-sky-200/60 dark:border-sky-800/50 text-[10px] font-medium cursor-pointer transition-colors"
                                                title="Test Outbound SMTP"
                                            >
                                                Test SMTP
                                            </button>
                                            <button
                                                onClick={() => handleTestImap(acc.id)}
                                                disabled={testingAccountId === acc.id}
                                                className="px-2 py-0.5 rounded bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-100 dark:hover:bg-indigo-900/50 border border-indigo-200/60 dark:border-indigo-800/50 text-[10px] font-medium cursor-pointer transition-colors"
                                                title="Test Inbound IMAP"
                                            >
                                                Test IMAP
                                            </button>
                                            <button
                                                onClick={() => handleOpenEditAccount(acc)}
                                                className="px-2 py-0.5 rounded hover:bg-gray-100 dark:hover:bg-slate-700 text-gray-600 dark:text-slate-300 text-[10px] cursor-pointer transition-colors"
                                            >
                                                Edit
                                            </button>
                                            <button
                                                onClick={() => handleDeleteAccount(acc.id)}
                                                className="px-1.5 py-0.5 rounded hover:bg-red-50 dark:hover:bg-red-950/40 text-red-600 dark:text-red-400 text-[10px] cursor-pointer transition-colors"
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
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800">
                <h3 className="text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2 mb-1">
                    <Send className="w-4 h-4 text-emerald-500" />
                    Notification Recipients
                </h3>
                <p className="text-xs text-gray-500 dark:text-slate-400 mb-4">
                    External user email addresses and distribution groups that receive sensor alerts and telemetry.
                </p>

                <div className="flex flex-col sm:flex-row gap-2 mb-4">
                    <input
                        type="email"
                        placeholder="Recipient Email (e.g. user@domain.com)"
                        value={newRecipientEmail}
                        onChange={(e) => setNewRecipientEmail(e.target.value)}
                        className="flex-1 px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <input
                        type="text"
                        placeholder="Group (e.g. dev, ops, default)"
                        value={newRecipientGroup}
                        onChange={(e) => setNewRecipientGroup(e.target.value)}
                        className="w-36 px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <button
                        onClick={handleAddRecipient}
                        className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-medium transition-colors shadow-sm cursor-pointer"
                    >
                        Add Recipient
                    </button>
                </div>

                <div className="flex flex-wrap gap-2">
                    {recipients.map((rec) => (
                        <div
                            key={rec.id}
                            className="bg-gray-50 dark:bg-slate-800/80 border border-gray-200 dark:border-slate-700 rounded-lg px-3 py-1.5 flex items-center gap-2 text-xs"
                        >
                            <span className="font-medium text-gray-700 dark:text-slate-200">{rec.email}</span>
                            <span className="text-[10px] px-1.5 py-0.5 bg-gray-200 dark:bg-slate-700 text-gray-600 dark:text-slate-300 rounded font-mono">
                                {rec.group_name}
                            </span>
                            <button
                                onClick={() => handleDeleteRecipient(rec.id)}
                                className="text-gray-400 hover:text-red-500 cursor-pointer"
                            >
                                <Trash2 className="w-3 h-3" />
                            </button>
                        </div>
                    ))}
                    {recipients.length === 0 && (
                        <span className="text-xs text-gray-400 dark:text-slate-500 italic">No recipients registered. Alerts will be skipped.</span>
                    )}
                </div>
            </div>

            {/* Watcher Daemon & Sensors Card */}
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800 space-y-4">
                <div className="flex items-center justify-between">
                    <div>
                        <h3 className="text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Sparkles className="w-4 h-4 text-purple-500" />
                            Background Watcher Daemon & Multi-Trigger Sensors
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400">
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
                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-slate-200 block">
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
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.polling_interval_minutes} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            Interval for telemetry collection and quota/idle workspace sensor sampling.
                        </p>
                    </div>

                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-slate-200 block">
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
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.inbox_check_interval_minutes} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            Frequency of reading the last 5 unread instructions via IMAP.
                        </p>
                    </div>
                </div>

                {/* Sensor Toggles */}
                <div className="space-y-3 border-t border-gray-100 dark:border-slate-800 pt-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-gray-400 dark:text-slate-400">Sensor Notification Triggers</h4>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_quota_drop"
                                checked={settings.notify_on_quota_drop}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_quota_drop: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_quota_drop" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Quota Drop Alert</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Trigger when credit &lt; 15%</div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_ws_switch"
                                checked={settings.notify_on_workspace_switch}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_workspace_switch: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_ws_switch" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Workspace Switch Notice</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Email notice before auto-rotation</div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_idle_ws"
                                checked={settings.notify_on_idle_workspace}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_idle_workspace: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_idle_ws" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Idle Workspace Alert</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Ask for prompt when queue is empty</div>
                            </label>
                        </div>
                    </div>
                </div>

                {/* Remote Execution Toggles */}
                <div className="space-y-3 border-t border-gray-100 dark:border-slate-800 pt-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-gray-400 dark:text-slate-400">Inbound Mailbox Command Capabilities</h4>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_allow_prompt"
                                checked={settings.allow_remote_prompt_execution}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_prompt_execution: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_prompt" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Prompt Injection</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Match <code>Project: &lt;name&gt;</code></div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_allow_cli"
                                checked={settings.allow_remote_cli_execution}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_cli_execution: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_cli" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">CLI Execution</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Match <code>exec: &lt;ip&gt;</code></div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_allow_instance"
                                checked={settings.allow_remote_instance_rotation}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_instance_rotation: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_instance" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Instance Launch / Rotate</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Match <code>instance: new</code></div>
                            </label>
                        </div>
                    </div>
                </div>

                <div className="flex items-center justify-between pt-4 border-t border-gray-100 dark:border-slate-800">
                    <button
                        onClick={handleTriggerManualCheck}
                        className="px-4 py-2 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-200 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer"
                    >
                        <Send className="w-3.5 h-3.5 text-blue-500" />
                        Dispatch Test Cheat Sheet
                    </button>

                    <button
                        onClick={handleSaveSettings}
                        disabled={isSaving}
                        className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm cursor-pointer"
                    >
                        {isSaving ? 'Saving Settings...' : 'Save Watcher Settings'}
                    </button>
                </div>
            </div>

            {/* Account Add/Edit Modal */}
            <ModalDialog
                isOpen={isAccountModalOpen}
                title={editingAccount.id ? 'Edit Mailbox Configuration' : 'Add Mailbox to Secure Split Vault'}
                type="confirm"
                maxWidth="max-w-lg"
                confirmText={editingAccount.id ? 'Save Mailbox' : 'Add Mailbox to Vault'}
                cancelText="Cancel"
                onConfirm={handleSaveAccount}
                onCancel={() => setIsAccountModalOpen(false)}
            >
                <div className="space-y-3.5 text-xs">
                    {/* One-Click AI Instructions Copy */}
                    <div className="flex items-center justify-between p-2.5 rounded-lg bg-blue-50/70 dark:bg-blue-950/30 border border-blue-100 dark:border-blue-900/40">
                        <div className="flex items-center gap-1.5 text-blue-700 dark:text-blue-300">
                            <Sparkles className="w-3.5 h-3.5 shrink-0 text-blue-600 dark:text-blue-400" />
                            <span className="font-medium text-[11px]">AI Automated Mailbox Configuration</span>
                        </div>
                        <button
                            type="button"
                            onClick={() => {
                                const instructions = `Configure Mailbox in Antigravity Manager:
- Alias: Primary Mailbox
- Email: your-email@gmail.com
- App Password: <generated-app-password>
- SMTP: smtp.gmail.com (Port: 587, TLS)
- IMAP: imap.gmail.com (Port: 993, TLS)`;
                                navigator.clipboard.writeText(instructions);
                                showToast('AI instructions copied to clipboard', 'success');
                            }}
                            className="px-2.5 py-1 text-[11px] font-semibold bg-white dark:bg-slate-800 hover:bg-blue-50 dark:hover:bg-blue-900/40 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-800 rounded-md flex items-center gap-1.5 transition-all shadow-xs cursor-pointer"
                            title="Copy AI Instructions to Clipboard"
                        >
                            <Copy className="w-3 h-3" />
                            <span>Copy AI Instructions</span>
                        </button>
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Email Address</label>
                        <input
                            type="email"
                            placeholder="e.g. ai-agm-tool-v2@hire-seoexperts.com"
                            value={editingAccount.email}
                            onChange={(e) => handleEmailChange(e.target.value)}
                            className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                        {emailFormatStatus === 'valid' && (
                            <div className="flex items-center gap-1 text-[11px] text-emerald-600 dark:text-emerald-400 mt-1">
                                <CheckCircle2 className="w-3.5 h-3.5 shrink-0" />
                                <span>Valid email detected. Host and port settings auto-configured.</span>
                            </div>
                        )}
                        {emailFormatStatus === 'invalid' && (
                            <div className="flex items-center gap-1 text-[11px] text-amber-600 dark:text-amber-400 mt-1">
                                <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                                <span>Please enter a complete email address (e.g. user@domain.com)</span>
                            </div>
                        )}
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Account Alias</label>
                        <input
                            type="text"
                            placeholder="e.g. Primary Gmail, Alerts Mailer"
                            value={editingAccount.alias}
                            onChange={(e) => setEditingAccount({ ...editingAccount, alias: e.target.value })}
                            className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">
                            Password / App Password
                            {editingAccount.id && <span className="text-gray-400 dark:text-slate-500 ml-1 font-normal">(Leave blank to keep existing)</span>}
                        </label>
                        <div className="relative">
                            <input
                                type="password"
                                placeholder={editingAccount.id ? '••••••••' : 'Application password / secret'}
                                value={editingAccount.password || ''}
                                onChange={(e) => setEditingAccount({ ...editingAccount, password: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                            <Key className="w-3.5 h-3.5 text-gray-400 absolute right-3 top-2.5" />
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400 mt-1">
                            Stored in isolated split database <code>email_passwords.db</code> with salted SSH RSA identity and machine-bound encryption.
                        </p>
                    </div>

                    <div className="grid grid-cols-3 gap-3">
                        <div className="col-span-2">
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Outgoing Server (SMTP Host)</label>
                            <input
                                type="text"
                                value={editingAccount.smtp_host}
                                onChange={(e) => setEditingAccount({ ...editingAccount, smtp_host: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                            />
                        </div>
                        <div>
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">SMTP Port</label>
                            <input
                                type="number"
                                value={editingAccount.smtp_port}
                                onChange={(e) =>
                                    setEditingAccount({ ...editingAccount, smtp_port: parseInt(e.target.value) || 465 })
                                }
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                            />
                        </div>
                    </div>
                    {/* SMTP Port Preset Pills */}
                    <div className="flex items-center gap-1.5 -mt-1.5">
                        <span className="text-[10px] text-gray-400 dark:text-slate-400">Presets:</span>
                        <button
                            type="button"
                            onClick={() => setEditingAccount({ ...editingAccount, smtp_port: 465, encryption_type: 'SSL' })}
                            className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.smtp_port === 465 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                        >
                            465 (SSL)
                        </button>
                        <button
                            type="button"
                            onClick={() => setEditingAccount({ ...editingAccount, smtp_port: 587, encryption_type: 'TLS' })}
                            className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.smtp_port === 587 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                        >
                            587 (TLS)
                        </button>
                        <button
                            type="button"
                            onClick={() => setEditingAccount({ ...editingAccount, smtp_port: 25, encryption_type: 'NONE' })}
                            className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.smtp_port === 25 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                        >
                            25 (Plain)
                        </button>
                    </div>

                    <div className="grid grid-cols-3 gap-3">
                        <div className="col-span-2">
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Incoming Server (IMAP Host)</label>
                            <input
                                type="text"
                                value={editingAccount.imap_host}
                                onChange={(e) => setEditingAccount({ ...editingAccount, imap_host: e.target.value })}
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                            />
                        </div>
                        <div>
                            <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">IMAP Port</label>
                            <input
                                type="number"
                                value={editingAccount.imap_port}
                                onChange={(e) =>
                                    setEditingAccount({ ...editingAccount, imap_port: parseInt(e.target.value) || 993 })
                                }
                                className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                            />
                        </div>
                    </div>
                    {/* IMAP Port Preset Pills */}
                    <div className="flex items-center gap-1.5 -mt-1.5">
                        <span className="text-[10px] text-gray-400 dark:text-slate-400">Presets:</span>
                        <button
                            type="button"
                            onClick={() => setEditingAccount({ ...editingAccount, imap_port: 993, encryption_type: 'SSL' })}
                            className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.imap_port === 993 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                        >
                            993 (IMAP SSL)
                        </button>
                        <button
                            type="button"
                            onClick={() => setEditingAccount({ ...editingAccount, imap_port: 143, encryption_type: 'TLS' })}
                            className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.imap_port === 143 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                        >
                            143 (IMAP)
                        </button>
                        <button
                            type="button"
                            onClick={() => setEditingAccount({ ...editingAccount, imap_port: 995, encryption_type: 'SSL' })}
                            className={`px-1.5 py-0.5 text-[10px] font-medium rounded border transition-colors cursor-pointer ${editingAccount.imap_port === 995 ? 'bg-blue-50 border-blue-300 text-blue-700 dark:bg-blue-950/50 dark:border-blue-700 dark:text-blue-300' : 'bg-gray-50 border-gray-200 text-gray-600 dark:bg-slate-800 dark:border-slate-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-700'}`}
                        >
                            995 (POP3)
                        </button>
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">Encryption Type</label>
                        <select
                            value={editingAccount.encryption_type}
                            onChange={(e) => setEditingAccount({ ...editingAccount, encryption_type: e.target.value })}
                            className="w-full px-3 py-2 border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100"
                        >
                            <option value="SSL">SSL / TLS (Recommended for Custom Domain)</option>
                            <option value="TLS">TLS (Recommended for Gmail / Outlook)</option>
                            <option value="STARTTLS">STARTTLS</option>
                            <option value="NONE">None / Plain</option>
                        </select>
                    </div>

                    <div className="flex items-center gap-4 pt-1">
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

                    {/* Dedicated Test Section */}
                    <div className="p-3 rounded-xl border border-blue-100 dark:border-blue-900/40 bg-blue-50/40 dark:bg-blue-950/20 space-y-2 mt-2">
                        <div className="flex items-center justify-between gap-2">
                            <div>
                                <div className="font-semibold text-xs text-gray-800 dark:text-gray-200 flex items-center gap-1.5">
                                    <Send className="w-3.5 h-3.5 text-blue-500" />
                                    <span>Test Connection & Self-Test Email</span>
                                </div>
                                <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                    Sends a self-test email to verify outgoing SMTP & auth before saving.
                                </p>
                            </div>
                            <button
                                type="button"
                                onClick={handleTestDirectConnection}
                                disabled={isTestingDirect}
                                className="px-3 py-1.5 text-xs font-semibold rounded-lg bg-blue-600 hover:bg-blue-700 text-white flex items-center gap-1.5 shadow-sm transition-all disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer shrink-0"
                            >
                                {isTestingDirect ? (
                                    <>
                                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                        <span>Testing...</span>
                                    </>
                                ) : (
                                    <>
                                        <RefreshCw className="w-3.5 h-3.5" />
                                        <span>Send Test Email</span>
                                    </>
                                )}
                            </button>
                        </div>

                        {testResultStatus === 'success' && (
                            <div className="p-2.5 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800/60 flex items-start gap-2 text-emerald-800 dark:text-emerald-200 animate-in fade-in">
                                <span className="relative flex h-3 w-3 mt-0.5 shrink-0">
                                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
                                </span>
                                <div className="flex-1 text-[11px]">
                                    <div className="font-bold flex items-center gap-1">
                                        <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                                        <span>Connection & Delivery Successful (Green Signal)</span>
                                    </div>
                                    <div className="text-emerald-700 dark:text-emerald-300 mt-0.5">
                                        {testResult?.message}
                                    </div>
                                </div>
                            </div>
                        )}

                        {testResultStatus === 'failed' && (
                            <div className="p-2.5 rounded-lg bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-800/60 flex items-start gap-2 text-rose-800 dark:text-rose-200 animate-in fade-in">
                                <AlertCircle className="w-3.5 h-3.5 text-rose-600 dark:text-rose-400 mt-0.5 shrink-0" />
                                <div className="flex-1 text-[11px]">
                                    <div className="font-bold">Connection Verification Failed</div>
                                    <div className="text-rose-700 dark:text-rose-300 mt-0.5 break-all">
                                        {testResult?.message}
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </ModalDialog>

            {/* Import Modal */}
            <ModalDialog
                isOpen={isImportModalOpen}
                title="Import Email Accounts & Settings"
                type="confirm"
                confirmText="Execute Import"
                cancelText="Cancel"
                onConfirm={handleImportSubmit}
                onCancel={() => setIsImportModalOpen(false)}
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
                                className="px-2.5 py-1 text-[11px] font-medium bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 rounded text-gray-700 dark:text-slate-200 border border-gray-200 dark:border-slate-700 flex items-center gap-1.5 transition-colors cursor-pointer"
                            >
                                <Upload className="w-3 h-3 text-blue-500" />
                                Browse File
                            </button>
                        </div>
                    </div>

                    <div>
                        <label className="block font-medium text-gray-700 dark:text-slate-300 mb-1">
                            Paste or Load {importFormat.toUpperCase()} Content
                        </label>
                        <textarea
                            rows={8}
                            value={importPayload}
                            onChange={(e) => setImportPayload(e.target.value)}
                            placeholder={`Paste your ${importFormat.toUpperCase()} here or click Browse File...`}
                            className="w-full px-3 py-2 font-mono text-[11px] border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                </div>
            </ModalDialog>

            {/* AI Sample Instructions & Template Modal */}
            <AiSampleTemplatesModal
                isOpen={isSampleTemplatesOpen}
                onClose={() => setIsSampleTemplatesOpen(false)}
            />
        </div>
    );
}
