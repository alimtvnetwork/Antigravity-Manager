import { SettingsPageApi } from '../useSettingsPage';
import { Github, User, Sparkles, ExternalLink, RefreshCw, Heart, CheckCircle2, Send } from 'lucide-react';
import { request as invoke } from '../../../utils/request';
import { showToast } from '../../../components/common/ToastContainer';

export function AboutTab(props: SettingsPageApi) {
    const { t, appVersion, formData, setFormData, setIsSupportModalOpen, isCheckingUpdate, updateInfo, setUpdateInfo, isInstallerUpdating, isBrewInstalled, isBrewUpgrading, setIsBrewConfirmOpen, handleCheckUpdate, handleRunInstallerUpdate } = props;

    return (
                        <div className="flex flex-col h-full animate-in fade-in duration-500 max-w-4xl space-y-4">
                            {/* Main Identity & Tech Card */}
                            <div className="p-5 rounded-2xl bg-white dark:bg-base-200 border border-gray-200/80 dark:border-base-100 shadow-xs flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                                <div className="flex items-center gap-3.5 min-w-0">
                                    <img
                                        src="/icon.png"
                                        alt="Antigravity Logo"
                                        className="w-12 h-12 rounded-xl object-cover bg-white dark:bg-black shadow-xs ring-1 ring-black/5 dark:ring-white/10 shrink-0"
                                    />
                                    <div className="min-w-0">
                                        <div className="flex items-center gap-2 flex-wrap">
                                            <h3 className="text-base font-bold text-gray-900 dark:text-base-content leading-tight">
                                                {t('common.app_name', 'Agm Tool By Alim')}
                                            </h3>
                                            <span className="px-2 py-0.5 rounded-full text-[11px] font-bold bg-gradient-to-r from-blue-600 to-indigo-600 text-white shadow-xs">
                                                v{appVersion}
                                            </span>
                                        </div>
                                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">
                                            {t('settings.branding.subtitle')}
                                        </p>
                                    </div>
                                </div>
                                <div className="flex items-center gap-1.5 flex-wrap self-start sm:self-center">
                                    <span className="px-2.5 py-1 rounded-lg text-[10px] font-semibold bg-gray-100 dark:bg-base-300 text-gray-600 dark:text-gray-300 border border-gray-200/70 dark:border-base-200">
                                        Tauri v2
                                    </span>
                                    <span className="px-2.5 py-1 rounded-lg text-[10px] font-semibold bg-gray-100 dark:bg-base-300 text-gray-600 dark:text-gray-300 border border-gray-200/70 dark:border-base-200">
                                        React 19
                                    </span>
                                    <span className="px-2.5 py-1 rounded-lg text-[10px] font-semibold bg-gray-100 dark:bg-base-300 text-gray-600 dark:text-gray-300 border border-gray-200/70 dark:border-base-200">
                                        Rust Core
                                    </span>
                                </div>
                            </div>

                            {/* Action Pills Row */}
                            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
                                <a
                                    href="https://alimkarim.com"
                                    target="_blank"
                                    rel="noreferrer"
                                    className="flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-gray-200/80 dark:border-base-100 bg-white dark:bg-base-200 text-xs font-medium text-gray-700 dark:text-gray-200 hover:border-blue-500 hover:text-blue-600 dark:hover:text-blue-400 hover:shadow-xs transition-all duration-200 group cursor-pointer"
                                >
                                    <User className="w-4 h-4 text-blue-500 group-hover:scale-110 transition-transform" />
                                    <span>Md. Alim Ul Karim</span>
                                </a>
                                <a
                                    href="https://t.me/AntigravityManager"
                                    target="_blank"
                                    rel="noreferrer"
                                    className="flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-gray-200/80 dark:border-base-100 bg-white dark:bg-base-200 text-xs font-medium text-gray-700 dark:text-gray-200 hover:border-sky-500 hover:text-sky-600 dark:hover:text-sky-400 hover:shadow-xs transition-all duration-200 group cursor-pointer"
                                >
                                    <Send className="w-4 h-4 text-sky-500 group-hover:scale-110 transition-transform" />
                                    <span>{t('settings.about.telegram')}</span>
                                </a>
                                <a
                                    href="https://github.com/alimtvnetwork/Antigravity-Manager"
                                    target="_blank"
                                    rel="noreferrer"
                                    className="flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-gray-200/80 dark:border-base-100 bg-white dark:bg-base-200 text-xs font-medium text-gray-700 dark:text-gray-200 hover:border-gray-400 dark:hover:border-gray-500 hover:text-gray-900 dark:hover:text-white hover:shadow-xs transition-all duration-200 group cursor-pointer"
                                >
                                    <Github className="w-4 h-4 text-gray-600 dark:text-gray-300 group-hover:scale-110 transition-transform" />
                                    <span>{t('settings.about.view_code')}</span>
                                </a>
                                <button
                                    type="button"
                                    onClick={() => setIsSupportModalOpen(true)}
                                    className="flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-pink-200/80 dark:border-pink-900/40 bg-pink-50/50 dark:bg-pink-950/20 text-xs font-medium text-pink-700 dark:text-pink-300 hover:border-pink-400 hover:bg-pink-100/60 dark:hover:bg-pink-950/40 hover:shadow-xs transition-all duration-200 group cursor-pointer"
                                >
                                    <Heart className="w-4 h-4 text-pink-500 group-hover:scale-110 transition-transform fill-pink-500/20" />
                                    <span>{t('settings.about.support_btn')}</span>
                                </button>
                            </div>

                            {/* Update Management Card */}
                            <div className="p-4 sm:p-5 rounded-2xl bg-white dark:bg-base-200 border border-gray-200/80 dark:border-base-100 shadow-xs flex flex-col gap-4">
                                <div className="flex flex-col md:flex-row md:items-center justify-between gap-3">
                                    {/* Update Channel Selector */}
                                    <div className="flex items-center gap-2.5 flex-wrap">
                                        <span className="text-xs font-semibold text-gray-600 dark:text-gray-300">
                                            {t('settings.about.update_channel')}:
                                        </span>
                                        <div className="inline-flex p-1 bg-gray-100 dark:bg-base-300 rounded-xl border border-gray-200/60 dark:border-base-200">
                                            <button
                                                type="button"
                                                onClick={async () => {
                                                    try {
                                                        await invoke('save_update_settings', {
                                                            settings: {
                                                                auto_check: formData.auto_check_update ?? true,
                                                                last_check_time: 0,
                                                                check_interval_hours: formData.update_check_interval ?? 24,
                                                                update_channel: 'stable',
                                                            }
                                                        });
                                                        setFormData(prev => ({ ...prev, update_channel: 'stable' }));
                                                        setUpdateInfo(null);
                                                        showToast(`${t('settings.about.update_channel')}: ${t('settings.about.channel_stable')}`, 'info');
                                                    } catch (err) {
                                                        showToast(`${t('common.error')}: ${err}`, 'error');
                                                    }
                                                }}
                                                className={`px-3 py-1 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                                                    (formData.update_channel || 'stable') === 'stable'
                                                        ? 'bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-sm'
                                                        : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                                                }`}
                                            >
                                                {t('settings.about.channel_stable')}
                                            </button>
                                            <button
                                                type="button"
                                                onClick={async () => {
                                                    try {
                                                        await invoke('save_update_settings', {
                                                            settings: {
                                                                auto_check: formData.auto_check_update ?? true,
                                                                last_check_time: 0,
                                                                check_interval_hours: formData.update_check_interval ?? 24,
                                                                update_channel: 'beta',
                                                            }
                                                        });
                                                        setFormData(prev => ({ ...prev, update_channel: 'beta' }));
                                                        setUpdateInfo(null);
                                                        showToast(`${t('settings.about.update_channel')}: ${t('settings.about.channel_beta')}`, 'info');
                                                    } catch (err) {
                                                        showToast(`${t('common.error')}: ${err}`, 'error');
                                                    }
                                                }}
                                                className={`px-3 py-1 rounded-lg text-xs font-semibold transition-all flex items-center gap-1 cursor-pointer ${
                                                    formData.update_channel === 'beta'
                                                        ? 'bg-white dark:bg-base-100 text-amber-600 dark:text-amber-400 shadow-sm'
                                                        : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                                                }`}
                                            >
                                                <span>{t('settings.about.channel_beta')}</span>
                                                <span className="w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse" />
                                            </button>
                                        </div>
                                        {formData.update_channel === 'beta' && (
                                            <span className="text-[11px] text-amber-600/90 dark:text-amber-400/90">
                                                {t('settings.about.channel_beta_hint')}
                                            </span>
                                        )}
                                    </div>

                                    {/* Check for Updates Action */}
                                    <div className="flex items-center gap-2">
                                        <button
                                            onClick={handleCheckUpdate}
                                            disabled={isCheckingUpdate}
                                            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-300 dark:disabled:bg-gray-700 text-white text-xs font-semibold rounded-xl transition-all flex items-center gap-2 shadow-sm hover:shadow-md disabled:cursor-not-allowed cursor-pointer"
                                        >
                                            <RefreshCw className={`w-3.5 h-3.5 ${isCheckingUpdate ? 'animate-spin' : ''}`} />
                                            {isCheckingUpdate ? t('settings.about.checking_update') : t('settings.about.check_update')}
                                        </button>
                                    </div>
                                </div>

                                {/* Inline Status Messaging */}
                                {updateInfo && !isCheckingUpdate && (
                                    <div className="pt-3 border-t border-gray-100 dark:border-base-100">
                                        {updateInfo.hasUpdate ? (
                                            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 p-3 rounded-xl bg-amber-50/60 dark:bg-amber-950/20 border border-amber-200/60 dark:border-amber-800/40">
                                                <div className="flex items-center gap-2 text-xs text-orange-700 dark:text-orange-300 font-medium">
                                                    <span>{t('settings.about.new_version_available', { version: updateInfo.latestVersion })}</span>
                                                    {updateInfo.channel === 'beta' && (
                                                        <span className="px-1.5 py-0.2 text-[10px] font-semibold rounded bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300 border border-amber-200 dark:border-amber-800">
                                                            Beta
                                                        </span>
                                                    )}
                                                </div>
                                                <div className="flex items-center gap-2 flex-wrap">
                                                    {isBrewInstalled && (
                                                        <button
                                                            onClick={() => setIsBrewConfirmOpen(true)}
                                                            disabled={isBrewUpgrading}
                                                            className="px-3 py-1.5 bg-green-600 hover:bg-green-700 disabled:bg-gray-300 dark:disabled:bg-gray-700 text-white text-xs rounded-lg transition-colors flex items-center gap-1.5 disabled:cursor-not-allowed cursor-pointer"
                                                        >
                                                            {isBrewUpgrading ? (
                                                                <>
                                                                    <RefreshCw className="w-3 h-3 animate-spin" />
                                                                    {t('settings.about.brew_upgrading')}
                                                                </>
                                                            ) : (
                                                                t('settings.about.brew_upgrade')
                                                            )}
                                                        </button>
                                                    )}
                                                    <button
                                                        onClick={handleRunInstallerUpdate}
                                                        disabled={isInstallerUpdating}
                                                        className="px-3.5 py-1.5 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 disabled:opacity-50 text-white text-xs font-semibold rounded-lg transition-all shadow-xs flex items-center gap-1.5 cursor-pointer disabled:cursor-not-allowed"
                                                    >
                                                        {isInstallerUpdating ? (
                                                            <>
                                                                <RefreshCw className="w-3 h-3 animate-spin" />
                                                                <span>{t('settings.about.installer_running', 'Running Installer...')}</span>
                                                            </>
                                                        ) : (
                                                            <>
                                                                <Sparkles className="w-3 h-3" />
                                                                <span>{t('settings.about.run_installer', 'Install Update Now')}</span>
                                                            </>
                                                        )}
                                                    </button>
                                                    <a
                                                        href={updateInfo.downloadUrl}
                                                        target="_blank"
                                                        rel="noreferrer"
                                                        className="px-3 py-1.5 bg-orange-500 hover:bg-orange-600 text-white text-xs rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer"
                                                    >
                                                        {t('settings.about.download_update')}
                                                        <ExternalLink className="w-3 h-3" />
                                                    </a>
                                                </div>
                                            </div>
                                        ) : (
                                            <div className="flex items-center gap-2 text-xs text-emerald-600 dark:text-emerald-400 font-medium">
                                                <CheckCircle2 className="w-4 h-4" />
                                                <span>{t('settings.about.latest_version')}</span>
                                            </div>
                                        )}
                                    </div>
                                )}
                            </div>

                            <div className="text-center text-[10px] text-gray-400 dark:text-gray-500 pt-2 mt-auto">
                                {t('settings.about.copyright')}
                            </div>
                        </div>
    );
}
