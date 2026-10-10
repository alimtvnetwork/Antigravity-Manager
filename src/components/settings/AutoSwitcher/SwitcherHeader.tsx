import { AutoSwitcherApi } from './useAutoSwitcher';

export function SwitcherHeader(props: AutoSwitcherApi) {
    const { t, currentConfig, isActionsOpen, setIsActionsOpen, actionsRef, fileInputRef, handleExport, handleImportFile, handleResetDefaults, handleToggleEnabled, onChange } = props;

    return (
        <>
                <div className="flex items-center justify-between gap-3 flex-wrap">
                    <div className="flex items-center gap-4">
                        <div
                            className={`w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-300 ${currentConfig.is_enabled
                                ? 'bg-blue-600 text-white shadow-md shadow-blue-500/20'
                                : 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400'
                                }`}
                        >
                            <RotateCw size={20} className={currentConfig.is_enabled ? 'animate-spin-slow' : ''} />
                        </div>
                        <div>
                            <div className="font-bold text-gray-900 dark:text-gray-100">
                                {t('settings.auto_switcher.title', 'Auto Profile Switcher & Task Resumption')}
                            </div>
                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                {t(
                                    'settings.auto_switcher.desc',
                                    'Monitors the active IDE profile on a timer and automatically switches to the next best profile when quota drops below threshold, resuming pending tasks.'
                                )}
                            </p>
                        </div>
                    </div>

                    <div className="flex items-center gap-3">
                        <input
                            type="file"
                            ref={fileInputRef}
                            accept=".json,application/json"
                            style={{ display: 'none' }}
                            onChange={handleImportFile}
                        />

                        {/* Import / Export Actions Dropdown */}
                        <div className="relative" ref={actionsRef}>
                            <button
                                type="button"
                                onClick={() => setIsActionsOpen(!isActionsOpen)}
                                className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-800 text-gray-700 dark:text-slate-300 hover:bg-gray-50 dark:hover:bg-slate-700 transition flex items-center gap-1.5 cursor-pointer shadow-2xs"
                                title="Import, Export, or Reset Switcher Config"
                            >
                                <SlidersHorizontal className="w-3.5 h-3.5" />
                                <span>Actions</span>
                                <ChevronDown className={`w-3.5 h-3.5 transition-transform duration-150 ${isActionsOpen ? 'rotate-180' : ''}`} />
                            </button>

                            {isActionsOpen && (
                                <div className="absolute right-0 mt-1.5 w-44 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-lg z-50 py-1 text-xs animate-in fade-in zoom-in-95">
                                    <button
                                        type="button"
                                        onClick={handleExport}
                                        className="w-full px-3 py-1.5 text-left text-gray-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                    >
                                        <Download className="w-3.5 h-3.5 text-blue-500" />
                                        <span>Export Config</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setIsActionsOpen(false);
                                            fileInputRef.current?.click();
                                        }}
                                        className="w-full px-3 py-1.5 text-left text-gray-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                    >
                                        <Upload className="w-3.5 h-3.5 text-emerald-500" />
                                        <span>Import Config</span>
                                    </button>
                                    <div className="border-t border-gray-100 dark:border-slate-800 my-1" />
                                    <button
                                        type="button"
                                        onClick={handleResetDefaults}
                                        className="w-full px-3 py-1.5 text-left text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 flex items-center gap-2 transition-colors cursor-pointer"
                                    >
                                        <RotateCcw className="w-3.5 h-3.5" />
                                        <span>Reset to Defaults</span>
                                    </button>
                                </div>
                            )}
                        </div>

                        <label className="relative inline-flex items-center cursor-pointer">
                            <input
                                type="checkbox"
                                className="sr-only peer"
                                checked={currentConfig.is_enabled}
                                onChange={(e) => handleToggleEnabled(e.target.checked)}
                            />
                            <div className="w-11 h-6 bg-gray-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600 shadow-inner"></div>
                        </label>
                    </div>
                </div>

        </>
    );
}
