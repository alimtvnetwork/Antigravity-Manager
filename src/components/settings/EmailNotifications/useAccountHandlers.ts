import { showToast } from '../common/ToastContainer';
import { EmailNotificationsState } from './useEmailCore';

export function useAccountHandlers(s: EmailNotificationsState) {
    const { accounts, setAccounts, recipients, setRecipients, setIsAccountModalOpen, editingAccount, setEditingAccount, newRecipientEmail, setNewRecipientEmail, newRecipientGroup, setTestingAccountId, setIsModalQuickImportOpen, setModalQuickImportText, setShowAiJsonSyntax, setIsTestingDirect, testResult, setTestResult } = s;

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


    return {
        handleOpenAddAccount,
        handleOpenEditAccount,
        handleEmailChange,
        trimmed,
        parts,
        userPart,
        domain,
        handleTestDirectConnection,
        isEmailFormatValid,
        hasMissingPassword,
        res,
        err,
        handleSaveAccount,
        accs,
        handleDeleteAccount,
        handleSetDefault,
        handleTestSmtp,
        handleTestImap,
        handleAddRecipient,
        added,
        handleDeleteRecipient,
    };
}
