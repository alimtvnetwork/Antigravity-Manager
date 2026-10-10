import { SettingsPageApi } from '../useSettingsPage';
import { Bug } from 'lucide-react';
import DebugConsole from '../components/debug/DebugConsole';

export function DebugTab(props: SettingsPageApi) {
    const { t, enable, disable, isEnabled } = props;

    return (
                        <div className="space-y-4 animate-in fade-in duration-500">
                            {/* Header Quick-Access Tip */}
                            <div className="flex items-center gap-3 p-3 bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-900/30 rounded-xl text-xs text-amber-800 dark:text-amber-300">
                                <Bug className="w-4 h-4 shrink-0 text-amber-500" />
                                <span>
                                    <strong>Header Quick-Access:</strong> You can toggle the global floating debug overlay from any page at any time by clicking the <strong>Bug icon</strong> in the top header navbar.
                                </span>
                            </div>

                            {/* Title and toggle */}
                            <div className="flex items-center justify-between">
                                <div>
                                    <h2 className="text-lg font-semibold text-gray-900 dark:text-base-content">
                                        {t('settings.debug.title')}
                                    </h2>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                                        {t('settings.debug.desc')}
                                    </p>
                                </div>
                                <label className="relative inline-flex items-center cursor-pointer">
                                    <input
                                        type="checkbox"
                                        className="sr-only peer"
                                        checked={isEnabled}
                                        onChange={(e) => e.target.checked ? enable() : disable()}
                                    />
                                    <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 dark:peer-focus:ring-blue-800 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                                    <span className="ml-3 text-sm font-medium text-gray-700 dark:text-gray-300">
                                        {isEnabled ? t('settings.debug.enabled') : t('settings.debug.disabled')}
                                    </span>
                                </label>
                            </div>

                            {/* Console or placeholder */}
                            {isEnabled ? (
                                <div className="h-[calc(100vh-320px)] min-h-[400px]">
                                    <DebugConsole embedded />
                                </div>
                            ) : (
                                <div className="h-[calc(100vh-320px)] min-h-[400px] flex items-center justify-center bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-200 dark:border-base-300">
                                    <div className="text-center">
                                        <p className="text-gray-500 dark:text-gray-400 text-lg font-medium">
                                            {t('settings.debug.disabled_hint')}
                                        </p>
                                        <p className="text-gray-400 dark:text-gray-500 text-sm mt-2">
                                            {t('settings.debug.disabled_desc')}
                                        </p>
                                    </div>
                                </div>
                            )}
                        </div>
    );
}
