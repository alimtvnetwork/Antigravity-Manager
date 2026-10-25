import { useState } from 'react';
import {
    X,
    AlertTriangle,
    RotateCw,
    Trash2,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../utils/cn';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';

export interface PurgeReinstallModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
}

interface PurgeSummary {
    processes_terminated: number;
    directories_purged: string[];
    bytes_reclaimed: number;
    accounts_safeguarded: boolean;
}

export function PurgeReinstallModal({
    isOpen,
    onClose,
    onSuccess,
}: PurgeReinstallModalProps) {
    const [keepAccounts, setKeepAccounts] = useState(true);
    const [purgeInstances, setPurgeInstances] = useState(true);
    const [uninstallBinaries, setUninstallBinaries] = useState(false);
    const [force, setForce] = useState(false);
    const [confirmationInput, setConfirmationInput] = useState('');
    const [isExecuting, setIsExecuting] = useState(false);
    const [activeStage, setActiveStage] = useState<number | null>(null);

    if (!isOpen) return null;

    const isConfirmed = confirmationInput.trim().toUpperCase() === 'PURGE';

    const handleExecutePurge = async () => {
        if (!isConfirmed) {
            showToast('Please type "PURGE" to confirm this destructive operation', 'warning');
            return;
        }

        setIsExecuting(true);
        setActiveStage(1);

        try {
            // Stage progression simulation while awaiting IPC call
            const timer1 = setTimeout(() => setActiveStage(2), 500);
            const timer2 = setTimeout(() => setActiveStage(3), 1200);

            const summary = await invoke<PurgeSummary>('execute_purge_reinstall', {
                options: {
                    keep_accounts: keepAccounts,
                    purge_instances: purgeInstances,
                    uninstall_binaries: uninstallBinaries,
                    force,
                },
            });

            clearTimeout(timer1);
            clearTimeout(timer2);
            setActiveStage(4);

            const reclaimedMb = (summary.bytes_reclaimed / (1024 * 1024)).toFixed(1);
            showToast(
                `Clean purge completed! Reclaimed ${reclaimedMb} MB across ${summary.directories_purged.length} directories`,
                'success'
            );
            onSuccess?.();
            setTimeout(() => {
                onClose();
            }, 800);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'PurgeReinstallModal.handleExecutePurge',
                triggerAction: 'execute_purge_reinstall',
            });
            showToast(`Purge failed: ${String(err)}`, 'error');
        } finally {
            setIsExecuting(false);
            setActiveStage(null);
        }
    };

    return (
        <div
            className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={onClose}
        >
            <div
                className="bg-white dark:bg-[#171025] rounded-2xl w-full max-w-xl max-h-[90vh] shadow-2xl border border-rose-300 dark:border-rose-900/60 flex flex-col overflow-hidden text-xs text-gray-900 dark:text-gray-100 animate-in fade-in zoom-in-95"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Modal Header */}
                <div className="flex items-center justify-between px-5 py-3.5 border-b border-rose-100 dark:border-rose-950/60 bg-rose-50/70 dark:bg-[#200c17]">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2 rounded-xl bg-rose-100 dark:bg-rose-900/40 text-rose-600 dark:text-rose-400">
                            <Trash2 className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold text-gray-900 dark:text-rose-100 flex items-center gap-2">
                                <span>Clean Purge & Reinstall</span>
                                {isExecuting && (
                                    <RotateCw className="w-3.5 h-3.5 animate-spin text-rose-500" />
                                )}
                            </h2>
                            <p className="text-[11px] text-gray-500 dark:text-rose-300/70">
                                Eradicate memory fog, corrupted brain state, transcript logs, and caches
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        disabled={isExecuting}
                        className="p-1.5 rounded-lg hover:bg-rose-100 dark:hover:bg-rose-900/40 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors cursor-pointer disabled:opacity-50"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Modal Body */}
                <div className="flex-1 overflow-y-auto p-5 space-y-4">
                    {/* Caution Banner */}
                    <div className="p-3.5 bg-rose-500/10 dark:bg-rose-950/30 border border-rose-300 dark:border-rose-800/80 rounded-xl flex items-start gap-3">
                        <AlertTriangle className="w-5 h-5 text-rose-600 dark:text-rose-400 shrink-0 mt-0.5" />
                        <div className="space-y-1 text-[11px] text-rose-900 dark:text-rose-200">
                            <span className="font-bold block">Destructive Lifecycle Maintenance</span>
                            <p>
                                This operation stops all background Antigravity processes, purges SQLite transcripts,
                                wipes <code className="px-1 py-0.5 bg-rose-100 dark:bg-rose-900/50 rounded text-rose-700 dark:text-rose-300 font-mono">~/.gemini/antigravity</code> caches,
                                and re-provisions a pristine default runtime environment.
                            </p>
                        </div>
                    </div>

                    {/* Safeguard & Scope Checklist */}
                    <div className="space-y-2.5 bg-slate-50 dark:bg-[#1a122c] p-3.5 rounded-xl border border-gray-200/80 dark:border-[#40285e]">
                        <span className="text-[11px] font-bold text-gray-600 dark:text-gray-400 uppercase tracking-wider block mb-1">
                            Purge Scope & Safeguards
                        </span>
                        <label className="flex items-center gap-2.5 cursor-pointer text-xs">
                            <input
                                type="checkbox"
                                checked={keepAccounts}
                                onChange={(e) => setKeepAccounts(e.target.checked)}
                                className="rounded border-gray-300 text-blue-600 focus:ring-0"
                            />
                            <span className="font-semibold text-emerald-700 dark:text-emerald-400">
                                Safeguard and restore authenticated accounts (Recommended)
                            </span>
                        </label>
                        <label className="flex items-center gap-2.5 cursor-pointer text-xs">
                            <input
                                type="checkbox"
                                checked={purgeInstances}
                                onChange={(e) => setPurgeInstances(e.target.checked)}
                                className="rounded border-gray-300 text-blue-600 focus:ring-0"
                            />
                            <span>Purge multi-instance workspaces & local data caches</span>
                        </label>
                        <label className="flex items-center gap-2.5 cursor-pointer text-xs">
                            <input
                                type="checkbox"
                                checked={uninstallBinaries}
                                onChange={(e) => setUninstallBinaries(e.target.checked)}
                                className="rounded border-gray-300 text-rose-600 focus:ring-0"
                            />
                            <span>Full application binary uninstall and fresh re-provisioning</span>
                        </label>
                        <label className="flex items-center gap-2.5 cursor-pointer text-xs pt-1 border-t border-gray-200/60 dark:border-[#40285e]/60">
                            <input
                                type="checkbox"
                                checked={force}
                                onChange={(e) => setForce(e.target.checked)}
                                className="rounded border-gray-300 text-amber-600 focus:ring-0"
                            />
                            <span className="text-amber-700 dark:text-amber-400">
                                Force bypass Caller IDE Self-Preservation protection
                            </span>
                        </label>
                    </div>

                    {/* 4-Stage Progress Tracker */}
                    {isExecuting && (
                        <div className="p-3.5 bg-slate-100/90 dark:bg-[#1b142d] rounded-xl border border-gray-200 dark:border-[#40285e] space-y-2">
                            <span className="text-[11px] font-bold text-gray-700 dark:text-gray-300 uppercase tracking-wider block">
                                Execution Progress
                            </span>
                            <div className="space-y-1.5 text-[11px]">
                                <div className={cn('flex items-center gap-2', (activeStage ?? 0) >= 1 ? 'text-blue-600 dark:text-cyan-400 font-semibold' : 'text-gray-400')}>
                                    <span className="w-4 h-4 rounded-full flex items-center justify-center text-[10px] bg-blue-100 dark:bg-blue-900/50">1</span>
                                    <span>Caller IDE Protection & Process Sweep</span>
                                </div>
                                <div className={cn('flex items-center gap-2', (activeStage ?? 0) >= 2 ? 'text-blue-600 dark:text-cyan-400 font-semibold' : 'text-gray-400')}>
                                    <span className="w-4 h-4 rounded-full flex items-center justify-center text-[10px] bg-blue-100 dark:bg-blue-900/50">2</span>
                                    <span>Credential Safeguard Vault Snapshot</span>
                                </div>
                                <div className={cn('flex items-center gap-2', (activeStage ?? 0) >= 3 ? 'text-blue-600 dark:text-cyan-400 font-semibold' : 'text-gray-400')}>
                                    <span className="w-4 h-4 rounded-full flex items-center justify-center text-[10px] bg-blue-100 dark:bg-blue-900/50">3</span>
                                    <span>Deep Filesystem, Brain State & Electron Cache Wipe</span>
                                </div>
                                <div className={cn('flex items-center gap-2', (activeStage ?? 0) >= 4 ? 'text-emerald-600 dark:text-emerald-400 font-semibold' : 'text-gray-400')}>
                                    <span className="w-4 h-4 rounded-full flex items-center justify-center text-[10px] bg-emerald-100 dark:bg-emerald-900/50">4</span>
                                    <span>Automated Clean Runtime Re-provisioning</span>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Confirmation Input Guard */}
                    <div className="space-y-1.5">
                        <label className="block text-[11px] font-bold text-gray-700 dark:text-gray-300">
                            Type <span className="font-mono text-rose-600 dark:text-rose-400 font-bold uppercase">PURGE</span> to enable execution:
                        </label>
                        <input
                            type="text"
                            placeholder="Type PURGE here..."
                            value={confirmationInput}
                            onChange={(e) => setConfirmationInput(e.target.value)}
                            disabled={isExecuting}
                            className="w-full px-3 py-2 rounded-lg bg-white dark:bg-[#1f1633] border border-gray-300 dark:border-[#40285e] text-xs font-mono font-bold text-rose-600 dark:text-rose-400 focus:outline-none focus:ring-1 focus:ring-rose-500 uppercase placeholder:normal-case placeholder:font-normal"
                        />
                    </div>
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-end gap-2.5 px-5 py-3.5 border-t border-gray-100 dark:border-[#40285e] bg-gray-50/70 dark:bg-[#091e30]">
                    <button
                        type="button"
                        onClick={onClose}
                        disabled={isExecuting}
                        className="px-4 py-1.5 rounded-lg border border-gray-300 dark:border-[#40285e] hover:bg-gray-100 dark:hover:bg-[#1f1633] text-gray-700 dark:text-gray-300 font-medium transition-colors cursor-pointer disabled:opacity-50"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={handleExecutePurge}
                        disabled={!isConfirmed || isExecuting}
                        className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-semibold transition-all shadow-xs cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
                    >
                        {isExecuting ? (
                            <RotateCw className="w-3.5 h-3.5 animate-spin" />
                        ) : (
                            <Trash2 className="w-3.5 h-3.5" />
                        )}
                        <span>Execute Clean Purge & Reinstall</span>
                    </button>
                </div>
            </div>
        </div>
    );
}
