import { Mail, ArrowLeft } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Link } from 'react-router-dom';
import EmailNotificationSettings from '../components/settings/EmailNotificationSettings';

/**
 * Email Management & Alerts Page
 *
 * Dedicated top-level view for SMTP/IMAP credentials, split security vault DB,
 * automated notification triggers, and remote mailbox control.
 */
export default function Email() {
    const { t } = useTranslation();

    return (
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-6 animate-in fade-in duration-300">
            {/* Page Header */}
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-2 border-b border-gray-200 dark:border-base-200">
                <div className="flex items-center gap-3">
                    <div className="p-2.5 rounded-xl bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400">
                        <Mail className="w-6 h-6" />
                    </div>
                    <div>
                        <h1 className="text-xl font-bold text-gray-900 dark:text-base-content flex items-center gap-2">
                            {t('nav.email', 'Email & Alerts')}
                            <span className="text-[10px] uppercase font-semibold px-2 py-0.5 rounded-full bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300">
                                Active
                            </span>
                        </h1>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                            {t('email.page_desc', 'Configure notification dispatch, failover swapping pools, and remote mailbox control')}
                        </p>
                    </div>
                </div>

                <Link
                    to="/settings"
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white bg-gray-100 dark:bg-base-200 rounded-lg hover:bg-gray-200 dark:hover:bg-base-100 transition-colors self-start sm:self-auto"
                >
                    <ArrowLeft className="w-3.5 h-3.5" />
                    <span>{t('settings.title', 'Settings')}</span>
                </Link>
            </div>

            {/* Email Notification & Vault Management Component */}
            <div className="bg-white dark:bg-base-100 rounded-2xl shadow-sm border border-gray-100 dark:border-base-200 p-4 sm:p-6">
                <EmailNotificationSettings />
            </div>
        </div>
    );
}
