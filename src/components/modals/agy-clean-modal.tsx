import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import {
    Sparkles,
    Trash2,
    RotateCcw,
    X,
    FolderArchive,
    CheckCircle2,
    AlertTriangle,
    Database,
    Loader2,
    Info,
} from 'lucide-react';
import { request as invoke } from '../../utils/request';
import { showToast } from '../common/ToastContainer';

export interface PreflightReport {
    total_conversations: number;
    keep_count: number;
    preserved_count: number;
    pruned_count: number;
    total_conversation_bytes: number;
    projected_reclaimed_bytes: number;
    cache_paths_count: number;
    cache_bytes: number;
    cache_targets: string[];
    staging_dir: string;
}

export interface PruneResult {
    transaction_id: string;
    keep_count: number;
    preserved_count: number;
    pruned_count: number;
    pruned_bytes: number;
    cache_cleared_bytes: number;
    total_freed_bytes: number;
    staging_dir: string;
    errors: string[];
}

export interface UndoResult {
    transaction_id: string;
    restored_conversations: number;
    restored_bytes: number;
    errors: string[];
}

interface AgyCleanModalProps {
    isOpen: boolean;
    onClose: () => void;
}

function formatBytes(bytes?: number | null): string {
    if (!bytes || isNaN(bytes) || bytes <= 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const safeI = Math.min(Math.max(0, i), units.length - 1);
    const converted = bytes / Math.pow(1024, safeI);
    return `${converted.toFixed(converted >= 10 || safeI === 0 ? 1 : 2)} ${units[safeI]}`;
}

export function AgyCleanModal({ isOpen, onClose }: AgyCleanModalProps) {
    const { t } = useTranslation();
    const [keepCount, setKeepCount] = useState<number>(10);
    const [preflight, setPreflight] = useState<PreflightReport | null>(null);
    const [isLoadingPreflight, setIsLoadingPreflight] = useState<boolean>(false);
    const [isPruning, setIsPruning] = useState<boolean>(false);
    const [isUndoing, setIsUndoing] = useState<boolean>(false);
    const [lastPruned, setLastPruned] = useState<PruneResult | null>(null);

    const loadPreflight = useCallback(async (count: number) => {
        setIsLoadingPreflight(true);
        try {
            const report = await invoke<PreflightReport>('preflight_antigravity_clean', { keepCount: count });
            setPreflight(report);
        } catch (err) {
            showToast(`${t('common.error', 'Error')}: ${err}`, 'error');
        } finally {
            setIsLoadingPreflight(false);
        }
    }, [t]);

    useEffect(() => {
        if (isOpen) {
            loadPreflight(keepCount);
        }
    }, [isOpen, keepCount, loadPreflight]);

    if (!isOpen) {
        return null;
    }

    const handleApplyPreset = (count: number) => {
        setKeepCount(count);
    };

    const handleCleanNow = async () => {
        setIsPruning(true);
        try {
            const result = await invoke<PruneResult>('prune_antigravity_conversations', { keepCount });
            setLastPruned(result);
            showToast(
                t(
                    'agy_clean.clean_success',
                    `Pruned ${result.pruned_count} conversations. Freed ${formatBytes(result.total_freed_bytes)}.`
                ),
                'success'
            );
            await loadPreflight(keepCount);
        } catch (err) {
            showToast(`${t('common.error', 'Error')}: ${err}`, 'error');
        } finally {
            setIsPruning(false);
        }
    };

    const handleCleanConversationsOnly = async () => {
        setIsPruning(true);
        try {
            const result = await invoke<PruneResult>('prune_antigravity_conversations_only', { keepCount });
            setLastPruned(result);
            showToast(
                t(
                    'agy_clean.clean_convs_success',
                    `Pruned ${result.pruned_count} conversations. Freed ${formatBytes(result.total_freed_bytes)}.`
                ),
                'success'
            );
            await loadPreflight(keepCount);
        } catch (err) {
            showToast(`${t('common.error', 'Error')}: ${err}`, 'error');
        } finally {
            setIsPruning(false);
        }
    };

    const handleUndoLast = async () => {
        setIsUndoing(true);
        try {
            const result = await invoke<UndoResult>('undo_antigravity_prune', { transactionId: null });
            showToast(
                t(
                    'agy_clean.undo_success',
                    `Restored ${result.restored_conversations} conversations (${formatBytes(result.restored_bytes)}).`
                ),
                'success'
            );
            setLastPruned(null);
            await loadPreflight(keepCount);
        } catch (err) {
            showToast(`${t('common.error', 'Error')}: ${err}`, 'error');
        } finally {
            setIsUndoing(false);
        }
    };

    const estFreedBytes = (preflight?.projected_reclaimed_bytes || 0) + (preflight?.cache_bytes || 0);

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs animate-in fade-in duration-200">
            <div className="relative w-full max-w-xl bg-white dark:bg-slate-900 rounded-2xl shadow-2xl border border-gray-200 dark:border-slate-800 overflow-hidden">
                {/* Modal Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100 dark:border-slate-800 bg-gray-50/50 dark:bg-slate-800/50">
                    <div className="flex items-center gap-3">
                        <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
                            <RotateCcw className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-base font-semibold text-gray-900 dark:text-gray-100">
                                {t('agy_clean.title', 'Antigravity Quick Clean & Retention')}
                            </h2>
                            <p className="text-xs text-gray-500 dark:text-gray-400">
                                {t('agy_clean.subtitle', 'Prune older conversations to OS temp storage & flush caches')}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Modal Body */}
                <div className="p-6 space-y-5 max-h-[80vh] overflow-y-auto">
                    {/* Retention Presets */}
                    <div>
                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider mb-2">
                            {t('agy_clean.keep_label', 'Recent Conversations to Retain')}
                        </label>
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={() => handleApplyPreset(1)}
                                className={`px-3 py-1.5 text-xs font-medium rounded-lg border transition-all ${
                                    keepCount === 1
                                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
                                        : 'border-gray-200 dark:border-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-800'
                                }`}
                            >
                                {t('agy_clean.keep_one', 'Keep 1 (ccko)')}
                            </button>
                            <button
                                type="button"
                                onClick={() => handleApplyPreset(5)}
                                className={`px-3 py-1.5 text-xs font-medium rounded-lg border transition-all ${
                                    keepCount === 5
                                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
                                        : 'border-gray-200 dark:border-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-800'
                                }`}
                            >
                                {t('agy_clean.keep_five', 'Keep 5 (cckf)')}
                            </button>
                            <button
                                type="button"
                                onClick={() => handleApplyPreset(10)}
                                className={`px-3 py-1.5 text-xs font-medium rounded-lg border transition-all ${
                                    keepCount === 10
                                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
                                        : 'border-gray-200 dark:border-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-800'
                                }`}
                            >
                                {t('agy_clean.keep_ten', 'Keep 10 (Default)')}
                            </button>
                            <button
                                type="button"
                                onClick={() => handleApplyPreset(40)}
                                className={`px-3 py-1.5 text-xs font-medium rounded-lg border transition-all ${
                                    keepCount === 40
                                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 font-semibold shadow-xs'
                                        : 'border-gray-200 dark:border-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-slate-800'
                                }`}
                            >
                                {t('agy_clean.keep_forty', 'Keep 40 (Telemetry)')}
                            </button>
                            <div className="ml-auto flex items-center gap-1.5">
                                <span className="text-xs text-gray-500 dark:text-gray-400">Custom:</span>
                                <input
                                    type="number"
                                    min="1"
                                    max="500"
                                    className="w-16 px-2.5 py-1 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-gray-50 dark:bg-slate-800 text-gray-900 dark:text-gray-100 focus:outline-hidden focus:ring-2 focus:ring-blue-500"
                                    value={keepCount}
                                    onChange={(e) => setKeepCount(Math.max(1, parseInt(e.target.value) || 1))}
                                />
                            </div>
                        </div>
                    </div>

                    {/* Preflight Statistics */}
                    {isLoadingPreflight ? (
                        <div className="flex items-center justify-center py-8 text-gray-500 dark:text-gray-400 gap-2">
                            <Loader2 className="w-5 h-5 animate-spin text-blue-500" />
                            <span className="text-sm">{t('agy_clean.calculating', 'Analyzing conversations & caches...')}</span>
                        </div>
                    ) : preflight ? (
                        <div className="space-y-3">
                            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
                                <div className="p-3 bg-gray-50 dark:bg-slate-800/80 rounded-xl border border-gray-100 dark:border-slate-800">
                                    <div className="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1">
                                        <Database className="w-3.5 h-3.5" />
                                        <span>{t('agy_clean.total', 'Total Found')}</span>
                                    </div>
                                    <div className="mt-1 text-lg font-bold text-gray-900 dark:text-gray-100">
                                        {preflight.total_conversations}
                                    </div>
                                </div>
                                <div className="p-3 bg-emerald-50 dark:bg-emerald-950/20 rounded-xl border border-emerald-100 dark:border-emerald-900/30">
                                    <div className="text-xs text-emerald-600 dark:text-emerald-400 flex items-center gap-1">
                                        <CheckCircle2 className="w-3.5 h-3.5" />
                                        <span>{t('agy_clean.preserved', 'Preserved')}</span>
                                    </div>
                                    <div className="mt-1 text-lg font-bold text-emerald-700 dark:text-emerald-300">
                                        {preflight.preserved_count}
                                    </div>
                                </div>
                                <div className="p-3 bg-amber-50 dark:bg-amber-950/20 rounded-xl border border-amber-100 dark:border-amber-900/30">
                                    <div className="text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1">
                                        <FolderArchive className="w-3.5 h-3.5" />
                                        <span>{t('agy_clean.staged', 'To Stage')}</span>
                                    </div>
                                    <div className="mt-1 text-lg font-bold text-amber-700 dark:text-amber-300">
                                        {preflight.pruned_count}
                                    </div>
                                </div>
                                <div className="p-3 bg-blue-50 dark:bg-blue-950/20 rounded-xl border border-blue-100 dark:border-blue-900/30">
                                    <div className="text-xs text-blue-600 dark:text-blue-400 flex items-center gap-1">
                                        <Sparkles className="w-3.5 h-3.5" />
                                        <span>{t('agy_clean.freed', 'Est. Freed')}</span>
                                    </div>
                                    <div className="mt-1 text-lg font-bold text-blue-700 dark:text-blue-300">
                                        {formatBytes(estFreedBytes)}
                                    </div>
                                </div>
                            </div>

                            {/* Safety & Temporary Storage Information */}
                            <div className="p-3.5 bg-blue-50/60 dark:bg-blue-950/20 border border-blue-200/50 dark:border-blue-800/40 rounded-xl text-xs space-y-1.5">
                                <div className="flex items-center gap-1.5 font-medium text-blue-800 dark:text-blue-300">
                                    <Info className="w-4 h-4 shrink-0 text-blue-600 dark:text-blue-400" />
                                    <span>{t('agy_clean.safety_title', 'Reversible Staging Protection')}</span>
                                </div>
                                <p className="text-blue-700 dark:text-blue-300/90 leading-relaxed">
                                    {t(
                                        'agy_clean.safety_desc',
                                        'Older conversations are moved safely into OS temporary storage with transaction manifests. You can undo anytime via this modal or running `agy undo` in terminal.'
                                    )}
                                </p>
                                <div className="pt-1 text-[11px] font-mono text-gray-500 dark:text-gray-400 break-all">
                                    {preflight.staging_dir}
                                </div>
                            </div>

                            {/* OS Temp Caveat */}
                            <div className="p-3 bg-amber-50/50 dark:bg-amber-950/20 border border-amber-200/40 dark:border-amber-800/30 rounded-xl text-xs flex items-start gap-2">
                                <AlertTriangle className="w-4 h-4 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
                                <span className="text-amber-800 dark:text-amber-300 leading-relaxed">
                                    {t(
                                        'agy_clean.temp_notice',
                                        'Note: Operating systems can purge temporary storage during disk cleanup or reboot. Undo backups while current OS session is active.'
                                    )}
                                </span>
                            </div>
                        </div>
                    ) : null}

                    {/* Last Result summary if available */}
                    {lastPruned ? (
                        <div className="p-3 bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-200 dark:border-emerald-800/40 rounded-xl text-xs text-emerald-800 dark:text-emerald-300 flex items-center justify-between">
                            <span>
                                {t('agy_clean.last_clean_msg', 'Last transaction completed:')}{' '}
                                <strong>{lastPruned.pruned_count} pruned</strong>,{' '}
                                <strong>{formatBytes(lastPruned.total_freed_bytes)} freed</strong>.
                            </span>
                            <span className="font-mono text-[10px] text-emerald-600 dark:text-emerald-400">
                                {lastPruned.transaction_id}
                            </span>
                        </div>
                    ) : null}
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-between px-6 py-4 border-t border-gray-100 dark:border-slate-800 bg-gray-50/50 dark:bg-slate-800/50">
                    <button
                        type="button"
                        onClick={handleUndoLast}
                        disabled={isUndoing || isPruning}
                        className="inline-flex items-center gap-1.5 px-3.5 py-2 text-xs font-medium rounded-xl border border-gray-300 dark:border-slate-700 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-slate-800 disabled:opacity-50 transition-all cursor-pointer"
                    >
                        {isUndoing ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        ) : (
                            <RotateCcw className="w-3.5 h-3.5 text-blue-500" />
                        )}
                        <span>{t('agy_clean.undo_btn', 'Undo Last Clean')}</span>
                    </button>

                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-3 py-2 text-xs font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 rounded-xl transition-colors cursor-pointer"
                        >
                            {t('common.cancel', 'Cancel')}
                        </button>
                        <button
                            type="button"
                            onClick={handleCleanConversationsOnly}
                            disabled={isPruning || isUndoing || isLoadingPreflight || (preflight?.pruned_count === 0)}
                            className="inline-flex items-center gap-1.5 px-3.5 py-2 text-xs font-medium text-blue-700 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800 hover:bg-blue-100 dark:hover:bg-blue-900/50 rounded-xl shadow-xs disabled:opacity-50 transition-all cursor-pointer"
                            title="Prune old conversations without wiping application caches"
                        >
                            {isPruning ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Database className="w-3.5 h-3.5" />
                            )}
                            <span>{t('agy_clean.conversations_only_btn', 'Clean Conversations Only')}</span>
                        </button>
                        <button
                            type="button"
                            onClick={handleCleanNow}
                            disabled={isPruning || isUndoing || isLoadingPreflight || (preflight?.pruned_count === 0 && estFreedBytes === 0)}
                            className="inline-flex items-center gap-1.5 px-4 py-2 text-xs font-medium text-white bg-blue-600 hover:bg-blue-500 rounded-xl shadow-xs disabled:opacity-50 transition-all cursor-pointer"
                        >
                            {isPruning ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Trash2 className="w-3.5 h-3.5" />
                            )}
                            <span>{t('agy_clean.execute_btn', 'Clean & Prune All')}</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
