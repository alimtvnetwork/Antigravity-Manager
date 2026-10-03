import { Database } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import SupabaseSyncSettings from '../components/settings/SupabaseSyncSettings';

/**
 * Supabase Sync & Cluster Governance Page
 *
 * Dedicated top-level view for multi-endpoint database sync,
 * cluster connectivity health, workspace leases, cross-DB migrations,
 * and Telegram relay governance.
 */
export default function Supabase() {
    const { t } = useTranslation();

    return (
        <div className="h-full overflow-y-auto bg-slate-50/50 dark:bg-[#071a27] text-slate-900 dark:text-slate-100">
            <div className="max-w-[1400px] mx-auto px-4 sm:px-6 py-6 space-y-6">
                {/* Header Banner */}
                <div className="rounded-2xl border border-slate-200/80 dark:border-[#15334d] bg-white dark:bg-[#0c2438] p-5 shadow-xs transition-all">
                    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                        <div className="flex items-center gap-3.5">
                            <div className="w-12 h-12 rounded-2xl bg-emerald-500/10 dark:bg-emerald-500/15 border border-emerald-500/20 text-emerald-600 dark:text-emerald-400 flex items-center justify-center shadow-xs">
                                <Database className="w-6 h-6" />
                            </div>
                            <div>
                                <h1 className="text-xl font-bold text-slate-900 dark:text-white tracking-tight">
                                    {t('supabase.page_title', 'Supabase Sync & Cluster Governance')}
                                </h1>
                                <p className="text-xs text-slate-500 dark:text-slate-400 mt-1 max-w-2xl leading-relaxed">
                                    {t(
                                        'supabase.page_desc',
                                        'Orchestrate dual Lovable and Secondary database endpoints, verify cluster node connectivity, govern real-time workspace leases, perform cross-DB migrations, and configure Telegram relay.'
                                    )}
                                </p>
                            </div>
                        </div>

                        <div className="flex items-center gap-2 self-stretch sm:self-auto">
                            <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d] text-xs font-medium text-slate-700 dark:text-slate-300">
                                <span className="relative flex h-2 w-2">
                                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                                </span>
                                <span>{t('supabase.cluster_active', 'Cluster Sync Active')}</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Embedded Supabase Settings & Governance Component */}
                <div className="rounded-2xl border border-slate-200/80 dark:border-[#15334d] bg-white dark:bg-[#0c2438] p-6 shadow-xs">
                    <SupabaseSyncSettings />
                </div>
            </div>
        </div>
    );
}
