import { SettingsPageApi } from '../useSettingsPage';
import { Save, User, RefreshCw, ChevronDown } from 'lucide-react';
import { showToast } from '../components/common/ToastContainer';
import QuotaProtection from '../components/settings/QuotaProtection';
import SmartWarmup from '../components/settings/SmartWarmup';
import PinnedQuotaModels from '../components/settings/PinnedQuotaModels';
import AutoSwitcherSettings from '../components/settings/AutoSwitcherSettings';

export function AccountTab(props: SettingsPageApi) {
    const { t, config, saveConfig, formData, setFormData } = props;

    return (
                        <div className="space-y-4 animate-in fade-in duration-500">
                            {/* Auto refresh quota */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-blue-200 transition-all duration-300 shadow-sm">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-4">
                                        <div className="w-10 h-10 rounded-xl bg-blue-50 dark:bg-blue-900/20 flex items-center justify-center text-blue-500 group-hover:bg-blue-500 group-hover:text-white transition-all duration-300">
                                            <RefreshCw size={20} />
                                        </div>
                                        <div>
                                            <div className="font-bold text-gray-900 dark:text-gray-100">{t('settings.account.auto_refresh')}</div>
                                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{t('settings.account.auto_refresh_desc')}</p>
                                        </div>
                                    </div>
                                    <label className={`relative inline-flex items-center ${formData.quota_protection.enabled ? 'cursor-not-allowed opacity-80' : 'cursor-pointer'}`}>
                                        <input
                                            type="checkbox"
                                            className="sr-only peer"
                                            checked={formData.auto_refresh}
                                            disabled={formData.quota_protection.enabled}
                                            onChange={async (e) => {
                                                const enabled = e.target.checked;
                                                const newConfig = { ...formData, auto_refresh: enabled };
                                                setFormData(newConfig);
                                                // Hot Save
                                                try {
                                                    await saveConfig(newConfig);
                                                } catch (error) {
                                                    showToast(`${t('common.error')}: ${error}`, 'error');
                                                }
                                            }}
                                        />
                                        <div className={`w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500 shadow-inner ${formData.quota_protection.enabled ? 'peer-checked:bg-blue-500' : ''}`}></div>
                                    </label>
                                </div>

                                <div className="mt-5 pt-5 border-t border-gray-50 dark:border-base-300 flex items-center gap-4 animate-in slide-in-from-top-1 duration-200">
                                    <label className="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('settings.account.refresh_interval')}</label>
                                    <div className="relative">
                                        <input
                                            type="number"
                                            className="w-24 px-3 py-2 bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none text-sm font-bold text-blue-600 dark:text-blue-400"
                                            min="1"
                                            max="35791"
                                            value={formData.refresh_interval}
                                            onChange={(e) => setFormData({ ...formData, refresh_interval: isNaN(parseInt(e.target.value)) ? 1 : Math.min(Math.max(parseInt(e.target.value), 1), 35791) })}
                                        />
                                    </div>
                                </div>
                            </div>

                            {/* Auto sync current account */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-emerald-200 transition-all duration-300 shadow-sm">
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center gap-4">
                                        <div className="w-10 h-10 rounded-xl bg-emerald-50 dark:bg-emerald-900/20 flex items-center justify-center text-emerald-500 group-hover:bg-emerald-500 group-hover:text-white transition-all duration-300">
                                            <User size={20} />
                                        </div>
                                        <div>
                                            <div className="font-bold text-gray-900 dark:text-gray-100">{t('settings.account.auto_sync')}</div>
                                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{t('settings.account.auto_sync_desc')}</p>
                                        </div>
                                    </div>
                                    <label className="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            className="sr-only peer"
                                            checked={formData.auto_sync}
                                            onChange={(e) => setFormData({ ...formData, auto_sync: e.target.checked })}
                                        />
                                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-500 shadow-inner"></div>
                                    </label>
                                </div>

                                {formData.auto_sync && (
                                    <div className="mt-5 pt-5 border-t border-gray-50 dark:border-base-300 flex items-center gap-4 animate-in slide-in-from-top-1 duration-200">
                                        <label className="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">{t('settings.account.sync_interval')}</label>
                                        <input
                                            type="number"
                                            className="w-24 px-3 py-2 bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300 rounded-lg focus:ring-2 focus:ring-emerald-500 outline-none text-sm font-bold text-emerald-600 dark:text-emerald-400"
                                            min="1"
                                            max="35791"
                                            value={formData.sync_interval}
                                            onChange={(e) => setFormData({ ...formData, sync_interval: isNaN(parseInt(e.target.value)) ? 1 : Math.min(Math.max(parseInt(e.target.value), 1), 35791) })}
                                        />
                                    </div>
                                )}
                            </div>

                            {/* Smart Warmup */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-orange-200 transition-all duration-300 shadow-sm">
                                <SmartWarmup
                                    config={formData.scheduled_warmup}
                                    onChange={async (newConfig) => {
                                        const newFormData = {
                                            ...formData,
                                            scheduled_warmup: newConfig
                                        };
                                        setFormData(newFormData);
                                        // Hot Save
                                        try {
                                            await saveConfig(newFormData);
                                        } catch (error) {
                                            showToast(`${t('common.error')}: ${error}`, 'error');
                                        }
                                    }}
                                />
                            </div>

                            {/* Auto Profile Switcher */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-blue-200 transition-all duration-300 shadow-sm space-y-4">
                                <AutoSwitcherSettings
                                    config={formData.auto_profile_switcher}
                                    onChange={async (newConfig) => {
                                        const mins = newConfig.account_cooldown_minutes ?? newConfig.account_lockout_window_minutes ?? 60;
                                        const syncedConfig = {
                                            ...newConfig,
                                            account_cooldown_minutes: mins,
                                            account_lockout_window_minutes: mins,
                                        };
                                        const newFormData = {
                                            ...formData,
                                            auto_profile_switcher: syncedConfig
                                        };
                                        setFormData(newFormData);
                                        try {
                                            await saveConfig(newFormData);
                                        } catch (error) {
                                            showToast(`${t('common.error')}: ${error}`, 'error');
                                        }
                                    }}
                                />

                                {/* Account Lockout Window Setting */}
                                <div className="pt-4 border-t border-slate-100 dark:border-slate-800 flex flex-col sm:flex-row sm:items-center justify-between gap-3">
                                    <div className="space-y-0.5">
                                        <div className="text-xs font-semibold text-slate-800 dark:text-slate-200">
                                            {t('settings.auto_switcher.account_lockout_window', 'Account Lockout Window')}
                                        </div>
                                        <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                            {t('settings.auto_switcher.account_lockout_window_desc', 'Duration an account remains temporarily locked out after quota exhaustion or rate limits before re-evaluation.')}
                                        </p>
                                    </div>
                                    <div className="flex flex-col items-end gap-1.5 shrink-0">
                                        <div className="relative">
                                            <select
                                                value={(formData.auto_profile_switcher as any)?.account_cooldown_minutes ?? (formData.auto_profile_switcher as any)?.account_lockout_window_minutes ?? 60}
                                                onChange={async (e) => {
                                                    const minutes = Number(e.target.value);
                                                    const newConfig = {
                                                        ...(formData.auto_profile_switcher || {}),
                                                        account_lockout_window_minutes: minutes,
                                                        account_cooldown_minutes: minutes,
                                                    };
                                                    const newFormData = {
                                                        ...formData,
                                                        auto_profile_switcher: newConfig as any,
                                                    };
                                                    setFormData(newFormData);
                                                    try {
                                                        await saveConfig(newFormData);
                                                    } catch (error) {
                                                        showToast(`${t('common.error')}: ${error}`, 'error');
                                                    }
                                                }}
                                                className="appearance-none px-3 py-1.5 pr-8 bg-slate-50 dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] rounded-[5px] text-xs font-medium text-slate-800 dark:text-slate-200 shadow-2xs focus:outline-none focus:ring-1 focus:ring-blue-500/40 cursor-pointer"
                                            >
                                                <option value={15}>{t('settings.auto_switcher.lockout_15m', '15 minutes')}</option>
                                                <option value={30}>{t('settings.auto_switcher.lockout_30m', '30 minutes')}</option>
                                                <option value={45}>{t('settings.auto_switcher.lockout_45m', '45 minutes')}</option>
                                                <option value={60}>{t('settings.auto_switcher.lockout_60m', '60 minutes (Default)')}</option>
                                                <option value={120}>{t('settings.auto_switcher.lockout_120m', '120 minutes')}</option>
                                            </select>
                                            <ChevronDown className="w-3.5 h-3.5 text-slate-400 pointer-events-none absolute right-2.5 top-2.5" />
                                        </div>
                                        <div className="flex gap-1 flex-wrap">
                                            {[15, 30, 45, 60, 120].map((mins) => {
                                                const currentMins = (formData.auto_profile_switcher as any)?.account_cooldown_minutes ?? (formData.auto_profile_switcher as any)?.account_lockout_window_minutes ?? 60;
                                                return (
                                                    <button
                                                        key={mins}
                                                        type="button"
                                                        onClick={async () => {
                                                            const newConfig = {
                                                                ...(formData.auto_profile_switcher || {}),
                                                                account_lockout_window_minutes: mins,
                                                                account_cooldown_minutes: mins,
                                                            };
                                                            const newFormData = {
                                                                ...formData,
                                                                auto_profile_switcher: newConfig as any,
                                                            };
                                                            setFormData(newFormData);
                                                            try {
                                                                await saveConfig(newFormData);
                                                            } catch (error) {
                                                                showToast(`${t('common.error')}: ${error}`, 'error');
                                                            }
                                                        }}
                                                        className={`px-1.5 py-0.5 text-[10px] font-mono font-medium rounded-[5px] border transition-all cursor-pointer ${
                                                            currentMins === mins
                                                                ? 'bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400 border-indigo-300 dark:border-indigo-800 shadow-xs'
                                                                : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-slate-300'
                                                        }`}
                                                    >
                                                        {mins}m
                                                    </button>
                                                );
                                            })}
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {/* Quota Protection */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-rose-200 transition-all duration-300 shadow-sm">
                                <QuotaProtection
                                    config={formData.quota_protection}
                                    onChange={async (newConfig) => {
                                        const updates: any = {
                                            quota_protection: newConfig
                                        };
                                        // When quota protection is enabled, enforce background auto-refresh
                                        if (newConfig.enabled) {
                                            updates.auto_refresh = true;
                                        }

                                        const newFormData = {
                                            ...formData,
                                            ...updates
                                        };
                                        setFormData(newFormData);

                                        // Hot Save
                                        try {
                                            await saveConfig(newFormData);
                                        } catch (error) {
                                            showToast(`${t('common.error')}: ${error}`, 'error');
                                        }
                                    }}
                                />
                            </div>

                            {/* Pinned Quota Models */}
                            <div className="group bg-white dark:bg-base-100 rounded-xl p-5 border border-gray-100 dark:border-base-200 hover:border-indigo-200 transition-all duration-300 shadow-sm">
                                <PinnedQuotaModels
                                    config={formData.pinned_quota_models}
                                    onChange={(newConfig) => setFormData({
                                        ...formData,
                                        pinned_quota_models: newConfig
                                    })}
                                />
                            </div>
                        </div>
    );
}
