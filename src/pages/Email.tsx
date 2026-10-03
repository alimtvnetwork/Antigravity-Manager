import { useEffect, useState } from 'react';
import { Mail, ArrowLeft, Cpu, Radio, Check, X, Server, ShieldCheck, Activity, Send } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import EmailNotificationSettings from '../components/settings/EmailNotificationSettings';
import {
    getEmailWatcherStatus,
    WatcherStatus,
    getEmailSettings,
    saveEmailSettings,
    listEmailAccounts,
    EmailAccount,
    EmailNotificationSettings as IEmailSettings,
} from '../services/emailService';
import { showToast } from '../components/common/ToastContainer';

/**
 * Email Management & Alerts Page
 *
 * Dedicated top-level view for SMTP/IMAP credentials, split security vault DB,
 * automated notification triggers, and remote mailbox control.
 */
export default function Email() {
    const { t } = useTranslation();
    const [watcherStatus, setWatcherStatus] = useState<WatcherStatus | null>(null);
    const [accounts, setAccounts] = useState<EmailAccount[]>([]);
    const [emailSettings, setEmailSettings] = useState<IEmailSettings | null>(null);
    const [isEditingNode, setIsEditingNode] = useState(false);
    const [nodeNameInput, setNodeNameInput] = useState('');

    useEffect(() => {
        getEmailWatcherStatus()
            .then(setWatcherStatus)
            .catch(console.error);
        listEmailAccounts()
            .then(setAccounts)
            .catch(console.error);
        getEmailSettings()
            .then(setEmailSettings)
            .catch(console.error);
    }, []);

    const handleStartEditNode = () => {
        setNodeNameInput(watcherStatus?.machine_name || '');
        setIsEditingNode(true);
    };

    const handleSaveNodeName = async () => {
        const trimmed = nodeNameInput.trim();
        if (!trimmed) {
            setIsEditingNode(false);
            return;
        }
        try {
            const currentSettings = await getEmailSettings();
            await saveEmailSettings({
                ...currentSettings,
                local_machine_name: trimmed,
            });
            const updatedStatus = await getEmailWatcherStatus();
            setWatcherStatus({
                ...updatedStatus,
                machine_name: trimmed,
            });
            setIsEditingNode(false);
            showToast(`Node name updated to "${trimmed}"`, 'success');
        } catch (e: any) {
            showToast('Failed to save node name: ' + (e?.message || e), 'error');
        }
    };

    const defaultAccount = accounts.find((a) => a.is_default) || accounts[0];
    const isWatcherRunning = Boolean(watcherStatus?.is_running);
    const hasImap = Boolean(defaultAccount?.imap_host);
    const hasSmtp = Boolean(defaultAccount?.smtp_host);

    return (
        <div className="h-full w-full overflow-y-auto">
            <div className="max-w-[1920px] mx-auto px-4 sm:px-6 py-3 space-y-3 animate-in fade-in duration-300">
                {/* Page Header */}
                <div className="flex items-center justify-between gap-3 pb-2 border-b border-gray-200 dark:border-base-200 flex-wrap">
                    <div className="flex items-center gap-2.5 min-w-0">
                        <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 shrink-0">
                            <Mail className="w-5 h-5" />
                        </div>
                        <div>
                            <h1 className="text-base sm:text-lg font-bold text-gray-900 dark:text-base-content leading-tight">
                                {t('nav.email', 'Email & Alerts')}
                            </h1>
                            <p className="text-xs text-gray-500 dark:text-gray-400">
                                {t('email.subtitle', 'SMTP/IMAP telemetry, automated notification triggers, and mailbox control')}
                            </p>
                        </div>
                    </div>

                    <Link
                        to="/settings"
                        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white bg-gray-100 dark:bg-base-200 rounded-lg hover:bg-gray-200 dark:hover:bg-base-100 transition-colors shrink-0"
                    >
                        <ArrowLeft className="w-3.5 h-3.5" />
                        <span>{t('settings.title', 'Settings')}</span>
                    </Link>
                </div>

                {/* Summary Metrics & Connection Status Strip */}
                <div className="flex items-center gap-2 flex-wrap p-2 rounded-xl bg-white dark:bg-base-200 border border-gray-200/80 dark:border-base-100 shadow-2xs text-xs">
                    {/* 1. Node Name (Editable) */}
                    {isEditingNode ? (
                        <div className="flex items-center gap-1.5 bg-slate-100 dark:bg-slate-800/90 border border-emerald-500/80 px-2 py-1 rounded-lg text-xs shadow-xs">
                            <Cpu className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                            <span className="text-slate-500 font-medium">Node:</span>
                            <input
                                type="text"
                                value={nodeNameInput}
                                onChange={(e) => setNodeNameInput(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') handleSaveNodeName();
                                    if (e.key === 'Escape') setIsEditingNode(false);
                                }}
                                autoFocus
                                placeholder="Node name"
                                className="w-28 px-1.5 py-0 text-xs bg-white dark:bg-slate-900 border border-emerald-400 rounded text-slate-900 dark:text-slate-100 font-bold focus:outline-none"
                            />
                            <button
                                type="button"
                                onClick={handleSaveNodeName}
                                className="p-0.5 text-emerald-600 hover:bg-emerald-100 dark:hover:bg-emerald-950/50 rounded transition-colors cursor-pointer"
                                title="Save node name"
                            >
                                <Check className="w-3.5 h-3.5" />
                            </button>
                            <button
                                type="button"
                                onClick={() => setIsEditingNode(false)}
                                className="p-0.5 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-950/30 rounded transition-colors cursor-pointer"
                                title="Cancel"
                            >
                                <X className="w-3.5 h-3.5" />
                            </button>
                        </div>
                    ) : (
                        <div
                            onDoubleClick={handleStartEditNode}
                            className="bg-slate-100 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 px-2.5 py-1 rounded-lg font-medium flex items-center gap-1.5 cursor-pointer select-none hover:border-emerald-500/60 dark:hover:border-emerald-500/60 transition-colors group/node shadow-2xs"
                            title="Double-click to edit machine node name"
                        >
                            <Cpu className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                            <span>Node: <strong className="group-hover/node:underline">{watcherStatus?.machine_name || 'Detecting...'}</strong></span>
                        </div>
                    )}

                    {/* 2. Machine IP */}
                    <div className="bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-800 text-sky-700 dark:text-sky-300 px-2.5 py-1 rounded-lg font-mono flex items-center gap-1.5">
                        <Radio className="w-3.5 h-3.5 text-sky-500 shrink-0" />
                        <span>IP: <strong>{watcherStatus?.machine_ip || '127.0.0.1'}</strong></span>
                    </div>

                    {/* 3. Watchers Status */}
                    <div className={`px-2.5 py-1 rounded-lg font-medium flex items-center gap-1.5 border ${
                        isWatcherRunning
                            ? 'bg-emerald-50 dark:bg-emerald-950/40 border-emerald-200 dark:border-emerald-800 text-emerald-700 dark:text-emerald-300'
                            : 'bg-amber-50 dark:bg-amber-950/40 border-amber-200 dark:border-amber-800 text-amber-700 dark:text-amber-300'
                    }`}>
                        <Activity className="w-3.5 h-3.5 shrink-0" />
                        <span className="flex items-center gap-1">
                            <span className={`w-2 h-2 rounded-full shrink-0 ${isWatcherRunning ? 'bg-emerald-500 animate-pulse' : 'bg-amber-500'}`} />
                            <span>Watchers: <strong>{isWatcherRunning ? 'Active' : 'Standby'}</strong>{emailSettings?.polling_interval_minutes ? ` (${emailSettings.polling_interval_minutes}m)` : ''}</span>
                        </span>
                    </div>

                    {/* 4. IMAP Connection Status */}
                    <div className={`px-2.5 py-1 rounded-lg font-medium flex items-center gap-1.5 border ${
                        hasImap
                            ? 'bg-indigo-50 dark:bg-indigo-950/40 border-indigo-200 dark:border-indigo-800 text-indigo-700 dark:text-indigo-300'
                            : 'bg-gray-100 dark:bg-base-300 border-gray-200 dark:border-base-200 text-gray-500 dark:text-gray-400'
                    }`}>
                        <Server className="w-3.5 h-3.5 shrink-0" />
                        <span>
                            IMAP:{' '}
                            <strong>
                                {hasImap
                                    ? `${defaultAccount.imap_host}:${defaultAccount.imap_port}`
                                    : 'Not Configured'}
                            </strong>
                        </span>
                    </div>

                    {/* 5. SMTP Connection Status */}
                    <div className={`px-2.5 py-1 rounded-lg font-medium flex items-center gap-1.5 border ${
                        hasSmtp
                            ? 'bg-blue-50 dark:bg-blue-950/40 border-blue-200 dark:border-blue-800 text-blue-700 dark:text-blue-300'
                            : 'bg-gray-100 dark:bg-base-300 border-gray-200 dark:border-base-200 text-gray-500 dark:text-gray-400'
                    }`}>
                        <Send className="w-3.5 h-3.5 shrink-0" />
                        <span>
                            SMTP:{' '}
                            <strong>
                                {hasSmtp
                                    ? `${defaultAccount.smtp_host}:${defaultAccount.smtp_port}`
                                    : 'Not Configured'}
                            </strong>
                        </span>
                    </div>

                    {/* 6. Security Vault DB Status */}
                    <div className="bg-purple-50 dark:bg-purple-950/40 border border-purple-200 dark:border-purple-800 text-purple-700 dark:text-purple-300 px-2.5 py-1 rounded-lg font-medium flex items-center gap-1.5 ml-auto">
                        <ShieldCheck className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                        <span>
                            Vault: <strong>Encrypted Split-DB ({accounts.length} {accounts.length === 1 ? 'account' : 'accounts'})</strong>
                        </span>
                    </div>
                </div>

                {/* Email Notification & Vault Management Component */}
                <EmailNotificationSettings />
            </div>
        </div>
    );
}
