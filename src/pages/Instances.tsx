import { useState, useEffect } from 'react';
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
} from 'lucide-react';
import { Gemini } from '@lobehub/icons';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../stores/useInstanceStore';
import { useAccountStore } from '../stores/useAccountStore';
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
        closeInstance,
        setActiveInstance,
        smartRotateProfileAccount,
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
    const [copyTargetId, setCopyTargetId] = useState<string | null>(null);
    const [copyInstanceName, setCopyInstanceName] = useState('');
    const [editTargetId, setEditTargetId] = useState<string | null>(null);
    const [editInstanceName, setEditInstanceName] = useState('');
    const [actionError, setActionError] = useState<string | null>(null);

    useEffect(() => {
        if (!isTauri()) return;
        fetchInstances();
        fetchSwitcherStatus();
        fetchAccounts();
        const timer = setInterval(() => {
            fetchInstances(true);
            fetchSwitcherStatus();
        }, 3000);
        return () => clearInterval(timer);
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
            const created = await createInstance(newInstanceName.trim());
            await setActiveInstance(created.id);
            setNewInstanceName('');
            setIsCreateOpen(false);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to create instance');
        }
    };

    const handleCopy = async () => {
        if (!copyTargetId || !copyInstanceName.trim()) return;
        setActionError(null);
        try {
            const copied = await copyInstance(copyTargetId, copyInstanceName.trim());
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
            alert(`Executable cloned successfully:\n${cloned}`);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to clone executable');
        }
    };

    return (
        <div className="h-full w-full overflow-y-auto">
            <div className="max-w-[1920px] mx-auto px-4 sm:px-6 lg:px-8 pt-2 pb-4 space-y-3">
                {/* Header */}
                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
                    <div>
                        <div className="flex items-center gap-2.5">
                            <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
                                <Laptop className="w-5 h-5" />
                            </div>
                            <div>
                                <h1 className="text-lg sm:text-xl font-bold text-gray-900 dark:text-base-content">
                                    {t('instances.page_title', 'Instances & Profiles')}
                                </h1>
                                <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                    {t('instances.page_desc', 'Run multiple Antigravity windows in parallel with isolated credentials and extensions')}
                                </p>
                            </div>
                        </div>
                    </div>

                <div className="flex items-center gap-3 w-full sm:w-auto">
                    <div className="px-3 py-1.5 rounded-lg bg-gray-100 dark:bg-base-200 text-xs text-gray-600 dark:text-gray-300 font-medium">
                        {t('instances.running_summary', 'Running: {{running}} / {{total}}', {
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
                <div className="p-3.5 rounded-xl bg-gradient-to-r from-blue-50/80 to-indigo-50/50 dark:from-blue-900/20 dark:to-indigo-900/10 border border-blue-200/60 dark:border-blue-800/40 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 text-xs">
                    <div className="flex items-center gap-2.5">
                        <span className="relative flex h-2.5 w-2.5">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-blue-500"></span>
                        </span>
                        <div>
                            <span className="font-semibold text-gray-900 dark:text-gray-100">
                                {t('instances.auto_switcher_active', 'Auto Profile Switcher Active')}
                            </span>
                            <span className="text-gray-500 dark:text-gray-400 ml-2">
                                {switcherStatus.current_quota_percent !== undefined && switcherStatus.current_quota_percent !== null
                                    ? `Current Active Quota: ${switcherStatus.current_quota_percent.toFixed(0)}%`
                                    : 'Monitoring active profile quota'}
                                {switcherStatus.last_switch_reason ? ` • Last switch: ${switcherStatus.last_switch_reason}` : ''}
                            </span>
                        </div>
                    </div>
                    <button
                        onClick={async () => {
                            try {
                                const msg = await triggerManualRotation();
                                alert(msg);
                            } catch (e: any) {
                                setActionError(e?.toString() || 'Rotation failed');
                            }
                        }}
                        className="btn btn-xs btn-outline btn-primary gap-1 shrink-0"
                    >
                        <RotateCcw className="w-3 h-3" />
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
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
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
                                className={cn(
                                    "rounded-2xl border transition-all flex flex-col justify-between bg-white dark:bg-base-200 overflow-hidden shadow-xs",
                                    isActive
                                        ? "border-blue-500 shadow-md ring-2 ring-blue-500/20"
                                        : "border-gray-200/80 dark:border-base-100 hover:border-gray-300 dark:hover:border-base-content/20"
                                )}
                            >
                                {/* Top Accent Bar identifying profile color */}
                                <div className={cn("h-1.5 w-full bg-gradient-to-r", theme.accentBar)} />

                                <div className="p-6 flex flex-col flex-1 justify-between min-h-[280px]">
                                    {/* Card Top */}
                                    <div>
                                        <div className="flex items-start justify-between gap-2 mb-3">
                                            <div className="flex items-center gap-2 min-w-0">
                                                <span
                                                    className={cn(
                                                        "w-2.5 h-2.5 rounded-full shrink-0",
                                                        inst.is_running ? "bg-emerald-500 shadow-xs shadow-emerald-500/50 animate-pulse" : "bg-gray-300 dark:bg-gray-600"
                                                    )}
                                                />
                                                <span className="px-1.5 py-0.5 rounded text-[11px] font-black bg-blue-500/10 text-blue-600 dark:text-blue-400 border border-blue-500/20 shrink-0">
                                                    #{seqNumber}
                                                </span>
                                                <h3 className="font-bold text-sm text-gray-900 dark:text-base-content truncate" title={inst.config.name}>
                                                    {inst.config.name}
                                                </h3>
                                                {inst.config.is_default ? (
                                                    <span className="px-1.5 py-0.5 rounded text-[10px] font-bold bg-gray-100 dark:bg-base-100 text-gray-600 dark:text-gray-400 shrink-0">
                                                        DEFAULT
                                                    </span>
                                                ) : null}
                                            </div>
                                            <div className="shrink-0 flex items-center gap-1">
                                                {isActive ? (
                                                    <span className="px-2 py-0.5 rounded-md text-[10px] font-semibold bg-blue-100 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300">
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
                                                    className="btn btn-ghost btn-xs p-1 text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-base-100 cursor-pointer"
                                                    title="Clone profile settings and extensions"
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

                                        {/* Bound Account / Email Section with distinct color pill */}
                                        <div className="py-2 px-3 rounded-xl bg-gray-50/80 dark:bg-base-100/60 border border-gray-100 dark:border-base-100/80 mb-3 flex items-center justify-between gap-2">
                                            <div className="flex items-center gap-1.5 min-w-0 flex-1">
                                                <Mail className="w-3.5 h-3.5 text-gray-400 shrink-0" />
                                                <span className="text-[11px] text-gray-500 dark:text-gray-400 font-medium shrink-0">
                                                    Account:
                                                </span>
                                                {displayEmail ? (
                                                    <span
                                                        className={cn(
                                                            "px-2 py-0.5 rounded-md text-xs font-semibold font-mono border flex items-center gap-1.5 truncate max-w-[210px] shadow-2xs",
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
                                                    <span className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-gray-100 dark:bg-white/10 text-gray-600 dark:text-gray-400 text-[9px] font-bold shadow-xs border border-gray-200 dark:border-white/10 shrink-0">
                                                        <Circle className="w-2.5 h-2.5" />
                                                        FREE
                                                    </span>
                                                );
                                            })() : null}
                                        </div>

                                        {/* Colorful Gemini Quota Progress Bar Section */}
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
                                                    <div className="p-3 rounded-xl bg-gradient-to-br from-gray-50/90 to-blue-50/20 dark:from-base-100/60 dark:to-blue-950/10 border border-gray-200/70 dark:border-base-100 space-y-2">
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
                                                                <span className={cn("font-mono font-bold text-xs px-1.5 py-0.5 rounded-md bg-white dark:bg-base-200 shadow-2xs border border-gray-100 dark:border-base-100", getTextClass(pct))}>
                                                                    {pct}%
                                                                </span>
                                                            </div>
                                                        </div>

                                                        {/* Progress bar with glowing animated gradient */}
                                                        <div className="h-2.5 w-full bg-gray-200/90 dark:bg-base-200 rounded-full overflow-hidden p-0.5 relative shadow-inner">
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
                                                <div className="p-2.5 rounded-xl bg-gray-50/60 dark:bg-base-100/40 border border-gray-100 dark:border-base-100 flex items-center justify-between text-xs text-gray-500">
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
                                                <div className="p-2.5 rounded-xl bg-gray-50/40 dark:bg-base-100/20 border border-dashed border-gray-200 dark:border-base-100 flex items-center justify-between text-xs text-gray-400">
                                                    <div className="flex items-center gap-2">
                                                        <Gemini.Color className="w-4 h-4 shrink-0 opacity-40 grayscale" />
                                                        <span className="text-[11px] italic">Gemini Quota (No profile bound)</span>
                                                    </div>
                                                    <span className="text-[10px] text-gray-400/80">Launch to assign</span>
                                                </div>
                                            )}
                                        </div>

                                        {/* Status and Profile details */}
                                        <div className="space-y-1.5 text-xs py-2 border-t border-gray-100 dark:border-base-100 text-gray-600 dark:text-gray-400">
                                            <div className="flex justify-between items-center">
                                                <span className="text-gray-400">Status:</span>
                                                <span className={cn("font-medium", inst.is_running ? "text-emerald-600 dark:text-emerald-400 font-semibold" : "text-gray-500")}>
                                                    {inst.is_running ? `Running (PID: ${inst.pid})` : 'Idle'}
                                                </span>
                                            </div>
                                            <div className="flex justify-between items-center">
                                                <span className="text-gray-400">Profile ID:</span>
                                                <span className="font-mono text-[11px] text-gray-500 truncate max-w-[170px]">
                                                    {inst.config.id}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1.5 text-[11px] text-gray-400 truncate pt-0.5" title={inst.config.data_dir}>
                                                <Folder className="w-3.5 h-3.5 shrink-0" />
                                                <span className="truncate">{inst.config.data_dir}</span>
                                            </div>
                                            {inst.config.executable_path ? (
                                                <div className="flex items-center gap-1.5 text-[11px] text-purple-600 dark:text-purple-400 truncate pt-0.5" title={inst.config.executable_path}>
                                                    <Cpu className="w-3.5 h-3.5 shrink-0" />
                                                    <span className="truncate font-mono">EXE: {inst.config.executable_path}</span>
                                                </div>
                                            ) : null}
                                        </div>
                                    </div>

                                    {/* Card Actions */}
                                    <div className="pt-3 border-t border-gray-100 dark:border-base-100 flex items-center justify-between gap-1.5 mt-2">
                                        <div className="flex items-center gap-1.5 flex-wrap">
                                            {inst.is_running ? (
                                                <button
                                                    onClick={() => closeInstance(inst.config.id)}
                                                    className="btn btn-xs btn-error btn-outline gap-1 cursor-pointer"
                                                    title="Gracefully close this instance window"
                                                >
                                                    <Square className="w-3 h-3" />
                                                    <span>Close</span>
                                                </button>
                                            ) : (
                                                <button
                                                    onClick={() => handleLaunch(inst.config.id)}
                                                    className="btn btn-xs btn-primary gap-1 shadow-xs cursor-pointer"
                                                    title="Launch instance window"
                                                >
                                                    <Play className="w-3 h-3" />
                                                    <span>Launch</span>
                                                </button>
                                            )}
                                            {/* Smart Switch Button with Process Teardown & Candidate Refill Runway */}
                                            <button
                                                onClick={async () => {
                                                    try {
                                                        const result = await smartRotateProfileAccount(inst.config.id);
                                                        const runwayInfo = result.daysUntilRefill > 0 ? ` (${result.daysUntilRefill}d refill runway)` : '';
                                                        const resumeInfo = (result.resumedProjectsCount ?? 0) > 0
                                                            ? ` · Auto-resumed ${result.resumedProjectsCount} active project(s) (<1h)`
                                                            : '';
                                                        showToast(
                                                            `Closed process & switched ${result.instanceName} to ${result.accountEmail}${runwayInfo}${resumeInfo}`,
                                                            'success'
                                                        );
                                                    } catch (e: any) {
                                                        setActionError(e?.toString() || 'Smart switch failed');
                                                    }
                                                }}
                                                className="btn btn-xs btn-primary btn-outline gap-1 cursor-pointer"
                                                title="Smart Switch: Close process, pick account with longest refill runway, and switch"
                                            >
                                                <FastForward className="w-3 h-3" />
                                                <span>Smart Switch</span>
                                            </button>
                                            <button
                                                onClick={() => handleCloneExecutable(inst.config.id)}
                                                className="btn btn-xs btn-ghost text-purple-600 dark:text-purple-400 cursor-pointer"
                                                title="Clone executable binary for this profile"
                                            >
                                                <Cpu className="w-3.5 h-3.5" />
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
                                            <button
                                                onClick={() => handleWipeSession(inst.config.id)}
                                                disabled={inst.is_running}
                                                className="btn btn-xs btn-ghost text-amber-600 dark:text-amber-400 cursor-pointer disabled:opacity-30"
                                                title="Wipe auth credentials (keep settings)"
                                            >
                                                <RotateCcw className="w-3.5 h-3.5" />
                                            </button>
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
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100">
                        <div className="flex items-center gap-2.5 mb-4">
                            <Laptop className="w-5 h-5 text-blue-600" />
                            <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                {t('instances.create_modal_title', 'Create New Profile')}
                            </h3>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                            Creates an isolated Antigravity profile folder with its own SQLite token store, extensions, and configuration.
                        </p>
                        <input
                            type="text"
                            placeholder={t('instances.name_placeholder', 'Profile name (e.g., Personal, Client Work)')}
                            value={newInstanceName}
                            onChange={(e) => setNewInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                            className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-5 text-sm"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2.5">
                            <button
                                onClick={() => setIsCreateOpen(false)}
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

            {/* Copy Modal */}
            {copyTargetId && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100">
                        <div className="flex items-center gap-2.5 mb-4">
                            <Copy className="w-5 h-5 text-indigo-600" />
                            <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                {t('instances.copy_modal_title', 'Duplicate Profile')}
                            </h3>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
                            Copies extensions, preferences, and editor settings into a new isolated profile.
                        </p>
                        <input
                            type="text"
                            placeholder={t('instances.copy_placeholder', 'New profile name')}
                            value={copyInstanceName}
                            onChange={(e) => setCopyInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCopy()}
                            className="input w-full bg-gray-50 dark:bg-base-100 border border-gray-200 dark:border-base-100 rounded-xl mb-5 text-sm"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2.5">
                            <button
                                onClick={() => setCopyTargetId(null)}
                                className="btn btn-ghost btn-sm text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleCopy}
                                disabled={!copyInstanceName.trim()}
                                className="btn btn-primary btn-sm"
                            >
                                {t('instances.duplicate', 'Duplicate')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Edit Modal */}
            {editTargetId && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-50 p-4">
                    <div className="bg-white dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-100 dark:border-base-100">
                        <div className="flex items-center gap-2.5 mb-4">
                            <Pencil className="w-5 h-5 text-blue-600" />
                            <h3 className="font-bold text-base text-gray-900 dark:text-base-content">
                                {t('instances.edit_modal_title', 'Rename Profile')}
                            </h3>
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
