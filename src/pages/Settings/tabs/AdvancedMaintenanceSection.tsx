import { SettingsPageApi } from '../useSettingsPage';
import { Bug, Terminal } from 'lucide-react';
import { isTauri } from '../../../utils/env';
import versionData from '../../../../version.json';

export function AdvancedMaintenanceSection(props: SettingsPageApi) {
    const { t, enable, disable, isEnabled, formData, setFormData, setIsClearLogsOpen, dataDirPath, handleSelectDebugLogDir, handleOpenClearCacheDialog } = props;

    return (
        <>
            {/* Clear log cache */}
            <div className="border-t border-gray-200 dark:border-base-200 pt-4">
                <h3 className="font-medium text-gray-900 dark:text-base-content mb-3">{t('settings.advanced.logs_title')}</h3>
                <div className="bg-gray-50 dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-lg p-3 mb-3">
                    <p className="text-sm text-gray-600 dark:text-gray-400">{t('settings.advanced.logs_desc')}</p>
                </div>
                <div className="flex items-center gap-4">
                    <button
                        className="px-4 py-2 border border-gray-300 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-base-200 transition-colors"
                        onClick={() => setIsClearLogsOpen(true)}
                    >
                        {t('settings.advanced.clear_logs')}
                    </button>
                </div>
            </div>

            {/* Clear Antigravity cache */}
            <div className="border-t border-gray-200 dark:border-base-200 pt-4">
                <h3 className="font-medium text-gray-900 dark:text-base-content mb-3">{t('settings.advanced.antigravity_cache_title')}</h3>
                <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700/30 rounded-lg p-3 mb-3">
                    <p className="text-sm text-amber-700 dark:text-amber-400">{t('settings.advanced.antigravity_cache_warning')}</p>
                </div>
                <div className="bg-gray-50 dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-lg p-3 mb-3">
                    <p className="text-sm text-gray-600 dark:text-gray-400">{t('settings.advanced.antigravity_cache_desc')}</p>
                </div>
                <div className="flex items-center gap-4">
                    <button
                        className="px-4 py-2 border border-orange-300 dark:border-orange-700 text-orange-700 dark:text-orange-400 rounded-lg hover:bg-orange-50 dark:hover:bg-orange-900/20 transition-colors"
                        onClick={handleOpenClearCacheDialog}
                    >
                        {t('settings.advanced.clear_antigravity_cache')}
                    </button>
                </div>
            </div>

            {/* Auto Conversation Pruning & Retention */}
            <div className="border-t border-gray-200 dark:border-base-200 pt-4">
                <div className="flex items-center justify-between mb-3">
                    <div>
                        <h3 className="font-medium text-gray-900 dark:text-base-content">
                            {t('settings.advanced.auto_cleanup_title', 'Periodic Conversation Auto-Cleanup')}
                        </h3>
                        <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                            {t('settings.advanced.auto_cleanup_desc', 'Automatically keep recent conversations and prune older conversations to temporary storage every 1 hour.')}
                        </p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={formData.conversation_cleanup?.is_enabled ?? false}
                            onChange={(e) => setFormData({
                                ...formData,
                                conversation_cleanup: {
                                    is_enabled: e.target.checked,
                                    interval_hours: formData.conversation_cleanup?.interval_hours ?? 1,
                                    keep_count: formData.conversation_cleanup?.keep_count ?? 40,
                                },
                            })}
                        />
                        <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                    </label>
                </div>

                {(formData.conversation_cleanup?.is_enabled ?? false) && (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 bg-gray-50 dark:bg-base-200 p-4 rounded-lg border border-gray-200 dark:border-base-300 mb-3">
                        <div>
                            <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">
                                {t('settings.advanced.cleanup_keep_count', 'Keep Recent Conversations')}
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="500"
                                className="w-full px-4 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-white dark:bg-base-100 text-gray-900 dark:text-base-content"
                                value={formData.conversation_cleanup?.keep_count ?? 40}
                                onChange={(e) => setFormData({
                                    ...formData,
                                    conversation_cleanup: {
                                        is_enabled: formData.conversation_cleanup?.is_enabled ?? true,
                                        interval_hours: formData.conversation_cleanup?.interval_hours ?? 1,
                                        keep_count: parseInt(e.target.value) || 40,
                                    },
                                })}
                            />
                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                {t('settings.advanced.cleanup_keep_desc', 'Default: 40 conversations. Older conversations are staged into temporary backup and can be undone.')}
                            </p>
                        </div>
                        <div>
                            <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">
                                {t('settings.advanced.cleanup_interval_hours', 'Execution Interval (Hours)')}
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="168"
                                className="w-full px-4 py-2 border border-gray-200 dark:border-base-300 rounded-lg bg-white dark:bg-base-100 text-gray-900 dark:text-base-content"
                                value={formData.conversation_cleanup?.interval_hours ?? 1}
                                onChange={(e) => setFormData({
                                    ...formData,
                                    conversation_cleanup: {
                                        is_enabled: formData.conversation_cleanup?.is_enabled ?? true,
                                        interval_hours: parseInt(e.target.value) || 1,
                                        keep_count: formData.conversation_cleanup?.keep_count ?? 40,
                                    },
                                })}
                            />
                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                {t('settings.advanced.cleanup_interval_desc', 'Periodic interval in hours (default: 1 hour).')}
                            </p>
                        </div>
                    </div>
                )}
            </div>



            <div className="border-t border-gray-200 dark:border-base-200 pt-4">
                <div className="space-y-3">
                    <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-base-200 rounded-lg border border-gray-100 dark:border-base-300">
                        <div>
                            <div className="font-medium text-gray-900 dark:text-base-content">
                                {t('settings.advanced.debug_logs_title')}
                            </div>
                            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                                {t('settings.advanced.debug_logs_enable_desc')}
                            </p>
                        </div>
                        <label className="relative inline-flex items-center cursor-pointer">
                            <input
                                type="checkbox"
                                className="sr-only peer"
                                checked={formData.proxy?.debug_logging?.enabled ?? false}
                                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({
                                    ...formData,
                                    proxy: {
                                        ...formData.proxy,
                                        debug_logging: {
                                            enabled: e.target.checked,
                                            output_dir: formData.proxy?.debug_logging?.output_dir,
                                        },
                                    },
                                })}
                            />
                            <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                        </label>
                    </div>
                    {(formData.proxy?.debug_logging?.enabled ?? false) && (
                        <>
                            <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700/30 rounded-lg p-3">
                                <p className="text-sm text-amber-700 dark:text-amber-400">
                                    {t('settings.advanced.debug_logs_desc')}
                                </p>
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">
                                    {t('settings.advanced.debug_log_dir')}
                                </label>
                                <div className="flex gap-2">
                                    <input
                                        type="text"
                                        className="flex-1 px-4 py-3 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-base-content font-medium"
                                        value={formData.proxy?.debug_logging?.output_dir || ''}
                                        placeholder={`${dataDirPath.replace(/\/$/, '')}/debug_logs`}
                                        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({
                                            ...formData,
                                            proxy: {
                                                ...formData.proxy,
                                                debug_logging: {
                                                    enabled: formData.proxy?.debug_logging?.enabled ?? false,
                                                    output_dir: e.target.value || undefined,
                                                },
                                            },
                                        })}
                                    />
                                    {isTauri() && (
                                        <button
                                            className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors"
                                            onClick={handleSelectDebugLogDir}
                                        >
                                            {t('settings.advanced.select_btn')}
                                        </button>
                                    )}
                                </div>
                                <p className="text-xs text-gray-500 dark:text-gray-400 mt-2">
                                    {t('settings.advanced.debug_log_dir_hint', { path: dataDirPath.replace(/\/$/, '') })}
                                </p>
                            </div>
                        </>
                    )}
                </div>
            </div>

            {/* Debug Console Quick-Access Card */}
            <div className="border-t border-gray-200 dark:border-base-200 pt-4">
                <div className="flex items-center justify-between p-4 bg-amber-50/50 dark:bg-amber-950/20 rounded-lg border border-amber-200/60 dark:border-amber-900/30">
                    <div className="flex items-center gap-3">
                        <Bug className="w-5 h-5 text-amber-500 shrink-0" />
                        <div>
                            <div className="font-medium text-gray-900 dark:text-base-content text-sm">
                                {t('settings.debug.title', 'Debug Console & Logs')}
                            </div>
                            <p className="text-xs text-gray-600 dark:text-gray-400 mt-0.5">
                                Inspect IPC traffic and live runtime logs. Also accessible via the Bug icon in the top header navbar.
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        className="px-3 py-1.5 bg-amber-500 hover:bg-amber-600 text-white text-xs font-medium rounded-lg transition-colors shadow-xs cursor-pointer"
                        onClick={() => {
                            if (isEnabled) {
                                disable();
                            } else {
                                enable();
                            }
                        }}
                    >
                        {isEnabled ? 'Close Debug Overlay' : 'Open Debug Console'}
                    </button>
                </div>
            </div>

            {/* CLI & Fleet Automation Quick Reference */}
            <div className="border-t border-gray-200 dark:border-base-200 pt-4">
                <div className="p-4 bg-slate-50 dark:bg-base-200/50 rounded-xl border border-slate-200 dark:border-base-300 space-y-3">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2.5">
                            <div className="w-8 h-8 rounded-lg bg-blue-500/10 dark:bg-blue-400/10 flex items-center justify-center text-blue-600 dark:text-blue-400 font-mono font-bold text-xs">
                                <Terminal className="w-4 h-4" />
                            </div>
                            <div>
                                <h4 className="text-sm font-semibold text-gray-900 dark:text-base-content">
                                    AGM CLI & Fleet Automation Quick Reference
                                </h4>
                                <p className="text-xs text-gray-500 dark:text-gray-400">
                                    Run agm commands from your terminal or remote SSH workflows for headless operations.
                                </p>
                            </div>
                        </div>
                        <span className="px-2.5 py-0.5 text-[11px] font-mono font-semibold rounded-full bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300">
                            agm v{versionData.version}
                        </span>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5 pt-1 text-xs">
                        <div className="p-2.5 bg-white dark:bg-base-100 rounded-lg border border-gray-100 dark:border-base-200 font-mono">
                            <div className="text-blue-600 dark:text-blue-400 font-semibold mb-0.5">agm status [--json]</div>
                            <div className="text-[11px] text-gray-500 dark:text-gray-400 font-sans">Fleet health, active account & live prompt status</div>
                        </div>
                        <div className="p-2.5 bg-white dark:bg-base-100 rounded-lg border border-gray-100 dark:border-base-200 font-mono">
                            <div className="text-emerald-600 dark:text-emerald-400 font-semibold mb-0.5">agm update all [--json]</div>
                            <div className="text-[11px] text-gray-500 dark:text-gray-400 font-sans">Fleet-wide update check & diagnostic probe (alias: agm ua)</div>
                        </div>
                        <div className="p-2.5 bg-white dark:bg-base-100 rounded-lg border border-gray-100 dark:border-base-200 font-mono">
                            <div className="text-indigo-600 dark:text-indigo-400 font-semibold mb-0.5">agm prompt "&lt;task&gt;"</div>
                            <div className="text-[11px] text-gray-500 dark:text-gray-400 font-sans">Inject & track workspace prompt in SQLite DB</div>
                        </div>
                        <div className="p-2.5 bg-white dark:bg-base-100 rounded-lg border border-gray-100 dark:border-base-200 font-mono">
                            <div className="text-purple-600 dark:text-purple-400 font-semibold mb-0.5">agm backup / agm restore</div>
                            <div className="text-[11px] text-gray-500 dark:text-gray-400 font-sans">Snapshot & re-inject running prompts across all projects</div>
                        </div>
                    </div>
                </div>
            </div>
        </>
    );
}
