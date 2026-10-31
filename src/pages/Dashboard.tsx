import {
    AlertTriangle,
    ArrowRight,
    Ban,
    Bot,
    Download,
    RefreshCw,
    ShieldAlert,
    ShieldCheck,
    Sparkles,
    Users,
} from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import AddAccountDialog from '../components/accounts/AddAccountDialog';
import { UnifiedBackupModal } from '../components/modals/UnifiedBackupModal';
import { showToast } from '../components/common/ToastContainer';
import BestAccounts from '../components/dashboard/BestAccounts';
import CurrentAccount from '../components/dashboard/CurrentAccount';
import { useAccountStore } from '../stores/useAccountStore';
import { useDashboardStats } from './dashboard/useDashboardStats';

function Dashboard() {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const {
        accounts,
        currentAccount,
        fetchAccounts,
        fetchCurrentAccount,
        switchAccount,
        addAccount,
        refreshQuota,
        loading
    } = useAccountStore();

    useEffect(() => {
        fetchAccounts();
        fetchCurrentAccount();
    }, []);

    const [onlyAvailable, setOnlyAvailable] = useState(true);

    // Full-dimensional account health & quota calculation matrix
    const stats = useDashboardStats(accounts, onlyAvailable);


    const isSwitchingRef = useRef(false);

    const handleSwitch = async (accountId: string) => {
        if (loading || isSwitchingRef.current) return;

        isSwitchingRef.current = true;
        console.log('[Dashboard] handleSwitch called for', accountId);
        try {
            await switchAccount(accountId);
            showToast(t('dashboard.toast.switch_success'), 'success');
        } catch (error) {
            console.error('[Dashboard] Switch account failed:', error);
            showToast(`${t('dashboard.toast.switch_error')}: ${error}`, 'error');
        } finally {
            setTimeout(() => {
                isSwitchingRef.current = false;
            }, 1000);
        }
    };

    const handleAddAccount = async (email: string, refreshToken: string) => {
        await addAccount(email, refreshToken);
        await fetchAccounts(); // Refresh list
    };

    const [isRefreshing, setIsRefreshing] = useState(false);

    const handleRefreshCurrent = async () => {
        if (!currentAccount) return;

        setIsRefreshing(true);
        try {
            await refreshQuota(currentAccount.id);
            // Refresh successful, fetch latest data
            await fetchCurrentAccount();
            showToast(t('dashboard.toast.refresh_success'), 'success');
        } catch (error) {
            console.error('[Dashboard] Refresh failed:', error);
            showToast(`${t('dashboard.toast.refresh_error')}: ${error}`, 'error');
        } finally {
            setIsRefreshing(false);
        }
    };

    const [isBackupModalOpen, setIsBackupModalOpen] = useState(false);

    const handleExport = () => {
        setIsBackupModalOpen(true);
    };

    return (
        <div className="h-full w-full overflow-y-auto">
            <div
                className="px-4 sm:px-6 pt-2 pb-4 space-y-4 max-w-[1920px] mx-auto"
                onMouseMove={() => console.log('Mouse moving over Dashboard')}
                style={{ position: 'relative', zIndex: 1 }}
            >
                {/* Greetings and action buttons */}
                <div
                    className="flex justify-between items-center"
                >
                    <div>
                        <h1 className="text-2xl font-bold text-gray-900 dark:text-base-content">
                            {currentAccount
                                ? t('dashboard.hello').replace('user', currentAccount.name || currentAccount.email.split('@')[0])
                                : t('dashboard.hello')
                            }
                        </h1>
                    </div>
                    <div className="flex gap-2">
                        <AddAccountDialog onAdd={handleAddAccount} />
                        <button
                            className={`px-3 py-1.5 bg-blue-500 text-white text-xs font-medium rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-1.5 shadow-sm ${isRefreshing || !currentAccount ? 'opacity-70 cursor-not-allowed' : ''}`}
                            onClick={handleRefreshCurrent}
                            disabled={isRefreshing || !currentAccount}
                            title={isRefreshing ? t('dashboard.refreshing') : t('dashboard.refresh_quota')}
                        >
                            <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
                            <span className="hidden sm:inline">{isRefreshing ? t('dashboard.refreshing') : t('dashboard.refresh_quota')}</span>
                        </button>
                    </div>
                </div>

                {/* 1. Account ecological health status array (4 core cards, risk control and configuration determine life and death) */}
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
                    {/* Total number of accounts */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200">
                        <div className="flex items-center justify-between mb-2">
                            <div className="p-1.5 bg-blue-50 dark:bg-blue-900/20 rounded-md">
                                <Users className="w-4 h-4 text-blue-500 dark:text-blue-400" />
                            </div>
                            <span className="text-[10px] font-mono font-bold text-gray-400 dark:text-gray-500">TOTAL</span>
                        </div>
                        <div className="text-2xl font-bold text-gray-900 dark:text-base-content mb-0.5">{stats.total}</div>
                        <div className="text-xs text-gray-500 dark:text-gray-400 font-medium">
                            {t('dashboard.total_accounts', 'Total number of accounts')}
                        </div>
                        <div className="text-[10px] text-gray-400 dark:text-gray-500 mt-1.5">
                            {t('dashboard.total_accounts_desc', 'All accounts that have been entered into the configuration')}
                        </div>
                    </div>

                    {/* Number of available accounts (enabled and risk control is normal, life and death are not determined by quota) */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-emerald-100 dark:border-emerald-950/40">
                        <div className="flex items-center justify-between mb-2">
                            <div className="p-1.5 bg-emerald-50 dark:bg-emerald-900/20 rounded-md">
                                <ShieldCheck className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                            </div>
                            <span className="text-[10px] font-mono font-bold text-emerald-600 dark:text-emerald-400">ACTIVE</span>
                        </div>
                        <div className="text-2xl font-bold text-emerald-600 dark:text-emerald-400 mb-0.5">{stats.available}</div>
                        <div className="text-xs text-gray-700 dark:text-gray-300 font-bold">
                            {t('dashboard.available_accounts', 'Number of available accounts')}
                        </div>
                        <div className="text-[10px] text-emerald-600/80 dark:text-emerald-400/80 mt-1.5">
                            {t('dashboard.available_accounts_desc', '✓ Enabled and in normal status · Reverse generation service is in progress')}
                        </div>
                    </div>

                    {/* Number of disabled accounts (manually disabled) */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200">
                        <div className="flex items-center justify-between mb-2">
                            <div className="p-1.5 bg-gray-100 dark:bg-base-300 rounded-md">
                                <Ban className="w-4 h-4 text-gray-500 dark:text-gray-400" />
                            </div>
                            <span className="text-[10px] font-mono font-bold text-gray-400 dark:text-gray-500">DISABLED</span>
                        </div>
                        <div className="text-2xl font-bold text-gray-600 dark:text-gray-300 mb-0.5">{stats.disabled}</div>
                        <div className="text-xs text-gray-500 dark:text-gray-400 font-medium">
                            {t('dashboard.disabled_accounts', 'Number of disabled accounts')}
                        </div>
                        <div className="text-[10px] text-gray-400 dark:text-gray-500 mt-1.5">
                            {t('dashboard.disabled_accounts_desc', 'Manual deactivation / anti-generation disabled')}
                        </div>
                    </div>

                    {/* Number of abnormal accounts (risk control blocking or 403 ban) */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200">
                        <div className="flex items-center justify-between mb-2">
                            <div className={`p-1.5 rounded-md ${stats.abnormal > 0 ? 'bg-rose-50 dark:bg-rose-900/20' : 'bg-gray-50 dark:bg-base-300'}`}>
                                <ShieldAlert className={`w-4 h-4 ${stats.abnormal > 0 ? 'text-rose-600 dark:text-rose-400' : 'text-gray-400'}`} />
                            </div>
                            <span className={`text-[10px] font-mono font-bold ${stats.abnormal > 0 ? 'text-rose-500' : 'text-gray-400'}`}>RISK</span>
                        </div>
                        <div className={`text-2xl font-bold mb-0.5 ${stats.abnormal > 0 ? 'text-rose-600 dark:text-rose-400' : 'text-gray-900 dark:text-base-content'}`}>{stats.abnormal}</div>
                        <div className="text-xs text-gray-500 dark:text-gray-400 font-medium">
                            {t('dashboard.abnormal_accounts', 'Number of abnormal accounts')}
                        </div>
                        <div className={`text-[10px] mt-1.5 ${stats.abnormal > 0 ? 'text-rose-600 font-bold' : 'text-gray-400 dark:text-gray-500'}`}>
                            {stats.abnormal > 0
                                ? t('dashboard.abnormal_accounts_has_risk', '⚠ Requires risk control verification/banning')
                                : t('dashboard.abnormal_accounts_no_risk', '✓ Zero risk control abnormal accounts')
                            }
                        </div>
                    </div>
                </div>

                {/* 2. Quota productivity matrix and conspicuous dual-selection capsule controller (Pill Capsule Control) */}
                <div className="flex flex-wrap items-center justify-between gap-2 pt-1 pb-1">
                    <div className="flex items-center gap-2">
                        <span className="text-xs font-bold text-gray-800 dark:text-gray-200">
                            {onlyAvailable
                                ? t('dashboard.quota_matrix_active', 'Available account quota productivity matrix')
                                : t('dashboard.quota_matrix_all_normal', 'Full normal account quota matrix (including disabled ones)')
                            }
                        </span>
                        <span className="text-[10px] text-gray-400 dark:text-gray-500 font-mono">
                            ({t('dashboard.base_pool_count', { count: stats.basePoolCount })})
                        </span>
                    </div>

                    {/* Conspicuous dual-select capsule controller (Pill Capsule) */}
                    <div className="flex items-center gap-1 bg-gray-200/80 dark:bg-base-300 p-1 rounded-full shadow-inner border border-gray-200/80 dark:border-base-200 select-none">
                        <button
                            type="button"
                            onClick={() => setOnlyAvailable(true)}
                            className={`px-3.5 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 cursor-pointer ${
                                onlyAvailable
                                    ? 'bg-emerald-600 text-white shadow-md scale-100 ring-2 ring-emerald-400/30'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-black/5 dark:hover:bg-white/5'
                            }`}
                            title={t('dashboard.btn_title_only_available', 'Current: Only view open and normal available accounts (click to include disabled accounts)')}
                        >
                            <ShieldCheck className="w-3.5 h-3.5" />
                            <span>
                                {t('dashboard.btn_only_available', { count: stats.available })}
                            </span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setOnlyAvailable(false)}
                            className={`px-3.5 py-1.5 rounded-full text-xs font-bold transition-all flex items-center gap-1.5 cursor-pointer ${
                                !onlyAvailable
                                    ? 'bg-blue-600 text-white shadow-md scale-100 ring-2 ring-blue-400/30'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 hover:bg-black/5 dark:hover:bg-white/5'
                            }`}
                            title={t('dashboard.btn_title_include_disabled', 'Current: View all normal status accounts including disabled ones (click to view only available accounts)')}
                        >
                            <Users className="w-3.5 h-3.5" />
                            <span>
                                {t('dashboard.btn_include_disabled', { count: stats.normalCount })}
                            </span>
                        </button>
                    </div>
                </div>

                {/* 3 large model cards: display 5H average, weekly quota average and weekly quota circuit breaker weighted quota */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                    {/* Gemini text model quota */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-col justify-between">
                        <div>
                            <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                    <div className="p-1.5 bg-green-50 dark:bg-green-900/20 rounded-md">
                                        <Sparkles className="w-4 h-4 text-green-500 dark:text-green-400" />
                                    </div>
                                    <span className="text-xs font-bold text-gray-800 dark:text-gray-200">
                                        {t('dashboard.gemini_available_quota', 'Gemini available quota')}
                                    </span>
                                </div>
                                <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${stats.gemini.weightedEffective >= 50 ? 'bg-green-50 text-green-700 dark:bg-green-900/30 dark:text-green-300' : 'bg-amber-50 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300'}`}>
                                    {stats.gemini.weightedEffective >= 50
                                        ? t('dashboard.quota_sufficient_short', 'adequate')
                                        : t('dashboard.quota_tight_short', 'Tight')
                                    }
                                </span>
                            </div>

                            <div className="flex items-baseline gap-2 mb-2">
                                <span className="text-3xl font-extrabold text-gray-900 dark:text-base-content font-mono">
                                    {stats.gemini.weightedEffective}%
                                </span>
                                <span className="text-[11px] text-gray-400 dark:text-gray-500 font-medium">
                                    {t('dashboard.weighted_available', 'Comprehensive weighting available')}
                                </span>
                            </div>
                        </div>

                        <div className="pt-2 border-t border-gray-100 dark:border-base-300/60 flex items-center justify-between text-[11px]">
                            <div className="flex items-center gap-1.5">
                                <span className="text-gray-400 dark:text-gray-500">{t('dashboard.rolling_5h', '5 hours rolling:')}</span>
                                <span className="font-mono font-bold text-emerald-600 dark:text-emerald-400">{stats.gemini.avg5h}%</span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <span className="text-gray-400 dark:text-gray-500">{t('dashboard.weekly_7d', '7-day weekly quota:')}</span>
                                <span className={`font-mono font-bold ${stats.gemini.avgWeekly <= 10 ? 'text-amber-600 dark:text-amber-400' : 'text-gray-700 dark:text-gray-300'}`}>{stats.gemini.avgWeekly}%</span>
                            </div>
                        </div>
                        {stats.gemini.zeroWeeklyCount > 0 && (
                            <div className="text-[10px] text-amber-600 dark:text-amber-400 mt-1.5 font-medium flex items-center gap-1">
                                <AlertTriangle className="w-3 h-3 shrink-0" />
                                <span>{t('dashboard.zero_weekly_warning', { count: stats.gemini.zeroWeeklyCount })}</span>
                            </div>
                        )}
                    </div>

                    {/* Gemini drawing model quota */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-col justify-between">
                        <div>
                            <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                    <div className="p-1.5 bg-purple-50 dark:bg-purple-900/20 rounded-md">
                                        <Sparkles className="w-4 h-4 text-purple-500 dark:text-purple-400" />
                                    </div>
                                    <span className="text-xs font-bold text-gray-800 dark:text-gray-200">
                                        {t('dashboard.gemini_image_quota', 'Gemini drawing quota')}
                                    </span>
                                </div>
                                <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${stats.geminiImage.weightedEffective >= 50 ? 'bg-purple-50 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300' : 'bg-amber-50 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300'}`}>
                                    {stats.geminiImage.weightedEffective >= 50
                                        ? t('dashboard.quota_sufficient_short', 'adequate')
                                        : t('dashboard.quota_tight_short', 'Tight')
                                    }
                                </span>
                            </div>

                            <div className="flex items-baseline gap-2 mb-2">
                                <span className="text-3xl font-extrabold text-gray-900 dark:text-base-content font-mono">
                                    {stats.geminiImage.weightedEffective}%
                                </span>
                                <span className="text-[11px] text-gray-400 dark:text-gray-500 font-medium">
                                    {t('dashboard.weighted_available', 'Comprehensive weighting available')}
                                </span>
                            </div>
                        </div>

                        <div className="pt-2 border-t border-gray-100 dark:border-base-300/60 flex items-center justify-between text-[11px]">
                            <div className="flex items-center gap-1.5">
                                <span className="text-gray-400 dark:text-gray-500">{t('dashboard.rolling_5h', '5 hours rolling:')}</span>
                                <span className="font-mono font-bold text-emerald-600 dark:text-emerald-400">{stats.geminiImage.avg5h}%</span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <span className="text-gray-400 dark:text-gray-500">{t('dashboard.weekly_7d', '7-day weekly quota:')}</span>
                                <span className={`font-mono font-bold ${stats.geminiImage.avgWeekly <= 10 ? 'text-amber-600 dark:text-amber-400' : 'text-gray-700 dark:text-gray-300'}`}>{stats.geminiImage.avgWeekly}%</span>
                            </div>
                        </div>
                        {stats.geminiImage.zeroWeeklyCount > 0 && (
                            <div className="text-[10px] text-amber-600 dark:text-amber-400 mt-1.5 font-medium flex items-center gap-1">
                                <AlertTriangle className="w-3 h-3 shrink-0" />
                                <span>{t('dashboard.zero_weekly_warning', { count: stats.geminiImage.zeroWeeklyCount })}</span>
                            </div>
                        )}
                    </div>

                    {/* Claude model quota */}
                    <div className="bg-white dark:bg-base-100 rounded-xl p-4 shadow-sm border border-gray-100 dark:border-base-200 flex flex-col justify-between">
                        <div>
                            <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-2">
                                    <div className="p-1.5 bg-cyan-50 dark:bg-cyan-900/20 rounded-md">
                                        <Bot className="w-4 h-4 text-cyan-500 dark:text-cyan-400" />
                                    </div>
                                    <span className="text-xs font-bold text-gray-800 dark:text-gray-200">
                                        {t('dashboard.claude_available_quota', 'Claude available quota')}
                                    </span>
                                </div>
                                <span className={`px-2 py-0.5 rounded text-[10px] font-bold ${stats.claude.weightedEffective >= 50 ? 'bg-cyan-50 text-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-300' : 'bg-amber-50 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300'}`}>
                                    {stats.claude.weightedEffective >= 50
                                        ? t('dashboard.quota_sufficient_short', 'adequate')
                                        : t('dashboard.quota_tight_short', 'Tight')
                                    }
                                </span>
                            </div>

                            <div className="flex items-baseline gap-2 mb-2">
                                <span className="text-3xl font-extrabold text-gray-900 dark:text-base-content font-mono">
                                    {stats.claude.weightedEffective}%
                                </span>
                                <span className="text-[11px] text-gray-400 dark:text-gray-500 font-medium">
                                    {t('dashboard.weighted_available', 'Comprehensive weighting available')}
                                </span>
                            </div>
                        </div>

                        <div className="pt-2 border-t border-gray-100 dark:border-base-300/60 flex items-center justify-between text-[11px]">
                            <div className="flex items-center gap-1.5">
                                <span className="text-gray-400 dark:text-gray-500">{t('dashboard.rolling_5h', '5 hours rolling:')}</span>
                                <span className="font-mono font-bold text-emerald-600 dark:text-emerald-400">{stats.claude.avg5h}%</span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <span className="text-gray-400 dark:text-gray-500">{t('dashboard.weekly_7d', '7-day weekly quota:')}</span>
                                <span className={`font-mono font-bold ${stats.claude.avgWeekly <= 10 ? 'text-amber-600 dark:text-amber-400' : 'text-gray-700 dark:text-gray-300'}`}>{stats.claude.avgWeekly}%</span>
                            </div>
                        </div>
                        {stats.claude.zeroWeeklyCount > 0 && (
                            <div className="text-[10px] text-amber-600 dark:text-amber-400 mt-1.5 font-medium flex items-center gap-1">
                                <AlertTriangle className="w-3 h-3 shrink-0" />
                                <span>{t('dashboard.zero_weekly_warning', { count: stats.claude.zeroWeeklyCount })}</span>
                            </div>
                        )}
                    </div>
                </div>

                {/* Two column layout */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <CurrentAccount
                        account={currentAccount}
                        onSwitch={() => navigate('/accounts')}
                    />
                    <BestAccounts
                        accounts={accounts}
                        currentAccountId={currentAccount?.id}
                        onSwitch={handleSwitch}
                    />
                </div>

                {/* Quick links */}
                <div className="grid grid-cols-2 gap-3">
                    <button
                        className="bg-indigo-50 dark:bg-indigo-900/20 rounded-lg p-3 shadow-sm border border-indigo-100 dark:border-indigo-900/30 hover:border-indigo-300 dark:hover:border-indigo-700 hover:shadow-md transition-all flex items-center justify-between group"
                        onClick={() => navigate('/accounts')}
                    >
                        <span className="text-indigo-700 dark:text-indigo-300 font-medium text-sm">{t('dashboard.view_all_accounts')}</span>
                        <ArrowRight className="w-4 h-4 text-indigo-400 dark:text-indigo-500 group-hover:text-indigo-600 dark:group-hover:text-indigo-300 group-hover:translate-x-1 transition-all" />
                    </button>
                    <button
                        className="bg-purple-50 dark:bg-purple-900/20 rounded-lg p-3 shadow-sm border border-purple-100 dark:border-purple-900/30 hover:border-purple-300 dark:hover:border-purple-700 hover:shadow-md transition-all flex items-center justify-between group"
                        onClick={handleExport}
                    >
                        <span className="text-purple-700 dark:text-purple-300 font-medium text-sm">{t('dashboard.export_data')}</span>
                        <Download className="w-4 h-4 text-purple-400 dark:text-purple-500 group-hover:text-purple-600 dark:group-hover:text-purple-300 transition-all" />
                    </button>
                </div>
            </div>

            <UnifiedBackupModal
                isOpen={isBackupModalOpen}
                onClose={() => setIsBackupModalOpen(false)}
                initialTab="export"
            />
        </div>
    );
}

export default Dashboard;
