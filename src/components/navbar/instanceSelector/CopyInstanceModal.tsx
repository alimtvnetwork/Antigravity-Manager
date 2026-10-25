import { useState, useEffect } from 'react';
import { Copy, X, Check, Layers, FolderSync, RotateCw } from 'lucide-react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useErrorStore } from '../../../stores/error-store';
import type { CloneProgressPayload } from '../../../services/instanceService/crud';
import type { InstanceSelectorApi } from './useInstanceSelector';

export function CopyInstanceModal({ api }: { api: InstanceSelectorApi }) {
    const {
        t,
        isCopyOpen,
        setIsCopyOpen,
        copyInstanceName,
        setCopyInstanceName,
        handleCopy,
    } = api;

    const [isCloning, setIsCloning] = useState(false);
    const [cloneProgress, setCloneProgress] = useState<CloneProgressPayload | null>(null);

    useEffect(() => {
        let unlisten: UnlistenFn | undefined;
        listen<CloneProgressPayload>('instance-clone-progress', (event) => {
            setCloneProgress(event.payload);
        }).then((fn) => {
            unlisten = fn;
        }).catch((e) => {
            useErrorStore.getState().trackWarning(e, {
                source: 'CopyInstanceModal.listen',
                triggerAction: 'instance-clone-progress',
            });
        });

        return () => {
            if (unlisten) unlisten();
        };
    }, []);

    if (!isCopyOpen) return null;

    const close = () => {
        if (isCloning) return;
        setIsCopyOpen(false);
    };

    const onSubmit = async () => {
        if (!copyInstanceName.trim() || isCloning) return;
        setIsCloning(true);
        setCloneProgress({
            source_id: '',
            target_name: copyInstanceName.trim(),
            stage: 'initializing',
            percentage: 10,
            message: 'Initializing clone environment...',
            timestamp_ms: Date.now(),
        });
        try {
            await handleCopy();
        } finally {
            setIsCloning(false);
            setCloneProgress(null);
        }
    };

    return (
        <div
            className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[99999] p-4"
            onClick={close}
        >
            <div
                className="bg-white dark:bg-[#090611] rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-200 dark:border-[#171025] ring-1 ring-black/5 dark:ring-white/5 transition-all"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between pb-3.5 border-b border-gray-100 dark:border-[#171025] mb-4">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2.5 rounded-xl bg-blue-50 dark:bg-[#171025] text-blue-600 dark:text-cyan-400 border border-blue-200/60 dark:border-cyan-500/30 shadow-xs">
                            <Copy className="w-5 h-5" />
                        </div>
                        <div>
                            <h3 className="font-bold text-sm text-gray-900 dark:text-white tracking-tight">
                                {t('instances.copy_modal_title', 'Duplicate / Clone Profile')}
                            </h3>
                            <p className="text-[11px] text-gray-500 dark:text-slate-400 mt-0.5">
                                {t('instances.copy_modal_subtitle', 'Create an isolated duplicate of this profile environment')}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={close}
                        disabled={isCloning}
                        className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:text-slate-400 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-[#122e47] transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                <label className="block text-xs font-semibold text-gray-700 dark:text-slate-200 mb-1.5">
                    {t('instances.copy_name_label', 'New Profile Name')}
                </label>
                <input
                    type="text"
                    disabled={isCloning}
                    placeholder={t('instances.copy_placeholder', 'New profile name')}
                    value={copyInstanceName}
                    onChange={(e) => setCopyInstanceName(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && onSubmit()}
                    className="w-full px-4 py-2.5 bg-gray-50 dark:bg-[#051320] border border-gray-200 dark:border-[#173d5c] text-gray-900 dark:text-white rounded-xl mb-4 text-xs font-medium focus:outline-none focus:ring-2 focus:ring-blue-500/40 dark:focus:ring-cyan-500/40 focus:border-blue-500 dark:focus:border-cyan-400 transition-all shadow-inner placeholder-gray-400 dark:placeholder-slate-500 disabled:opacity-50"
                    autoFocus
                />

                {/* Locked Full Clone Information Card */}
                <div className="p-3.5 rounded-xl border border-cyan-500/40 bg-cyan-50/70 dark:bg-[#171025] ring-1 ring-cyan-500/20 shadow-xs mb-3.5">
                    <div className="flex items-center justify-between gap-2 mb-1">
                        <div className="flex items-center gap-1.5 font-bold text-xs text-gray-900 dark:text-white">
                            <Layers className="w-3.5 h-3.5 text-cyan-600 dark:text-cyan-400" />
                            <span>{t('instances.clone_mode_full', 'IDE Copy (Full Environment & Sessions)')}</span>
                        </div>
                        <span className="px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-emerald-100 dark:bg-emerald-950/80 text-emerald-800 dark:text-emerald-300 border border-emerald-300/50 dark:border-emerald-500/30 flex items-center gap-1">
                            <Check className="w-2.5 h-2.5 stroke-[3]" />
                            Locked: Full Clone
                        </span>
                    </div>
                    <p className="text-[11px] text-gray-600 dark:text-slate-300 leading-relaxed">
                        {t('instances.clone_mode_full_desc', 'Clones complete isolated environment, sessions, extensions, cache, and state.')}
                    </p>
                </div>

                {/* Locked Workspace Projects Pill */}
                <div className="p-2.5 rounded-xl border border-gray-200 dark:border-[#171025] bg-gray-50/60 dark:bg-[#051320] flex items-center justify-between gap-2 mb-4 text-[11px] font-medium text-gray-700 dark:text-slate-300">
                    <div className="flex items-center gap-2">
                        <FolderSync className="w-3.5 h-3.5 text-emerald-500 dark:text-emerald-400 shrink-0" />
                        <span>{t('instances.copy_projects_label', 'Workspace projects, recent paths, and states included')}</span>
                    </div>
                    <span className="text-[10px] font-bold text-emerald-600 dark:text-emerald-400">Always Included</span>
                </div>

                {/* Real-time Clone Progress Indicator */}
                {isCloning && (
                    <div className="mb-4 p-3 rounded-xl border border-cyan-500/30 bg-cyan-50/50 dark:bg-cyan-950/20">
                        <div className="flex items-center justify-between text-xs mb-1.5 font-medium">
                            <span className="text-gray-700 dark:text-slate-200 truncate pr-2">
                                {cloneProgress?.message || 'Cloning instance...'}
                            </span>
                            <span className="font-bold text-cyan-600 dark:text-cyan-400 shrink-0">
                                {cloneProgress?.percentage ?? 10}%
                            </span>
                        </div>
                        <div className="w-full bg-gray-200 dark:bg-gray-800 rounded-full h-1.5 overflow-hidden">
                            <div
                                className="bg-gradient-to-r from-cyan-500 to-blue-500 h-1.5 rounded-full transition-all duration-300 ease-out"
                                style={{ width: `${cloneProgress?.percentage ?? 10}%` }}
                            />
                        </div>
                    </div>
                )}

                <div className="flex justify-end items-center gap-2.5 pt-3 border-t border-gray-100 dark:border-[#171025]">
                    <button
                        type="button"
                        onClick={close}
                        disabled={isCloning}
                        className="px-4 py-2.5 rounded-xl text-xs font-semibold text-gray-600 dark:text-slate-300 border border-gray-200 dark:border-[#171025] hover:bg-gray-100 dark:hover:bg-[#102a40] transition-all duration-200 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>
                    <button
                        type="button"
                        onClick={onSubmit}
                        disabled={!copyInstanceName.trim() || isCloning}
                        className="px-5 py-2.5 bg-gradient-to-r from-blue-600 via-indigo-600 to-cyan-500 hover:from-blue-500 hover:via-indigo-500 hover:to-cyan-400 text-white font-bold text-xs rounded-xl shadow-lg shadow-blue-500/25 dark:shadow-cyan-500/20 hover:shadow-cyan-500/40 transition-all duration-200 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
                    >
                        {isCloning ? (
                            <>
                                <RotateCw className="w-3.5 h-3.5 animate-spin" />
                                <span>Cloning ({cloneProgress?.percentage ?? 10}%)...</span>
                            </>
                        ) : (
                            <>
                                <Copy className="w-3.5 h-3.5" />
                                <span>{t('instances.duplicate', 'Duplicate')}</span>
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>
    );
}
