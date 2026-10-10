import { useState } from 'react';
import { showToast } from '../../common/ToastContainer';
import { EmailNotificationsState } from './useEmailCore';
import {
    EmailAccount,
    EmailAccountInput,
    addEmailAccount,
    importEmailData,
    exportEmailData,
    backupEmailDb,
    restoreEmailDb,
    triggerManualEmailCheck,
    dispatchEmailTestPing,
    dispatchCustomEmailTask,
    testExecuteCliCommand,
    CliExecResult,
    saveEmailSettings,
} from '../../../services/emailService';
import {
    downloadOrSaveFile,
    parseAccountsFromText,
} from '../../../utils/emailFormatters';

export function useEmailUtilities(s: EmailNotificationsState) {
    const { accounts, settings, setIsPinging, HELP_COMMAND_PAYLOAD, developerTaskType, setDeveloperTaskType, developerTargetRecipient, developerTaskPayload, setDeveloperTaskPayload, developerCustomSubject, setDeveloperCustomSubject, setIsDispatchingTask, editingAccount, setEditingAccount, setIsImportModalOpen, importFormat, setImportFormat, importPayload, setImportPayload, setExportModalState, setIsModalQuickImportOpen, setModalQuickImportText, loadAll, testResult } = s;

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

    const handleLoadSampleMailboxes = async () => {
        const sampleAccounts: EmailAccountInput[] = [
            {
                alias: "Primary Google Workspace",
                email: "alerts@example.com",
                password: "app-password-sample",
                smtp_host: "smtp.gmail.com",
                smtp_port: 587,
                imap_host: "imap.gmail.com",
                imap_port: 993,
                encryption_type: "TLS",
                is_default: true,
                is_active: true
            },
            {
                alias: "Backup Microsoft 365",
                email: "agent@outlook.com",
                password: "token-secret-sample",
                smtp_host: "smtp.office365.com",
                smtp_port: 587,
                imap_host: "outlook.office365.com",
                imap_port: 993,
                encryption_type: "STARTTLS",
                is_default: false,
                is_active: true
            }
        ];
        try {
            let count = 0;
            for (const acc of sampleAccounts) {
                await addEmailAccount(acc);
                count++;
            }
            await loadAll();
            showToast(`Loaded ${count} sample mailboxes successfully`, 'success');
        } catch (e: any) {
            showToast('Failed to load sample mailboxes: ' + (e?.message || e), 'error');
        }
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

    const [testCliCommand, setTestCliCommand] = useState<string>('Get-Process | Select-Object -First 5');
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
        if (type === 'help') {
            setDeveloperCustomSubject('* | help');
            setDeveloperTaskPayload(HELP_COMMAND_PAYLOAD);
        } else if (type === 'prompt') {
            setDeveloperCustomSubject('* | prompt | proj-Antigravity-Manager');
            setDeveloperTaskPayload('Scan repository health and report open issues');
        } else if (type === 'powershell') {
            setDeveloperCustomSubject('* | ps | Get-Process');
            setDeveloperTaskPayload('Get-Process | Select-Object -First 10');
        } else if (type === 'cmd') {
            setDeveloperCustomSubject('* | cmd | dir');
            setDeveloperTaskPayload('dir /b & whoami');
        } else if (type === 'gitmap') {
            setDeveloperCustomSubject('* | gitmap | status');
            setDeveloperTaskPayload('gitmap status --json');
        } else if (type === 'status') {
            setDeveloperCustomSubject('* | status');
            setDeveloperTaskPayload('Report quota and running profile status');
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

    return {
        handleExport,
        handleExportSingleAccount,
        handleQuickImportSingle,
        handleSingleFileUpload,
        handleImportSubmit,
        handleFileUpload,
        handleLoadSampleMailboxes,
        handleBackupDb,
        handleRestoreDb,
        handleTriggerManualCheck,
        handleDispatchPingTest,
        testCliCommand,
        setTestCliCommand,
        isExecutingCli,
        setIsExecutingCli,
        cliExecResult,
        setCliExecResult,
        handleTestExecuteCli,
        handleTaskTypeChange,
        handleDispatchDeveloperTask,
        handleSaveRecipientsIntervals,
        trimmedEmail,
        emailFormatStatus,
        testResultStatus,
    };
}
