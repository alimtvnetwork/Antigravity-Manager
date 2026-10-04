import { useState, useEffect, useRef } from 'react';
import {
    Laptop,
    Play,
    Square,
    Copy,
    Trash2,
    RotateCcw,
    RotateCw,
    Plus,
    Folder,
    Search,
    AlertCircle,
    Cpu,
    Pencil,
    FastForward,
    Mail,
    Clock,
    Gem,
    Diamond,
    Circle,
    Sparkles,
    Star,
    ToggleLeft,
    ToggleRight,
    ArrowRightLeft,
    X,
    SlidersHorizontal,
    LayoutGrid,
    List,
    Layers,
    History,
} from 'lucide-react';
import { Gemini } from '@lobehub/icons';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../stores/useInstanceStore';
import { useAccountStore } from '../stores/useAccountStore';
import type { InstanceStatus } from '../services/instanceService';
import { InstanceSettingsModal } from '../components/instances/InstanceSettingsModal';
import InstanceTable from '../components/instances/InstanceTable';
import PromptTreeViewModal from '../components/instances/PromptTreeViewModal';
import InstanceAuditTrailModal from '../components/instances/InstanceAuditTrailModal';
import { findQuotaModel } from '../config/modelConfig';
import { formatTimeRemaining } from '../utils/format';
import { isTauri } from '../utils/env';
import { cn } from '../utils/cn';
import { showToast } from '../components/common/ToastContainer';
import { WaterDrainProgressBar } from '../components/common/WaterDrainProgressBar';

function truncatePath(fullPath?: string | null): string {
    if (!fullPath) return '';
    const isWindows = fullPath.includes('\\') || !fullPath.includes('/');
    const parts = fullPath.split(/[\\/]/).filter(Boolean);
    if (parts.length <= 2) return fullPath;
    const sep = isWindows ? '\\' : '/';
    return `...${sep}${parts.slice(-2).join(sep)}`;
}

const INSTANCE_THEMES = [
    {
        name: 'Indigo',
        accentBar: 'from-indigo-500 via-blue-500 to-indigo-600',
        badge: 'bg-indigo-500/10 text-indigo-700 dark:text-indigo-300 border-indigo-400/30',
        emailPill: 'bg-indigo-50 dark:bg-indigo-950/40 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800/60',
        dot: 'bg-indigo-500',
    },
    {
        name: 'Emerald',
        accentBar: 'from-emerald-500 via-teal-500 to-emerald-600',
        badge: 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border-emerald-400/30',
        emailPill: 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800/60',
        dot: 'bg-emerald-500',
    },
    {
        name: 'Purple',
        accentBar: 'from-purple-500 via-fuchsia-500 to-purple-600',
        badge: 'bg-purple-500/10 text-purple-700 dark:text-purple-300 border-purple-400/30',
        emailPill: 'bg-purple-50 dark:bg-purple-950/40 text-purple-700 dark:text-purple-300 border-purple-200 dark:border-purple-800/60',
        dot: 'bg-purple-500',
    },
    {
        name: 'Amber',
        accentBar: 'from-amber-500 via-yellow-500 to-orange-500',
        badge: 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-400/30',
        emailPill: 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800/60',
        dot: 'bg-amber-500',
    },
    {
        name: 'Cyan',
        accentBar: 'from-cyan-500 via-sky-500 to-blue-500',
        badge: 'bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 border-cyan-400/30',
        emailPill: 'bg-cyan-50 dark:bg-cyan-950/40 text-cyan-700 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800/60',
        dot: 'bg-cyan-500',
    },
    {
        name: 'Rose',
        accentBar: 'from-rose-500 via-pink-500 to-red-500',
        badge: 'bg-rose-500/10 text-rose-700 dark:text-rose-300 border-rose-400/30',
        emailPill: 'bg-rose-50 dark:bg-rose-950/40 text-rose-700 dark:text-rose-300 border-rose-200 dark:border-rose-800/60',
        dot: 'bg-rose-500',
    },
    {
        name: 'Violet',
        accentBar: 'from-violet-500 via-purple-500 to-indigo-500',
        badge: 'bg-violet-500/10 text-violet-700 dark:text-violet-300 border-violet-400/30',
        emailPill: 'bg-violet-50 dark:bg-violet-950/40 text-violet-700 dark:text-violet-300 border-violet-200 dark:border-violet-800/60',
        dot: 'bg-violet-500',
    },
];

