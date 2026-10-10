import { SettingsPageApi } from './useSettingsPage';
import { Coffee } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import ModalDialog from '../components/common/ModalDialog';
import { UnifiedBackupModal } from '../components/modals/UnifiedBackupModal';
import { showToast } from '../components/common/ToastContainer';
import { relaunch } from '@tauri-apps/plugin-process';

export function SettingsModals(props: SettingsPageApi) {
    const { t, isBackupModalOpen, setIsBackupModalOpen, isClearLogsOpen, setIsClearLogsOpen, isSupportModalOpen, setIsSupportModalOpen, pendingDataDir, setPendingDataDir, isMigrateDataDirOpen, setIsMigrateDataDirOpen, isMigratingDataDir, isClearCacheOpen, setIsClearCacheOpen, cachePaths, isClearingCache, isBrewConfirmOpen, setIsBrewConfirmOpen, isBrewSuccessOpen, setIsBrewSuccessOpen, confirmClearLogs, confirmMigrateDataDir, path, handleBrewUpgrade, confirmClearAntigravityCache } = props;

    return (
        <>
            {/* Data Directory Migration Modal */}
            <ModalDialog
                isOpen={isClearLogsOpen}
                title={t('settings.advanced.clear_logs_title')}
                message={t('settings.advanced.clear_logs_msg')}
                type="confirm"
                confirmText={t('common.clear')}
                cancelText={t('common.cancel')}
                isDestructive={true}
                onConfirm={confirmClearLogs}
                onCancel={() => setIsClearLogsOpen(false)}
            />

            <ModalDialog
                isOpen={isMigrateDataDirOpen}
                title={t('settings.advanced.data_dir_migrate_title')}
                type="confirm"
                confirmText={isMigratingDataDir ? t('common.loading') : t('common.confirm')}
                cancelText={t('common.cancel')}
                onConfirm={confirmMigrateDataDir}
                onCancel={() => {
                    if (!isMigratingDataDir) {
                        setIsMigrateDataDirOpen(false);
                        setPendingDataDir('');
                    }
                }}
            >
                <p className="text-sm text-gray-600 dark:text-gray-400 whitespace-pre-line">
                    {t('settings.advanced.data_dir_migrate_msg', { path: pendingDataDir })}
                </p>
            </ModalDialog>

            {/* Antigravity Cache Clear Modal */}
            <ModalDialog
                isOpen={isClearCacheOpen}
                title={t('settings.advanced.clear_cache_confirm_title')}
                type="confirm"
                confirmText={isClearingCache ? t('common.clearing') : t('common.clear')}
                cancelText={t('common.cancel')}
                isDestructive={true}
                onConfirm={confirmClearAntigravityCache}
                onCancel={() => setIsClearCacheOpen(false)}
            >
                <div className="space-y-3">
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                        {t('settings.advanced.clear_cache_confirm_msg')}
                    </p>
                    {cachePaths.length > 0 ? (
                        <div className="bg-gray-50 dark:bg-base-200 rounded-lg p-3 max-h-40 overflow-y-auto">
                            <ul className="text-xs font-mono text-gray-600 dark:text-gray-400 space-y-1">
                                {cachePaths.map((path, index) => (
                                    <li key={index} className="truncate">• {path}</li>
                                ))}
                            </ul>
                        </div>
                    ) : (
                        <div className="bg-gray-50 dark:bg-base-200 rounded-lg p-3">
                            <p className="text-xs text-gray-500 dark:text-gray-400">
                                {t('settings.advanced.cache_not_found')}
                            </p>
                        </div>
                    )}
                    <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700/30 rounded-lg p-2">
                        <p className="text-xs text-amber-700 dark:text-amber-400">
                            {t('settings.advanced.antigravity_cache_warning')}
                        </p>
                    </div>
                </div>
            </ModalDialog>

            {/* Homebrew Upgrade Confirm Modal */}
            <ModalDialog
                isOpen={isBrewConfirmOpen}
                title={t('settings.about.brew_confirm_title')}
                type="confirm"
                confirmText={t('settings.about.brew_confirm_btn')}
                cancelText={t('common.cancel')}
                onConfirm={handleBrewUpgrade}
                onCancel={() => setIsBrewConfirmOpen(false)}
            >
                <div className="space-y-3">
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                        {t('settings.about.brew_confirm_desc')}
                    </p>
                    <div className="bg-gray-50 dark:bg-base-200 rounded-lg p-3">
                        <div className="flex items-center justify-between gap-2">
                            <code className="text-xs text-gray-700 dark:text-gray-300 break-all">brew upgrade --cask antigravity-tools</code>
                            <button
                                className="shrink-0 px-2 py-1 text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 border border-gray-200 dark:border-base-300 rounded hover:bg-gray-100 dark:hover:bg-base-300 transition-colors"
                                onClick={() => {
                                    navigator.clipboard.writeText('brew upgrade --cask antigravity-tools');
                                    showToast(t('common.copied', 'Copied'), 'success');
                                }}
                            >
                                {t('common.copy', 'Copy')}
                            </button>
                        </div>
                    </div>
                    <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700/30 rounded-lg p-3">
                        <p className="text-xs text-amber-700 dark:text-amber-400 mb-2">{t('settings.about.brew_quarantine_hint')}</p>
                        <div className="flex items-center justify-between gap-2">
                            <code className="text-xs text-amber-800 dark:text-amber-300 break-all">sudo xattr -rd com.apple.quarantine "/Applications/Antigravity Tools.app"</code>
                            <button
                                className="shrink-0 px-2 py-1 text-xs text-amber-600 hover:text-amber-800 dark:text-amber-400 dark:hover:text-amber-200 border border-amber-200 dark:border-amber-700 rounded hover:bg-amber-100 dark:hover:bg-amber-900/30 transition-colors"
                                onClick={() => {
                                    navigator.clipboard.writeText('sudo xattr -rd com.apple.quarantine "/Applications/Antigravity Tools.app"');
                                    showToast(t('common.copied', 'Copied'), 'success');
                                }}
                            >
                                {t('common.copy', 'Copy')}
                            </button>
                        </div>
                    </div>
                </div>
            </ModalDialog>

            {/* Homebrew Upgrade Success Modal */}
            <ModalDialog
                isOpen={isBrewSuccessOpen}
                title={t('settings.about.brew_success_title')}
                type="success"
                confirmText={t('settings.about.brew_restart_btn')}
                onConfirm={async () => {
                    try {
                        await relaunch();
                    } catch {
                        setIsBrewSuccessOpen(false);
                        showToast(t('settings.about.brew_restart_failed'), 'error');
                    }
                }}
            >
                <p className="text-sm text-gray-600 dark:text-gray-400">
                    {t('settings.about.brew_upgrade_success')}
                </p>
            </ModalDialog>


            {/* Support Modal */}
            <div className={`modal ${isSupportModalOpen ? 'modal-open' : ''} z-[100]`}>
                <div data-tauri-drag-region className="fixed top-0 left-0 right-0 h-8 z-[110]" />
                <div className="modal-box relative max-w-2xl bg-white dark:bg-base-100 shadow-2xl rounded-3xl p-0 overflow-hidden transform transition-all animate-in fade-in zoom-in-95 duration-300">
                    <div className="flex flex-col items-center p-8">
                        <div className="w-16 h-16 bg-pink-50 dark:bg-pink-900/20 rounded-2xl flex items-center justify-center mb-6 shadow-sm">
                            <Coffee className="w-8 h-8 text-pink-500" />
                        </div>

                        <h3 className="text-2xl font-black text-gray-900 dark:text-base-content mb-3">{t('settings.about.support_title')}</h3>
                        <p className="text-gray-500 dark:text-gray-400 text-sm text-center mb-8 max-w-md leading-relaxed">
                            {t('settings.about.support_desc')}
                        </p>

                        {/* QR Codes Grid */}
                        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full mb-8">
                            {/* Alipay */}
                            <div className="flex flex-col items-center gap-3 p-4 rounded-2xl bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300">
                                <div className="w-full aspect-square relative bg-white rounded-xl overflow-hidden shadow-sm border border-gray-100">
                                    <img src="/images/donate/alipay.png" alt="Alipay" className="w-full h-full object-contain" />
                                </div>
                                <span className="text-xs font-bold text-gray-700 dark:text-gray-300">{t('settings.about.support_alipay')}</span>
                            </div>

                            {/* WeChat */}
                            <div className="flex flex-col items-center gap-3 p-4 rounded-2xl bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300">
                                <div className="w-full aspect-square relative bg-white rounded-xl overflow-hidden shadow-sm border border-gray-100">
                                    <img src="/images/donate/wechat.png" alt="WeChat" className="w-full h-full object-contain" />
                                </div>
                                <span className="text-xs font-bold text-gray-700 dark:text-gray-300">{t('settings.about.support_wechat')}</span>
                            </div>

                            {/* Buy Me a Coffee */}
                            <div className="flex flex-col items-center gap-3 p-4 rounded-2xl bg-gray-50 dark:bg-base-200 border border-gray-100 dark:border-base-300">
                                <div className="w-full aspect-square relative bg-white rounded-xl overflow-hidden shadow-sm border border-gray-100">
                                    <img src="/images/donate/coffee.png" alt="Buy Me A Coffee" className="w-full h-full object-contain" />
                                </div>
                                <span className="text-xs font-bold text-gray-700 dark:text-gray-300">{t('settings.about.support_buymeacoffee')}</span>
                            </div>
                        </div>

                        <button
                            onClick={() => setIsSupportModalOpen(false)}
                            className="w-full md:w-auto px-12 py-3 bg-gray-100 dark:bg-base-300 text-gray-700 dark:text-gray-200 font-bold rounded-xl hover:bg-gray-200 dark:hover:bg-base-200 transition-all"
                        >
                            {t('common.close') || 'Close'}
                        </button>
                    </div>
                </div>
                <div className="modal-backdrop bg-black/60 backdrop-blur-md fixed inset-0 z-[-1]" onClick={() => setIsSupportModalOpen(false)}></div>
            </div>

            {/* Unified Backup Modal */}
            <UnifiedBackupModal
                isOpen={isBackupModalOpen}
                onClose={() => setIsBackupModalOpen(false)}
            />
        </>
    );
}
