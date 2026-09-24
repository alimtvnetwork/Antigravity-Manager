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
    Zap,
    Terminal,
    Play,
    Eye,
    EyeOff,
    MessageSquare,
    FileCode,
} from 'lucide-react';
import AiSampleTemplatesModal from './ai-sample-templates-modal';
import MailboxExportModal from './MailboxExportModal';
import {
    downloadOrSaveFile,
    parseAccountsFromText,
} from '../../utils/emailFormatters';
import {
    telegramService,
    TelegramConfig,
    TelegramWatcherStatus,
} from '../../services/telegramService';
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
    dispatchEmailTestPing,
    dispatchCustomEmailTask,
    testExecuteCliCommand,
    CliExecResult,
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
        baseline_polling_interval_minutes: 5,
        active_awaiting_interval_seconds: 10,
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
    const [isPinging, setIsPinging] = useState(false);

    // Developer Task Quick Dispatch state
    const [developerTaskType, setDeveloperTaskType] = useState<string>('prompt');
    const [developerTargetRecipient, setDeveloperTargetRecipient] = useState<string>('');
    const [developerTaskPayload, setDeveloperTaskPayload] = useState<string>(
        'Project: Antigravity-Manager\nScan repository health and report open issues'
    );
    const [developerCustomSubject, setDeveloperCustomSubject] = useState<string>('');
    const [isDispatchingTask, setIsDispatchingTask] = useState<boolean>(false);

    // Telegram Bot Integration State
    const [telegramConfig, setTelegramConfig] = useState<TelegramConfig | null>(null);
    const [telegramStatus, setTelegramStatus] = useState<TelegramWatcherStatus | null>(null);
    const [isTestingTelegram, setIsTestingTelegram] = useState<boolean>(false);
    const [isSavingTelegram, setIsSavingTelegram] = useState<boolean>(false);
    const [isSendingTelegramPing, setIsSendingTelegramPing] = useState<boolean>(false);
    const [telegramBotUsername, setTelegramBotUsername] = useState<string | null>(null);
    const [showBotToken, setShowBotToken] = useState<boolean>(false);

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
    const [importFormat, setImportFormat] = useState<'json' | 'yaml' | 'csv' | 'xlsx'>('json');
    const [importPayload, setImportPayload] = useState('');
    const [testingAccountId, setTestingAccountId] = useState<string | null>(null);
    const fileInputRef = React.useRef<HTMLInputElement>(null);
    const [isActionsOpen, setIsActionsOpen] = useState(false);
    const [isSampleTemplatesOpen, setIsSampleTemplatesOpen] = useState(false);
    const actionsDropdownRef = React.useRef<HTMLDivElement>(null);

    // Export preview modal state (all accounts or single account)
    const [exportModalState, setExportModalState] = useState<{
        isOpen: boolean;
        singleAccount?: Partial<EmailAccount> | null;
        allAccounts?: EmailAccount[];
        initialFormat?: 'json' | 'yaml' | 'csv';
    }>({ isOpen: false });

    // Single account modal quick import/export state
    const [isModalQuickImportOpen, setIsModalQuickImportOpen] = useState(false);
    const [modalQuickImportText, setModalQuickImportText] = useState('');
    const [showAiJsonSyntax, setShowAiJsonSyntax] = useState(false);
    const singleAccountFileInputRef = React.useRef<HTMLInputElement>(null);

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
            const [accs, recs, sets, tgConf, tgStat] = await Promise.all([
                listEmailAccounts(),
                listNotifyRecipients(),
                getEmailSettings(),
                telegramService.getConfig().catch(() => null),
                telegramService.getStatus().catch(() => null),
            ]);
            setAccounts(accs);
            setRecipients(recs);
            setSettings(sets);
            if (tgConf) setTelegramConfig(tgConf);
            if (tgStat) setTelegramStatus(tgStat);
        } catch (e: any) {
            console.error('Failed to load email settings:', e);
            showToast('Failed to load email configurations', 'error');
        }
    };

    useEffect(() => {
        loadAll();
    }, []);

    const handleSaveTelegram = async () => {
        if (!telegramConfig) return;
        setIsSavingTelegram(true);
        try {
            await telegramService.saveConfig(telegramConfig);
            showToast('Telegram bot settings saved successfully', 'success');
            const st = await telegramService.getStatus();
            setTelegramStatus(st);
        } catch (e: any) {
            showToast(`Failed to save Telegram settings: ${e?.message || e}`, 'error');
        } finally {
            setIsSavingTelegram(false);
        }
    };

    const handleTestTelegram = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim()) {
            showToast('Please enter a Telegram Bot Token', 'warning');
            return;
        }
        setIsTestingTelegram(true);
        try {
            const username = await telegramService.testBot(telegramConfig.bot_token);
            setTelegramBotUsername(username);
            showToast(`Connected to Telegram bot: @${username}`, 'success');
        } catch (e: any) {
            setTelegramBotUsername(null);
            showToast(`Telegram connection failed: ${e?.message || e}`, 'error');
        } finally {
            setIsTestingTelegram(false);
        }
    };

    const handleSendTelegramPing = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim() || !telegramConfig.allowed_chat_id) {
            showToast('Please specify Bot Token and Allowed Chat ID first', 'warning');
            return;
        }
        setIsSendingTelegramPing(true);
        try {
            await telegramService.sendTestMessage(telegramConfig.bot_token, telegramConfig.allowed_chat_id);
            showToast('Test alert sent to your Telegram chat!', 'success');
        } catch (e: any) {
            showToast(`Failed to send test alert: ${e?.message || e}`, 'error');
        } finally {
            setIsSendingTelegramPing(false);
        }
    };

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
        setIsModalQuickImportOpen(false);
        setModalQuickImportText('');
        setShowAiJsonSyntax(false);
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
        setIsModalQuickImportOpen(false);
        setModalQuickImportText('');
        setShowAiJsonSyntax(false);
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

    const handleExport = (format: 'json' | 'yaml' | 'csv' | 'xlsx') => {
        if (format === 'xlsx') {
            exportEmailData('xlsx')
                .then((content) =>
                    downloadOrSaveFile(`antigravity-mailboxes-${Date.now()}.xls`, content, 'xlsx')
                )
                .catch((e) => showToast('Export failed: ' + (e?.message || e), 'error'));
            return;
        }
        setExportModalState({
            isOpen: true,
            allAccounts: accounts,
            initialFormat: format,
        });
    };

    const handleExportSingleAccount = (
        account: Partial<EmailAccount>,
        initialFormat: 'json' | 'yaml' | 'csv' = 'json'
    ) => {
        setExportModalState({
            isOpen: true,
            singleAccount: account,
            initialFormat,
        });
    };

    const handleQuickImportSingle = (text: string) => {
        if (!text.trim()) {
            showToast('Please paste JSON, YAML, or CSV content first', 'warning');
            return;
        }
        const parsed = parseAccountsFromText(text);
        if (parsed.error || parsed.accounts.length === 0) {
            showToast(parsed.error || 'Failed to parse mailbox data', 'error');
            return;
        }
        const target = parsed.accounts[0];
        setEditingAccount((prev) => ({
            ...prev,
            alias: target.alias || prev.alias,
            email: target.email || prev.email,
            password: target.password || prev.password,
            smtp_host: target.smtp_host || prev.smtp_host,
            smtp_port: target.smtp_port || prev.smtp_port,
            imap_host: target.imap_host || prev.imap_host,
            imap_port: target.imap_port || prev.imap_port,
            encryption_type: target.encryption_type || prev.encryption_type,
            is_default: target.is_default !== undefined ? target.is_default : prev.is_default,
            is_active: target.is_active !== undefined ? target.is_active : prev.is_active,
        }));
        showToast(`Loaded mailbox details from ${parsed.format.toUpperCase()}`, 'success');
        setIsModalQuickImportOpen(false);
        setModalQuickImportText('');
    };

    const handleSingleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = (event) => {
            const text = event.target?.result as string;
            if (text) {
                handleQuickImportSingle(text);
            }
        };
        reader.readAsText(file);
        e.target.value = '';
    };

    const handleImportSubmit = async () => {
        if (!importPayload.trim()) {
            showToast('Payload cannot be empty', 'error');
            return;
        }
        try {
            if (importFormat === 'yaml') {
                const parsed = parseAccountsFromText(importPayload);
                if (parsed.error || parsed.accounts.length === 0) {
                    showToast(parsed.error || 'No valid YAML accounts found', 'error');
                    return;
                }
                let count = 0;
                for (const acc of parsed.accounts) {
                    await addEmailAccount(acc);
                    count++;
                }
                showToast(`Imported ${count} mailboxes from YAML`, 'success');
                setIsImportModalOpen(false);
                setImportPayload('');
                await loadAll();
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
            } catch (err: any) {
                // Fallback to parseAccountsFromText for array/single JSON or CSV
                const parsed = parseAccountsFromText(importPayload);
                if (parsed.accounts.length > 0) {
                    let count = 0;
                    for (const acc of parsed.accounts) {
                        await addEmailAccount(acc);
                        count++;
                    }
                    showToast(`Imported ${count} mailboxes successfully`, 'success');
                    setIsImportModalOpen(false);
                    setImportPayload('');
                    await loadAll();
                } else {
                    throw err;
                }
            }
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
        } else if (lowerName.endsWith('.yaml') || lowerName.endsWith('.yml')) {
            setImportFormat('yaml');
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

    const handleDispatchPingTest = async () => {
        setIsPinging(true);
        try {
            const res = await dispatchEmailTestPing();
            showToast(res, 'success');
        } catch (e: any) {
            showToast('Ping test failed: ' + (e?.message || e), 'error');
        } finally {
            setIsPinging(false);
        }
    };

    const [testCliCommand, setTestCliCommand] = useState<string>('powershell: Get-Process | Select-Object -First 5');
    const [isExecutingCli, setIsExecutingCli] = useState<boolean>(false);
    const [cliExecResult, setCliExecResult] = useState<CliExecResult | null>(null);

    const handleTestExecuteCli = async () => {
        if (!testCliCommand.trim()) {
            showToast('Please enter a command to test', 'warning');
            return;
        }
        setIsExecutingCli(true);
        try {
            const res = await testExecuteCliCommand(testCliCommand.trim());
            setCliExecResult(res);
            if (res.success) {
                showToast(`Command executed successfully (Node: ${res.machine_name})`, 'success');
            } else {
                showToast(`Command exited with code ${res.exit_code}`, 'error');
            }
        } catch (e: any) {
            showToast('CLI execution failed: ' + (e?.message || e), 'error');
        } finally {
            setIsExecutingCli(false);
        }
    };

    const handleTaskTypeChange = (type: string) => {
        setDeveloperTaskType(type);
        if (type === 'prompt') {
            setDeveloperTaskPayload('Project: Antigravity-Manager\nScan repository health and report open issues');
        } else if (type === 'powershell') {
            setDeveloperTaskPayload('Get-Process | Select-Object -First 10');
        } else if (type === 'cmd') {
            setDeveloperTaskPayload('dir /b & whoami');
        } else if (type === 'gitmap') {
            setDeveloperTaskPayload('gitmap status --json');
        } else if (type === 'status') {
            setDeveloperTaskPayload('telemetry: report quota and running profile status');
        }
    };

    const handleDispatchDeveloperTask = async () => {
        if (!developerTaskPayload.trim()) {
            showToast('Please enter a task payload or command', 'warning');
            return;
        }
        setIsDispatchingTask(true);
        try {
            const res = await dispatchCustomEmailTask(
                developerTaskType,
                developerTaskPayload.trim(),
                developerTargetRecipient || undefined,
                developerCustomSubject.trim() || undefined
            );
            showToast(res, 'success');
        } catch (e: any) {
            showToast('Dispatch failed: ' + (e?.message || e), 'error');
        } finally {
            setIsDispatchingTask(false);
        }
    };

    const handleSaveRecipientsIntervals = async () => {
        try {
            await saveEmailSettings(settings);
            showToast('Monitoring and retry intervals saved successfully', 'success');
        } catch (e: any) {
            showToast('Failed to save intervals: ' + (e?.message || e), 'error');
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
                                title="Import / Export Mailboxes & Database Actions"
                            >
                                <SlidersHorizontal className="w-3.5 h-3.5 text-gray-600 dark:text-slate-300" />
                                <span>Export / Import Actions</span>
                                <ChevronDown className={`w-3.5 h-3.5 text-gray-500 dark:text-slate-400 transition-transform duration-150 ${isActionsOpen ? 'rotate-180' : ''}`} />
                            </button>

                            {isActionsOpen && (
                                <div className="absolute right-0 mt-1.5 w-52 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-lg z-50 py-1 divide-y divide-gray-100 dark:divide-slate-800 animate-in fade-in slide-in-from-top-1">
                                    <div className="py-1">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('json');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Export all mailboxes as structured JSON"
                                        >
                                            <FileJson className="w-3.5 h-3.5 text-amber-500" />
                                            <span>Export JSON</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('yaml');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Export all mailboxes as clean YAML"
                                        >
                                            <FileCode className="w-3.5 h-3.5 text-emerald-500" />
                                            <span>Export YAML</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsActionsOpen(false);
                                                handleExport('csv');
                                            }}
                                            className="w-full px-3 py-1.5 text-xs text-left text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                            title="Export all mailboxes in CSV format"
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
                                            title="Export all mailboxes to Microsoft Excel format"
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
                                            title="Import mailboxes from JSON, YAML, or CSV files"
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
                                    <th className="py-2 px-2.5 font-semibold text-right whitespace-nowrap min-w-[290px]">Actions</th>
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
                                        <td className="py-2 px-2.5 text-right space-x-1 whitespace-nowrap min-w-[290px]">
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
                                                type="button"
                                                onClick={() => handleExportSingleAccount(acc)}
                                                className="px-2 py-0.5 rounded bg-amber-50 dark:bg-amber-950/40 text-amber-600 dark:text-amber-400 hover:bg-amber-100 dark:hover:bg-amber-900/50 border border-amber-200/60 dark:border-amber-800/50 text-[10px] font-medium cursor-pointer transition-colors"
                                                title="Export this account (JSON / YAML / CSV)"
                                            >
                                                Export
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
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800 space-y-6">
                <div>
                    <div className="flex items-center justify-between mb-3">
                        <h3 className="text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Send className="w-4 h-4 text-emerald-500" />
                            Notification Recipients & Worker Contacts
                        </h3>
                        <span className="text-xs text-gray-400 dark:text-slate-500 font-mono">
                            {recipients.length} registered
                        </span>
                    </div>

                    <form
                        onSubmit={(e) => {
                            e.preventDefault();
                            handleAddRecipient();
                        }}
                        className="flex flex-wrap items-center gap-2 mb-3"
                    >
                        <input
                            type="email"
                            placeholder="Recipient Email (e.g. user@domain.com)"
                            value={newRecipientEmail}
                            onChange={(e) => setNewRecipientEmail(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    handleAddRecipient();
                                }
                            }}
                            className="w-64 max-w-xs px-3 py-1.5 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                        <input
                            type="text"
                            placeholder="Group (e.g. default)"
                            value={newRecipientGroup}
                            onChange={(e) => setNewRecipientGroup(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    handleAddRecipient();
                                }
                            }}
                            className="w-28 px-2.5 py-1.5 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                        <button
                            type="submit"
                            className="px-3.5 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-medium transition-colors shadow-sm cursor-pointer shrink-0"
                        >
                            Add Recipient
                        </button>
                    </form>

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

                {/* Developer Task Quick Dispatch */}
                <div className="pt-4 border-t border-gray-100 dark:border-slate-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <Terminal className="w-4 h-4 text-blue-500" />
                            <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-700 dark:text-slate-300">
                                Developer Task Quick Dispatch
                            </h4>
                        </div>
                        <span className="text-[11px] text-blue-600 dark:text-blue-400 font-medium">
                            Auto Receipt Active
                        </span>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-slate-400 mb-3">
                        Dispatch a structured remote task to worker email addresses with instantaneous receipt acknowledgements.
                    </p>

                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-3">
                        <div>
                            <label className="text-[11px] font-semibold text-gray-600 dark:text-slate-400 mb-1 block">
                                Task Type
                            </label>
                            <select
                                value={developerTaskType}
                                onChange={(e) => handleTaskTypeChange(e.target.value)}
                                className="w-full px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                            >
                                <option value="prompt">prompt: - AI Prompt Injection</option>
                                <option value="powershell">powershell: - Windows PowerShell Execution</option>
                                <option value="cmd">cmd: - Windows Command Prompt Execution</option>
                                <option value="gitmap">gitmap: - GitMap Autonomous CLI</option>
                                <option value="status">status: - Telemetry & Quota Status Query</option>
                            </select>
                        </div>
                        <div>
                            <label className="text-[11px] font-semibold text-gray-600 dark:text-slate-400 mb-1 block">
                                Target Recipient
                            </label>
                            <select
                                value={developerTargetRecipient}
                                onChange={(e) => setDeveloperTargetRecipient(e.target.value)}
                                className="w-full px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            >
                                <option value="">All Registered Recipients (Broadcast)</option>
                                {recipients.map((rec) => (
                                    <option key={rec.id} value={rec.email}>
                                        {rec.email} ({rec.group_name})
                                    </option>
                                ))}
                            </select>
                        </div>
                    </div>

                    <div className="mb-3">
                        <label className="text-[11px] font-semibold text-gray-600 dark:text-slate-400 mb-1 block">
                            Custom Subject (Optional)
                        </label>
                        <input
                            type="text"
                            placeholder="e.g. [Antigravity-Node-1][192.168.1.100][Instance-1] prompt: Review PR"
                            value={developerCustomSubject}
                            onChange={(e) => setDeveloperCustomSubject(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    handleDispatchDeveloperTask();
                                }
                            }}
                            className="w-full px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                        />
                    </div>

                    <div className="mb-3">
                        <label className="text-[11px] font-semibold text-gray-600 dark:text-slate-400 mb-1 block">
                            Command / Prompt Payload (Press Ctrl+Enter to dispatch)
                        </label>
                        <textarea
                            rows={3}
                            value={developerTaskPayload}
                            onChange={(e) => setDeveloperTaskPayload(e.target.value)}
                            onKeyDown={(e) => {
                                if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                                    e.preventDefault();
                                    handleDispatchDeveloperTask();
                                }
                            }}
                            className="w-full px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                            placeholder="Enter command or AI prompt payload... (Ctrl+Enter to dispatch)"
                        />
                    </div>

                    <div className="flex justify-end">
                        <button
                            onClick={handleDispatchDeveloperTask}
                            disabled={isDispatchingTask}
                            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-xs font-medium transition-colors shadow-sm cursor-pointer flex items-center gap-1.5 disabled:opacity-50"
                        >
                            {isDispatchingTask ? (
                                <>
                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                    <span>Dispatching...</span>
                                </>
                            ) : (
                                <>
                                    <Play className="w-3.5 h-3.5" />
                                    <span>Dispatch Task</span>
                                </>
                            )}
                        </button>
                    </div>
                </div>

                {/* Adaptive Monitoring & Retry Cadence */}
                <div className="pt-4 border-t border-gray-100 dark:border-slate-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <SlidersHorizontal className="w-4 h-4 text-purple-500" />
                            <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-700 dark:text-slate-300">
                                Adaptive Monitoring & Retry Intervals
                            </h4>
                        </div>
                        <button
                            onClick={handleSaveRecipientsIntervals}
                            className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white rounded-md text-[11px] font-medium transition-colors cursor-pointer shadow-sm"
                        >
                            Save Monitoring Intervals
                        </button>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-slate-400 mb-3">
                        Configure initial mailbox wait cadence and fast-polling responsiveness for interactive remote replies.
                    </p>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="p-3.5 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-2">
                            <div className="flex justify-between items-center">
                                <label className="text-xs font-semibold text-gray-700 dark:text-slate-200">
                                    Initial Wait Cadence
                                </label>
                                <span className="text-xs font-bold text-purple-600 dark:text-purple-400">
                                    {settings.baseline_polling_interval_minutes || 5} min
                                </span>
                            </div>
                            <input
                                type="range"
                                min={5}
                                max={10}
                                step={1}
                                value={settings.baseline_polling_interval_minutes || 5}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        baseline_polling_interval_minutes: parseInt(e.target.value) || 5,
                                    })
                                }
                                className="w-full accent-purple-600"
                            />
                            <p className="text-[11px] text-gray-400 dark:text-slate-400">
                                Initial delay (5–10 min) before checking mailboxes for external instructions.
                            </p>
                        </div>

                        <div className="p-3.5 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-2">
                            <div className="flex justify-between items-center">
                                <label className="text-xs font-semibold text-gray-700 dark:text-slate-200">
                                    Fast Reply Quick-Poll
                                </label>
                                <span className="text-xs font-bold text-purple-600 dark:text-purple-400">
                                    {settings.active_awaiting_interval_seconds || 10} sec
                                </span>
                            </div>
                            <input
                                type="range"
                                min={5}
                                max={10}
                                step={1}
                                value={settings.active_awaiting_interval_seconds || 10}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        active_awaiting_interval_seconds: parseInt(e.target.value) || 10,
                                    })
                                }
                                className="w-full accent-purple-600"
                            />
                            <p className="text-[11px] text-gray-400 dark:text-slate-400">
                                High-speed polling (5–10 sec) during active command dispatch and receipt awaiting.
                            </p>
                        </div>
                    </div>
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
                            Baseline Polling Interval (Idle Cadence)
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={5}
                                max={10}
                                value={settings.baseline_polling_interval_minutes || 5}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        baseline_polling_interval_minutes: parseInt(e.target.value) || 5,
                                    })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.baseline_polling_interval_minutes || 5} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            Standard background frequency for monitoring incoming email instructions.
                        </p>
                    </div>

                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-slate-200 block">
                            Active Awaiting Interval (Fast Adaptive)
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={5}
                                max={10}
                                value={settings.active_awaiting_interval_seconds || 10}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        active_awaiting_interval_seconds: parseInt(e.target.value) || 10,
                                    })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.active_awaiting_interval_seconds || 10} sec
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            High-speed 5–10s polling interval when awaiting reply after outbound dispatch.
                        </p>
                    </div>

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

                <div className="flex flex-wrap items-center justify-between gap-3 pt-4 border-t border-gray-100 dark:border-slate-800">
                    <div className="flex flex-wrap items-center gap-2">
                        <button
                            type="button"
                            onClick={handleTriggerManualCheck}
                            className="px-4 py-2 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-200 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer"
                        >
                            <Send className="w-3.5 h-3.5 text-blue-500" />
                            Dispatch Test Cheat Sheet
                        </button>
                        <button
                            type="button"
                            onClick={handleDispatchPingTest}
                            disabled={isPinging}
                            className="px-4 py-2 text-xs font-medium border border-blue-200 dark:border-blue-900/60 bg-blue-50/50 dark:bg-blue-950/30 hover:bg-blue-100/50 dark:hover:bg-blue-900/50 text-blue-600 dark:text-blue-400 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                        >
                            {isPinging ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Zap className="w-3.5 h-3.5 text-amber-500" />
                            )}
                            Dispatch Ping Command Test
                        </button>
                    </div>

                    <button
                        type="button"
                        onClick={handleSaveSettings}
                        disabled={isSaving}
                        className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm cursor-pointer disabled:opacity-60"
                    >
                        {isSaving ? 'Saving Settings...' : 'Save Watcher Settings'}
                    </button>
                </div>
            </div>

            {/* Interactive Remote Command & Prompt Testing Card */}
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800 space-y-4">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                    <div>
                        <h3 className="text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Terminal className="w-4 h-4 text-purple-500" />
                            Interactive Remote Command &amp; Prompt Testing
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400 mt-0.5">
                            Simulate and execute CLI instructions, GitMap scans, or AI prompts locally on this machine node before sending via email.
                        </p>
                    </div>
                    <button
                        type="button"
                        onClick={() => setIsSampleTemplatesOpen(true)}
                        className="px-3 py-1.5 text-xs font-medium border border-purple-200 dark:border-purple-900/60 bg-purple-50/50 dark:bg-purple-950/30 hover:bg-purple-100/50 dark:hover:bg-purple-900/50 text-purple-600 dark:text-purple-400 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer self-start sm:self-auto"
                    >
                        <Sparkles className="w-3.5 h-3.5 text-purple-500" />
                        Command Cheat Sheets
                    </button>
                </div>

                {/* Quick Templates */}
                <div className="flex flex-wrap items-center gap-1.5">
                    <span className="text-[11px] font-semibold text-gray-400 dark:text-slate-500 mr-1">Templates:</span>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('powershell: Get-Process | Select-Object -First 5')}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                    >
                        PowerShell Processes
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand("powershell: Get-Service -Name '*wsl*', '*docker*' -ErrorAction SilentlyContinue")}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                    >
                        PowerShell Services
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('gitmap --version')}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                    >
                        GitMap Version
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('gitmap status')}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                    >
                        GitMap Status
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('Project: Antigravity-Manager\nScan repository health and report open issues')}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                    >
                        Prompt Injection
                    </button>
                </div>

                {/* Command Input Area */}
                <div className="space-y-2">
                    <div className="relative">
                        <textarea
                            rows={3}
                            value={testCliCommand}
                            onChange={(e) => setTestCliCommand(e.target.value)}
                            placeholder="Enter CLI command (e.g. powershell: Get-Process) or AI prompt instruction..."
                            className="w-full px-3 py-2.5 font-mono text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-gray-900 text-gray-100 placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500 leading-relaxed resize-y"
                        />
                    </div>
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <div className="text-[11px] text-gray-500 dark:text-slate-400">
                            Prefix with <code className="text-purple-600 dark:text-purple-400 font-bold">powershell:</code>, <code className="text-purple-600 dark:text-purple-400 font-bold">bash:</code>, or execute directly on path.
                        </div>
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={() => {
                                    navigator.clipboard.writeText(testCliCommand);
                                    showToast('Command copied to clipboard', 'info');
                                }}
                                className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer"
                            >
                                <Copy className="w-3.5 h-3.5" />
                                Copy
                            </button>
                            <button
                                type="button"
                                onClick={handleTestExecuteCli}
                                disabled={isExecutingCli}
                                className="px-4 py-1.5 bg-purple-600 hover:bg-purple-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isExecutingCli ? (
                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                ) : (
                                    <Play className="w-3.5 h-3.5 fill-current" />
                                )}
                                {isExecutingCli ? 'Executing...' : 'Run Local Test'}
                            </button>
                        </div>
                    </div>
                </div>

                {/* Execution Output Console */}
                {cliExecResult ? (
                    <div className="mt-3 p-3.5 rounded-xl bg-gray-950 border border-gray-800 text-gray-200 space-y-2">
                        <div className="flex flex-wrap items-center justify-between gap-2 text-xs border-b border-gray-800 pb-2">
                            <div className="flex items-center gap-2">
                                <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider ${
                                    cliExecResult.success ? 'bg-emerald-900/60 text-emerald-300 border border-emerald-700/60' : 'bg-rose-900/60 text-rose-300 border border-rose-700/60'
                                }`}>
                                    {cliExecResult.success ? <CheckCircle2 className="w-3 h-3" /> : <AlertCircle className="w-3 h-3" />}
                                    Exit {cliExecResult.exit_code}
                                </span>
                                <span className="font-mono text-[11px] text-gray-400">
                                    Node: <span className="text-gray-200 font-semibold">{cliExecResult.machine_name}</span> ({cliExecResult.machine_ip})
                                </span>
                            </div>
                            <button
                                type="button"
                                onClick={() => setCliExecResult(null)}
                                className="text-[11px] text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
                            >
                                Clear Console
                            </button>
                        </div>

                        {cliExecResult.stdout.length > 0 ? (
                            <div>
                                <div className="text-[10px] font-bold text-gray-400 uppercase tracking-wider mb-1">Standard Output:</div>
                                <pre className="p-2.5 bg-black/50 rounded-lg font-mono text-[11px] text-emerald-400 overflow-x-auto max-h-48 whitespace-pre-wrap leading-relaxed">
                                    {cliExecResult.stdout}
                                </pre>
                            </div>
                        ) : null}

                        {cliExecResult.stderr.length > 0 ? (
                            <div>
                                <div className="text-[10px] font-bold text-rose-400 uppercase tracking-wider mb-1">Standard Error:</div>
                                <pre className="p-2.5 bg-black/50 rounded-lg font-mono text-[11px] text-rose-400 overflow-x-auto max-h-36 whitespace-pre-wrap leading-relaxed">
                                    {cliExecResult.stderr}
                                </pre>
                            </div>
                        ) : null}

                        {cliExecResult.stdout === '' && cliExecResult.stderr === '' ? (
                            <div className="text-[11px] text-gray-400 italic py-1">
                                Command executed with empty output stream.
                            </div>
                        ) : null}
                    </div>
                ) : null}
            </div>

            {/* Telegram Bot & Alerts Card */}
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
                    <div>
                        <h3 className="text-sm sm:text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <MessageSquare className="w-4 h-4 text-sky-500" />
                            Telegram Bot & Alert Notifications
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400">
                            Receive real-time push alerts, account switch logs, and remote command executions directly on Telegram.
                        </p>
                    </div>

                    <div className="flex items-center gap-2">
                        {telegramStatus?.is_running ? (
                            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
                                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                Active & Polling
                            </span>
                        ) : (
                            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-gray-100 dark:bg-slate-800 text-gray-600 dark:text-slate-400 border border-gray-200 dark:border-slate-700">
                                Inactive
                            </span>
                        )}

                        <button
                            type="button"
                            onClick={() => {
                                const cmd = `.\\03-ai-scripts\\setup-telegram-bot.ps1 -BotToken "${telegramConfig?.bot_token || 'TOKEN'}" -ChatId "${telegramConfig?.allowed_chat_id || 'CHAT_ID'}"`;
                                navigator.clipboard.writeText(cmd);
                                showToast('PowerShell verification command copied', 'info');
                            }}
                            className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-sky-200 dark:border-sky-900/50 bg-sky-50/50 dark:bg-sky-950/20 text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/40 transition-all flex items-center gap-1.5 shadow-xs cursor-pointer"
                            title="Copy PowerShell script command"
                        >
                            <Copy className="w-3.5 h-3.5" />
                            <span>Copy PS Script</span>
                        </button>
                    </div>
                </div>

                <div className="space-y-4">
                    {/* Bot Token & Allowed Chat ID */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                        <div className="space-y-1.5">
                            <label className="text-xs font-medium text-gray-700 dark:text-slate-300 flex items-center justify-between">
                                <span>Telegram Bot Token</span>
                                {telegramBotUsername && (
                                    <span className="text-[11px] text-sky-600 dark:text-sky-400 font-semibold">
                                        @{telegramBotUsername} Verified
                                    </span>
                                )}
                            </label>
                            <div className="relative">
                                <input
                                    type={showBotToken ? 'text' : 'password'}
                                    value={telegramConfig?.bot_token || ''}
                                    onChange={(e) =>
                                        setTelegramConfig((prev) =>
                                            prev
                                                ? { ...prev, bot_token: e.target.value }
                                                : {
                                                      bot_token: e.target.value,
                                                      allowed_chat_id: null,
                                                      is_enabled: false,
                                                      poll_interval_secs: 5,
                                                  }
                                        )
                                    }
                                    placeholder="123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ"
                                    className="w-full px-3 py-2 pr-10 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-gray-50 dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-sky-500 font-mono"
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowBotToken(!showBotToken)}
                                    className="absolute right-2.5 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-slate-300 p-0.5"
                                >
                                    {showBotToken ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                                </button>
                            </div>
                        </div>

                        <div className="space-y-1.5">
                            <label className="text-xs font-medium text-gray-700 dark:text-slate-300">
                                Allowed Chat ID (Numeric)
                            </label>
                            <input
                                type="number"
                                value={telegramConfig?.allowed_chat_id ?? ''}
                                onChange={(e) => {
                                    const val = e.target.value.trim();
                                    const parsed = val ? parseInt(val, 10) : null;
                                    setTelegramConfig((prev) =>
                                        prev
                                            ? { ...prev, allowed_chat_id: isNaN(parsed as any) ? null : parsed }
                                            : {
                                                  bot_token: '',
                                                  allowed_chat_id: isNaN(parsed as any) ? null : parsed,
                                                  is_enabled: false,
                                                  poll_interval_secs: 5,
                                              }
                                    );
                                }}
                                placeholder="987654321"
                                className="w-full px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-gray-50 dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-sky-500 font-mono"
                            />
                        </div>
                    </div>

                    {/* Enable Toggle & Polling Interval */}
                    <div className="flex flex-wrap items-center justify-between gap-4 p-3 rounded-lg bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-750">
                        <label className="flex items-center gap-2.5 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={telegramConfig?.is_enabled || false}
                                onChange={(e) =>
                                    setTelegramConfig((prev) =>
                                        prev
                                            ? { ...prev, is_enabled: e.target.checked }
                                            : {
                                                  bot_token: '',
                                                  allowed_chat_id: null,
                                                  is_enabled: e.target.checked,
                                                  poll_interval_secs: 5,
                                              }
                                    )
                                }
                                className="w-4 h-4 rounded text-sky-600 focus:ring-sky-500 border-gray-300 dark:border-slate-600"
                            />
                            <div>
                                <span className="text-xs font-semibold text-gray-900 dark:text-slate-100">
                                    Enable Telegram Bot Notifications & Inbound Daemon
                                </span>
                                <p className="text-[11px] text-gray-500 dark:text-slate-400">
                                    Polls for commands and sends instant alerts when accounts switch or quota drops.
                                </p>
                            </div>
                        </label>

                        <div className="flex items-center gap-2">
                            <span className="text-xs text-gray-600 dark:text-slate-400">Poll Interval:</span>
                            <input
                                type="number"
                                min={2}
                                max={60}
                                value={telegramConfig?.poll_interval_secs || 5}
                                onChange={(e) => {
                                    const val = parseInt(e.target.value, 10) || 5;
                                    setTelegramConfig((prev) =>
                                        prev ? { ...prev, poll_interval_secs: val } : null
                                    );
                                }}
                                className="w-16 px-2 py-1 text-xs border border-gray-200 dark:border-slate-700 rounded bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 text-center font-mono"
                            />
                            <span className="text-xs text-gray-500">sec</span>
                        </div>
                    </div>

                    {/* Action Buttons */}
                    <div className="flex flex-wrap items-center justify-between gap-2 pt-1 border-t border-gray-100 dark:border-slate-800">
                        <div className="text-[11px] text-gray-500 dark:text-slate-400">
                            Create bot with <span className="font-semibold text-sky-600 dark:text-sky-400">@BotFather</span> and get ID from <span className="font-semibold text-sky-600 dark:text-sky-400">@userinfobot</span>.
                        </div>

                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={handleTestTelegram}
                                disabled={isTestingTelegram}
                                className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isTestingTelegram ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <RefreshCw className="w-3.5 h-3.5" />}
                                Test Bot Token
                            </button>
                            <button
                                type="button"
                                onClick={handleSendTelegramPing}
                                disabled={isSendingTelegramPing}
                                className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isSendingTelegramPing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Send className="w-3.5 h-3.5" />}
                                Send Test Alert
                            </button>
                            <button
                                type="button"
                                onClick={handleSaveTelegram}
                                disabled={isSavingTelegram}
                                className="px-4 py-1.5 bg-sky-600 hover:bg-sky-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isSavingTelegram ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <CheckCircle2 className="w-3.5 h-3.5" />}
                                Save Telegram Settings
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            {/* Account Add/Edit Modal */}
            <ModalDialog
                isOpen={isAccountModalOpen}
                title={editingAccount.id ? 'Edit Mailbox Configuration' : 'Add Mailbox to Secure Split Vault'}
                type="confirm"
                maxWidth="max-w-xl"
                confirmText={editingAccount.id ? 'Save Mailbox' : 'Add Mailbox to Vault'}
                cancelText="Cancel"
                onConfirm={handleSaveAccount}
                onCancel={() => setIsAccountModalOpen(false)}
            >
                <form
                    onSubmit={(e) => {
                        e.preventDefault();
                        handleSaveAccount();
                    }}
                    className="space-y-3.5 text-xs"
                >
                    {/* Single Account Import / Export Action Bar */}
                    <div className="flex flex-wrap items-center justify-between gap-2 p-2 rounded-lg bg-gray-50 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-700">
                        <div className="flex items-center gap-1.5">
                            <span className="text-[11px] font-semibold text-gray-500 dark:text-slate-400">Account IO:</span>
                            <button
                                type="button"
                                onClick={() => setIsModalQuickImportOpen(!isModalQuickImportOpen)}
                                className={`px-2 py-1 text-[11px] font-medium rounded-md border flex items-center gap-1 transition-colors cursor-pointer ${
                                    isModalQuickImportOpen
                                        ? 'bg-blue-600 text-white border-blue-600'
                                        : 'bg-white dark:bg-slate-800 border-gray-200 dark:border-slate-700 text-gray-700 dark:text-slate-200 hover:bg-gray-100 dark:hover:bg-slate-700'
                                }`}
                                title="Import single account from JSON, YAML, or CSV"
                            >
                                <Upload className="w-3 h-3 text-indigo-500" />
                                <span>Import (JSON / YAML / CSV)</span>
                            </button>
                        </div>

                        <div className="flex items-center gap-1">
                            <span className="text-[10px] text-gray-400 dark:text-slate-500 mr-0.5">Export:</span>
                            <button
                                type="button"
                                onClick={() => handleExportSingleAccount(editingAccount, 'json')}
                                className="px-1.5 py-0.5 text-[10px] font-medium rounded border border-amber-200 dark:border-amber-800/60 bg-amber-50/50 dark:bg-amber-950/20 text-amber-700 dark:text-amber-300 hover:bg-amber-100 dark:hover:bg-amber-900/40 flex items-center gap-1 cursor-pointer transition-colors"
                                title="Export single account as JSON"
                            >
                                <FileJson className="w-3 h-3 text-amber-500" />
                                <span>JSON</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => handleExportSingleAccount(editingAccount, 'yaml')}
                                className="px-1.5 py-0.5 text-[10px] font-medium rounded border border-emerald-200 dark:border-emerald-800/60 bg-emerald-50/50 dark:bg-emerald-950/20 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 flex items-center gap-1 cursor-pointer transition-colors"
                                title="Export single account as YAML"
                            >
                                <FileCode className="w-3 h-3 text-emerald-500" />
                                <span>YAML</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => handleExportSingleAccount(editingAccount, 'csv')}
                                className="px-1.5 py-0.5 text-[10px] font-medium rounded border border-blue-200 dark:border-blue-800/60 bg-blue-50/50 dark:bg-blue-950/20 text-blue-700 dark:text-blue-300 hover:bg-blue-100 dark:hover:bg-blue-900/40 flex items-center gap-1 cursor-pointer transition-colors"
                                title="Export single account as CSV"
                            >
                                <FileText className="w-3 h-3 text-blue-500" />
                                <span>CSV</span>
                            </button>
                        </div>
                    </div>

                    {/* Expandable Quick Import Panel */}
                    {isModalQuickImportOpen && (
                        <div className="p-3 rounded-lg border border-indigo-200 dark:border-indigo-900/50 bg-indigo-50/40 dark:bg-indigo-950/20 space-y-2 animate-in fade-in slide-in-from-top-1">
                            <div className="flex items-center justify-between">
                                <span className="text-[11px] font-semibold text-indigo-900 dark:text-indigo-200 flex items-center gap-1.5">
                                    <Upload className="w-3.5 h-3.5 text-indigo-600 dark:text-indigo-400" />
                                    Import Account Config (Paste or Browse File)
                                </span>
                                <div>
                                    <input
                                        type="file"
                                        ref={singleAccountFileInputRef}
                                        className="hidden"
                                        accept=".json,.yaml,.yml,.csv,.txt"
                                        onChange={handleSingleFileUpload}
                                    />
                                    <button
                                        type="button"
                                        onClick={() => singleAccountFileInputRef.current?.click()}
                                        className="px-2 py-0.5 text-[10px] font-medium bg-white dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-700 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 rounded flex items-center gap-1 cursor-pointer"
                                    >
                                        <Upload className="w-3 h-3" />
                                        Browse File
                                    </button>
                                </div>
                            </div>
                            <textarea
                                rows={3}
                                value={modalQuickImportText}
                                onChange={(e) => setModalQuickImportText(e.target.value)}
                                placeholder="Paste single account JSON, YAML, or CSV snippet here to auto-fill form..."
                                className="w-full px-2.5 py-1.5 font-mono text-[11px] border border-indigo-200 dark:border-indigo-800 rounded bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 focus:outline-none focus:ring-1 focus:ring-indigo-500"
                            />
                            <div className="flex items-center justify-end gap-2">
                                <button
                                    type="button"
                                    onClick={() => {
                                        setIsModalQuickImportOpen(false);
                                        setModalQuickImportText('');
                                    }}
                                    className="px-2 py-1 text-[11px] text-gray-500 hover:text-gray-700 dark:text-slate-400 cursor-pointer"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="button"
                                    onClick={() => handleQuickImportSingle(modalQuickImportText)}
                                    className="px-3 py-1 text-[11px] font-semibold bg-indigo-600 hover:bg-indigo-700 text-white rounded transition-colors cursor-pointer shadow-xs"
                                >
                                    Apply to Form
                                </button>
                            </div>
                        </div>
                    )}

                    {/* One-Click AI Instructions with Embedded JSON Syntax Segment */}
                    <div className="rounded-lg bg-blue-50/70 dark:bg-blue-950/30 border border-blue-100 dark:border-blue-900/40 p-2.5 space-y-2">
                        <div className="flex items-center justify-between gap-2">
                            <div className="flex items-center gap-1.5 text-blue-700 dark:text-blue-300">
                                <Sparkles className="w-3.5 h-3.5 shrink-0 text-blue-600 dark:text-blue-400" />
                                <span className="font-semibold text-[11px]">AI Automated Mailbox Configuration</span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <button
                                    type="button"
                                    onClick={() => setShowAiJsonSyntax(!showAiJsonSyntax)}
                                    className="px-2 py-0.5 text-[10px] font-medium bg-blue-100/70 dark:bg-blue-900/50 hover:bg-blue-200/70 dark:hover:bg-blue-900/80 text-blue-800 dark:text-blue-200 rounded flex items-center gap-1 transition-colors cursor-pointer"
                                >
                                    <FileJson className="w-3 h-3" />
                                    <span>{showAiJsonSyntax ? 'Hide JSON Format' : 'View JSON Format'}</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => {
                                        const instructions = `Generate or configure a mailbox for Antigravity-Manager matching this exact JSON specification segment:

\`\`\`json
{
  "alias": "${editingAccount.alias || 'Primary Mailbox'}",
  "email": "${editingAccount.email || 'your-email@gmail.com'}",
  "password": "<generated-app-password>",
  "smtp_host": "${editingAccount.smtp_host || 'smtp.gmail.com'}",
  "smtp_port": ${editingAccount.smtp_port || 587},
  "imap_host": "${editingAccount.imap_host || 'imap.gmail.com'}",
  "imap_port": ${editingAccount.imap_port || 993},
  "encryption_type": "${editingAccount.encryption_type || 'TLS'}",
  "is_default": ${editingAccount.is_default},
  "is_active": ${editingAccount.is_active}
}
\`\`\`

Instructions: Provide the app-password and verified SMTP/IMAP settings inside the JSON segment so it can be pasted directly into Antigravity-Manager's mailbox import.`;
                                        navigator.clipboard.writeText(instructions);
                                        showToast('AI instructions with embedded JSON syntax copied to clipboard', 'success');
                                    }}
                                    className="px-2.5 py-1 text-[11px] font-semibold bg-white dark:bg-slate-800 hover:bg-blue-50 dark:hover:bg-blue-900/40 text-blue-600 dark:text-blue-400 border border-blue-200 dark:border-blue-800 rounded-md flex items-center gap-1.5 transition-all shadow-xs cursor-pointer"
                                    title="Copy AI Instructions with Embedded JSON Syntax to Clipboard"
                                >
                                    <Copy className="w-3 h-3" />
                                    <span>Copy AI Instructions</span>
                                </button>
                            </div>
                        </div>

                        {/* Embedded JSON Syntax Segment Preview */}
                        {showAiJsonSyntax && (
                            <div className="pt-1.5 border-t border-blue-200/50 dark:border-blue-900/40">
                                <div className="text-[10px] text-blue-700 dark:text-blue-300 font-medium mb-1">
                                    Embedded JSON Syntax Segment:
                                </div>
                                <pre className="p-2 bg-slate-950 text-amber-300 font-mono text-[10px] rounded-md overflow-x-auto leading-tight select-all">
{JSON.stringify(
    {
        alias: editingAccount.alias || 'Primary Mailbox',
        email: editingAccount.email || 'your-email@gmail.com',
        password: editingAccount.password || '<app-password>',
        smtp_host: editingAccount.smtp_host || 'smtp.gmail.com',
        smtp_port: editingAccount.smtp_port || 587,
        imap_host: editingAccount.imap_host || 'imap.gmail.com',
        imap_port: editingAccount.imap_port || 993,
        encryption_type: editingAccount.encryption_type || 'TLS',
        is_default: editingAccount.is_default,
        is_active: editingAccount.is_active,
    },
    null,
    2
)}
                                </pre>
                            </div>
                        )}
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
                </form>
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
                                    value="yaml"
                                    checked={importFormat === 'yaml'}
                                    onChange={() => setImportFormat('yaml')}
                                />
                                <span>YAML</span>
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
                                accept=".json,.yaml,.yml,.csv,.xml,.xlsx,.xls"
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

            {/* Mailbox Export Preview & Save Modal */}
            <MailboxExportModal
                isOpen={exportModalState.isOpen}
                onClose={() => setExportModalState({ isOpen: false })}
                singleAccount={exportModalState.singleAccount}
                allAccounts={exportModalState.allAccounts}
                initialFormat={exportModalState.initialFormat}
            />
        </div>
    );
}
