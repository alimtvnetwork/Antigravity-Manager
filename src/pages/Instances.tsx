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
} from 'lucide-react';
import { Gemini } from '@lobehub/icons';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../stores/useInstanceStore';
import { useAccountStore } from '../stores/useAccountStore';
import type { InstanceStatus } from '../services/instanceService';
import { findQuotaModel } from '../config/modelConfig';
import { formatTimeRemaining } from '../utils/format';
import { isTauri } from '../utils/env';
import { cn } from '../utils/cn';
import { showToast } from '../components/common/ToastContainer';

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
    const [editTargetId, setEditTargetId] = useState<string | null>(null);
    const [editInstanceName, setEditInstanceName] = useState('');
    const [actionError, setActionError] = useState<string | null>(null);
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

    const filteredInstances = instances.filter((inst) => {
        const query = searchQuery.toLowerCase();
        return (
            inst.config.name.toLowerCase().includes(query) ||
            inst.config.id.toLowerCase().includes(query) ||
            (inst.config.bound_email && inst.config.bound_email.toLowerCase().includes(query))
        );
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
            const copied = await copyInstance(copyTargetId, copyInstanceName.trim(), cloneMode);
            await setActiveInstance(copied.id);
            setCopyInstanceName('');
            setCopyTargetId(null);
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

                <div className="flex flex-wrap items-center gap-2">
                    <div className="px-3 py-1.5 rounded-lg bg-gray-100 dark:bg-base-200 text-xs text-gray-600 dark:text-gray-300 font-medium whitespace-nowrap">
                        {t('instances.running_summary', 'Running {{running}} of {{total}}', {
                            running: runningCount,
                            total: instances.length,
                        })}
                    </div>
                    <button
                        onClick={() => {
                            fetchInstances();
                            fetchSwitcherStatus();
                        }}
                        disabled={isLoading}
                        className="btn btn-ghost btn-sm gap-1.5 border border-gray-200 dark:border-base-100 text-xs"
                        title={t('common.refresh', 'Refresh')}
                    >
                        <RotateCw className={cn("w-3.5 h-3.5", isLoading ? "animate-spin" : "")} />
                        <span className="hidden sm:inline">{t('common.refresh', 'Refresh')}</span>
                    </button>
                    <button
                        onClick={async () => {
                            try {
                                const msg = await cleanAndRestartWorkspace();
                                showToast(msg || 'Stuck Electron processes cleared & Antigravity restarted!', 'success');
                            } catch (e: any) {
                                setActionError(e?.toString() || 'Failed to clean and restart workspace');
                            }
                        }}
                        disabled={isLoading}
                        className="btn btn-warning btn-sm gap-1.5 shadow-sm text-xs font-semibold cursor-pointer"
                        title="Force-terminate lingering background Electron/Antigravity processes, purge lockfiles, and cleanly relaunch Antigravity"
                    >
                        <Sparkles className="w-3.5 h-3.5" />
                        <span className="hidden md:inline">Clean & Restart</span>
                    </button>
                    <button
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
                            "btn btn-sm gap-1.5 shadow-sm text-xs font-semibold cursor-pointer border",
                            switcherStatus?.is_running
                                ? "bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800"
                                : "btn-ghost border-gray-200 dark:border-base-100 text-gray-600 dark:text-gray-400"
                        )}
                        title="Toggle background auto-profile switcher daemon"
                    >
                        {switcherStatus?.is_running ? (
                            <ToggleRight className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                        ) : (
                            <ToggleLeft className="w-4 h-4 text-gray-400" />
                        )}
                        <span>Auto-Switch: {switcherStatus?.is_running ? 'ON' : 'OFF'}</span>
                    </button>
                    <button
                        onClick={async () => {
                            try {
                                const msg = await triggerManualRotation();
                                showToast(msg || 'Evaluated quota across instances!', 'success');
                            } catch (e: any) {
                                setActionError(e?.toString() || 'Failed to trigger quota evaluation');
                            }
                        }}
                        disabled={isLoading}
                        className="btn btn-ghost border border-gray-200 dark:border-base-100 text-gray-600 dark:text-gray-400 btn-sm gap-1.5 shadow-sm text-xs font-semibold cursor-pointer hover:text-blue-600"
                        title="Evaluate rolling quota across all monitored instances and auto-rotate any low quota accounts"
                    >
                        <RotateCw className={cn("w-3.5 h-3.5", isLoading ? "animate-spin" : "")} />
                        <span className="hidden lg:inline">Eval Quota</span>
                    </button>
                    <button
                        onClick={() => {
                            setNewInstanceName('');
                            setIsCreateOpen(true);
                        }}
                        className="btn btn-primary btn-sm gap-1.5 shadow-sm"
                    >
                        <Plus className="w-4 h-4" />
                        <span>{t('instances.create_btn', 'New Instance')}</span>
                    </button>
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
                    <button
                        onClick={async () => {
                            try {
                                const msg = await triggerManualRotation();
                                showToast(msg || 'Rotated to next best profile!', 'success');
                            } catch (e: any) {
                                setActionError(e?.toString() || 'Rotation failed');
                            }
                        }}
                        className="btn btn-xs sm:btn-sm gap-1.5 shrink-0 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 text-white font-semibold shadow-md shadow-blue-500/20 border-none rounded-xl cursor-pointer transition-all active:scale-95"
                    >
                        <RotateCcw className="w-3.5 h-3.5" />
                        <span>{t('instances.rotate_next_best', 'Rotate to Next Best')}</span>
                    </button>
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
                    className="btn btn-primary btn-sm gap-1.5 shadow-sm"
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
                            className="btn btn-primary btn-sm gap-1.5 shadow-sm"
                        >
                            <Play className="w-3.5 h-3.5" />
                            <span>{t('instances.init_default', 'Initialize Default Profile')}</span>
                        </button>
                        <button
                            onClick={() => {
                                setNewInstanceName('');
                                setIsCreateOpen(true);
                            }}
                            className="btn btn-outline btn-sm gap-1.5"
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
            ) : (
                <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
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
                                    "rounded-2xl border transition-all flex flex-col justify-between bg-white dark:bg-[#0a1e30] overflow-hidden shadow-xs backdrop-blur-xs",
                                    isActive
                                        ? "border-blue-500 shadow-lg ring-2 ring-blue-500/30 bg-blue-50/15 dark:bg-[#0c2438]"
                                        : "border-gray-200/80 dark:border-[#15334d] hover:border-gray-300 dark:hover:border-blue-500/40"
                                )}
                            >
                                {/* Top Accent Bar identifying profile color */}
                                <div className={cn("h-1.5 w-full bg-gradient-to-r", theme.accentBar)} />

                                <div className="p-4 sm:p-5 flex flex-col flex-1 justify-between min-h-[280px] min-w-0">
                                    {/* Card Top */}
                                    <div className="min-w-0">
                                        <div className="flex items-start justify-between gap-3 mb-3">
                                            <div className="flex items-center gap-2 min-w-0 flex-wrap">
                                                <span
                                                    className={cn(
                                                        "w-2.5 h-2.5 rounded-full shrink-0",
                                                        inst.is_running ? "bg-emerald-500 shadow-xs shadow-emerald-500/50 animate-pulse" : "bg-gray-300 dark:bg-gray-600"
                                                    )}
                                                />
                                                <span className="px-2 py-0.5 rounded-md text-xs font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 shrink-0">
                                                    #{seqNumber}
                                                </span>
                                                <h3 className={cn("font-bold text-sm truncate", isActive ? "text-blue-900 dark:text-blue-100" : "text-gray-900 dark:text-base-content")} title={inst.config.name}>
                                                    {inst.config.name}
                                                </h3>
                                                {inst.config.is_default ? (
                                                    <span className="px-2 py-0.5 rounded text-[10px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-400/30 shrink-0">
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
                                                        className="px-1.5 py-0.5 rounded text-[10px] font-medium text-gray-400 hover:text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-950/30 border border-dashed border-gray-300 dark:border-base-100 transition-colors cursor-pointer flex items-center gap-1 shrink-0"
                                                        title="Set as default profile"
                                                    >
                                                        <Star className="w-2.5 h-2.5" />
                                                        <span>Set Default</span>
                                                    </button>
                                                )}
                                            </div>
                                            <div className="shrink-0 flex items-center gap-1">
                                                {isActive ? (
                                                    <span className="px-2 py-0.5 rounded-md text-[10px] font-bold bg-blue-600 text-white shadow-xs tracking-wider">
                                                        Active Target
                                                    </span>
                                                ) : (
                                                    <button
                                                        onClick={() => setActiveInstance(inst.config.id)}
                                                        className="text-[10px] text-gray-500 hover:text-blue-600 transition-colors font-medium mr-1 cursor-pointer"
                                                        title="Set as active instance for account switches"
                                                    >
                                                        Set Active
                                                    </button>
                                                )}
                                                {/* Top Quick Actions: Edit, Duplicate, Delete */}
                                                <button
                                                    onClick={() => {
                                                        setEditTargetId(inst.config.id);
                                                        setEditInstanceName(inst.config.name);
                                                    }}
                                                    className="btn btn-ghost btn-xs p-1 text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30 cursor-pointer"
                                                    title={t('instances.edit_title', 'Rename profile')}
                                                >
                                                    <Pencil className="w-3.5 h-3.5" />
                                                </button>
                                                <button
                                                    onClick={() => {
                                                        setCopyTargetId(inst.config.id);
                                                        setCopyInstanceName(`${inst.config.name} Copy`);
                                                    }}
                                                    className="btn btn-ghost btn-xs p-1 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 cursor-pointer"
                                                    title="Clone / Duplicate profile settings and extensions"
                                                >
                                                    <Copy className="w-3.5 h-3.5" />
                                                </button>
                                                {inst.config.is_default ? null : (
                                                    <button
                                                        onClick={() => handleDelete(inst.config.id)}
                                                        disabled={inst.is_running}
                                                        className="btn btn-ghost btn-xs p-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-900/30 disabled:opacity-30 cursor-pointer"
                                                        title="Delete profile"
                                                    >
                                                        <Trash2 className="w-3.5 h-3.5" />
                                                    </button>
                                                )}
                                            </div>
                                        </div>

                                        {/* Bound Account / Email Section with dark glass capsule */}
                                        <div className="py-2 px-3 rounded-xl bg-gray-50/80 dark:bg-[#0c2438]/90 border border-gray-100 dark:border-[#15334d] mb-3 flex items-center justify-between gap-2">
                                            <div className="flex items-center gap-1.5 min-w-0 flex-1">
                                                <Mail className="w-3.5 h-3.5 text-gray-400 shrink-0" />
                                                <span className="text-[11px] text-gray-500 dark:text-gray-400 font-medium shrink-0">
                                                    Account:
                                                </span>
                                                {displayEmail ? (
                                                    <span
                                                        className={cn(
                                                            "px-2 py-0.5 rounded-md text-xs font-semibold font-mono border flex items-center gap-1.5 min-w-0 shadow-2xs",
                                                            theme.emailPill
                                                        )}
                                                        title={displayEmail}
                                                    >
                                                        <span className={cn("w-1.5 h-1.5 rounded-full shrink-0", theme.dot)} />
                                                        <span className="truncate">{displayEmail}</span>
                                                    </span>
                                                ) : (
                                                    <span className="text-xs text-gray-400 italic">
                                                        Unassigned
                                                    </span>
                                                )}
                                            </div>
                                            {boundAccount?.quota?.subscription_tier ? (() => {
                                                const tier = boundAccount.quota.subscription_tier.toLowerCase();
                                                if (tier.includes('ultra')) {
                                                    return (
                                                        <span className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-gradient-to-r from-purple-600 to-pink-600 text-white text-[9px] font-bold shadow-xs shrink-0">
                                                            <Gem className="w-2.5 h-2.5 fill-current" />
                                                            ULTRA
                                                        </span>
                                                    );
                                                }
                                                if (tier.includes('pro')) {
                                                    return (
                                                        <span className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-[9px] font-bold shadow-xs shrink-0">
                                                            <Diamond className="w-2.5 h-2.5 fill-current" />
                                                            PRO
                                                        </span>
                                                    );
                                                }
                                                return (
                                                    <span className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-gray-100 dark:bg-[#15334d] text-gray-600 dark:text-slate-300 text-[9px] font-bold shadow-xs border border-gray-200 dark:border-[#15334d] shrink-0">
                                                        <Circle className="w-2.5 h-2.5" />
                                                        FREE
                                                    </span>
                                                );
                                            })() : null}
                                        </div>

                                        {/* Colorful Gemini Quota Progress Bar Section with dark glass Quota box */}
                                        <div className="mb-3">
                                            {geminiModel ? (() => {
                                                const pct = Math.min(100, Math.max(0, geminiModel.percentage));
                                                const getGradient = (percentage: number) => {
                                                    if (percentage >= 50) return 'from-emerald-500 via-teal-400 to-emerald-400 shadow-emerald-500/20';
                                                    if (percentage >= 20) return 'from-amber-500 via-yellow-400 to-orange-400 shadow-amber-500/20';
                                                    return 'from-rose-500 via-red-500 to-pink-500 shadow-rose-500/20';
                                                };
                                                const getTextClass = (percentage: number) => {
                                                    if (percentage >= 50) return 'text-emerald-600 dark:text-emerald-400';
                                                    if (percentage >= 20) return 'text-amber-600 dark:text-amber-400';
                                                    return 'text-rose-600 dark:text-rose-400';
                                                };

                                                return (
                                                    <div className="p-3 rounded-xl bg-gray-50/90 dark:bg-[#0c2438]/90 border border-gray-200/70 dark:border-[#15334d] space-y-2">
                                                        <div className="flex items-center justify-between text-xs">
                                                            <div className="flex items-center gap-1.5">
                                                                <Gemini.Color className="w-4 h-4 shrink-0" />
                                                                <span className="font-semibold text-gray-800 dark:text-gray-200 text-xs">
                                                                    {geminiModel.display_name || 'Gemini 3.1 Pro'}
                                                                </span>
                                                            </div>
                                                            <div className="flex items-center gap-2">
                                                                {geminiModel.reset_time ? (
                                                                    <span className="text-[10px] text-gray-500 dark:text-gray-400 flex items-center gap-1 font-mono" title={`Resets in ${formatTimeRemaining(geminiModel.reset_time)}`}>
                                                                        <Clock className="w-3 h-3 text-gray-400" />
                                                                        {formatTimeRemaining(geminiModel.reset_time)}
                                                                    </span>
                                                                ) : null}
                                                                <span className={cn("font-mono font-bold text-xs px-2 py-0.5 rounded-md bg-white dark:bg-[#081a2b] shadow-2xs border border-gray-200 dark:border-[#15334d]", getTextClass(pct))}>
                                                                    {pct}%
                                                                </span>
                                                            </div>
                                                        </div>

                                                        {/* Progress bar with glowing animated gradient */}
                                                        <div className="h-2.5 w-full bg-gray-200/90 dark:bg-[#081a2b] border border-transparent dark:border-[#15334d]/60 rounded-full overflow-hidden p-0.5 relative shadow-inner">
                                                            <div
                                                                className={cn(
                                                                    "h-full rounded-full bg-gradient-to-r transition-all duration-700 ease-out shadow-xs",
                                                                    getGradient(pct)
                                                                )}
                                                                style={{ width: `${pct}%` }}
                                                            />
                                                        </div>
                                                    </div>
                                                );
                                            })() : boundAccount ? (
                                                <div className="p-2.5 rounded-xl bg-gray-50/60 dark:bg-[#0c2438]/90 border border-gray-100 dark:border-[#15334d] flex items-center justify-between text-xs text-gray-500">
                                                    <div className="flex items-center gap-2">
                                                        <Gemini.Color className="w-4 h-4 shrink-0 opacity-70" />
                                                        <span className="text-[11px] text-gray-500 dark:text-gray-400">Gemini quota not synced</span>
                                                    </div>
                                                    <button
                                                        onClick={() => refreshQuota(boundAccount.id)}
                                                        className="btn btn-ghost btn-xs text-blue-600 dark:text-blue-400 hover:underline gap-1 text-[11px] cursor-pointer"
                                                    >
                                                        <RotateCw className="w-3 h-3" />
                                                        <span>Sync</span>
                                                    </button>
                                                </div>
                                            ) : (
                                                <div className="p-2.5 rounded-xl bg-gray-50/40 dark:bg-[#0c2438]/60 border border-dashed border-gray-200 dark:border-[#15334d] flex items-center justify-between text-xs text-gray-400">
                                                    <div className="flex items-center gap-2">
                                                        <Gemini.Color className="w-4 h-4 shrink-0 opacity-40 grayscale" />
                                                        <span className="text-[11px] italic">Gemini Quota (No profile bound)</span>
                                                    </div>
                                                    <span className="text-[10px] text-gray-400/80">Launch to assign</span>
                                                </div>
                                            )}
                                        </div>

                                        {/* Status and Profile details: Modern compact badge layout */}
                                        <div className="space-y-2 py-2.5 border-t border-gray-100 dark:border-[#15334d]/80 text-xs">
                                            <div className="flex items-center justify-between gap-2 flex-wrap">
                                                {/* Status Badge */}
                                                {inst.is_running ? (
                                                    <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-semibold bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/20 shadow-2xs">
                                                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                                        Running {inst.pid ? `(PID: ${inst.pid})` : ''}
                                                    </span>
                                                ) : (
                                                    <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-medium bg-slate-500/10 text-slate-600 dark:text-slate-400 border border-slate-400/20">
                                                        <span className="w-1.5 h-1.5 rounded-full bg-slate-400" />
                                                        Idle
                                                    </span>
                                                )}

                                                {/* Profile ID Monospace Pill */}
                                                <span
                                                    className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-mono font-medium bg-gray-100 dark:bg-[#0c2438] text-gray-600 dark:text-slate-300 border border-gray-200 dark:border-[#15334d] truncate max-w-[150px]"
                                                    title={`Profile ID: ${inst.config.id}`}
                                                >
                                                    <span className="text-gray-400 dark:text-slate-500">ID:</span>
                                                    <span className="truncate">{inst.config.id}</span>
                                                </span>
                                            </div>

                                            {/* Path with clean folder badge and copy action */}
                                            <div className="flex items-center justify-between gap-2 px-2.5 py-1.5 rounded-xl bg-gray-50/80 dark:bg-[#0c2438]/70 border border-gray-200/70 dark:border-[#15334d] text-[11px] group transition-colors hover:border-gray-300 dark:hover:border-blue-500/30">
                                                <div className="flex items-center gap-1.5 min-w-0 flex-1 text-gray-500 dark:text-slate-400" title={inst.config.data_dir}>
                                                    <Folder className="w-3.5 h-3.5 shrink-0 text-blue-500/80" />
                                                    <span className="truncate font-mono text-[10px]">{inst.config.data_dir}</span>
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
                                                    className="opacity-60 group-hover:opacity-100 p-1 rounded-md hover:bg-gray-200 dark:hover:bg-[#15334d] text-gray-500 dark:text-gray-300 transition-opacity cursor-pointer shrink-0"
                                                    title="Copy directory path"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                </button>
                                            </div>

                                            {/* Custom executable path badge */}
                                            {inst.config.executable_path ? (
                                                <div className="flex items-center justify-between gap-2 px-2.5 py-1.5 rounded-xl bg-purple-50/50 dark:bg-purple-950/20 border border-purple-200/40 dark:border-purple-800/30 text-[11px] group transition-colors">
                                                    <div className="flex items-center gap-1.5 min-w-0 flex-1 text-purple-700 dark:text-purple-300" title={inst.config.executable_path}>
                                                        <Cpu className="w-3.5 h-3.5 shrink-0 text-purple-500" />
                                                        <span className="truncate font-mono text-[10px]">{inst.config.executable_path}</span>
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
                                                        className="opacity-60 group-hover:opacity-100 p-1 rounded-md hover:bg-purple-200/60 dark:hover:bg-purple-900/40 text-purple-700 dark:text-purple-300 transition-opacity cursor-pointer shrink-0"
                                                        title="Copy executable path"
                                                    >
                                                        <Copy className="w-3 h-3" />
                                                    </button>
                                                </div>
                                            ) : null}
                                        </div>
                                    </div>

                                    {/* Card Actions with clear visual hierarchy */}
                                    <div className="pt-3 border-t border-gray-100 dark:border-[#15334d]/80 mt-2">
                                        <div className="flex items-center justify-between gap-2 flex-wrap">
                                            {/* Primary Action Buttons (Launch/Stop, Switch, FF) */}
                                            <div className="flex items-center gap-1.5">
                                                {inst.is_running ? (
                                                    <button
                                                        onClick={() => stopInstance(inst.config.id)}
                                                        className="btn btn-xs bg-rose-500/15 hover:bg-rose-500 text-rose-600 dark:text-rose-400 hover:text-white border border-rose-500/30 rounded-lg gap-1 font-semibold cursor-pointer transition-colors shadow-2xs"
                                                        title="Gracefully stop this instance window"
                                                    >
                                                        <Square className="w-3 h-3" />
                                                        <span>Stop</span>
                                                    </button>
                                                ) : (
                                                    <button
                                                        onClick={() => handleLaunch(inst.config.id)}
                                                        className="btn btn-xs bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 text-white border-none rounded-lg gap-1 font-semibold cursor-pointer shadow-xs shadow-blue-500/25 transition-all active:scale-95"
                                                        title="Launch instance window"
                                                    >
                                                        <Play className="w-3 h-3" />
                                                        <span>Launch</span>
                                                    </button>
                                                )}

                                                {/* Sleek Switch Account Button */}
                                                <button
                                                    onClick={() => setSwitchTargetInstance(inst)}
                                                    className="btn btn-xs bg-sky-500/10 hover:bg-sky-500 text-sky-700 dark:text-sky-300 hover:text-white border border-sky-500/25 rounded-lg gap-1 font-semibold cursor-pointer transition-colors"
                                                    title="Explicitly select and switch this instance to any registered account"
                                                >
                                                    <ArrowRightLeft className="w-3 h-3" />
                                                    <span>Switch</span>
                                                </button>

                                                {/* Fast Forward Badge Button */}
                                                <button
                                                    onClick={async () => {
                                                        try {
                                                            const msg = await fastForwardInstance(inst.config.id);
                                                            showToast(msg || `Rotated ${inst.config.name} to next best profile!`, 'success');
                                                        } catch (e: any) {
                                                            setActionError(e?.toString() || 'Fast forward failed');
                                                        }
                                                    }}
                                                    className="btn btn-xs bg-purple-500/10 hover:bg-purple-500 text-purple-700 dark:text-purple-300 hover:text-white border border-purple-500/25 rounded-lg gap-1 font-bold cursor-pointer transition-colors"
                                                    title="Fast Forward: Automatically rotate to the next healthiest profile in pool"
                                                >
                                                    <FastForward className="w-3 h-3" />
                                                    <span>FF</span>
                                                </button>
                                            </div>

                                            {/* Clean Secondary Utility Icon Buttons (Clone, Executable, Wipe, Delete) */}
                                            <div className="flex items-center gap-1">
                                                <button
                                                    onClick={() => {
                                                        setCopyTargetId(inst.config.id);
                                                        setCopyInstanceName(`${inst.config.name} Copy`);
                                                    }}
                                                    className="p-1.5 rounded-lg text-gray-500 dark:text-slate-400 hover:text-indigo-600 dark:hover:text-indigo-300 hover:bg-indigo-50 dark:hover:bg-indigo-950/40 border border-gray-200/70 dark:border-[#15334d] transition-colors cursor-pointer"
                                                    title="Clone profile settings and extensions"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                </button>

                                                <button
                                                    onClick={() => handleCloneExecutable(inst.config.id)}
                                                    className="p-1.5 rounded-lg text-gray-500 dark:text-slate-400 hover:text-purple-600 dark:hover:text-purple-300 hover:bg-purple-50 dark:hover:bg-purple-950/40 border border-gray-200/70 dark:border-[#15334d] transition-colors cursor-pointer"
                                                    title="Clone executable binary for this profile"
                                                >
                                                    <Cpu className="w-3 h-3" />
                                                </button>

                                                <button
                                                    onClick={() => handleWipeSession(inst.config.id)}
                                                    disabled={inst.is_running}
                                                    className="p-1.5 rounded-lg text-gray-500 dark:text-slate-400 hover:text-amber-600 dark:hover:text-amber-300 hover:bg-amber-50 dark:hover:bg-amber-950/40 border border-gray-200/70 dark:border-[#15334d] transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                                    title="Wipe auth credentials (keep settings)"
                                                >
                                                    <RotateCcw className="w-3 h-3" />
                                                </button>

                                                {inst.config.is_default ? null : (
                                                    <button
                                                        onClick={() => handleDelete(inst.config.id)}
                                                        disabled={inst.is_running}
                                                        className="p-1.5 rounded-lg text-gray-500 dark:text-slate-400 hover:text-rose-600 dark:hover:text-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40 border border-gray-200/70 dark:border-[#15334d] transition-colors cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
                                                        title="Delete profile"
                                                    >
                                                        <Trash2 className="w-3 h-3" />
                                                    </button>
                                                )}
                                            </div>
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
                            className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-4 text-xs font-medium focus:ring-2 focus:ring-indigo-500/30"
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
        </div>
        </div>
    );
}
