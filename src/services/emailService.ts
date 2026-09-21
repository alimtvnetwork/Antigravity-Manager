import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export interface EmailAccount {
    id: string;
    alias: string;
    email: string;
    smtp_host: string;
    smtp_port: number;
    imap_host: string;
    imap_port: number;
    encryption_type: string;
    is_default: boolean;
    is_active: boolean;
    created_at: number;
    updated_at: number;
}

export interface EmailAccountInput {
    id?: string;
    alias: string;
    email: string;
    password?: string;
    smtp_host: string;
    smtp_port: number;
    imap_host: string;
    imap_port: number;
    encryption_type: string;
    is_default: boolean;
    is_active: boolean;
}

export interface NotifyRecipient {
    id: string;
    email: string;
    group_name: string;
    is_active: boolean;
    created_at: number;
}

export interface NotifyRecipientInput {
    email: string;
    group_name?: string;
    is_active?: boolean;
}

export interface EmailNotificationSettings {
    id: string;
    is_enabled: boolean;
    polling_interval_minutes: number;
    inbox_check_interval_minutes: number;
    baseline_polling_interval_minutes: number;
    active_awaiting_interval_seconds: number;
    notify_on_quota_drop: boolean;
    quota_drop_threshold_percent: number;
    notify_on_workspace_switch: boolean;
    notify_on_idle_workspace: boolean;
    allow_remote_prompt_execution: boolean;
    allow_remote_cli_execution: boolean;
    allow_remote_instance_rotation: boolean;
    local_machine_name: string;
    local_machine_ip: string;
    updated_at: number;
}

export interface ImportSummary {
    accounts_imported: number;
    recipients_imported: number;
    settings_updated: boolean;
    errors: string[];
}

export interface WatcherStatus {
    is_running: boolean;
    machine_name: string;
    machine_ip: string;
    last_telemetry_check: number;
    last_inbox_check: number;
    last_alert_sent?: number;
}

export async function getEmailSettings(): Promise<EmailNotificationSettings> {
    try {
        return await invoke('get_email_settings');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.getEmailSettings',
            endpoint: 'get_email_settings',
        });
        throw e;
    }
}

export async function saveEmailSettings(settings: EmailNotificationSettings): Promise<void> {
    try {
        return await invoke('save_email_settings', { settings });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.saveEmailSettings',
            endpoint: 'save_email_settings',
        });
        throw e;
    }
}

export async function listEmailAccounts(): Promise<EmailAccount[]> {
    try {
        return await invoke('list_email_accounts');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.listEmailAccounts',
            endpoint: 'list_email_accounts',
        });
        throw e;
    }
}

export async function addEmailAccount(account: EmailAccountInput): Promise<EmailAccount> {
    try {
        return await invoke('add_email_account', { account });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.addEmailAccount',
            endpoint: 'add_email_account',
        });
        throw e;
    }
}

export async function updateEmailAccount(account: EmailAccountInput): Promise<EmailAccount> {
    try {
        return await invoke('update_email_account', { account });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.updateEmailAccount',
            endpoint: 'update_email_account',
        });
        throw e;
    }
}

export async function deleteEmailAccount(id: string): Promise<void> {
    try {
        return await invoke('delete_email_account', { id });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.deleteEmailAccount',
            endpoint: 'delete_email_account',
        });
        throw e;
    }
}

export async function setDefaultEmailAccount(id: string): Promise<void> {
    try {
        return await invoke('set_default_email_account', { id });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.setDefaultEmailAccount',
            endpoint: 'set_default_email_account',
        });
        throw e;
    }
}

export async function listNotifyRecipients(): Promise<NotifyRecipient[]> {
    try {
        return await invoke('list_notify_recipients');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.listNotifyRecipients',
            endpoint: 'list_notify_recipients',
        });
        throw e;
    }
}

export async function addNotifyRecipient(recipient: NotifyRecipientInput): Promise<NotifyRecipient> {
    try {
        return await invoke('add_notify_recipient', { recipient });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.addNotifyRecipient',
            endpoint: 'add_notify_recipient',
        });
        throw e;
    }
}

export async function deleteNotifyRecipient(id: string): Promise<void> {
    try {
        return await invoke('delete_notify_recipient', { id });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.deleteNotifyRecipient',
            endpoint: 'delete_notify_recipient',
        });
        throw e;
    }
}

export async function testSmtpConnection(accountId: string): Promise<string> {
    try {
        return await invoke('test_smtp_connection', { accountId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.testSmtpConnection',
            endpoint: 'test_smtp_connection',
        });
        throw e;
    }
}

export async function testImapConnection(accountId: string): Promise<string> {
    try {
        return await invoke('test_imap_connection', { accountId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.testImapConnection',
            endpoint: 'test_imap_connection',
        });
        throw e;
    }
}

export async function testDirectEmailConnection(account: EmailAccountInput): Promise<string> {
    try {
        return await invoke('test_direct_email_connection', { account });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.testDirectEmailConnection',
            endpoint: 'test_direct_email_connection',
        });
        throw e;
    }
}


export async function exportEmailData(format: string): Promise<string> {
    try {
        return await invoke('export_email_data', { format });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.exportEmailData',
            endpoint: 'export_email_data',
        });
        throw e;
    }
}

export async function importEmailData(format: string, payload: string): Promise<ImportSummary> {
    try {
        return await invoke('import_email_data', { format, payload });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.importEmailData',
            endpoint: 'import_email_data',
        });
        throw e;
    }
}

export async function backupEmailDb(targetPath: string): Promise<string> {
    try {
        return await invoke('backup_email_db', { targetPath });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.backupEmailDb',
            endpoint: 'backup_email_db',
        });
        throw e;
    }
}

export async function restoreEmailDb(sourcePath: string): Promise<string> {
    try {
        return await invoke('restore_email_db', { sourcePath });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.restoreEmailDb',
            endpoint: 'restore_email_db',
        });
        throw e;
    }
}

export async function getEmailWatcherStatus(): Promise<WatcherStatus> {
    try {
        return await invoke('get_email_watcher_status');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.getEmailWatcherStatus',
            endpoint: 'get_email_watcher_status',
        });
        throw e;
    }
}

export async function triggerManualEmailCheck(): Promise<string> {
    try {
        return await invoke('trigger_manual_email_check');
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.triggerManualEmailCheck',
            endpoint: 'trigger_manual_email_check',
        });
        throw e;
    }
}

export async function dispatchEmailTestPing(projectName?: string): Promise<string> {
    try {
        return await invoke('dispatch_email_test_ping', {
            projectName: projectName || 'Antigravity-Workspace',
        });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.dispatchEmailTestPing',
            endpoint: 'dispatch_email_test_ping',
        });
        throw e;
    }
}

export interface CliExecResult {
    exit_code: number;
    stdout: string;
    stderr: string;
    success: boolean;
    machine_name: string;
    machine_ip: string;
}

export async function testExecuteCliCommand(command: string): Promise<CliExecResult> {
    try {
        return await invoke('test_execute_cli_command', { command });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'emailService.testExecuteCliCommand',
            endpoint: 'test_execute_cli_command',
        });
        throw e;
    }
}
