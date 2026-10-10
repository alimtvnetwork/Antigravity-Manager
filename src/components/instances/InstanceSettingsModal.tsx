import { X, SlidersHorizontal, RotateCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useInstanceSettings } from './settingsModal/useInstanceSettings';
import type { InstanceSettingsModalProps } from './settingsModal/instanceSettingsTypes';
import { SettingsQuickToggles } from './settingsModal/SettingsQuickToggles';
import { SettingsSyncSection } from './settingsModal/SettingsSyncSection';
import { SettingsJsonSection } from './settingsModal/SettingsJsonSection';

export function InstanceSettingsModal({
    isOpen,
    onClose,
    targetInstance,
    instances,
    onInstancesUpdated,
}: InstanceSettingsModalProps) {
    const { t } = useTranslation();
    const api = useInstanceSettings(isOpen, onClose, targetInstance, instances, onInstancesUpdated);
    const {
        selectedTargetId,
        currentInstanceObj,
        isLoading,
        fileInputRef,
        handleTargetChange,
        handleImportJsonFile,
    } = api;

    if (!isOpen) return null;

    return (
        <div
            className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={onClose}
        >
            <input
                ref={fileInputRef}
                type="file"
                accept=".json,application/json"
                style={{ display: 'none' }}
                onChange={handleImportJsonFile}
            />

            <div
                className="bg-white dark:bg-[#0c2438] rounded-2xl w-full max-w-2xl max-h-[90vh] shadow-2xl border border-gray-200 dark:border-[#15334d] flex flex-col overflow-hidden text-xs text-gray-900 dark:text-gray-100 animate-in fade-in zoom-in-95"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Modal Header */}
                <div className="flex items-center justify-between px-5 py-3.5 border-b border-gray-100 dark:border-[#15334d] bg-gray-50/70 dark:bg-[#091e30]">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-cyan-400">
                            <SlidersHorizontal className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                                <span>{t('instances.settings_modal_title', 'Instance Settings & Deep Sync')}</span>
                                {isLoading && <RotateCw className="w-3.5 h-3.5 animate-spin text-blue-500" />}
                            </h2>
                            <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                {t(
                                    'instances.settings_modal_desc',
                                    'Synchronize Turbo Mode, Plan Review, themes, projects, and JSON configurations across profiles'
                                )}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Target Instance Bar */}
                <div className="px-5 py-2.5 bg-blue-50/40 dark:bg-[#081a28] border-b border-gray-100 dark:border-[#15334d] flex items-center justify-between gap-3 flex-wrap">
                    <div className="flex items-center gap-2 flex-1 min-w-0">
                        <span className="font-semibold text-gray-700 dark:text-gray-300 shrink-0">Target Profile:</span>
                        <select
                            value={selectedTargetId}
                            onChange={(e) => handleTargetChange(e.target.value)}
                            className="min-w-0 flex-1 truncate bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] rounded-lg px-2.5 py-1 text-xs font-bold text-blue-700 dark:text-cyan-300 focus:outline-none focus:ring-1 focus:ring-blue-500 cursor-pointer"
                        >
                            {instances.map((inst, idx) => {
                                const seq = inst.config.seq_num ?? idx + 1;
                                const idSuffix = inst.config.id.startsWith('antigravity-')
                                    ? inst.config.id
                                    : `antigravity-${inst.config.id}`;
                                const isDefault = inst.config.is_default || inst.config.id === 'default';
                                return (
                                    <option key={inst.config.id} value={inst.config.id}>
                                        #{seq} {inst.config.name} ({idSuffix}){isDefault ? ' [Default]' : ''}
                                    </option>
                                );
                            })}
                        </select>
                    </div>

                    <div className="flex items-center gap-1.5 text-[11px] text-gray-500 dark:text-gray-400 font-mono shrink-0">
                        <span className="px-2 py-0.5 rounded bg-gray-100 dark:bg-[#15334d] border border-gray-200 dark:border-[#1e466b]">
                            ID: {selectedTargetId}
                        </span>
                        {currentInstanceObj?.is_running && (
                            <span className="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-bold border border-emerald-500/20">
                                Running
                            </span>
                        )}
                    </div>
                </div>

                {/* Scrollable Content */}
                <div className="flex-1 overflow-y-auto overflow-x-hidden p-5 space-y-4">
                    <SettingsQuickToggles api={api} />
                    <SettingsSyncSection api={api} instances={instances} />
                    <SettingsJsonSection api={api} />
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-between px-5 py-3 border-t border-gray-100 dark:border-[#15334d] bg-gray-50/70 dark:bg-[#091e30]">
                    <span className="text-[11px] text-gray-400">
                        Targeting:{' '}
                        <strong className="text-gray-700 dark:text-gray-200">
                            {currentInstanceObj?.config.name || selectedTargetId}
                        </strong>
                    </span>
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-1.5 rounded-xl text-xs font-semibold text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-[#15334d] hover:bg-gray-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                    >
                        {t('common.done', 'Done')}
                    </button>
                </div>
            </div>
        </div>
    );
}
