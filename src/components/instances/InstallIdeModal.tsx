import { useState, useEffect } from 'react';
import {
    X,
    Sparkles,
    RotateCw,
    CheckCircle2,
    Download,
    Terminal,
    ChevronDown,
    ChevronUp,
    Laptop,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../utils/cn';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';

export interface InstallIdeModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSuccess?: () => void;
}

interface IdeStatusInfo {
    target: 'Antigravity' | 'Cursor' | 'VsCode';
    display_name: string;
    is_installed: boolean;
    version: string | null;
    executable_path: string | null;
}

interface InstallResult {
    target: string;
    success: boolean;
    installed_path: string | null;
    command_executed: string;
    output: string;
}

export function InstallIdeModal({
    isOpen,
    onClose,
    onSuccess,
}: InstallIdeModalProps) {
    const [ideList, setIdeList] = useState<IdeStatusInfo[]>([
        {
            target: 'Antigravity',
            display_name: 'Google Antigravity IDE',
            is_installed: false,
            version: null,
            executable_path: null,
        },
        {
            target: 'Cursor',
            display_name: 'Cursor AI',
            is_installed: false,
            version: null,
            executable_path: null,
        },
        {
            target: 'VsCode',
            display_name: 'Visual Studio Code',
            is_installed: false,
            version: null,
            executable_path: null,
        },
    ]);

    const [isLoadingStatus, setIsLoadingStatus] = useState(false);
    const [activeInstallingTarget, setActiveInstallingTarget] = useState<string | null>(null);
    const [lastInstallResult, setLastInstallResult] = useState<InstallResult | null>(null);
    const [isLogDrawerOpen, setIsLogDrawerOpen] = useState(false);

    const refreshStatus = async () => {
        setIsLoadingStatus(true);
        try {
            const statuses = await invoke<IdeStatusInfo[]>('detect_ide_installation_status');
            if (Array.isArray(statuses) && statuses.length > 0) {
                setIdeList(statuses);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstallIdeModal.refreshStatus',
                triggerAction: 'detect_ide_installation_status',
            });
        } finally {
            setIsLoadingStatus(false);
        }
    };

    useEffect(() => {
        if (isOpen) {
            refreshStatus();
        }
    }, [isOpen]);

    if (!isOpen) return null;

    const handleInstallTarget = async (targetSlug: string) => {
        setActiveInstallingTarget(targetSlug);
        setLastInstallResult(null);
        setIsLogDrawerOpen(true);

        try {
            const res = await invoke<InstallResult>('install_ide_target', {
                target: targetSlug,
                dryRun: false,
            });

            setLastInstallResult(res);

            if (res.success) {
                showToast(`${res.target} installation completed successfully!`, 'success');
                await refreshStatus();
                onSuccess?.();
            } else {
                showToast(`Installation returned non-zero exit status`, 'warning');
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InstallIdeModal.handleInstallTarget',
                triggerAction: 'install_ide_target',
            });
            showToast(`Installation failed: ${String(err)}`, 'error');
        } finally {
            setActiveInstallingTarget(null);
        }
    };

    return (
        <div
            className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={onClose}
        >
            <div
                className="bg-white dark:bg-[#171025] rounded-2xl w-full max-w-2xl max-h-[90vh] shadow-2xl border border-gray-200 dark:border-[#40285e] flex flex-col overflow-hidden text-xs text-gray-900 dark:text-gray-100 animate-in fade-in zoom-in-95"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Modal Header */}
                <div className="flex items-center justify-between px-5 py-3.5 border-b border-gray-100 dark:border-[#40285e] bg-gray-50/70 dark:bg-[#091e30]">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2 rounded-xl bg-amber-50 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400">
                            <Sparkles className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                                <span>Multi-IDE Installation Engine</span>
                                {isLoadingStatus && (
                                    <RotateCw className="w-3.5 h-3.5 animate-spin text-amber-500" />
                                )}
                            </h2>
                            <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                Discover and automate package installation for supported AI-assisted code editors
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        disabled={activeInstallingTarget !== null}
                        className="p-1.5 rounded-lg hover:bg-gray-200 dark:hover:bg-[#40285e] text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors cursor-pointer disabled:opacity-50"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Modal Body */}
                <div className="flex-1 overflow-y-auto p-5 space-y-4">
                    {/* 3-Card Grid */}
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3.5">
                        {ideList.map((ide) => {
                            const isInstalling = activeInstallingTarget === ide.target.toLowerCase();
                            const targetSlug = ide.target.toLowerCase();

                            return (
                                <div
                                    key={ide.target}
                                    className="p-4 rounded-xl border border-gray-200/90 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#1b142d] flex flex-col justify-between space-y-3 shadow-2xs"
                                >
                                    <div className="space-y-2">
                                        <div className="flex items-center justify-between">
                                            <div className="w-8 h-8 rounded-lg bg-blue-100 dark:bg-blue-900/40 text-blue-600 dark:text-cyan-400 flex items-center justify-center font-bold">
                                                <Laptop className="w-4 h-4" />
                                            </div>
                                            {ide.is_installed ? (
                                                <span className="flex items-center gap-1 text-[10px] font-semibold text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/50 px-2 py-0.5 rounded-full border border-emerald-200 dark:border-emerald-800">
                                                    <CheckCircle2 className="w-3 h-3" />
                                                    {ide.version ? `v${ide.version}` : 'Installed'}
                                                </span>
                                            ) : (
                                                <span className="text-[10px] font-semibold text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-[#251b3d] px-2 py-0.5 rounded-full border border-slate-200 dark:border-[#40285e]">
                                                    Not Detected
                                                </span>
                                            )}
                                        </div>

                                        <div>
                                            <h3 className="font-bold text-xs text-gray-900 dark:text-slate-100">
                                                {ide.display_name}
                                            </h3>
                                            <p className="text-[11px] text-gray-500 dark:text-gray-400 line-clamp-2 mt-0.5">
                                                {ide.target === 'Antigravity'
                                                    ? 'Google official pair programming agentic environment'
                                                    : ide.target === 'Cursor'
                                                    ? 'AI-first code editor with Claude & GPT reasoning'
                                                    : 'Core extensible code editor by Microsoft'}
                                            </p>
                                        </div>

                                        {ide.executable_path && (
                                            <div className="text-[10px] text-gray-400 font-mono truncate" title={ide.executable_path}>
                                                {ide.executable_path}
                                            </div>
                                        )}
                                    </div>

                                    <button
                                        type="button"
                                        onClick={() => handleInstallTarget(targetSlug)}
                                        disabled={activeInstallingTarget !== null}
                                        className={cn(
                                            'w-full flex items-center justify-center gap-1.5 py-1.5 px-3 rounded-lg text-xs font-semibold transition-all shadow-xs cursor-pointer disabled:opacity-50',
                                            ide.is_installed
                                                ? 'bg-slate-200 dark:bg-[#2d2149] hover:bg-slate-300 dark:hover:bg-[#3d2d63] text-gray-800 dark:text-gray-200'
                                                : 'bg-blue-600 hover:bg-blue-500 text-white'
                                        )}
                                    >
                                        {isInstalling ? (
                                            <RotateCw className="w-3.5 h-3.5 animate-spin" />
                                        ) : (
                                            <Download className="w-3.5 h-3.5" />
                                        )}
                                        <span>{isInstalling ? 'Installing...' : ide.is_installed ? 'Reinstall' : 'Install'}</span>
                                    </button>
                                </div>
                            );
                        })}
                    </div>

                    {/* Installation Log Drawer */}
                    {lastInstallResult && (
                        <div className="border border-gray-200 dark:border-[#40285e] rounded-xl overflow-hidden">
                            <button
                                type="button"
                                onClick={() => setIsLogDrawerOpen(!isLogDrawerOpen)}
                                className="w-full flex items-center justify-between px-3.5 py-2 bg-slate-100 dark:bg-[#1a122c] text-xs font-semibold text-gray-700 dark:text-gray-300 cursor-pointer"
                            >
                                <span className="flex items-center gap-1.5">
                                    <Terminal className="w-3.5 h-3.5 text-blue-500" />
                                    <span>Last Command: {lastInstallResult.command_executed}</span>
                                </span>
                                {isLogDrawerOpen ? <ChevronUp className="w-3.5 h-3.5" /> : <ChevronDown className="w-3.5 h-3.5" />}
                            </button>
                            {isLogDrawerOpen && (
                                <pre className="p-3 bg-black/90 text-green-400 text-[10px] font-mono overflow-x-auto max-h-40 whitespace-pre-wrap">
                                    {lastInstallResult.output || 'No output recorded.'}
                                </pre>
                            )}
                        </div>
                    )}
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-between px-5 py-3.5 border-t border-gray-100 dark:border-[#40285e] bg-gray-50/70 dark:bg-[#091e30]">
                    <button
                        type="button"
                        onClick={refreshStatus}
                        disabled={isLoadingStatus}
                        className="flex items-center gap-1.5 text-xs text-blue-600 dark:text-cyan-400 hover:underline cursor-pointer"
                    >
                        <RotateCw className={cn('w-3.5 h-3.5', isLoadingStatus && 'animate-spin')} />
                        <span>Verify & Refresh Detection</span>
                    </button>
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-1.5 rounded-lg border border-gray-300 dark:border-[#40285e] hover:bg-gray-100 dark:hover:bg-[#1f1633] text-gray-700 dark:text-gray-300 font-medium transition-colors cursor-pointer"
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    );
}
