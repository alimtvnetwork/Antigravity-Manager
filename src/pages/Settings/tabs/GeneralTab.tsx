import { SettingsPageApi } from '../useSettingsPage';
import { startTransition } from 'react';
import { LayoutDashboard, Users, Network, Activity, BarChart3, Settings as SettingsIcon, Lock, CheckCircle2, Palette } from 'lucide-react';
import { request as invoke } from '../../../utils/request';
import { showToast } from '../../../components/common/ToastContainer';
import { isTauri } from '../../../utils/env';
import { DateTimeFormatSection } from '../../../components/settings/GeneralSettingsTab';
import { findPalette } from '../../../components/common/themePalettes';

export function GeneralTab(props: SettingsPageApi) {
    const { t, i18n, saveConfig, updateLanguage, updateTheme, formData, setFormData, saveUpdateSettingsHelper, setActiveTab } = props;
    const activeThemeId = formData.theme || 'dark';
    const activePalette = findPalette(activeThemeId);

    return (
        <div className="space-y-4">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-slate-100">{t('settings.general.title')}</h2>

            {/* 1. General Preferences & Appearance Card */}
            <div className="rounded-xl border border-gray-200/70 dark:border-[#40285e]/60 bg-white/60 dark:bg-[#171025]/60 backdrop-blur-md p-4.5 space-y-3.5">
                {/* Language selection */}
                <div>
                    <label className="block text-sm font-medium text-gray-900 dark:text-slate-100 mb-2">{t('settings.general.language')}</label>
                    <select
                        className="w-full h-10 px-3 py-1.5 text-sm border border-gray-200/70 dark:border-[#40285e]/60 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900 dark:text-slate-100 bg-white/80 dark:bg-[#171025]/80"
                        value={formData.language}
                        onChange={(e) => {
                            const newLang = e.target.value;
                            setFormData({ ...formData, language: newLang });
                            document.documentElement.dir = newLang === 'ar' ? 'rtl' : 'ltr';
                            startTransition(() => {
                                i18n.changeLanguage(newLang);
                            });
                            updateLanguage(newLang);
                        }}
                    >
                        <option value="zh">Simplified Chinese</option>
                        <option value="zh-TW">Traditional Chinese</option>
                        <option value="en">English</option>
                        <option value="ja">Japanese</option>
                        <option value="tr">Türkçe</option>
                        <option value="vi">Tiếng Việt</option>
                        <option value="pt">Português</option>
                        <option value="ko">한국어</option>
                        <option value="ru">Русский</option>
                        <option value="ar">العربية</option>
                    </select>
                </div>

                {/* Theme selection with Active Swatch & Quick Theme Picker */}
                <div className="border-t border-gray-200/50 dark:border-[#40285e]/40 pt-3.5 space-y-2.5">
                    <div className="flex items-center justify-between gap-2">
                        <label className="block text-sm font-medium text-gray-900 dark:text-slate-100">{t('settings.general.theme')}</label>
                        <button
                            type="button"
                            onClick={() => setActiveTab('themes')}
                            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg border border-blue-500/30 bg-blue-50/60 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 hover:bg-blue-100/60 dark:hover:bg-blue-900/30 text-xs font-semibold cursor-pointer transition-colors"
                            title="Open full 14-palette Theme Catalogue"
                        >
                            <Palette className="w-3.5 h-3.5 text-blue-500 shrink-0" />
                            <span>Theme Catalogue (14 Palettes) ↗</span>
                        </button>
                    </div>

                    <div className="flex flex-wrap items-center gap-3">
                        <div className="inline-flex items-center rounded-full border border-gray-200/70 dark:border-[#40285e]/60 bg-gray-50/80 dark:bg-[#1a122c]/60 p-0.5 divide-x divide-gray-200/70 dark:divide-[#40285e]/60">
                            {(
                                [
                                    { value: 'light', label: t('settings.general.theme_light') },
                                    { value: 'dark', label: t('settings.general.theme_dark') },
                                    { value: 'system', label: t('settings.general.theme_system') },
                                ] as const
                            ).map((opt) => (
                                <button
                                    key={opt.value}
                                    type="button"
                                    onClick={() => {
                                        setFormData({ ...formData, theme: opt.value });
                                        updateTheme(opt.value);
                                    }}
                                    className={`px-3.5 py-1.5 rounded-full text-xs font-medium transition-all cursor-pointer ${formData.theme === opt.value
                                        ? 'bg-blue-500 text-white shadow'
                                        : 'text-slate-600 dark:text-slate-300 hover:text-gray-900 dark:hover:text-white'
                                        }`}
                                >
                                    {opt.label}
                                </button>
                            ))}
                        </div>

                        {/* Active Theme Swatch Preview */}
                        <div className="flex items-center gap-2 px-2.5 py-1 rounded-lg border border-gray-200/70 dark:border-[#40285e]/60 bg-white/70 dark:bg-[#1a122c]/70 shadow-2xs">
                            <div
                                className="w-4 h-4 rounded-[4px] border border-black/15 dark:border-white/20 shrink-0"
                                style={{
                                    background: `linear-gradient(135deg, ${activePalette.bg} 50%, ${activePalette.primary} 50%)`,
                                }}
                            />
                            <span className="text-xs font-semibold text-gray-800 dark:text-slate-200">
                                {activePalette.label}
                            </span>
                            <span className="text-[10px] font-mono text-slate-500 dark:text-slate-400">
                                {activePalette.primary}
                            </span>
                        </div>
                    </div>
                </div>

                {/* Date & Time Format */}
                <div className="border-t border-gray-200/50 dark:border-[#40285e]/40 pt-3.5">
                    <DateTimeFormatSection
                        value={formData.date_format_mode}
                        onChange={(mode) => setFormData({ ...formData, date_format_mode: mode, time_format_preference: mode })}
                    />
                </div>
            </div>

            {/* 2. Startup, Updates & Notifications Card */}
            <div className="rounded-xl border border-gray-200/70 dark:border-[#40285e]/60 bg-white/60 dark:bg-[#171025]/60 backdrop-blur-md p-4.5 space-y-3.5">
                {/* Auto launch on system startup */}
                <div className="flex items-center justify-between p-3.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                    <div>
                        <div className="font-medium text-sm text-gray-900 dark:text-slate-100">
                            {t('settings.general.auto_launch')}
                            {!isTauri() && (
                                <span className="ml-2 text-xs text-orange-500 dark:text-orange-400">
                                    {t('settings.web_mode_limitation', '(Not supported in Web mode)')}
                                </span>
                            )}
                        </div>
                        <p className="text-xs text-slate-600 dark:text-slate-300 mt-1">{t('settings.general.auto_launch_desc')}</p>
                    </div>
                    <label className={`relative inline-flex items-center ${isTauri() ? 'cursor-pointer' : 'cursor-not-allowed opacity-50'}`}>
                        <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={formData.auto_launch ?? false}
                            disabled={!isTauri()}
                            onChange={async (e) => {
                                const enabled = e.target.checked;
                                try {
                                    await invoke('toggle_auto_launch', { enable: enabled });
                                    setFormData({ ...formData, auto_launch: enabled });
                                    showToast(enabled ? t('settings.general.auto_launch_enabled') : t('settings.general.auto_launch_disabled'), 'success');
                                } catch (error) {
                                    showToast(`${t('common.error')}: ${error}`, 'error');
                                }
                            }}
                        />
                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                    </label>
                </div>

                {/* Auto check for updates */}
                <div className="flex items-center justify-between p-3.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                    <div>
                        <div className="font-medium text-sm text-gray-900 dark:text-slate-100">{t('settings.general.auto_check_update')}</div>
                        <p className="text-xs text-slate-600 dark:text-slate-300 mt-1">{t('settings.general.auto_check_update_desc')}</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={formData.auto_check_update ?? true}
                            onChange={async (e) => {
                                const enabled = e.target.checked;
                                try {
                                    await saveUpdateSettingsHelper({ auto_check: enabled });
                                    setFormData({ ...formData, auto_check_update: enabled });
                                    showToast(enabled ? t('settings.general.auto_check_update_enabled') : t('settings.general.auto_check_update_disabled'), 'success');
                                } catch (error) {
                                    showToast(`${t('common.error')}: ${error}`, 'error');
                                }
                            }}
                        />
                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                    </label>
                </div>

                {/* Check interval */}
                {formData.auto_check_update && (
                    <div className="ml-4 pl-4 border-l-2 border-blue-500/30 space-y-1.5">
                        <label className="block text-sm font-medium text-gray-900 dark:text-slate-100 mb-1">{t('settings.general.update_check_interval')}</label>
                        <input
                            type="number"
                            className="w-32 h-9 px-3 py-1.5 text-sm border border-gray-200/70 dark:border-[#40285e]/60 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900 dark:text-slate-100 bg-white/80 dark:bg-[#171025]/80"
                            min="1"
                            max="168"
                            value={formData.update_check_interval ?? 24}
                            onChange={(e) => setFormData({ ...formData, update_check_interval: parseInt(e.target.value) })}
                            onBlur={async () => {
                                try {
                                    await saveUpdateSettingsHelper({ check_interval_hours: formData.update_check_interval ?? 24 });
                                    showToast(t('settings.general.update_check_interval_saved'), 'success');
                                } catch (error) {
                                    showToast(`${t('common.error')}: ${error}`, 'error');
                                }
                            }}
                        />
                        <p className="text-xs text-slate-600 dark:text-slate-300 mt-1">{t('settings.general.update_check_interval_desc')}</p>
                    </div>
                )}

                {/* System Update Notifications */}
                <div className="flex items-center justify-between p-3.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                    <div>
                        <div className="font-medium text-sm text-gray-900 dark:text-slate-100">{t('settings.general.notify_on_update')}</div>
                        <p className="text-xs text-slate-600 dark:text-slate-300 mt-1">{t('settings.general.notify_on_update_desc')}</p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={formData.notify_on_update ?? true}
                            onChange={async (e) => {
                                const enabled = e.target.checked;
                                try {
                                    await saveUpdateSettingsHelper({ notify_on_update: enabled });
                                    setFormData({ ...formData, notify_on_update: enabled });
                                    showToast(enabled ? t('settings.general.notify_on_update_enabled') : t('settings.general.notify_on_update_disabled'), 'success');
                                } catch (error) {
                                    showToast(`${t('common.error')}: ${error}`, 'error');
                                }
                            }}
                        />
                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                    </label>
                </div>

                {/* Channel Sub-toggles when notify_on_update is active */}
                {(formData.notify_on_update ?? true) && (
                    <div className="ml-4 pl-4 border-l-2 border-blue-500/30 space-y-2.5">
                        <div className="flex items-center justify-between p-2.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                            <div>
                                <div className="text-sm font-medium text-gray-900 dark:text-slate-100">{t('settings.general.notify_via_email')}</div>
                                <p className="text-xs text-slate-600 dark:text-slate-300 mt-0.5">{t('settings.general.notify_via_email_desc')}</p>
                            </div>
                            <label className="relative inline-flex items-center cursor-pointer">
                                <input
                                    type="checkbox"
                                    className="sr-only peer"
                                    checked={formData.notify_via_email ?? true}
                                    onChange={async (e) => {
                                        const enabled = e.target.checked;
                                        try {
                                            await saveUpdateSettingsHelper({ notify_via_email: enabled });
                                            setFormData({ ...formData, notify_via_email: enabled });
                                        } catch (error) {
                                            showToast(`${t('common.error')}: ${error}`, 'error');
                                        }
                                    }}
                                />
                                <div className="w-9 h-5 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-500"></div>
                            </label>
                        </div>

                        <div className="flex items-center justify-between p-2.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                            <div>
                                <div className="text-sm font-medium text-gray-900 dark:text-slate-100">{t('settings.general.notify_via_telegram')}</div>
                                <p className="text-xs text-slate-600 dark:text-slate-300 mt-0.5">{t('settings.general.notify_via_telegram_desc')}</p>
                            </div>
                            <label className="relative inline-flex items-center cursor-pointer">
                                <input
                                    type="checkbox"
                                    className="sr-only peer"
                                    checked={formData.notify_via_telegram ?? true}
                                    onChange={async (e) => {
                                        const enabled = e.target.checked;
                                        try {
                                            await saveUpdateSettingsHelper({ notify_via_telegram: enabled });
                                            setFormData({ ...formData, notify_via_telegram: enabled });
                                        } catch (error) {
                                            showToast(`${t('common.error')}: ${error}`, 'error');
                                        }
                                    }}
                                />
                                <div className="w-9 h-5 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-500"></div>
                            </label>
                        </div>
                    </div>
                )}

                {/* Lightweight Mode (Release Memory) */}
                {isTauri() && (
                    <div className="flex items-center justify-between p-3.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                        <div>
                            <div className="font-medium text-sm text-gray-900 dark:text-slate-100">{t('settings.general.lightweight_mode')}</div>
                            <p className="text-xs text-slate-600 dark:text-slate-300 mt-1">{t('settings.general.lightweight_mode_desc')}</p>
                        </div>
                        <label className="relative inline-flex items-center cursor-pointer">
                            <input
                                type="checkbox"
                                className="sr-only peer"
                                checked={formData.lightweight_mode ?? false}
                                onChange={(e) => {
                                    const enabled = e.target.checked;
                                    setFormData({ ...formData, lightweight_mode: enabled });
                                }}
                            />
                            <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                        </label>
                    </div>
                )}
            </div>

            {/* 3. REST API Integrations Card */}
            <div className="rounded-xl border border-gray-200/70 dark:border-[#40285e]/60 bg-white/60 dark:bg-[#171025]/60 backdrop-blur-md p-4.5 space-y-3.5">
                {/* Remote Control REST API */}
                <div className="flex items-center justify-between p-3.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                    <div>
                        <div className="font-medium text-sm text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <span>Remote Control REST API</span>
                            <span className="text-[11px] px-2 py-0.5 rounded-full bg-emerald-100 dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-300 font-mono">
                                /api/v1/remote/control
                            </span>
                        </div>
                        <p className="text-xs text-slate-600 dark:text-slate-300 mt-1">
                            Allow remote REST clients to trigger Smart Rotator account switches, bind instances, and modify machine routing parameters.
                        </p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={formData.remote_control_api_enabled ?? true}
                            onChange={(e) => {
                                const enabled = e.target.checked;
                                setFormData({ ...formData, remote_control_api_enabled: enabled });
                            }}
                        />
                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                    </label>
                </div>

                {/* Machine Training REST API */}
                <div className="flex items-center justify-between p-3.5 bg-gray-50/70 dark:bg-[#1a122c]/50 rounded-lg border border-gray-200/50 dark:border-[#40285e]/40">
                    <div>
                        <div className="font-medium text-sm text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <span>Machine Training REST API</span>
                            <span className="text-[11px] px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-mono">
                                /api/v1/training
                            </span>
                        </div>
                        <p className="text-xs text-slate-600 dark:text-slate-300 mt-1">
                            Expose external REST API endpoints for seeking into node telemetry, ingesting reinforcement learning feedback, and remotely managing machine models.
                        </p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={formData.training_api_enabled ?? true}
                            onChange={(e) => {
                                const enabled = e.target.checked;
                                setFormData({ ...formData, training_api_enabled: enabled });
                            }}
                        />
                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                    </label>
                </div>
            </div>

            {/* 4. Menu Display Settings Card */}
            <div className="rounded-xl border border-gray-200/70 dark:border-[#40285e]/60 bg-white/60 dark:bg-[#171025]/60 backdrop-blur-md p-4.5 space-y-3.5">
                <div>
                    <h3 className="font-medium text-sm text-gray-900 dark:text-slate-100 mb-1">{t('settings.menu.title')}</h3>
                    <p className="text-xs text-slate-600 dark:text-slate-300">
                        {t('settings.menu.desc')}
                    </p>
                </div>
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
                    {[
                        { path: '/', label: t('nav.dashboard'), icon: LayoutDashboard },
                        { path: '/accounts', label: t('nav.accounts'), icon: Users },
                        { path: '/api-proxy', label: t('nav.proxy'), icon: Network },
                        { path: '/monitor', label: t('nav.call_records'), icon: Activity },
                        { path: '/token-stats', label: t('nav.token_stats'), icon: BarChart3 },
                        { path: '/user-token', label: t('nav.user_token', 'User Tokens'), icon: Users },
                        { path: '/security', label: t('nav.security'), icon: Lock },
                        { path: '/settings', label: t('nav.settings'), icon: SettingsIcon },
                    ].map((item) => {
                        const hiddenItems = formData.hidden_menu_items || [];
                        const isVisible = !hiddenItems.includes(item.path);
                        const isSettings = item.path === '/settings';

                        return (
                            <div
                                key={item.path}
                                onClick={async () => {
                                    if (!isSettings) {
                                        const originalConfig = { ...formData };
                                        const hiddenItems = formData.hidden_menu_items || [];
                                        const newHiddenItems = isVisible
                                            ? [...hiddenItems, item.path]
                                            : hiddenItems.filter(p => p !== item.path);

                                        // Optimistic UI update
                                        const newConfig = {
                                            ...formData,
                                            hidden_menu_items: newHiddenItems
                                        };
                                        setFormData(newConfig);

                                        // Attempt save
                                        try {
                                            await saveConfig(newConfig);
                                        } catch (error) {
                                            // Save failed, rollback to original snapshot
                                            setFormData(originalConfig);
                                            showToast(`Save failed, restored settings: ${error}`, 'error');
                                        }
                                    }
                                }}
                                className={`
                                    relative flex flex-col items-center justify-center gap-3 p-4 rounded-xl border-2 transition-all cursor-pointer select-none
                                    ${isSettings
                                        ? 'bg-gray-50/70 dark:bg-[#1a122c]/50 border-gray-200/50 dark:border-[#40285e]/40 opacity-60 cursor-not-allowed'
                                        : isVisible
                                            ? 'bg-blue-50/70 dark:bg-blue-900/20 border-blue-500 dark:border-blue-500 shadow-sm'
                                            : 'bg-white/70 dark:bg-[#1a122c]/40 border-gray-200/70 dark:border-[#40285e]/50 hover:border-gray-300 dark:hover:border-slate-500 text-slate-500'
                                    }
                                `}
                            >
                                {/* Selected checkmark */}
                                {isVisible && (
                                    <div className="absolute top-2 right-2 text-blue-500">
                                        <CheckCircle2 size={16} fill="currentColor" className="text-white dark:text-[#171025]" />
                                    </div>
                                )}

                                {isSettings && (
                                    <div className="absolute top-2 right-2 text-xs font-bold text-gray-400 bg-gray-200 dark:bg-base-300 px-1.5 py-0.5 rounded">
                                        {t('settings.menu.required')}
                                    </div>
                                )}

                                <div className={`
                                    p-3 rounded-xl transition-colors
                                    ${isVisible
                                        ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
                                        : 'bg-gray-100 dark:bg-base-200 text-gray-400 dark:text-slate-400'
                                    }
                                `}>
                                    <item.icon size={24} />
                                </div>

                                <span className={`font-medium text-sm ${isVisible ? 'text-blue-900 dark:text-blue-100' : 'text-slate-500 dark:text-slate-400'}`}>
                                    {item.label}
                                </span>
                            </div>
                        );
                    })}
                </div>
                <p className="text-xs text-slate-600 dark:text-slate-300 mt-3 flex items-center gap-1.5">
                    <span className="w-1.5 h-1.5 rounded-full bg-slate-400"></span>
                    {t('settings.menu.selected_items_note')}
                </p>
            </div>

            {/* 5. Instance Preferences Card */}
            <div className="rounded-xl border border-gray-200/70 dark:border-[#40285e]/60 bg-white/60 dark:bg-[#171025]/60 backdrop-blur-md p-4.5 space-y-3.5">
                {/* Instance clone mode settings */}
                <div>
                    <h3 className="font-medium text-sm text-gray-900 dark:text-slate-100 mb-1">
                        {t('settings.instance.clone_mode_title', 'Instance Duplication Mode')}
                    </h3>
                    <p className="text-xs text-slate-600 dark:text-slate-300 mb-2">
                        {t('settings.instance.clone_mode_desc', 'Choose whether cloning an instance copies the full directory or only profile configuration.')}
                    </p>
                    <select
                        className="w-full h-10 px-3 py-1.5 border border-gray-200/70 dark:border-[#40285e]/60 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900 dark:text-slate-100 bg-white/80 dark:bg-[#171025]/80 text-sm"
                        value={formData.instance_clone_mode || 'full'}
                        onChange={(e) => {
                            const mode = e.target.value as 'full' | 'profile';
                            setFormData({ ...formData, instance_clone_mode: mode });
                        }}
                    >
                        <option value="full">
                            {t('settings.instance.mode_full', 'Full Directory Copy (Default - complete settings, sessions, and extensions)')}
                        </option>
                        <option value="profile">
                            {t('settings.instance.mode_profile', 'Profile Only (Only User configuration and keybindings)')}
                        </option>
                    </select>
                </div>

                {/* Max projects on instance cards */}
                <div className="border-t border-gray-200/50 dark:border-[#40285e]/40 pt-3.5">
                    <h3 className="font-medium text-sm text-gray-900 dark:text-slate-100 mb-1">
                        {t('settings.instance.max_projects_title', 'Max Projects on Instance Cards')}
                    </h3>
                    <p className="text-xs text-slate-600 dark:text-slate-300 mb-2">
                        {t('settings.instance.max_projects_desc', 'Set the maximum number of recent or active projects displayed directly on each instance card (1 to 3).')}
                    </p>
                    <select
                        className="w-full h-10 px-3 py-1.5 border border-gray-200/70 dark:border-[#40285e]/60 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-gray-900 dark:text-slate-100 bg-white/80 dark:bg-[#171025]/80 text-sm"
                        value={formData.instance_card_max_projects ?? 3}
                        onChange={(e) => {
                            const count = parseInt(e.target.value, 10) || 3;
                            setFormData({ ...formData, instance_card_max_projects: count });
                        }}
                    >
                        <option value={1}>
                            1 {t('settings.instance.projects_count_1', 'Project')}
                        </option>
                        <option value={2}>
                            2 {t('settings.instance.projects_count_2', 'Projects')}
                        </option>
                        <option value={3}>
                            3 {t('settings.instance.projects_count_3', 'Projects (Default)')}
                        </option>
                    </select>
                </div>
            </div>
        </div>
    );
}
