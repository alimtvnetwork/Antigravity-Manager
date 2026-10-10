import { startTransition } from 'react';
import { Save, Bug, Menu, ChevronDown, Sliders, Info, ShieldCheck } from 'lucide-react';
import { useSettingsPage } from './Settings/useSettingsPage';
import { GeneralTab } from './Settings/tabs/GeneralTab';
import { AccountTab } from './Settings/tabs/AccountTab';
import { AdvancedTab } from './Settings/tabs/AdvancedTab';
import { DebugTab } from './Settings/tabs/DebugTab';
import { ProxyTab } from './Settings/tabs/ProxyTab';
import { AboutTab } from './Settings/tabs/AboutTab';
import { SettingsModals } from './Settings/SettingsModals';
import EmailNotificationSettings from './components/settings/EmailNotificationSettings';
import ThemePicker from './components/settings/ThemePicker';
import SupabaseSyncSettings from './components/settings/SupabaseSyncSettings';

function Settings() {
    const page = useSettingsPage();
    const { t, activeTab } = page;

    return (
            <div className="h-full w-full overflow-y-auto">
                <div className="px-4 sm:px-6 pt-2 pb-4 space-y-4 max-w-[1920px] mx-auto">
                    {/* Top toolbar: Tab navigation and save button */}
                    <div className="flex flex-wrap gap-2 justify-between items-center">
                        {/* Tab navigation */}
                        <div className="flex items-center gap-1 bg-gray-100 dark:bg-base-200 rounded-full p-1 w-fit">
                            <button
                                className={`px-4 py-1.5 rounded-full text-sm font-medium transition-all cursor-pointer ${activeTab === 'general'
                                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-xs'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    }`}
                                onClick={() => startTransition(() => setActiveTab('general'))}
                            >
                                {t('settings.tabs.general')}
                            </button>
                            <button
                                className={`px-4 py-1.5 rounded-full text-sm font-medium transition-all cursor-pointer ${activeTab === 'account'
                                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-xs'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    }`}
                                onClick={() => startTransition(() => setActiveTab('account'))}
                            >
                                {t('settings.tabs.account')}
                            </button>
                            <button
                                className={`px-4 py-1.5 rounded-full text-sm font-medium transition-all cursor-pointer ${activeTab === 'proxy'
                                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-xs'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    }`}
                                onClick={() => startTransition(() => setActiveTab('proxy'))}
                            >
                                {t('settings.tabs.proxy', 'Proxy')}
                            </button>
                            <button
                                className={`px-4 py-1.5 rounded-full text-sm font-medium transition-all cursor-pointer ${activeTab === 'email'
                                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-xs'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    }`}
                                onClick={() => setActiveTab('email')}
                            >
                                {t('settings.tabs.email', 'Email-Alerts')}
                            </button>
                            <button
                                className={`px-4 py-1.5 rounded-full text-sm font-medium transition-all cursor-pointer ${activeTab === 'themes'
                                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-xs'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    }`}
                                onClick={() => setActiveTab('themes')}
                            >
                                {t('settings.tabs.themes', 'Themes')}
                            </button>
                            <button
                                className={`px-4 py-1.5 rounded-full text-sm font-medium transition-all cursor-pointer ${activeTab === 'supabase'
                                    ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-xs'
                                    : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                    }`}
                                onClick={() => setActiveTab('supabase')}
                            >
                                {t('settings.tabs.supabase', 'Supabase')}
                            </button>

                            {/* Hamburger Dropdown for Advance, Debug, and About */}
                            <div className="relative">
                                <button
                                    type="button"
                                    className={`px-3 py-1.5 rounded-full text-sm font-medium transition-all flex items-center gap-1.5 cursor-pointer ${['advanced', 'debug', 'about'].includes(activeTab)
                                        ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-xs'
                                        : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                                        }`}
                                    onClick={() => setIsMoreDropdownOpen(!isMoreDropdownOpen)}
                                    title={t('settings.tabs.more', 'More Options')}
                                >
                                    {activeTab === 'advanced' ? (
                                        <>
                                            <Sliders className="w-3.5 h-3.5" />
                                            <span>{t('settings.tabs.advanced', 'Advance')}</span>
                                        </>
                                    ) : activeTab === 'debug' ? (
                                        <>
                                            <Bug className="w-3.5 h-3.5 text-amber-500" />
                                            <span>{t('settings.tabs.debug', 'Debug')}</span>
                                        </>
                                    ) : activeTab === 'about' ? (
                                        <>
                                            <Info className="w-3.5 h-3.5 text-blue-500" />
                                            <span>{t('settings.tabs.about', 'About')}</span>
                                        </>
                                    ) : (
                                        <>
                                            <Menu className="w-4 h-4" />
                                        </>
                                    )}
                                    <ChevronDown className={`w-3 h-3 transition-transform ${isMoreDropdownOpen ? 'rotate-180' : ''}`} />
                                </button>

                                {isMoreDropdownOpen && (
                                    <>
                                        <div
                                            className="fixed inset-0 z-40"
                                            onClick={() => setIsMoreDropdownOpen(false)}
                                        />
                                        <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-base-100 rounded-2xl shadow-xl border border-gray-100 dark:border-base-300 py-1.5 z-50 animate-in fade-in zoom-in-95 duration-150">
                                            <button
                                                type="button"
                                                className={`w-full px-3.5 py-2 text-left text-sm flex items-center gap-2.5 transition-colors cursor-pointer ${activeTab === 'advanced'
                                                    ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 font-medium'
                                                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-base-200'
                                                    }`}
                                                onClick={() => {
                                                    startTransition(() => setActiveTab('advanced'));
                                                    setIsMoreDropdownOpen(false);
                                                }}
                                            >
                                                <Sliders className="w-4 h-4 text-gray-500 dark:text-gray-400" />
                                                <span>{t('settings.tabs.advanced', 'Advance')}</span>
                                            </button>
                                            <button
                                                type="button"
                                                className={`w-full px-3.5 py-2 text-left text-sm flex items-center gap-2.5 transition-colors cursor-pointer ${activeTab === 'debug'
                                                    ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 font-medium'
                                                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-base-200'
                                                    }`}
                                                onClick={() => {
                                                    openDebugModal();
                                                    startTransition(() => setActiveTab('debug'));
                                                    setIsMoreDropdownOpen(false);
                                                }}
                                                title="Debug is consolidated into the top-level titlebar Bug icon"
                                            >
                                                <Bug className="w-4 h-4 text-amber-500" />
                                                <div className="flex items-center justify-between flex-1">
                                                    <span className="line-through decoration-2 decoration-rose-500 text-gray-400 dark:text-gray-500">
                                                        {t('settings.tabs.debug', 'Debug')}
                                                    </span>
                                                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300 font-medium">
                                                        Top Bar ↗
                                                    </span>
                                                </div>
                                            </button>
                                            <div className="my-1 border-t border-gray-100 dark:border-base-300" />
                                            <button
                                                type="button"
                                                className={`w-full px-3.5 py-2 text-left text-sm flex items-center gap-2.5 transition-colors cursor-pointer ${activeTab === 'about'
                                                    ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 font-medium'
                                                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-base-200'
                                                    }`}
                                                onClick={() => {
                                                    startTransition(() => setActiveTab('about'));
                                                    setIsMoreDropdownOpen(false);
                                                }}
                                            >
                                                <Info className="w-4 h-4 text-blue-500" />
                                                <span>{t('settings.tabs.about', 'About')}</span>
                                            </button>
                                        </div>
                                    </>
                                )}
                            </div>
                        </div>

                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                className="px-3.5 py-2 bg-gray-100 dark:bg-base-200 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-lg hover:bg-gray-200 dark:hover:bg-base-300 transition-colors flex items-center gap-2 shadow-xs border border-gray-200 dark:border-base-100 cursor-pointer"
                                onClick={() => setIsBackupModalOpen(true)}
                                title="Encrypted Full System Backup & Restore"
                            >
                                <ShieldCheck className="w-4 h-4 text-emerald-500" />
                                <span>{t('settings.backup_restore', 'Backup')}</span>
                            </button>

                            <button
                                className="px-4 py-2 bg-blue-500 text-white text-sm font-medium rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-2 shadow-sm cursor-pointer"
                                onClick={handleSave}
                            >
                                <Save className="w-4 h-4" />
                                <span>{t('settings.save', 'Save')}</span>
                            </button>
                        </div>
                    </div>

                    {/* Settings form */}

                {/* Settings form */}
                <div className="bg-white dark:bg-base-100 rounded-2xl p-5 shadow-sm border border-gray-100 dark:border-base-200">
                    {activeTab === 'general' && <GeneralTab {...page} />}
                    {activeTab === 'account' && <AccountTab {...page} />}
                    {activeTab === 'advanced' && <AdvancedTab {...page} />}
                    {activeTab === 'debug' && <DebugTab {...page} />}
                    {activeTab === 'proxy' && <ProxyTab {...page} />}
                    {activeTab === 'email' && <EmailNotificationSettings />}
                    {activeTab === 'themes' && <ThemePicker />}
                    {activeTab === 'supabase' && <SupabaseSyncSettings />}
                    {activeTab === 'about' && <AboutTab {...page} />}
                </div>

                <SettingsModals {...page} />
            </div>
        </div>
    );
}

export default Settings;