export default function Instances() {
    const { t } = useTranslation();
    const {
        instances,
        activeInstanceId,
        switcherStatus,
        isLoading,
        error,
        fetchInstances,
        fetchSwitcherStatus,
        triggerManualRotation,
        createInstance,
        copyInstance,
        renameInstance,
        deleteInstance,
        wipeSession,
        launchInstance,
        cloneInstanceExecutable,
        stopInstance,
        fastForwardInstance,
        toggleAutoSwitcher,
        setActiveInstance,
        setDefaultInstance,
        switchAccountToInstance,
        cleanAndRestartWorkspace,
        syncInstance,
        syncAllInstances,
    } = useInstanceStore();

    const {
        accounts,
        currentAccount,
        fetchAccounts,
        refreshQuota,
    } = useAccountStore();

    const [searchQuery, setSearchQuery] = useState('');
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [newInstanceName, setNewInstanceName] = useState('');
    const [newInstanceBoundAccount, setNewInstanceBoundAccount] = useState<string>('');
    const [newInstanceFromInstance, setNewInstanceFromInstance] = useState<string>('');
    const [newInstanceLaunchImmediately, setNewInstanceLaunchImmediately] = useState<boolean>(false);
    const [switchTargetInstance, setSwitchTargetInstance] = useState<InstanceStatus | null>(null);
    const [accountSearchQuery, setAccountSearchQuery] = useState<string>('');
    const [copyTargetId, setCopyTargetId] = useState<string | null>(null);
    const [copyInstanceName, setCopyInstanceName] = useState('');
    const [cloneMode, setCloneMode] = useState<'full' | 'profile'>('full');
    const [copyProjects, setCopyProjects] = useState<boolean>(true);
    const [isSettingsModalOpen, setIsSettingsModalOpen] = useState<boolean>(false);
    const [settingsModalTarget, setSettingsModalTarget] = useState<InstanceStatus | null>(null);
    const [editTargetId, setEditTargetId] = useState<string | null>(null);
    const [editInstanceName, setEditInstanceName] = useState('');
    const [actionError, setActionError] = useState<string | null>(null);
    const [isSyncingAll, setIsSyncingAll] = useState(false);
    const [syncingInstanceIds, setSyncingInstanceIds] = useState<Record<string, boolean>>({});
    const [auditModalInstance, setAuditModalInstance] = useState<{ id: string; name: string; sequence_name?: string } | null>(null);
    const activeCardRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (activeInstanceId) {
            const timer = setTimeout(() => {
                activeCardRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
            }, 150);
            return () => clearTimeout(timer);
        }
    }, [activeInstanceId]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsCreateOpen(false);
                setCopyTargetId(null);
                setEditTargetId(null);
                setSwitchTargetInstance(null);
                setIsSettingsModalOpen(false);
                setAuditModalInstance(null);
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);

    }, []);

    useEffect(() => {
        if (!isTauri()) return;
        fetchInstances();
        fetchSwitcherStatus();
        fetchAccounts();
        const timer = setInterval(() => {
            fetchInstances(true);
            fetchSwitcherStatus();
        }, 3000);

        let unlistenList: (() => void)[] = [];
        import('@tauri-apps/api/event').then(({ listen }) => {
            const events = ['account://auto-switched', 'instance://switched', 'instance://rotated'];
            Promise.all(
                events.map((ev) =>
                    listen(ev, async () => {
                        await Promise.all([
                            fetchInstances(true),
                            fetchSwitcherStatus(),
                            fetchAccounts(),
                        ]);
                    })
                )
            ).then((unsubscribers) => {
                unlistenList = unsubscribers;
            });
        });

        return () => {
            clearInterval(timer);
            unlistenList.forEach((fn) => fn());
        };
    }, [fetchInstances, fetchSwitcherStatus, fetchAccounts]);

    const [viewMode, setViewMode] = useState<'card' | 'list'>(() => {
        return (localStorage.getItem('agm_instance_view_mode') as 'card' | 'list') || 'card';
    });
    const [promptTreeInstance, setPromptTreeInstance] = useState<{ id: string; name: string } | null>(null);

    const handleSetViewMode = (mode: 'card' | 'list') => {
        setViewMode(mode);
        localStorage.setItem('agm_instance_view_mode', mode);
    };

    const filteredInstances = instances.filter((inst) => {
        if (!searchQuery.trim()) return true;
        const query = searchQuery.toLowerCase();
        const nameMatch = inst.config.name.toLowerCase().includes(query);
        const idMatch = inst.config.id.toLowerCase().includes(query);
        const boundAcc = accounts.find((a) => a.id === inst.config.bound_account_id || a.email.toLowerCase() === (inst.config.bound_email || '').toLowerCase());
        const boundEmail = inst.config.bound_email || boundAcc?.email || '';
        const emailMatch = boundEmail.toLowerCase().includes(query);
        const pidMatch = inst.pid ? inst.pid.toString().includes(query) : false;
        const statusMatch = (inst.is_running ? 'running active' : 'idle').includes(query);
        const defaultMatch = (inst.config.is_default || inst.config.id === 'default') && 'default'.includes(query);
        return nameMatch || idMatch || emailMatch || pidMatch || statusMatch || defaultMatch;
    });

    const runningCount = instances.filter((i) => i.is_running).length;

    const handleCreate = async () => {
        if (!newInstanceName.trim()) return;
        setActionError(null);
        try {
            const created = await createInstance(
                newInstanceName.trim(),
                newInstanceBoundAccount ? newInstanceBoundAccount : undefined,
                newInstanceFromInstance ? newInstanceFromInstance : undefined
            );
            await setActiveInstance(created.id);
            if (newInstanceLaunchImmediately) {
                await launchInstance(created.id);
            }
            setNewInstanceName('');
            setNewInstanceBoundAccount('');
            setNewInstanceFromInstance('');
            setNewInstanceLaunchImmediately(false);
            setIsCreateOpen(false);
            showToast(
                `Created profile '${created.name}'${newInstanceLaunchImmediately ? ' & launched' : ''}`,
                'success'
            );
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to create instance');
        }
    };

    const handleCopy = async () => {
        if (!copyTargetId || !copyInstanceName.trim()) return;
        setActionError(null);
        try {
            const copied = await copyInstance(copyTargetId, copyInstanceName.trim(), cloneMode, copyProjects);
            await setActiveInstance(copied.id);
            await fetchInstances(true);
            setCopyInstanceName('');
            setCopyTargetId(null);
            showToast(t('instances.copied_toast', 'Instance profile duplicated'), 'success');
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to copy instance');
        }
    };

    const handleEdit = async () => {
        if (!editTargetId || !editInstanceName.trim()) return;
        setActionError(null);
        try {
            await renameInstance(editTargetId, editInstanceName.trim());
            setEditInstanceName('');
            setEditTargetId(null);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to rename instance');
        }
    };

    const handleDelete = async (id: string) => {
        setActionError(null);
        if (window.confirm(t('instances.confirm_delete', 'Are you sure you want to delete this profile?'))) {
            try {
                await deleteInstance(id);
            } catch (e: any) {
                setActionError(e?.toString() || 'Failed to delete instance');
            }
        }
    };

    const handleWipeSession = async (id: string) => {
        setActionError(null);
        if (window.confirm(t('instances.confirm_wipe', 'Wipe session tokens for this profile? User preferences will be kept.'))) {
            try {
                await wipeSession(id);
            } catch (e: any) {
                setActionError(e?.toString() || 'Failed to wipe session');
            }
        }
    };

    const handleLaunch = async (id: string) => {
        setActionError(null);
        try {
            await launchInstance(id);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to launch instance');
        }
    };

    const handleCloneExecutable = async (id: string) => {
        setActionError(null);
        try {
            const cloned = await cloneInstanceExecutable(id);
            showToast(`Executable cloned successfully: ${cloned}`, 'success');
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to clone executable');
        }
    };

    return (
        <div className="h-full w-full overflow-y-auto">
            <div className="max-w-[1920px] mx-auto px-4 sm:px-6 lg:px-8 pt-2 pb-4 space-y-3">
                {/* Header */}
                <div className="space-y-3">
                    <div className="flex items-center gap-2.5 min-w-0">
                        <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 shrink-0">
                            <Laptop className="w-5 h-5" />
                        </div>
                        <div className="min-w-0">
                            <h1 className="text-lg sm:text-xl font-bold text-gray-900 dark:text-base-content">
                                {t('instances.page_title', 'Instances & Profiles')}
                            </h1>
                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                {t('instances.page_desc', 'Run multiple Antigravity windows in parallel with isolated credentials and extensions')}
                            </p>
                        </div>
                    </div>

                <div className="flex flex-wrap items-center justify-between gap-3">
                    {/* Segmented Group 1: Status & Maintenance */}
                    <div className="flex items-center rounded-[5px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                        <div className="px-3 py-1 text-xs text-slate-700 dark:text-slate-300 font-semibold whitespace-nowrap">
                            {t('instances.running_summary', 'Running {{running}} of {{total}}', {
                                running: runningCount,
                                total: instances.length,
                            })}
                        </div>
                        <button
                            type="button"
                            onClick={() => {
                                fetchInstances();
                                fetchSwitcherStatus();
                            }}
                            disabled={isLoading}
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                            title={t('common.refresh', 'Refresh')}
                        >
                            <RotateCw className={cn("w-3.5 h-3.5 text-blue-500", isLoading && "animate-spin")} />
                            <span>{t('common.refresh', 'Refresh')}</span>
                        </button>
                        <button
                            type="button"
                            onClick={async () => {
                                setIsSyncingAll(true);
                                try {
                                    await syncAllInstances();
                                } catch (e: any) {
                                    setActionError(e?.toString() || 'Failed to sync all instances and quotas');
                                } finally {
                                    setIsSyncingAll(false);
                                }
                            }}
                            disabled={isLoading || isSyncingAll}
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                            title="Synchronize process PIDs and account quotas across all instances"
                        >
                            <RotateCw className={cn("w-3.5 h-3.5 text-cyan-500", isSyncingAll && "animate-spin")} />
                            <span>Sync All</span>
                        </button>
                        <button
                            type="button"
                            onClick={async () => {
                                try {
                                    const msg = await cleanAndRestartWorkspace();
                                    showToast(msg || 'Stuck Electron processes cleared & Antigravity restarted!', 'success');
                                } catch (e: any) {
                                    setActionError(e?.toString() || 'Failed to clean and restart workspace');
                                }
                            }}
                            disabled={isLoading}
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-amber-700 dark:text-amber-400 hover:bg-amber-100/50 dark:hover:bg-amber-950/40 rounded-r-[5px] transition-colors cursor-pointer"
                            title="Force-terminate lingering background Electron/Antigravity processes, purge lockfiles, and cleanly relaunch Antigravity"
                        >
                            <Sparkles className="w-3.5 h-3.5" />
                            <span>Clean & Restart</span>
                        </button>
                    </div>


                    {/* Segmented Group 2: Automation & Settings */}
                    <div className="flex items-center rounded-[5px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                        <button
                            type="button"
                            onClick={async () => {
                                try {
                                    await toggleAutoSwitcher();
                                    showToast(
                                        switcherStatus?.is_running
                                            ? 'Auto-Switcher disabled'
                                            : 'Auto-Switcher enabled',
                                        'success'
                                    );
                                } catch (e: any) {
                                    setActionError(e?.toString() || 'Failed to toggle Auto Switcher');
                                }
                            }}
                            disabled={isLoading}
                            className={cn(
                                "flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-l-[5px] transition-colors cursor-pointer",
                                switcherStatus?.is_running
                                    ? "text-emerald-700 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/40"
                                    : "text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-[#15334d]"
                            )}
                            title="Toggle background auto-profile switcher daemon"
                        >
                            {switcherStatus?.is_running ? (
                                <ToggleRight className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                            ) : (
                                <ToggleLeft className="w-4 h-4 text-slate-400" />
                            )}
                            <span>Auto-Switch: {switcherStatus?.is_running ? 'ON' : 'OFF'}</span>
                        </button>
                        <button
                            type="button"
                            onClick={async () => {
                                try {
                                    const msg = await triggerManualRotation();
                                    showToast(msg || 'Evaluated quota across instances!', 'success');
                                } catch (e: any) {
                                    setActionError(e?.toString() || 'Failed to trigger quota evaluation');
                                }
                            }}
                            disabled={isLoading}
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                            title="Evaluate rolling quota across all monitored instances and auto-rotate any low quota accounts"
                        >
                            <RotateCw className={cn("w-3.5 h-3.5", isLoading && "animate-spin")} />
                            <span>Eval Quota</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => {
                                setSettingsModalTarget(null);
                                setIsSettingsModalOpen(true);
                            }}
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-r-[5px] transition-colors cursor-pointer"
                            title="Instance Settings & Sync: Turbo mode, plan review, copy settings, folder sync, JSON tools"
                        >
                            <SlidersHorizontal className="w-3.5 h-3.5 text-blue-500" />
                            <span>Settings & Sync</span>
                        </button>
                    </div>

                    {/* Segmented Group 3: View Mode & Creation */}
                    <div className="flex items-center gap-2">
                        {/* View Switcher Capsule */}
                        <div className="flex items-center rounded-[5px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                            <button
                                type="button"
                                onClick={() => handleSetViewMode('card')}
                                className={cn(
                                    "flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-l-[5px] transition-colors cursor-pointer",
                                    viewMode === 'card'
                                        ? "bg-white dark:bg-[#15334d] text-blue-600 dark:text-cyan-300 shadow-xs"
                                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                                )}
                                title="Card View"
                            >
                                <LayoutGrid className="w-3.5 h-3.5" />
                                <span className="hidden sm:inline">Cards</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => handleSetViewMode('list')}
                                className={cn(
                                    "flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-r-[5px] transition-colors cursor-pointer",
                                    viewMode === 'list'
                                        ? "bg-white dark:bg-[#15334d] text-blue-600 dark:text-cyan-300 shadow-xs"
                                        : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                                )}
                                title="List View"
                            >
                                <List className="w-3.5 h-3.5" />
                                <span className="hidden sm:inline">List</span>
                            </button>
                        </div>

                        {/* New Instance Button */}
                        <button
                            type="button"
                            onClick={() => {
                                setNewInstanceName('');
                                setIsCreateOpen(true);
                            }}
                            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-[5px] bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold shadow-xs cursor-pointer transition-all"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            <span>{t('instances.create_btn', 'New Instance')}</span>
                        </button>
                    </div>
                </div>
            </div>

            {/* Store Error Alert */}
            {error ? (
                <div className="p-3.5 rounded-xl bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{error}</span>
                    </div>
                    <button
                        onClick={() => fetchInstances()}
                        className="btn btn-xs btn-outline btn-error gap-1 shrink-0"
                    >
                        <RotateCw className="w-3 h-3" />
                        <span>{t('common.retry', 'Retry')}</span>
                    </button>
                </div>
            ) : null}

            {/* Action Error Alert */}
            {actionError ? (
                <div className="p-3.5 rounded-xl bg-rose-50 dark:bg-rose-900/20 border border-rose-200 dark:border-rose-800 text-rose-700 dark:text-rose-300 text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{actionError}</span>
                    </div>
                    <button onClick={() => setActionError(null)} className="text-xs font-semibold">✕</button>
                </div>
            ) : null}

            {/* Auto-Switcher Status Banner */}
            {switcherStatus?.is_running ? (
                <div className="bg-gradient-to-r from-blue-50/90 via-sky-50/70 to-indigo-50/80 dark:from-blue-950/40 dark:via-[#0c2438]/60 dark:to-indigo-950/40 border border-blue-200/80 dark:border-blue-500/30 rounded-2xl p-4 backdrop-blur-md shadow-sm flex flex-col md:flex-row items-start md:items-center justify-between gap-3 text-xs">
                    <div className="flex items-center gap-3 min-w-0">
                        <span className="relative flex h-3 w-3 shrink-0">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500 shadow-xs shadow-emerald-500/50"></span>
                        </span>
                        <div className="flex flex-wrap items-center gap-2 min-w-0">
                            <span className="font-bold text-gray-900 dark:text-gray-100 flex items-center gap-1.5">
                                <Sparkles className="w-3.5 h-3.5 text-blue-500 dark:text-cyan-400" />
                                {t('instances.auto_switcher_active', 'Auto Profile Switcher Active')}
                            </span>
                            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[11px] font-semibold font-mono bg-blue-500/15 text-blue-700 dark:text-cyan-300 border border-blue-400/30 shadow-2xs">
                                Current Active Quota: {switcherStatus.current_quota_percent !== undefined && switcherStatus.current_quota_percent !== null
                                    ? `${switcherStatus.current_quota_percent.toFixed(0)}%`
                                    : '100%'}
                            </span>
                            {switcherStatus.last_switch_reason ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[11px] font-medium bg-gray-500/10 text-gray-600 dark:text-gray-300 border border-gray-400/20 max-w-xs truncate" title={switcherStatus.last_switch_reason}>
                                    Reason: {switcherStatus.last_switch_reason}
                                </span>
                            ) : null}
                        </div>
                    </div>
                    {(() => {
                        const activeInstance = instances.find(i => i.config.id === (activeInstanceId || 'default')) || instances.find(i => i.config.is_default) || instances[0];
                        const rotateTooltip = `Rotates active instance (${activeInstance?.config.name || 'current'}) to the highest health candidate`;
                        return (
                            <button
                                onClick={async () => {
                                    try {
                                        const msg = await triggerManualRotation();
                                        showToast(msg || 'Rotated to next best profile!', 'success');
                                    } catch (e: any) {
                                        setActionError(e?.toString() || 'Rotation failed');
                                    }
                                }}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-[5px] bg-blue-600 hover:bg-blue-500 text-white shadow-xs cursor-pointer transition-colors active:scale-95 shrink-0"
                                title={rotateTooltip}
                            >
                                <RotateCcw className="w-3.5 h-3.5" />
                                <span>{t('instances.rotate_next_best', 'Rotate to Next Best')}</span>
                            </button>
                        );
                    })()}
                </div>
            ) : null}

            {/* Search Filter & Quick Action */}
            <div className="flex items-center justify-between gap-3 flex-wrap">
                <div className="flex items-center gap-2 flex-1 max-w-md bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 rounded-xl px-3 py-2 shadow-xs">
                    <Search className="w-4 h-4 text-gray-400 shrink-0" />
                    <input
                        type="text"
                        placeholder={t('instances.search_placeholder', 'Search by profile name, ID, or bound email...')}
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full bg-transparent border-none outline-hidden text-xs text-gray-900 dark:text-base-content"
                    />
                </div>
                <button
                    onClick={() => {
                        setNewInstanceName('');
                        setIsCreateOpen(true);
                    }}
                    className="btn btn-primary btn-sm gap-1.5 shadow-sm rounded-[5px]"
                >
                    <Plus className="w-4 h-4" />
                    <span>{t('instances.create_btn', 'New Instance')}</span>
                </button>
            </div>

            {/* Instance Cards Grid or Empty / Loading States */}
            {isLoading && instances.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-20 bg-white dark:bg-base-200 rounded-2xl border border-gray-200/80 dark:border-base-100">
                    <RotateCw className="w-8 h-8 text-blue-600 dark:text-blue-400 animate-spin mb-3" />
                    <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {t('instances.loading', 'Loading instances and profiles...')}
                    </p>
                </div>
            ) : instances.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-16 px-4 text-center bg-white dark:bg-base-200 rounded-2xl border border-dashed border-gray-300 dark:border-base-100">
                    <div className="p-4 rounded-2xl bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 mb-4">
                        <Laptop className="w-10 h-10" />
                    </div>
                    <h3 className="text-base font-bold text-gray-900 dark:text-base-content mb-1">
                        {t('instances.empty_title', 'No Profiles Found')}
                    </h3>
                    <p className="text-xs text-gray-500 dark:text-gray-400 max-w-md mb-6 leading-relaxed">
                        {t('instances.empty_desc', 'Instances allow you to run isolated Antigravity windows with dedicated account tokens, extensions, and workspaces. Initialize your default profile or create a custom one to get started.')}
                    </p>
                    <div className="flex items-center gap-3">
                        <button
                            onClick={async () => {
                                setActionError(null);
                                try {
                                    await createInstance('Default');
                                } catch (e: any) {
                                    setActionError(e?.toString() || 'Failed to initialize default profile');
                                }
                            }}
                            className="btn btn-primary btn-sm gap-1.5 shadow-sm rounded-[5px]"
                        >
                            <Play className="w-3.5 h-3.5" />
                            <span>{t('instances.init_default', 'Initialize Default Profile')}</span>
                        </button>
                        <button
                            onClick={() => {
                                setNewInstanceName('');
                                setIsCreateOpen(true);
                            }}
                            className="btn btn-outline btn-sm gap-1.5 rounded-[5px]"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            <span>{t('instances.create_custom', 'Create Custom Profile')}</span>
                        </button>
                    </div>
                </div>
            ) : filteredInstances.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-12 text-center bg-white dark:bg-base-200 rounded-2xl border border-gray-200/80 dark:border-base-100">
                    <Search className="w-8 h-8 text-gray-400 mb-2" />
                    <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                        {t('instances.no_search_results', 'No profiles match your search')}
                    </p>
                    <p className="text-xs text-gray-400 mt-1">
                        {t('instances.no_search_results_desc', 'Try searching with a different name, profile ID, or email')}
                    </p>
                </div>
            ) : viewMode === 'list' ? (
                <InstanceTable
                    instances={filteredInstances}
                    activeInstanceId={activeInstanceId}
                    searchQuery={searchQuery}
                    onLaunch={handleLaunch}
                    onStop={stopInstance}
                    onSwitch={(id) => {
                        const target = instances.find((i) => i.config.id === id);
                        if (target) setSwitchTargetInstance(target);
                    }}
                    onFastForward={async (id) => {
                        try {
                            const msg = await fastForwardInstance(id);
                            showToast(msg || 'Rotated to next best profile!', 'success');
                        } catch (e: any) {
                            setActionError(e?.toString() || 'Fast forward failed');
                        }
                    }}
                    onAudit={(id, name) => {
                        const target = instances.find((i) => i.config.id === id);
                        setAuditModalInstance({
                            id,
                            name,
                            sequence_name: target?.config.seq_num ? `Instance #${target.config.seq_num}` : undefined
                        });
                    }}
                    onSync={async (id) => {
                        setSyncingInstanceIds(prev => ({ ...prev, [id]: true }));
                        try {
                            await syncInstance(id);
                        } catch (e: any) {
                            setActionError(e?.toString() || 'Failed to sync instance');
                        } finally {
                            setSyncingInstanceIds(prev => ({ ...prev, [id]: false }));
                        }
                    }}
                    syncingInstanceIds={syncingInstanceIds}
                    onSettings={(id) => {
                        const target = instances.find((i) => i.config.id === id);
                        setSettingsModalTarget(target || null);
                        setIsSettingsModalOpen(true);
                    }}
                    onClone={(id, name) => {
                        setCopyTargetId(id);
                        setCopyInstanceName(`${name} Copy`);
                        setCopyProjects(true);
                    }}
                    onDelete={handleDelete}
                    onOpenPromptTree={(id) => {
                        const target = instances.find((i) => i.config.id === id);
                        setPromptTreeInstance({ id, name: target?.config.name || id });
                    }}
                    onSetActive={setActiveInstance}
                    onSetDefault={setDefaultInstance}
                />
            ) : (

                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
                    {filteredInstances.map((inst, index) => {
                        const originalIndex = instances.findIndex((i) => i.config.id === inst.config.id);
                        const seqNumber = originalIndex !== -1 ? originalIndex + 1 : index + 1;
                        const isActive = inst.config.id === activeInstanceId;
                        const theme = INSTANCE_THEMES[index % INSTANCE_THEMES.length];

                        // Resolve bound account
                        const boundAccount = accounts.find((a) => {
                            if (inst.config.bound_account_id) {
                                return a.id === inst.config.bound_account_id;
                            }
                            if (inst.config.bound_email) {
                                return a.email.toLowerCase() === inst.config.bound_email.toLowerCase();
                            }
                            if (isActive) {
                                if (currentAccount) {
                                    return a.email.toLowerCase() === currentAccount.email.toLowerCase();
                                }
                            }
                            return false;
                        });

                        const displayEmail = inst.config.bound_email || boundAccount?.email || (isActive && currentAccount ? currentAccount.email : null);

                        const geminiPro = findQuotaModel(boundAccount?.quota?.models, 'gemini-pro');
                        const geminiFlash = findQuotaModel(boundAccount?.quota?.models, 'gemini-flash');
                        const geminiModel = geminiPro || geminiFlash;

                        return (
                            <div
                                key={inst.config.id}
                                ref={isActive ? activeCardRef : undefined}
                                className={cn(
                                    "rounded-xl border transition-all flex flex-col justify-between bg-white dark:bg-[#0a1e30] overflow-hidden shadow-xs backdrop-blur-xs",
                                    isActive
                                        ? "border-blue-500 shadow-lg ring-2 ring-blue-500/30 bg-blue-50/15 dark:bg-[#0c2438]"
                                        : "border-gray-200/80 dark:border-[#15334d] hover:border-gray-300 dark:hover:border-blue-500/40"
                                )}
                            >
                                {/* Top Accent Bar identifying profile color */}
                                <div className={cn("h-1.5 w-full bg-gradient-to-r", theme.accentBar)} />

                                <div className="p-3.5 flex flex-col flex-1 justify-between min-w-0">
                                    {/* Card Top */}
                                    <div className="min-w-0">
                                        <div className="flex items-start justify-between gap-2 mb-2.5">
                                            <div className="flex items-center gap-1.5 min-w-0 flex-wrap">
                                                <span
                                                    className={cn(
                                                        "w-2.5 h-2.5 rounded-full shrink-0",
                                                        inst.is_running ? "bg-emerald-500 shadow-xs shadow-emerald-500/50 animate-pulse" : "bg-gray-300 dark:bg-gray-600"
                                                    )}
                                                />
                                                <span className="px-1.5 py-0.5 rounded-[5px] text-xs font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 shrink-0">
                                                    #{seqNumber}
                                                </span>
                                                <h3 className={cn("font-bold text-xs truncate max-w-[120px]", isActive ? "text-blue-900 dark:text-blue-100" : "text-gray-900 dark:text-base-content")} title={inst.config.name}>
                                                    {inst.config.name}
                                                </h3>
                                                {inst.config.is_default ? (
                                                    <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-400/30 shrink-0">
                                                        DEFAULT
                                                    </span>
                                                ) : (
                                                    <button
                                                        onClick={async () => {
                                                            try {
                                                                await setDefaultInstance(inst.config.id);
                                                                showToast(t('instances.set_default_toast', 'Default profile updated successfully'), 'success');
                                                            } catch (e: any) {
                                                                setActionError(e?.toString() || 'Failed to set default profile');
                                                            }
                                                        }}
                                                        className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-medium text-gray-400 hover:text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-950/30 border border-dashed border-gray-300 dark:border-base-100 transition-colors cursor-pointer flex items-center gap-1 shrink-0"
                                                        title="Set as default profile"
                                                    >
                                                        <Star className="w-2.5 h-2.5" />
                                                        <span>Set Default</span>
                                                    </button>
                                                )}
                                            </div>
                                            <div className="shrink-0 flex items-center gap-1">
                                                {isActive ? (
                                                    <span className="px-1.5 py-0.5 rounded-[5px] text-[9px] font-bold bg-blue-600 text-white shadow-xs tracking-wider">
                                                        Active
                                                    </span>
                                                ) : (
                                                    <button
                                                        onClick={() => setActiveInstance(inst.config.id)}
                                                        className="text-[10px] text-gray-500 hover:text-blue-600 transition-colors font-medium mr-0.5 cursor-pointer"
                                                        title="Set as active instance for account switches"
                                                    >
                                                        Set Active
                                                    </button>
                                                )}
                                                <button
                                                    onClick={() => {
                                                        setEditTargetId(inst.config.id);
                                                        setEditInstanceName(inst.config.name);
                                                    }}
                                                    className="p-1 rounded-[5px] text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30 cursor-pointer"
                                                    title={t('instances.edit_title', 'Rename profile')}
                                                >
                                                    <Pencil className="w-3 h-3" />
                                                </button>
                                                <button
                                                    onClick={() => {
                                                        setCopyTargetId(inst.config.id);
                                                        setCopyInstanceName(`${inst.config.name} Copy`);
                                                        setCopyProjects(true);
                                                    }}
                                                    className="p-1 rounded-[5px] text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 cursor-pointer"
                                                    title="Clone / Duplicate profile settings and extensions"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                </button>
                                                {!inst.config.is_default && (
                                                    <button
                                                        onClick={() => handleDelete(inst.config.id)}
                                                        disabled={inst.is_running}
                                                        className="p-1 rounded-[5px] text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-900/30 disabled:opacity-30 cursor-pointer"
                                                        title="Delete profile"
                                                    >
                                                        <Trash2 className="w-3 h-3" />
                                                    </button>
                                                )}
                                            </div>
                                        </div>

                                        {/* Bound Account / Email Section: Compact py-1 px-2.5 rounded-md */}
                                        <div className="py-1 px-2.5 rounded-md bg-gray-50/80 dark:bg-[#0c2438]/90 border border-gray-100 dark:border-[#15334d] mb-2.5 flex items-center justify-between gap-1.5">
                                            <div className="flex items-center gap-1.5 min-w-0 flex-1">
                                                <Mail className="w-3 h-3 text-gray-400 shrink-0" />
                                                <span className="text-[10px] text-gray-500 dark:text-gray-400 font-medium shrink-0">
                                                    Account:
                                                </span>
                                                {displayEmail ? (
                                                    <span
                                                        className={cn(
                                                            "px-1.5 py-0.5 rounded-md text-[11px] font-semibold font-mono border flex items-center gap-1 min-w-0 shadow-2xs",
                                                            theme.emailPill
                                                        )}
                                                        title={displayEmail}
                                                    >
                                                        <span className={cn("w-1.5 h-1.5 rounded-full shrink-0", theme.dot)} />
                                                        <span className="truncate">{displayEmail}</span>
                                                    </span>
                                                ) : (
                                                    <span className="text-[11px] text-gray-400 italic">
                                                        Unassigned
                                                    </span>
                                                )}
                                            </div>
                                            {boundAccount?.quota?.subscription_tier ? (() => {
                                                const tier = boundAccount.quota.subscription_tier.toLowerCase();
                                                if (tier.includes('ultra')) {
                                                    return (
                                                        <span className="flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-gradient-to-r from-purple-600 to-pink-600 text-white text-[9px] font-bold shadow-xs shrink-0">
                                                            <Gem className="w-2.5 h-2.5 fill-current" />
                                                            ULTRA
                                                        </span>
                                                    );
                                                }
                                                if (tier.includes('pro')) {
                                                    return (
                                                        <span className="flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-[9px] font-bold shadow-xs shrink-0">
                                                            <Diamond className="w-2.5 h-2.5 fill-current" />
                                                            PRO
                                                        </span>
                                                    );
                                                }
                                                return (
                                                    <span className="flex items-center gap-0.5 px-1.5 py-0.5 rounded bg-gray-100 dark:bg-[#15334d] text-gray-600 dark:text-slate-300 text-[9px] font-bold shadow-xs border border-gray-200 dark:border-[#15334d] shrink-0">
                                                        <Circle className="w-2.5 h-2.5" />
                                                        FREE
                                                    </span>
                                                );
                                            })() : null}
                                        </div>

                                        {/* Colorful Gemini Quota Progress Bar: VS Code cyan-teal palette */}
                                        <div className="mb-2.5">
                                            {geminiModel ? (() => {
                                                const pct = Math.min(100, Math.max(0, geminiModel.percentage));
                                                const getTextClass = (percentage: number) => {
                                                    if (percentage >= 50) return 'text-teal-600 dark:text-cyan-400';
                                                    if (percentage >= 20) return 'text-amber-600 dark:text-amber-400';
                                                    return 'text-rose-600 dark:text-rose-400';
                                                };

                                                return (
                                                    <div className="p-2.5 rounded-md bg-gray-50/90 dark:bg-[#0c2438]/90 border border-gray-200/70 dark:border-[#15334d] space-y-1.5">
                                                        <div className="flex items-center justify-between text-xs">
                                                            <div className="flex items-center gap-1.5 min-w-0">
                                                                <Gemini.Color className="w-3.5 h-3.5 shrink-0" />
                                                                <span className="font-semibold text-gray-800 dark:text-gray-200 text-xs truncate">
                                                                    {geminiModel.display_name || 'Gemini 3.1 Pro'}
                                                                </span>
                                                            </div>
                                                            <div className="flex items-center gap-1.5 shrink-0">
                                                                {geminiModel.reset_time ? (
                                                                    <span className="text-[10px] text-gray-500 dark:text-gray-400 flex items-center gap-0.5 font-mono" title={`Resets in ${formatTimeRemaining(geminiModel.reset_time)}`}>
                                                                        <Clock className="w-2.5 h-2.5 text-gray-400" />
                                                                        {formatTimeRemaining(geminiModel.reset_time)}
                                                                    </span>
                                                                ) : null}
                                                                <span className={cn("font-mono font-bold text-[11px] px-1.5 py-0.5 rounded-md bg-white dark:bg-[#081a2b] shadow-2xs border border-gray-200 dark:border-[#15334d]", getTextClass(pct))}>
                                                                    {pct}%
                                                                </span>
                                                            </div>
                                                        </div>

                                                        {/* Stepper Progress Bar */}
                                                        <WaterDrainProgressBar percentage={pct} />
                                                    </div>
                                                );
                                            })() : boundAccount ? (
                                                <div className="p-2 rounded-md bg-gray-50/60 dark:bg-[#0c2438]/90 border border-gray-100 dark:border-[#15334d] flex items-center justify-between text-xs text-gray-500">
                                                    <div className="flex items-center gap-1.5">
                                                        <Gemini.Color className="w-3.5 h-3.5 shrink-0 opacity-70" />
                                                        <span className="text-[11px] text-gray-500 dark:text-gray-400">Quota not synced</span>
                                                    </div>
                                                    <button
                                                        onClick={() => refreshQuota(boundAccount.id)}
                                                        className="text-blue-600 dark:text-blue-400 hover:underline gap-1 text-[11px] cursor-pointer flex items-center"
                                                    >
                                                        <RotateCw className="w-3 h-3" />
                                                        <span>Sync</span>
                                                    </button>
                                                </div>
                                            ) : (
                                                <div className="p-2 rounded-md bg-gray-50/40 dark:bg-[#0c2438]/60 border border-dashed border-gray-200 dark:border-[#15334d] flex items-center justify-between text-xs text-gray-400">
                                                    <div className="flex items-center gap-1.5">
                                                        <Gemini.Color className="w-3.5 h-3.5 shrink-0 opacity-40 grayscale" />
                                                        <span className="text-[10px] italic">No profile bound</span>
                                                    </div>
                                                    <span className="text-[10px] text-gray-400/80">Launch to assign</span>
                                                </div>
                                            )}
                                        </div>

                                        {/* Status and Profile details: Compact badge layout */}
                                        <div className="space-y-1.5 py-2 border-t border-gray-100 dark:border-[#15334d]/80 text-xs">
                                            <div className="flex items-center justify-between gap-1.5 flex-wrap">
                                                {/* Status Badge */}
                                                {inst.is_running ? (
                                                    <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[10px] font-semibold bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/20 shadow-2xs">
                                                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                                        Running {inst.pid ? `(${inst.pid})` : ''}
                                                    </span>
                                                ) : (
                                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-medium bg-slate-500/10 text-slate-600 dark:text-slate-400 border border-slate-400/20">
                                                        <span className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                                                        Idle
                                                    </span>
                                                )}

                                                {/* Profile ID Monospace Pill */}
                                                <span
                                                    className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono font-medium bg-gray-100 dark:bg-[#0c2438] text-gray-600 dark:text-slate-300 border border-gray-200 dark:border-[#15334d] truncate max-w-[120px]"
                                                    title={`Profile ID: ${inst.config.id}`}
                                                >
                                                    <span className="text-gray-400 dark:text-slate-500">ID:</span>
                                                    <span className="truncate">{inst.config.id}</span>
                                                </span>
                                            </div>

                                            {/* Path with clean folder badge and copy action */}
                                            <div className="flex items-center justify-between gap-1.5 px-2 py-1 rounded-[5px] bg-gray-50/80 dark:bg-[#0c2438]/70 border border-gray-200/70 dark:border-[#15334d] text-[10px] group transition-colors hover:border-gray-300 dark:hover:border-blue-500/30">
                                                <div className="flex items-center gap-1 min-w-0 flex-1 text-gray-500 dark:text-slate-400" title={inst.config.data_dir}>
                                                    <Folder className="w-3 h-3 shrink-0 text-blue-500/80" />
                                                    <span className="truncate font-mono" title={inst.config.data_dir}>{truncatePath(inst.config.data_dir)}</span>
                                                </div>
                                                <button
                                                    type="button"
                                                    onClick={async (e) => {
                                                        e.stopPropagation();
                                                        try {
                                                            await navigator.clipboard.writeText(inst.config.data_dir);
                                                            showToast('Data dir path copied to clipboard', 'info');
                                                        } catch {
                                                            showToast('Failed to copy path', 'error');
                                                        }
                                                    }}
                                                    className="opacity-60 group-hover:opacity-100 p-0.5 rounded-[5px] hover:bg-gray-200 dark:hover:bg-[#15334d] text-gray-500 dark:text-gray-300 transition-opacity cursor-pointer shrink-0"
                                                    title="Copy directory path"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                </button>
                                            </div>

                                            {/* Custom executable path badge */}
                                            {inst.config.executable_path ? (
                                                <div className="flex items-center justify-between gap-1.5 px-2 py-1 rounded-[5px] bg-purple-50/50 dark:bg-purple-950/20 border border-purple-200/40 dark:border-purple-800/30 text-[10px] group transition-colors">
                                                    <div className="flex items-center gap-1 min-w-0 flex-1 text-purple-700 dark:text-purple-300" title={inst.config.executable_path}>
                                                        <Cpu className="w-3 h-3 shrink-0 text-purple-500" />
                                                        <span className="truncate font-mono" title={inst.config.executable_path}>{truncatePath(inst.config.executable_path)}</span>
                                                    </div>
                                                    <button
                                                        type="button"
                                                        onClick={async (e) => {
                                                            e.stopPropagation();
                                                            if (inst.config.executable_path) {
                                                                try {
                                                                    await navigator.clipboard.writeText(inst.config.executable_path);
                                                                    showToast('Executable path copied to clipboard', 'info');
                                                                } catch {
                                                                    showToast('Failed to copy executable path', 'error');
                                                                }
                                                            }
                                                        }}
                                                        className="opacity-60 group-hover:opacity-100 p-0.5 rounded-[5px] hover:bg-purple-200/60 dark:hover:bg-purple-900/40 text-purple-700 dark:text-purple-300 transition-opacity cursor-pointer shrink-0"
                                                        title="Copy executable path"
                                                    >
                                                        <Copy className="w-3 h-3" />
                                                    </button>
                                                </div>
                                            ) : null}
                                        </div>
                                    </div>

                                    {/* Card Actions Toolbar: 2 clean structured rows with 5-6px radius */}
                                    <div className="pt-2.5 border-t border-gray-100 dark:border-[#15334d]/80 mt-2 space-y-1.5">
                                        {/* Row 1: Stop/Launch, Switch, Fast-Forward, Audit, Sync */}
                                        <div className="flex items-center gap-1 w-full">
                                            {inst.is_running ? (
                                                <button
                                                    type="button"
                                                    onClick={() => stopInstance(inst.config.id)}
                                                    className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-rose-50 dark:bg-rose-950/40 text-rose-600 dark:text-rose-400 border border-rose-200 dark:border-rose-900/50 hover:bg-rose-100 dark:hover:bg-rose-900/40 transition-colors cursor-pointer"
                                                    title="Stop Instance"
                                                >
                                                    <Square className="w-3 h-3 fill-current" />
                                                </button>
                                            ) : (
                                                <button
                                                    type="button"
                                                    onClick={() => handleLaunch(inst.config.id)}
                                                    className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-emerald-50 dark:bg-emerald-950/40 text-emerald-600 dark:text-emerald-400 border border-emerald-200 dark:border-emerald-900/50 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 transition-colors cursor-pointer"
                                                    title="Launch Instance"
                                                >
                                                    <Play className="w-3 h-3 fill-current" />
                                                </button>
                                            )}

                                            <button
                                                type="button"
                                                onClick={() => setSwitchTargetInstance(inst)}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer"
                                                title="Switch Account"
                                            >
                                                <ArrowRightLeft className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={async () => {
                                                    try {
                                                        const msg = await fastForwardInstance(inst.config.id);
                                                        showToast(msg || `Rotated ${inst.config.name} to next best profile!`, 'success');
                                                    } catch (e: any) {
                                                        setActionError(e?.toString() || 'Fast forward failed');
                                                    }
                                                }}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer"
                                                title="Fast Forward to Next Best"
                                            >
                                                <FastForward className="w-3 h-3" />
                                            </button>

                                            {/* Softened Audit Button (VS Code slate theme) */}
                                            <button
                                                type="button"
                                                onClick={() => setAuditModalInstance({
                                                    id: inst.config.id,
                                                    name: inst.config.name,
                                                    sequence_name: inst.config.seq_num ? `Instance #${inst.config.seq_num}` : undefined
                                                })}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer"
                                                title="Audit Trail"
                                            >
                                                <History className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={async () => {
                                                    setSyncingInstanceIds(prev => ({ ...prev, [inst.config.id]: true }));
                                                    try {
                                                        await syncInstance(inst.config.id);
                                                    } catch (e: any) {
                                                        setActionError(e?.toString() || 'Failed to sync instance');
                                                    } finally {
                                                        setSyncingInstanceIds(prev => ({ ...prev, [inst.config.id]: false }));
                                                    }
                                                }}
                                                disabled={Boolean(syncingInstanceIds[inst.config.id])}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-teal-600 dark:text-teal-400 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer disabled:opacity-50"
                                                title="Sync PID and Quota"
                                            >
                                                <RotateCw className={cn("w-3 h-3 text-teal-500", syncingInstanceIds[inst.config.id] && "animate-spin")} />
                                            </button>
                                        </div>

                                        {/* Row 2: Prompts, Settings, Clone, Executable, Wipe, Delete */}
                                        <div className="flex items-center gap-1 w-full">
                                            <button
                                                type="button"
                                                onClick={() => setPromptTreeInstance({ id: inst.config.id, name: inst.config.name })}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 hover:text-cyan-600 dark:hover:text-cyan-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer"
                                                title="Prompt Tree"
                                            >
                                                <Layers className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={() => {
                                                    setSettingsModalTarget(inst);
                                                    setIsSettingsModalOpen(true);
                                                }}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 hover:text-blue-600 dark:hover:text-blue-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer"
                                                title="Settings & Sync"
                                            >
                                                <SlidersHorizontal className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={() => {
                                                    setCopyTargetId(inst.config.id);
                                                    setCopyInstanceName(`${inst.config.name} Copy`);
                                                    setCopyProjects(true);
                                                }}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer"
                                                title="Clone Profile"
                                            >
                                                <Copy className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={() => handleCloneExecutable(inst.config.id)}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 hover:text-purple-600 dark:hover:text-purple-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer"
                                                title="Clone Binary"
                                            >
                                                <Cpu className="w-3 h-3" />
                                            </button>

                                            <button
                                                type="button"
                                                onClick={() => handleWipeSession(inst.config.id)}
                                                disabled={inst.is_running}
                                                className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 hover:text-amber-600 dark:hover:text-amber-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                                title="Wipe Credentials"
                                            >
                                                <RotateCcw className="w-3 h-3" />
                                            </button>

                                            {!inst.config.is_default && (
                                                <button
                                                    type="button"
                                                    onClick={() => handleDelete(inst.config.id)}
                                                    disabled={inst.is_running}
                                                    className="flex-1 flex items-center justify-center text-xs px-2 py-1 rounded-[5px] bg-slate-100 dark:bg-slate-800/80 text-slate-600 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-300 border border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700 transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                                    title="Delete Profile"
                                                >
                                                    <Trash2 className="w-3 h-3" />
                                                </button>
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}

            {/* Create Instance Modal */}
            {isCreateOpen && (
                <div
                    className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4"
                    onClick={() => {
                        setIsCreateOpen(false);
                        setNewInstanceName('');
                        setNewInstanceBoundAccount('');
                        setNewInstanceFromInstance('');
                        setNewInstanceLaunchImmediately(false);
                    }}
                >
                    <div
                        className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="flex items-center justify-between pb-3 border-b border-gray-100 dark:border-base-100 mb-4">
                            <div className="flex items-center gap-2.5">
                                <Laptop className="w-5 h-5 text-blue-600" />
                                <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                    {t('instances.create_modal_title', 'Create New Profile')}
                                </h3>
                            </div>
                            <button
                                type="button"
                                onClick={() => {
                                    setIsCreateOpen(false);
                                    setNewInstanceName('');
                                    setNewInstanceBoundAccount('');
                                    setNewInstanceFromInstance('');
                                    setNewInstanceLaunchImmediately(false);
                                }}
                                className="btn btn-ghost btn-xs btn-circle text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 cursor-pointer"
                                title={t('common.close', 'Close')}
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                            Creates an isolated Antigravity profile folder with its own SQLite token store, extensions, and configuration.
                        </p>
                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1.5">
                            Profile Name
                        </label>
                        <input
                            type="text"
                            placeholder={t('instances.name_placeholder', 'Profile name (e.g., Personal, Client Work)')}
                            value={newInstanceName}
                            onChange={(e) => setNewInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                            className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-sm"
                            autoFocus
                        />
                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1.5">
                            Initial Account (Optional)
                        </label>
                        <select
                            value={newInstanceBoundAccount}
                            onChange={(e) => setNewInstanceBoundAccount(e.target.value)}
                            className="select select-sm w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-xs"
                        >
                            <option value="">Auto-assign next available account</option>
                            {accounts.map(acc => (
                                <option key={acc.id} value={acc.id}>
                                    {acc.email} ({acc.quota?.subscription_tier || 'FREE'})
                                </option>
                            ))}
                        </select>

                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1.5">
                            Clone Settings & Extensions From (Optional)
                        </label>
                        <select
                            value={newInstanceFromInstance}
                            onChange={(e) => setNewInstanceFromInstance(e.target.value)}
                            className="select select-sm w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-xs"
                        >
                            <option value="">Blank profile (clean isolated sandbox)</option>
                            {instances.map(inst => (
                                <option key={inst.config.id} value={inst.config.id}>
                                    {inst.config.name} {inst.config.is_default ? '(Default)' : ''}
                                </option>
                            ))}
                        </select>

                        <div className="flex items-center gap-2 mb-5">
                            <input
                                type="checkbox"
                                id="launch-immediately-check"
                                checked={newInstanceLaunchImmediately}
                                onChange={(e) => setNewInstanceLaunchImmediately(e.target.checked)}
                                className="checkbox checkbox-sm checkbox-primary rounded cursor-pointer"
                            />
                            <label htmlFor="launch-immediately-check" className="text-xs text-gray-700 dark:text-gray-300 cursor-pointer select-none">
                                Launch Antigravity window immediately after creation
                            </label>
                        </div>

                        <div className="flex justify-end gap-2.5">
                            <button
                                onClick={() => {
                                    setIsCreateOpen(false);
                                    setNewInstanceName('');
                                    setNewInstanceBoundAccount('');
                                    setNewInstanceFromInstance('');
                                    setNewInstanceLaunchImmediately(false);
                                }}
                                className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleCreate}
                                disabled={!newInstanceName.trim()}
                                className="btn btn-primary btn-sm"
                            >
                                {t('common.create', 'Create Profile')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Switch Account Modal */}
            {switchTargetInstance && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-lg shadow-2xl border border-gray-100 dark:border-base-100 max-h-[85vh] flex flex-col">
                        <div className="flex items-center justify-between pb-3 border-b border-gray-100 dark:border-base-100">
                            <div className="flex items-center gap-2.5">
                                <ArrowRightLeft className="w-5 h-5 text-blue-600" />
                                <div>
                                    <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                        Switch Account for {switchTargetInstance.config.name}
                                    </h3>
                                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                        Currently bound: <span className="font-mono font-medium">{switchTargetInstance.config.bound_email || 'None'}</span>
                                    </p>
                                </div>
                            </div>
                            <button
                                onClick={() => {
                                    setSwitchTargetInstance(null);
                                    setAccountSearchQuery('');
                                }}
                                className="btn btn-ghost btn-xs btn-circle"
                            >
                                ✕
                            </button>
                        </div>

                        <div className="pt-2 pb-1">
                            <div className="relative">
                                <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
                                <input
                                    type="text"
                                    value={accountSearchQuery}
                                    onChange={(e) => setAccountSearchQuery(e.target.value)}
                                    placeholder="Filter accounts by email or ID..."
                                    className="w-full pl-8 pr-2.5 py-1 text-xs bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500 text-gray-900 dark:text-base-content"
                                />
                            </div>
                        </div>

                        <div className="overflow-y-auto py-2 space-y-2 flex-1 my-1 pr-1">
                            {accounts.filter(a => {
                                const q = accountSearchQuery.toLowerCase().trim();
                                if (!q) return true;
                                return a.email.toLowerCase().includes(q) || a.id.toLowerCase().includes(q);
                            }).length === 0 ? (
                                <p className="text-xs text-gray-400 text-center py-4">No accounts matching filter.</p>
                            ) : (
                                accounts
                                    .filter(a => {
                                        const q = accountSearchQuery.toLowerCase().trim();
                                        if (!q) return true;
                                        return a.email.toLowerCase().includes(q) || a.id.toLowerCase().includes(q);
                                    })
                                    .map((acc) => {
                                    const isCurrent = switchTargetInstance.config.bound_account_id === acc.id ||
                                        (switchTargetInstance.config.bound_email && switchTargetInstance.config.bound_email.toLowerCase() === acc.email.toLowerCase());
                                    const proModel = findQuotaModel(acc.quota?.models, 'gemini-pro');
                                    const flashModel = findQuotaModel(acc.quota?.models, 'gemini-flash');
                                    const model = proModel || flashModel;
                                    const pct = model ? Math.min(100, Math.max(0, model.percentage)) : 0;
                                    const tier = (acc.quota?.subscription_tier || 'FREE').toUpperCase();

                                    return (
                                        <div
                                            key={acc.id}
                                            className={cn(
                                                "p-3 rounded-xl border flex items-center justify-between gap-3 transition-colors",
                                                isCurrent
                                                    ? "bg-blue-50/50 dark:bg-blue-950/20 border-blue-200 dark:border-blue-800"
                                                    : "bg-gray-50/60 dark:bg-base-100/60 border-gray-100 dark:border-base-100 hover:border-gray-300 dark:hover:border-base-300"
                                            )}
                                        >
                                            <div className="min-w-0 flex-1">
                                                <div className="flex items-center gap-2 mb-1">
                                                    <span className="font-mono text-xs font-semibold text-gray-900 dark:text-base-content truncate">
                                                        {acc.email}
                                                    </span>
                                                    <span className={cn(
                                                        "text-[9px] font-bold px-1.5 py-0.5 rounded",
                                                        tier.includes('ULTRA') ? "bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300" :
                                                        tier.includes('PRO') ? "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300" :
                                                        "bg-gray-100 text-gray-600 dark:bg-[#15334d] dark:text-slate-300"
                                                    )}>
                                                        {tier}
                                                    </span>
                                                    {isCurrent ? (
                                                        <span className="text-[10px] text-blue-600 dark:text-blue-400 font-semibold">
                                                            Active
                                                        </span>
                                                    ) : null}
                                                </div>
                                                <div className="flex items-center gap-2 text-[11px] text-gray-500">
                                                    <span>Quota: {pct}%</span>
                                                    <div className="w-24 h-1.5 bg-gray-200 dark:bg-base-300 rounded-full overflow-hidden">
                                                        <div
                                                            className={cn(
                                                                "h-full rounded-full",
                                                                pct >= 50 ? "bg-emerald-500" : pct >= 20 ? "bg-amber-500" : "bg-rose-500"
                                                            )}
                                                            style={{ width: `${pct}%` }}
                                                        />
                                                    </div>
                                                </div>
                                            </div>

                                            <button
                                                disabled={isCurrent || isLoading}
                                                onClick={async () => {
                                                    setActionError(null);
                                                    try {
                                                        await switchAccountToInstance(acc.id, switchTargetInstance.config.id);
                                                        showToast(`Switched ${switchTargetInstance.config.name} to ${acc.email}`, 'success');
                                                        setSwitchTargetInstance(null);
                                                        setAccountSearchQuery('');
                                                    } catch (e: any) {
                                                        setActionError(e?.toString() || 'Failed to switch account');
                                                    }
                                                }}
                                                className={cn(
                                                    "btn btn-xs",
                                                    isCurrent ? "btn-disabled opacity-50" : "btn-primary"
                                                )}
                                            >
                                                {isCurrent ? 'Current' : 'Select'}
                                            </button>
                                        </div>
                                    );
                                })
                            )}
                        </div>

                        <div className="flex justify-end pt-3 border-t border-gray-100 dark:border-base-100">
                            <button
                                onClick={() => {
                                    setSwitchTargetInstance(null);
                                    setAccountSearchQuery('');
                                }}
                                className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400"
                            >
                                {t('common.close', 'Close')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Copy / Duplicate Modal */}
            {copyTargetId && (
                <div
                    className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4"
                    onClick={() => {
                        setCopyTargetId(null);
                        setCopyInstanceName('');
                    }}
                >
                    <div
                        className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-200/80 dark:border-base-100 transition-all"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="flex items-center justify-between pb-3 border-b border-gray-100 dark:border-base-100 mb-4">
                            <div className="flex items-center gap-2.5">
                                <div className="p-2 rounded-xl bg-indigo-50 dark:bg-indigo-950/40 text-indigo-600 dark:text-indigo-400">
                                    <Copy className="w-5 h-5" />
                                </div>
                                <div>
                                    <h3 className="font-bold text-sm text-gray-900 dark:text-base-content">
                                        {t('instances.copy_modal_title', 'Duplicate / Clone Profile')}
                                    </h3>
                                    <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                        {t('instances.copy_modal_subtitle', 'Create an isolated duplicate of this profile environment')}
                                    </p>
                                </div>
                            </div>
                            <button
                                type="button"
                                onClick={() => {
                                    setCopyTargetId(null);
                                    setCopyInstanceName('');
                                }}
                                className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-base-100 transition-colors cursor-pointer"
                                title={t('common.close', 'Close')}
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>

                        <label className="block text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1.5">
                            {t('instances.copy_name_label', 'New Profile Name')}
                        </label>
                        <input
                            type="text"
                            placeholder={t('instances.copy_placeholder', 'New profile name')}
                            value={copyInstanceName}
                            onChange={(e) => setCopyInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCopy()}
                            className="w-full px-4 py-3 bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 text-gray-900 dark:text-slate-100 rounded-xl mb-4 text-xs font-medium focus:outline-none focus:ring-2 focus:ring-indigo-500/40 transition-all shadow-xs"
                            autoFocus
                        />

                        {/* Scope Selection Cards */}
                        <div className="mb-5 space-y-2">
                            <span className="block text-[11px] font-bold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                {t('instances.clone_mode_label', 'Duplication Scope / Clone Type')}
                            </span>
                            <div className="grid grid-cols-1 gap-2.5">
                                <div
                                    onClick={() => setCloneMode('full')}
                                    className={cn(
                                        "p-3 rounded-xl border-2 transition-all cursor-pointer flex items-start gap-3",
                                        cloneMode === 'full'
                                            ? "border-indigo-500 bg-indigo-50/60 dark:bg-indigo-950/30 ring-2 ring-indigo-500/20 shadow-xs"
                                            : "border-gray-200 dark:border-base-100 bg-gray-50/50 dark:bg-base-100/60 hover:border-gray-300 dark:hover:border-base-300"
                                    )}
                                >
                                    <div className="mt-0.5 shrink-0">
                                        <div className={cn(
                                            "w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors",
                                            cloneMode === 'full' ? "border-indigo-600 bg-indigo-600" : "border-gray-400 dark:border-gray-600"
                                        )}>
                                            {cloneMode === 'full' && <div className="w-1.5 h-1.5 rounded-full bg-white" />}
                                        </div>
                                    </div>
                                    <div className="min-w-0">
                                        <div className="font-bold text-xs text-gray-900 dark:text-gray-100 flex items-center gap-1.5">
                                            <span>{t('instances.clone_mode_full', 'IDE Copy (Full Environment & Sessions)')}</span>
                                            <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-indigo-100 dark:bg-indigo-900/60 text-indigo-700 dark:text-indigo-300">
                                                Full
                                            </span>
                                        </div>
                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 leading-relaxed mt-0.5">
                                            {t('instances.clone_mode_full_desc', 'Clones complete isolated environment, sessions, extensions, cache, and state.')}
                                        </p>
                                    </div>
                                </div>

                                <div
                                    onClick={() => setCloneMode('profile')}
                                    className={cn(
                                        "p-3 rounded-xl border-2 transition-all cursor-pointer flex items-start gap-3",
                                        cloneMode === 'profile'
                                            ? "border-indigo-500 bg-indigo-50/60 dark:bg-indigo-950/30 ring-2 ring-indigo-500/20 shadow-xs"
                                            : "border-gray-200 dark:border-base-100 bg-gray-50/50 dark:bg-base-100/60 hover:border-gray-300 dark:hover:border-base-300"
                                    )}
                                >
                                    <div className="mt-0.5 shrink-0">
                                        <div className={cn(
                                            "w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors",
                                            cloneMode === 'profile' ? "border-indigo-600 bg-indigo-600" : "border-gray-400 dark:border-gray-600"
                                        )}>
                                            {cloneMode === 'profile' && <div className="w-1.5 h-1.5 rounded-full bg-white" />}
                                        </div>
                                    </div>
                                    <div className="min-w-0">
                                        <div className="font-bold text-xs text-gray-900 dark:text-gray-100 flex items-center gap-1.5">
                                            <span>{t('instances.clone_mode_profile', 'Profile Copy (Preferences & Snippets)')}</span>
                                            <span className="px-1.5 py-0.2 rounded text-[9px] font-bold bg-blue-100 dark:bg-blue-900/60 text-blue-700 dark:text-blue-300">
                                                Preferences
                                            </span>
                                        </div>
                                        <p className="text-[11px] text-gray-500 dark:text-gray-400 leading-relaxed mt-0.5">
                                            {t('instances.clone_mode_profile_desc', 'Copies only User preferences, keybindings, and snippets without bulky runtime session state.')}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Copy Workspace Projects & Folders Option */}
                        <div className="mb-5 p-3 rounded-xl border border-gray-200 dark:border-base-100 bg-gray-50/50 dark:bg-base-100/50 flex items-center justify-between gap-3">
                            <div className="min-w-0 pr-2">
                                <label htmlFor="instances-page-copy-projects" className="font-bold text-xs text-gray-900 dark:text-gray-100 cursor-pointer block">
                                    {t('instances.copy_projects_label', 'Copy Workspace Projects & Folders')}
                                </label>
                                <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-0.5 leading-relaxed">
                                    {t('instances.copy_projects_desc', 'Duplicate opened workspaces, project states, and recent folder paths into the new profile.')}
                                </p>
                            </div>
                            <input
                                id="instances-page-copy-projects"
                                type="checkbox"
                                checked={copyProjects}
                                onChange={(e) => setCopyProjects(e.target.checked)}
                                className="checkbox checkbox-sm checkbox-primary rounded cursor-pointer shrink-0"
                            />
                        </div>

                        <div className="flex justify-end items-center gap-2.5 pt-2">
                            <button
                                type="button"
                                onClick={() => {
                                    setCopyTargetId(null);
                                    setCopyInstanceName('');
                                }}
                                className="px-4 py-2 rounded-xl text-xs font-medium text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-base-100 hover:bg-gray-100 dark:hover:bg-base-100 transition-all duration-200 active:scale-95 cursor-pointer"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleCopy}
                                disabled={!copyInstanceName.trim()}
                                className="px-5 py-2 bg-gradient-to-r from-blue-600 via-indigo-600 to-purple-600 hover:from-blue-500 hover:via-indigo-500 hover:to-purple-500 text-white font-semibold text-xs rounded-xl shadow-lg shadow-indigo-500/25 hover:shadow-indigo-500/40 transition-all duration-300 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer flex items-center gap-1.5"
                            >
                                <Copy className="w-3.5 h-3.5" />
                                <span>{t('instances.duplicate', 'Duplicate')}</span>
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Edit Modal */}
            {editTargetId && (
                <div
                    className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4"
                    onClick={() => {
                        setEditTargetId(null);
                        setEditInstanceName('');
                    }}
                >
                    <div
                        className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="flex items-center justify-between pb-3 border-b border-gray-100 dark:border-base-100 mb-4">
                            <div className="flex items-center gap-2.5">
                                <Pencil className="w-5 h-5 text-blue-600" />
                                <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                    {t('instances.edit_modal_title', 'Rename Profile')}
                                </h3>
                            </div>
                            <button
                                type="button"
                                onClick={() => {
                                    setEditTargetId(null);
                                    setEditInstanceName('');
                                }}
                                className="btn btn-ghost btn-xs btn-circle text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 cursor-pointer"
                                title={t('common.close', 'Close')}
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                            {t('instances.edit_modal_desc', 'Update display name for this isolated instance profile.')}
                        </p>
                        <input
                            type="text"
                            placeholder={t('instances.edit_placeholder', 'New profile name')}
                            value={editInstanceName}
                            onChange={(e) => setEditInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleEdit()}
                            className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-5 text-sm"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2.5">
                            <button
                                onClick={() => setEditTargetId(null)}
                                className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleEdit}
                                disabled={!editInstanceName.trim()}
                                className="btn btn-primary btn-sm"
                            >
                                {t('common.save', 'Save')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Instance Settings & Deep Sync Modal */}
            <InstanceSettingsModal
                isOpen={isSettingsModalOpen}
                onClose={() => {
                    setIsSettingsModalOpen(false);
                    setSettingsModalTarget(null);
                }}
                targetInstance={settingsModalTarget}
                instances={instances}
                onInstancesUpdated={() => fetchInstances(true)}
            />

            {/* Project & Conversation Prompt Tree View Modal */}
            <PromptTreeViewModal
                isOpen={Boolean(promptTreeInstance)}
                onClose={() => setPromptTreeInstance(null)}
                instanceId={promptTreeInstance?.id || ''}
                instanceName={promptTreeInstance?.name || ''}
            />

            {/* Instance Audit Trail & Switch History Modal */}
            <InstanceAuditTrailModal
                isOpen={Boolean(auditModalInstance)}
                onClose={() => setAuditModalInstance(null)}
                instance={auditModalInstance}
            />
        </div>
        </div>
    );
}

