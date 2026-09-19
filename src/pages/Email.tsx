import { useEffect, useState } from 'react';
import { Mail, ArrowLeft, Cpu, Radio } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import EmailNotificationSettings from '../components/settings/EmailNotificationSettings';
import { getEmailWatcherStatus, WatcherStatus } from '../services/emailService';

/**
 * Email Management & Alerts Page
 *
 * Dedicated top-level view for SMTP/IMAP credentials, split security vault DB,
 * automated notification triggers, and remote mailbox control.
 */
export default function Email() {
    const { t } = useTranslation();
    const [watcherStatus, setWatcherStatus] = useState<WatcherStatus | null>(null);

    useEffect(() => {
        getEmailWatcherStatus()
            .then(setWatcherStatus)
            .catch(console.error);
    }, []);

    return (
        <div className="h-full w-full overflow-y-auto">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 py-3 space-y-3 animate-in fade-in duration-300">
                {/* Page Header with Integrated Telemetry in same line */}
                <div className="flex items-center justify-between gap-3 pb-2 border-b border-gray-200 dark:border-base-200 flex-wrap">
                    <div className="flex items-center gap-2.5 flex-wrap min-w-0">
                        <div className="p-1.5 rounded-lg bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 shrink-0">
                            <Mail className="w-4.5 h-4.5" />
                        </div>
                        <div className="flex items-center gap-2 flex-wrap">
                            <h1 className="text-base sm:text-lg font-bold text-gray-900 dark:text-base-content flex items-center gap-2">
                                <span>{t('nav.email', 'Email & Alerts')}</span>
                                <span className="text-[10px] uppercase font-semibold px-2 py-0.5 rounded-full bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300">
                                    Active
                                </span>
                            </h1>

                            {/* Telemetry badges placed inline beside Active tag */}
                            <div className="flex items-center gap-1.5 text-xs">
                                {watcherStatus?.machine_name && (
                                    <div className="bg-slate-100 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 text-slate-700 dark:text-slate-300 px-2 py-0.5 rounded-md font-medium text-[11px] flex items-center gap-1">
                                        <Cpu className="w-3 h-3 text-emerald-500" />
                                        <span>Node: <strong>{watcherStatus.machine_name}</strong></span>
                                    </div>
                                )}
                                {watcherStatus?.machine_ip && (
                                    <div className="bg-sky-50 dark:bg-sky-950/40 border border-sky-200 dark:border-sky-800 text-sky-700 dark:text-sky-300 px-2 py-0.5 rounded-md font-mono text-[11px] flex items-center gap-1">
                                        <Radio className="w-3 h-3 text-sky-500" />
                                        <span>IP: <strong>{watcherStatus.machine_ip}</strong></span>
                                    </div>
                                )}
                                <div className={`px-2 py-0.5 rounded-full text-[10px] font-semibold flex items-center gap-1 border ${
                                    watcherStatus?.is_running
                                        ? 'bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 dark:text-emerald-400 border-emerald-300 dark:border-emerald-800'
                                        : 'bg-amber-50 dark:bg-amber-950/30 text-amber-600 dark:text-amber-400 border-amber-300 dark:border-amber-800'
                                }`}>
                                    <span className={`w-1.5 h-1.5 rounded-full ${watcherStatus?.is_running ? 'bg-emerald-500 animate-pulse' : 'bg-amber-500'}`}></span>
                                    <span>{watcherStatus?.is_running ? 'Watcher Active' : 'Watcher Idle'}</span>
                                </div>
                            </div>
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

                {/* Email Notification & Vault Management Component */}
                <EmailNotificationSettings />
            </div>
        </div>
    );
}
