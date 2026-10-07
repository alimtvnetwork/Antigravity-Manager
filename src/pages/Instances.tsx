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
    Sparkles,
    Star,
    ToggleLeft,
    ToggleRight,
    ArrowRightLeft,
    ArrowLeftRight,
    KeyRound,
    X,
    SlidersHorizontal,
    LayoutGrid,
    Grid3X3,
    List,
    Layers,
    History,
    Check,
    FolderSync,
    Sliders,
    MoreHorizontal,
} from 'lucide-react';
import { Gemini } from '@lobehub/icons';
import { useTranslation } from 'react-i18next';
import { TierBadge } from '../components/common/TierBadge';
import { SELECTED_CARD_CLASSES, ACTIVE_PILL_CLASSES } from '../components/common/selectedState';
import { useInstanceStore } from '../stores/useInstanceStore';
import { useAccountStore } from '../stores/useAccountStore';
import { useConfigStore } from '../stores/useConfigStore';
import {
    type InstanceStatus,
    getInstanceCardDensity,
    setInstanceCardDensity,
    getAutoSwitcherDaemonStatus,
    type AutoSwitcherDaemonStatus,
    rankSmartCandidates,
} from '../services/instanceService';
import { invoke } from '@tauri-apps/api/core';
import { InstanceSettingsModal } from '../components/instances/InstanceSettingsModal';
import InstanceTable, { type InstanceActionType } from '../components/instances/InstanceTable';
import PromptTreeViewModal, { type AgmProjectTreeNode } from '../components/instances/PromptTreeViewModal';
import InstanceAuditTrailModal from '../components/instances/InstanceAuditTrailModal';
import ModalDialog from '../components/common/ModalDialog';
import { findQuotaModel } from '../config/modelConfig';
import { isTauri } from '../utils/env';
import { cn } from '../utils/cn';
import { showToast } from '../components/common/ToastContainer';
import { QuotaProgressBar } from '../components/accounts/QuotaProgressBar';

function truncatePath(fullPath?: string | null): string {
    if (!fullPath) return '';
    const isWindows = fullPath.includes('\\') || /^[a-zA-Z]:/.test(fullPath);
    const sep = isWindows ? '\\' : '/';
    const parts = fullPath.split(/[\\/]/).filter(Boolean);
    if (parts.length === 0) return fullPath;
    return `...${sep}${parts[parts.length - 1]}`;
}

function getActionLabel(action: InstanceActionType): string {
    switch (action) {
        case 'launch':
            return 'Launching...';
        case 'stop':
            return 'Stopping...';
        case 'restart':
            return 'Restarting...';
        case 'switch':
            return 'Switching...';
        case 'fast-forward':
            return 'Rotating...';
        case 'sync':
            return 'Syncing...';
        case 'wipe':
            return 'Wiping...';
        case 'delete':
            return 'Deleting...';
        default:
            return 'Processing...';
    }
}

function isDefaultOwned(nodeInstId: string | undefined | null, cfgId: string): boolean {
    if (!nodeInstId || nodeInstId.trim() === '') {
        return false;
    }
    const isDefaultAlias = nodeInstId === 'default' || nodeInstId === '__default__';
    const isIdMatch = Boolean(cfgId) && nodeInstId === cfgId;
    return isDefaultAlias || isIdMatch;
}

export const isNodeOwnedByInstance = (
    node: AgmProjectTreeNode,
    instConfig: { id: string; is_default?: boolean; seq_num?: number }
): boolean => {
    if (!node.instance_id || node.instance_id.trim() === '') {
        return false;
    }
    const isDefaultInstance = instConfig.is_default || instConfig.id === 'default';
    if (isDefaultInstance) {
        return isDefaultOwned(node.instance_id, instConfig.id);
    }
    // Non-default instance never owns default nodes
    if (node.instance_id === 'default' || node.instance_id === '__default__') {
        return false;
    }
    if (node.instance_id === instConfig.id) {
        return true;
    }
    const hasSeqNum = typeof instConfig.seq_num === 'number' && instConfig.seq_num > 1;
    if (hasSeqNum && node.instance_seq_num === instConfig.seq_num) {
        return true;
    }
    return false;
};

// Data-ink exception (see ui-instance-identity-hues.md): per-profile identity
// hues are functional color coding (like chart series), not theme-ink, so the
// 7 literal ramps below are intentionally kept off daisyUI semantics.
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
        name: 'Cyan',
        accentBar: 'from-cyan-500 via-sky-500 to-blue-500',
        badge: 'bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 border-cyan-400/30',
        emailPill: 'bg-cyan-50 dark:bg-cyan-950/40 text-cyan-700 dark:text-cyan-300 border-cyan-200 dark:border-cyan-800/60',
        dot: 'bg-cyan-500',
    },
    {
        name: 'Amber',
        accentBar: 'from-amber-500 via-yellow-500 to-orange-500',
        badge: 'bg-amber-500/10 text-amber-700 dark:text-amber-300 border-amber-400/30',
        emailPill: 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800/60',
        dot: 'bg-amber-500',
    },
    {
        name: 'Sky',
        accentBar: 'from-sky-500 via-blue-500 to-cyan-600',
        badge: 'bg-sky-500/10 text-sky-700 dark:text-sky-300 border-sky-400/30',
        emailPill: 'bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 border-sky-200 dark:border-sky-800/60',
        dot: 'bg-sky-500',
    },
    {
        name: 'Teal',
        accentBar: 'from-teal-500 via-emerald-500 to-teal-600',
        badge: 'bg-teal-500/10 text-teal-700 dark:text-teal-300 border-teal-400/30',
        emailPill: 'bg-teal-50 dark:bg-teal-950/40 text-teal-700 dark:text-teal-300 border-teal-200 dark:border-teal-800/60',
        dot: 'bg-teal-500',
    },
    {
        name: 'Slate',
        accentBar: 'from-slate-500 via-gray-500 to-slate-600',
        badge: 'bg-slate-500/10 text-slate-700 dark:text-slate-300 border-slate-400/30',
        emailPill: 'bg-slate-100 dark:bg-slate-900/60 text-slate-700 dark:text-slate-300 border-slate-300 dark:border-slate-800',
        dot: 'bg-slate-500',
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
        restartInstance,
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
    const [deletingId, setDeletingId] = useState<string | null>(null);
    const [isSyncingAll, setIsSyncingAll] = useState(false);
    const [syncingInstanceIds, setSyncingInstanceIds] = useState<Record<string, boolean>>({});
    const [auditModalInstance, setAuditModalInstance] = useState<{ id: string; name: string; sequence_name?: string } | null>(null);
    const [actionState, setActionState] = useState<Record<string, InstanceActionType>>({});
    const [deleteModalTarget, setDeleteModalTarget] = useState<InstanceStatus | null>(null);
    const [wipeModalTarget, setWipeModalTarget] = useState<InstanceStatus | null>(null);
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
                setDeleteModalTarget(null);
                setWipeModalTarget(null);
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);

    }, []);

    const { config } = useConfigStore();
    const [runningTreeNodes, setRunningTreeNodes] = useState<AgmProjectTreeNode[]>([]);
    const [projectTreeNodes, setProjectTreeNodes] = useState<AgmProjectTreeNode[]>([]);
    const [daemonStatus, setDaemonStatus] = useState<AutoSwitcherDaemonStatus | null>(null);
    const [daemonCountdown, setDaemonCountdown] = useState<number>(0);

    const fetchRunningTasks = async () => {
        try {
            const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                maxWords: 50,
                onlyRunning: false,
                force: false,
            });
            if (Array.isArray(data)) {
                setProjectTreeNodes(data);
                const running = data.filter(
                    (node) => Boolean(node.is_running)
                );
                setRunningTreeNodes(running);
            }
        } catch {
            // Silently ignore background polling error
        }
    };

    const fetchDaemonTelemetry = async () => {
        try {
            const status = await getAutoSwitcherDaemonStatus();
            setDaemonStatus(status);
            if (status.next_check_in_seconds !== undefined) {
                setDaemonCountdown(Math.max(0, status.next_check_in_seconds));
            }
        } catch {
            // Silently ignore
        }
    };

    useEffect(() => {
        const ticker = setInterval(() => {
            setDaemonCountdown((prev) => (prev > 0 ? prev - 1 : 0));
        }, 1000);
        return () => clearInterval(ticker);
    }, []);

    useEffect(() => {
        if (!isTauri()) return;
        fetchInstances();
        fetchSwitcherStatus();
        fetchAccounts();
        fetchRunningTasks();
        fetchDaemonTelemetry();
        const timer = setInterval(() => {
            fetchInstances(true);
            fetchSwitcherStatus();
            fetchRunningTasks();
            fetchDaemonTelemetry();
        }, 3000);

        let unlistenList: (() => void)[] = [];
        import('@tauri-apps/api/event').then(({ listen }) => {
            const events = [
                'account://auto-switched',
                'instance://switched',
                'instance://rotated',
                'prompt://dispatched',
                'prompt://resumed',
            ];
            Promise.all([
                ...events.map((ev) =>
                    listen(ev, async () => {
                        await Promise.all([
                            fetchInstances(true),
                            fetchSwitcherStatus(),
                            fetchAccounts(),
                            fetchRunningTasks(),
                            fetchDaemonTelemetry(),
                        ]);
                    })
                ),
                listen<AutoSwitcherDaemonStatus>('auto-switcher://status-tick', (event) => {
                    if (event.payload) {
                        setDaemonStatus(event.payload);
                        if (event.payload.next_check_in_seconds !== undefined) {
                            setDaemonCountdown(Math.max(0, event.payload.next_check_in_seconds));
                        }
                    }
                }),
            ]).then((unsubscribers) => {
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
    const [cardDensity, setCardDensity] = useState<'normal' | 'compact'>(() => getInstanceCardDensity());
    const [promptTreeInstance, setPromptTreeInstance] = useState<{
        id: string;
        name: string;
        projectId?: string;
        seqNum?: number;
        executablePath?: string;
    } | null>(null);
    const [cardMoreId, setCardMoreId] = useState<string | null>(null);

    useEffect(() => {
        const handleClickOutside = () => setCardMoreId(null);
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const handleSetViewMode = (mode: 'card' | 'list') => {
        setViewMode(mode);
        localStorage.setItem('agm_instance_view_mode', mode);
    };

    const handleSetCardDensity = (density: 'normal' | 'compact') => {
        setCardDensity(density);
        setInstanceCardDensity(density);
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

    const handleDelete = (id: string) => {
        const target = instances.find((i) => i.config.id === id);
        if (target) {
            if (target.is_running) {
                showToast('Cannot delete an actively running instance. Please stop it first.', 'warning');
                return;
            }
            setDeleteModalTarget(target);
        }
    };

    const handleConfirmDelete = async () => {
        if (!deleteModalTarget) return;
        if (deleteModalTarget.is_running) {
            showToast('Cannot delete an actively running instance. Please stop it first.', 'warning');
            setDeleteModalTarget(null);
            return;
        }
        const targetId = deleteModalTarget.config.id;
        setActionState((prev) => ({ ...prev, [targetId]: 'delete' }));
        setDeletingId(targetId);
        setActionError(null);
        try {
            await deleteInstance(targetId);
            showToast(t('instances.deleted_toast', 'Profile deleted successfully'), 'success');
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to delete instance');
        } finally {
            setDeletingId(null);
            setActionState((prev) => ({ ...prev, [targetId]: null }));
            setDeleteModalTarget(null);
        }
    };

    const handleWipeSession = (id: string) => {
        const target = instances.find((i) => i.config.id === id);
        if (target) {
            setWipeModalTarget(target);
        }
    };

    const handleConfirmWipe = async () => {
        if (!wipeModalTarget) return;
        const targetId = wipeModalTarget.config.id;
        setActionState((prev) => ({ ...prev, [targetId]: 'wipe' }));
        setActionError(null);
        try {
            await wipeSession(targetId);
            showToast(`Session tokens wiped for '${wipeModalTarget.config.name}'`, 'success');
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to wipe session');
        } finally {
            setActionState((prev) => ({ ...prev, [targetId]: null }));
            setWipeModalTarget(null);
        }
    };

    const handleLaunch = async (id: string) => {
        if (actionState[id]) return;
        setActionState((prev) => ({ ...prev, [id]: 'launch' }));
        setActionError(null);
        try {
            await launchInstance(id);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to launch instance');
        } finally {
            setActionState((prev) => ({ ...prev, [id]: null }));
        }
    };

    const handleStop = async (id: string) => {
        if (actionState[id]) return;
        setActionState((prev) => ({ ...prev, [id]: 'stop' }));
        setActionError(null);
        try {
            await stopInstance(id);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to stop instance');
        } finally {
            setActionState((prev) => ({ ...prev, [id]: null }));
        }
    };

    const handleRestart = async (id: string) => {
        if (actionState[id]) return;
        setActionState((prev) => ({ ...prev, [id]: 'restart' }));
        setActionError(null);
        try {
            await restartInstance(id);
            showToast('Instance restarted successfully on current account', 'success');
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to restart instance');
            showToast(e?.toString() || 'Failed to restart instance', 'error');
        } finally {
            setActionState((prev) => ({ ...prev, [id]: null }));
        }
    };

    const handleFastForward = async (id: string) => {
        if (actionState[id]) return;
        setActionState((prev) => ({ ...prev, [id]: 'fast-forward' }));
        setActionError(null);
        try {
            const msg = await fastForwardInstance(id);
            showToast(msg || 'Rotated to next best profile!', 'success');
        } catch (e: any) {
            setActionError(e?.toString() || 'Fast forward failed');
        } finally {
            setActionState((prev) => ({ ...prev, [id]: null }));
        }
    };

    const handleSync = async (id: string) => {
        if (actionState[id]) return;
        setActionState((prev) => ({ ...prev, [id]: 'sync' }));
        setSyncingInstanceIds((prev) => ({ ...prev, [id]: true }));
        setActionError(null);
        try {
            await syncInstance(id);
        } catch (e: any) {
            setActionError(e?.toString() || 'Failed to sync instance');
        } finally {
            setSyncingInstanceIds((prev) => ({ ...prev, [id]: false }));
            setActionState((prev) => ({ ...prev, [id]: null }));
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
                        <div className="p-2 rounded-xl bg-primary/10 text-primary shrink-0">
                            <Laptop className="w-5 h-5" />
                        </div>
                        <div className="min-w-0">
                            <h1 className="text-lg sm:text-xl font-bold text-base-content">
                                {t('instances.page_title', 'Instances & Profiles')}
                            </h1>
                            <p className="text-xs text-base-content/60 mt-0.5">
                                {t('instances.page_desc', 'Run multiple Antigravity windows in parallel with isolated credentials and extensions')}
                            </p>
                        </div>
                    </div>

                <div className="flex flex-wrap items-center justify-between gap-3">
                    {/* Segmented Group 1: Status & Maintenance */}
                    <div className="flex items-center rounded-[4px] bg-base-200 border border-base-300/80 dark:border-base-300 p-0.5 divide-x divide-base-300 shadow-2xs">
                        <div className="px-3 py-1 text-xs text-base-content/80 font-semibold whitespace-nowrap rounded-l-[4px] rounded-r-none">
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
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-base-content/80 dark:text-base-content hover:bg-base-300 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title={t('common.refresh', 'Refresh')}
                        >
                            <RotateCw className={cn("w-3.5 h-3.5 text-primary", isLoading && "motion-safe:animate-spin")} />
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
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-base-content/80 dark:text-base-content hover:bg-base-300 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title="Synchronize process PIDs and account quotas across all instances"
                        >
                            <FolderSync className={cn("w-3.5 h-3.5 text-info", isSyncingAll && "motion-safe:animate-spin")} />
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
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-warning hover:bg-warning/10 rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title="Force-terminate lingering background Electron/Antigravity processes, purge lockfiles, and cleanly relaunch Antigravity"
                        >
                            <Sparkles className="w-3.5 h-3.5" />
                            <span>Clean & Restart</span>
                        </button>
                    </div>


                    {/* Segmented Group 2: Automation & Settings */}
                    <div className="flex items-center rounded-[4px] bg-base-200 border border-base-300/80 dark:border-base-300 p-0.5 divide-x divide-base-300 shadow-2xs">
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
                                "flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                switcherStatus?.is_running
                                    ? "text-success hover:bg-success/10"
                                    : "text-base-content/70 dark:text-base-content/60 hover:bg-base-300"
                            )}
                            title="Toggle background auto-profile switcher daemon"
                        >
                            {switcherStatus?.is_running ? (
                                <ToggleRight className="w-4 h-4 text-success" />
                            ) : (
                                <ToggleLeft className="w-4 h-4 text-base-content/50" />
                            )}
                            <span>Auto-Switch: {switcherStatus?.is_running ? 'ON' : 'OFF'}</span>
                        </button>
                        {switcherStatus?.is_running && (
                            <div
                                className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-base-content/80 rounded-none"
                                title={`Auto-Switcher daemon next check in ${daemonCountdown}s (${daemonStatus?.current_stage || 'Normal'} stage, interval ${daemonStatus?.check_interval_seconds || 60}s)`}
                            >
                                <span className="relative flex h-2 w-2 shrink-0">
                                    <span className="motion-safe:animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-2 w-2 bg-success shadow-xs shadow-success/50"></span>
                                </span>
                                <span className="font-mono text-[11px] whitespace-nowrap">
                                    ⏱ {daemonCountdown}s Next Check
                                </span>
                            </div>
                        )}
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
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-base-content/80 dark:text-base-content hover:bg-base-300 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title="Evaluate rolling quota across all monitored instances and auto-rotate any low quota accounts"
                        >
                            <Sparkles className={cn("w-3.5 h-3.5 text-warning", isLoading && "motion-safe:animate-spin")} />
                            <span>Eval Quota</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => {
                                setSettingsModalTarget(null);
                                setIsSettingsModalOpen(true);
                            }}
                            className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-base-content/80 dark:text-base-content hover:bg-base-300 rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                            title="Instance Settings & Sync: Turbo mode, plan review, copy settings, folder sync, JSON tools"
                        >
                            <SlidersHorizontal className="w-3.5 h-3.5 text-primary" />
                            <span>Settings & Sync</span>
                        </button>
                    </div>

                    {/* Segmented Group 3: View Mode & Creation */}
                    <div className="flex items-center gap-2">
                        {/* Density Switcher Capsule (when viewMode === 'card') */}
                        {viewMode === 'card' && (
                            <div className="flex items-center rounded-[4px] bg-base-200 border border-base-300/80 dark:border-base-300 p-0.5 divide-x divide-base-300 shadow-2xs">
                                <button
                                    type="button"
                                    onClick={() => handleSetCardDensity('normal')}
                                    className={cn(
                                        "flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                        cardDensity === 'normal'
                                            ? "bg-base-100 dark:bg-base-300 text-primary shadow-xs"
                                            : "text-base-content/70 dark:text-base-content/60 hover:text-base-content dark:hover:text-base-content"
                                    )}
                                    title="Normal Cards"
                                >
                                    <LayoutGrid className="w-3.5 h-3.5" />
                                    <span className="hidden sm:inline">Normal Cards</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => handleSetCardDensity('compact')}
                                    className={cn(
                                        "flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                        cardDensity === 'compact'
                                            ? "bg-base-100 dark:bg-base-300 text-primary shadow-xs"
                                            : "text-base-content/70 dark:text-base-content/60 hover:text-base-content dark:hover:text-base-content"
                                    )}
                                    title="Compact Cards"
                                >
                                    <Grid3X3 className="w-3.5 h-3.5" />
                                    <span className="hidden sm:inline">Compact Cards</span>
                                </button>
                            </div>
                        )}

                        {/* View Switcher Capsule */}
                        <div className="flex items-center rounded-[4px] bg-base-200 border border-base-300/80 dark:border-base-300 p-0.5 divide-x divide-base-300 shadow-2xs">
                            <button
                                type="button"
                                onClick={() => handleSetViewMode('card')}
                                className={cn(
                                    "flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-l-[4px] rounded-r-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                    viewMode === 'card'
                                        ? "bg-base-100 dark:bg-base-300 text-primary shadow-xs"
                                        : "text-base-content/70 dark:text-base-content/60 hover:text-base-content dark:hover:text-base-content"
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
                                    "flex items-center gap-1.5 px-2.5 py-1 text-xs font-semibold rounded-r-[4px] rounded-l-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                    viewMode === 'list'
                                        ? "bg-base-100 dark:bg-base-300 text-primary shadow-xs"
                                        : "text-base-content/70 dark:text-base-content/60 hover:text-base-content dark:hover:text-base-content"
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
                            className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-[4px] bg-primary hover:bg-[var(--primary-hover)] text-primary-content text-xs font-semibold shadow-xs cursor-pointer transition-all duration-150 ease-out active:scale-[0.98]"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            <span>{t('instances.create_btn', 'New Instance')}</span>
                        </button>
                    </div>
                </div>
            </div>

            {/* Store Error Alert */}
            {error ? (
                <div className="p-3.5 rounded-[5px] bg-error/10 border border-error/30 text-error text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{error}</span>
                    </div>
                    <button
                        onClick={() => fetchInstances()}
                        className="btn btn-xs btn-outline btn-error gap-1 shrink-0 rounded-[5px]"
                    >
                        <RotateCw className="w-3 h-3" />
                        <span>{t('common.retry', 'Retry')}</span>
                    </button>
                </div>
            ) : null}

            {/* Action Error Alert */}
            {actionError ? (
                <div className="p-3.5 rounded-[5px] bg-error/10 border border-error/30 text-error text-xs flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <AlertCircle className="w-4 h-4 shrink-0" />
                        <span>{actionError}</span>
                    </div>
                    <button onClick={() => setActionError(null)} className="text-xs font-semibold">✕</button>
                </div>
            ) : null}

            {/* Auto-Switcher Status Banner */}
            {switcherStatus?.is_running ? (
                <div className="bg-success/10 border border-success/30 rounded-[5px] p-4 shadow-sm flex flex-col md:flex-row items-start md:items-center justify-between gap-3 text-xs">
                    <div className="flex items-center gap-3 min-w-0">
                        <span className="relative flex h-3 w-3 shrink-0">
                            <span className="motion-safe:animate-ping absolute inline-flex h-full w-full rounded-full bg-success opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-3 w-3 bg-success shadow-xs shadow-success/50"></span>
                        </span>
                        <div className="flex flex-wrap items-center gap-2 min-w-0">
                            <span className="font-bold text-base-content flex items-center gap-1.5">
                                <Sparkles className="w-3.5 h-3.5 text-success" />
                                {t('instances.auto_switcher_active', 'Auto Profile Switcher Active')}
                            </span>
                            <span className="inline-flex items-center px-2.5 py-0.5 rounded-[5px] text-[11px] font-semibold font-mono bg-success/15 text-success border border-success/30 shadow-2xs">
                                Current Active Quota: {switcherStatus.current_quota_percent !== undefined && switcherStatus.current_quota_percent !== null
                                    ? `${switcherStatus.current_quota_percent.toFixed(0)}%`
                                    : '100%'}
                            </span>
                            {switcherStatus.last_switch_reason ? (
                                <span className="inline-flex items-center px-2.5 py-0.5 rounded-[5px] text-[11px] font-medium bg-base-300/10 text-base-content/70 dark:text-base-content/80 border border-base-300/20 max-w-xs truncate" title={switcherStatus.last_switch_reason}>
                                    Reason: {switcherStatus.last_switch_reason}
                                </span>
                            ) : null}
                        </div>
                    </div>
                    {(() => {
                        const activeInstance = instances.find(i => i.config.id === (activeInstanceId || 'default')) || instances.find(i => i.config.is_default) || instances[0];
                        const inUseAccountIds = instances.map(i => i.config.bound_account_id).filter(Boolean) as string[];
                        const rankedCandidates = rankSmartCandidates(accounts, inUseAccountIds, activeInstance?.config.bound_account_id);
                        const nextBestCandidate = rankedCandidates[0]?.account || null;
                        const rotateTooltip = `Target: ${activeInstance?.config.name || 'Current'} → Next Best: ${nextBestCandidate ? `${nextBestCandidate.email} (${nextBestCandidate.quota?.subscription_tier || 'PRO'} · 4H: ${nextBestCandidate.quota?.models?.[0]?.percentage ?? 100}%)` : 'No idle candidate available'}`;
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
                                className="flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold rounded-[5px] bg-primary hover:bg-[var(--primary-hover)] text-primary-content shadow-xs cursor-pointer transition-colors active:scale-95 shrink-0"
                                title={rotateTooltip}
                            >
                                <FastForward className="w-3.5 h-3.5" />
                                <span>{t('instances.rotate_next_best', 'Rotate to Next Best')}</span>
                                {nextBestCandidate && (
                                    <span className="px-1.5 py-0.5 rounded-[4px] bg-primary-content/25 text-[10px] font-mono font-normal max-w-[110px] truncate">
                                        {nextBestCandidate.email.split('@')[0]}
                                    </span>
                                )}
                            </button>
                        );
                    })()}
                </div>
            ) : null}

            {/* Search Filter & Quick Action */}
            <div className="flex items-center justify-between gap-3 flex-wrap">
                <div className="flex items-center gap-2 flex-1 max-w-md bg-base-100 dark:bg-base-200 border border-base-300 rounded-[5px] px-3 py-2 shadow-xs">
                    <Search className="w-4 h-4 text-base-content/50 shrink-0" />
                    <input
                        type="text"
                        placeholder={t('instances.search_placeholder', 'Search by profile name, ID, or bound email...')}
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full bg-transparent border-none outline-hidden text-xs text-base-content"
                    />
                </div>
                <button
                    onClick={() => {
                        setNewInstanceName('');
                        setIsCreateOpen(true);
                    }}
                    className="btn btn-primary btn-sm gap-1.5 shadow-sm rounded-[4px] transition-all duration-150 ease-out active:scale-[0.98]"
                >
                    <Plus className="w-4 h-4" />
                    <span>{t('instances.create_btn', 'New Instance')}</span>
                </button>
            </div>

            {/* Instance Cards Grid or Empty / Loading States */}
            {isLoading && instances.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-20 bg-base-100 dark:bg-base-200 rounded-[5px] border border-base-300/80 dark:border-base-300">
                    <RotateCw className="w-8 h-8 text-primary motion-safe:animate-spin mb-3" />
                    <p className="text-sm font-medium text-base-content/80">
                        {t('instances.loading', 'Loading instances and profiles...')}
                    </p>
                </div>
            ) : instances.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-16 px-4 text-center bg-base-100 dark:bg-base-200 rounded-[5px] border border-dashed border-base-300">
                    <div className="p-4 rounded-[5px] bg-primary/10 text-primary mb-4">
                        <Laptop className="w-10 h-10" />
                    </div>
                    <h3 className="text-base font-bold text-base-content mb-1">
                        {t('instances.empty_title', 'No Profiles Found')}
                    </h3>
                    <p className="text-xs text-base-content/60 max-w-md mb-6 leading-relaxed">
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
                <div className="flex flex-col items-center justify-center py-12 text-center bg-base-100 dark:bg-base-200 rounded-[5px] border border-base-300/80 dark:border-base-300">
                    <Search className="w-8 h-8 text-base-content/50 mb-2" />
                    <p className="text-sm font-medium text-base-content/80">
                        {t('instances.no_search_results', 'No profiles match your search')}
                    </p>
                    <p className="text-xs text-base-content/50 mt-1">
                        {t('instances.no_search_results_desc', 'Try searching with a different name, profile ID, or email')}
                    </p>
                </div>
            ) : viewMode === 'list' ? (
                <InstanceTable
                    instances={filteredInstances}
                    activeInstanceId={activeInstanceId}
                    searchQuery={searchQuery}
                    actionState={actionState}
                    onLaunch={handleLaunch}
                    onStop={handleStop}
                    onRestart={handleRestart}
                    onSwitch={(id) => {
                        const target = instances.find((i) => i.config.id === id);
                        if (target) setSwitchTargetInstance(target);
                    }}
                    onFastForward={handleFastForward}
                    onAudit={(id, name) => {
                        const target = instances.find((i) => i.config.id === id);
                        setAuditModalInstance({
                            id,
                            name,
                            sequence_name: target?.config.seq_num ? `Instance #${target.config.seq_num}` : undefined
                        });
                    }}
                    onSync={handleSync}
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
                        setPromptTreeInstance({
                            id,
                            name: target?.config.name || id,
                            seqNum: target?.config.seq_num,
                            executablePath: target?.config.executable_path,
                        });
                    }}
                    onSetActive={setActiveInstance}
                    onSetDefault={setDefaultInstance}
                />
            ) : (

                <div className={cn(
                    cardDensity === 'compact'
                        ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2.5"
                        : "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3"
                )}>
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

                        const weeklyBucket = (boundAccount?.quota?.quota_groups || [])
                            .find((item) => {
                                const name = (item.display_name || '').toLowerCase();
                                return name.includes('gemini') || (!name.includes('claude') && !name.includes('gpt'));
                            })
                            ?.buckets?.find((item) =>
                                (item.window || '').toLowerCase().includes('week') ||
                                (item.bucket_id || '').toLowerCase().includes('week')
                            );

                        const weeklyQuota = weeklyBucket ? {
                            percentage: Math.round((weeklyBucket.remaining_fraction || 0) * 100),
                            resetTime: weeklyBucket.reset_time,
                        } : ((boundAccount?.quota as any)?.weekly || null);

                        const hasActiveTask = Boolean(inst.is_running) && runningTreeNodes.some((node) => {
                            const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
                            const isNodeRunning = Boolean(node.is_running);
                            return isInstanceMatch && isNodeRunning;
                        });

                        // Resolve effective executable path for display
                        const defaultExePath =
                            (config as any)?.antigravity_ide_executable ||
                            (config as any)?.antigravity_executable ||
                            'Antigravity IDE (Default System Path)';

                        const effectiveExePath =
                            inst.config.executable_path || (inst.config.is_default ? defaultExePath : null);

                        const currentAction = actionState[inst.config.id] || null;
                        const isBusy = Boolean(currentAction);

                        return (
                            <div
                                key={inst.config.id}
                                ref={isActive ? activeCardRef : undefined}
                                className={cn(
                                    "group relative rounded-[5px] border transition-all duration-200 flex flex-col justify-between overflow-hidden backdrop-blur-xs",
                                    isActive
                                        ? cn(SELECTED_CARD_CLASSES, "border-warning/30 shadow-md")
                                        : "bg-base-100 dark:bg-base-200 border-base-300/50 dark:border-base-300/60 hover:border-base-300/80 dark:hover:border-primary/40 hover:bg-base-200/90 dark:hover:bg-base-100 shadow-xs"
                                )}
                            >
                                {/* Card Mutex Overlay when action is executing */}
                                {isBusy && (
                                    <div className="absolute inset-0 bg-white/75 dark:bg-black/70 backdrop-blur-[2px] z-30 flex flex-col items-center justify-center gap-2 rounded-[5px] pointer-events-auto cursor-wait select-none">
                                        <RotateCw className="w-5 h-5 motion-safe:animate-spin text-primary" />
                                        <span className="text-xs font-bold text-base-content tracking-wide font-mono">
                                            {getActionLabel(currentAction)}
                                        </span>
                                    </div>
                                )}

                                <div className={cn(
                                    "flex flex-col flex-1 justify-between min-w-0",
                                    cardDensity === 'compact' ? "p-2.5" : "p-3.5"
                                )}>
                                    {/* Card Top */}
                                    <div className="min-w-0">
                                        <div className="flex items-center justify-between gap-2 mb-2.5 h-6 flex-nowrap min-w-0">
                                            <div className="flex items-center gap-1.5 min-w-0 flex-1 flex-nowrap overflow-hidden">
                                                <span
                                                    className={cn(
                                                        "w-2.5 h-2.5 rounded-full shrink-0",
                                                        inst.is_running ? "bg-success shadow-xs shadow-success/50 motion-safe:animate-pulse" : "bg-base-300 dark:bg-base-200"
                                                    )}
                                                />
                                                <span className="px-1.5 py-0.5 rounded-[5px] text-xs font-black bg-primary/15 text-primary border border-primary/25 shrink-0">
                                                    #{seqNumber}
                                                </span>
                                                <h3 className={cn("font-bold text-xs truncate shrink min-w-0", isActive ? "text-primary" : "text-base-content")} title={inst.config.name}>
                                                    {inst.config.name}
                                                </h3>
                                                {hasActiveTask && (
                                                    <button
                                                        type="button"
                                                        onClick={() => setPromptTreeInstance({ id: inst.config.id, name: inst.config.name, seqNum: inst.config.seq_num, executablePath: inst.config.executable_path })}
                                                        className="inline-flex items-center gap-1 px-1.5 h-5 rounded-[5px] text-[9px] font-bold bg-info/15 hover:bg-info/25 text-info border border-info/30 shrink-0 cursor-pointer transition-colors shadow-2xs"
                                                        title="Active prompt/task running - Click to open Prompt Tree"
                                                    >
                                                        <span className="w-1.5 h-1.5 rounded-full bg-info motion-safe:animate-pulse shrink-0" />
                                                        <span>Prompt</span>
                                                    </button>
                                                )}
                                                {inst.config.is_default ? (
                                                    <span className="h-5 px-1.5 rounded-[5px] text-[9px] font-bold bg-accent/15 text-accent border border-accent/30 flex items-center justify-center shrink-0">
                                                        DEFAULT
                                                    </span>
                                                ) : (
                                                    <button
                                                        disabled={isBusy}
                                                        onClick={async () => {
                                                            try {
                                                                await setDefaultInstance(inst.config.id);
                                                                showToast(t('instances.set_default_toast', 'Default profile updated successfully'), 'success');
                                                            } catch (e: any) {
                                                                setActionError(e?.toString() || 'Failed to set default profile');
                                                            }
                                                        }}
                                                        className="h-5 px-1.5 rounded-[5px] text-[9px] font-medium text-base-content/50 hover:text-warning hover:bg-warning/10 border border-dashed border-base-300 transition-colors cursor-pointer flex items-center gap-1 shrink-0 disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Set as default profile"
                                                    >
                                                        <Star className="w-2.5 h-2.5" />
                                                        <span>Set Default</span>
                                                    </button>
                                                )}
                                            </div>
                                            <div className="shrink-0 flex items-center gap-1">
                                                {isActive ? (
                                                    <span className={cn(ACTIVE_PILL_CLASSES, "text-[9px] px-1.5 py-0.2 tracking-wider")}>
                                                        Active
                                                    </span>
                                                ) : (
                                                    <button
                                                        disabled={isBusy}
                                                        onClick={() => setActiveInstance(inst.config.id)}
                                                        className="text-[10px] text-base-content/60 hover:text-primary transition-colors font-medium mr-0.5 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                        title="Set as active instance for account switches"
                                                    >
                                                        Set Active
                                                    </button>
                                                )}
                                                <button
                                                    disabled={isBusy}
                                                    onClick={() => {
                                                        setEditTargetId(inst.config.id);
                                                        setEditInstanceName(inst.config.name);
                                                    }}
                                                    className="p-1 rounded-[5px] text-primary hover:bg-primary/10 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                    title={t('instances.edit_title', 'Rename profile')}
                                                >
                                                    <Pencil className="w-3 h-3" />
                                                </button>
                                                <button
                                                    disabled={isBusy}
                                                    onClick={() => {
                                                        setCopyTargetId(inst.config.id);
                                                        setCopyInstanceName(`${inst.config.name} Copy`);
                                                        setCopyProjects(true);
                                                    }}
                                                    className="p-1 rounded-[5px] text-accent hover:bg-accent/10 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                    title="Clone / Duplicate profile settings and extensions"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                </button>
                                                {!inst.config.is_default && (
                                                    <button
                                                        onClick={() => handleDelete(inst.config.id)}
                                                        disabled={inst.is_running || isBusy || deletingId === inst.config.id}
                                                        className="p-1 rounded-[5px] text-error hover:bg-error/10 disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer"
                                                        title="Delete profile"
                                                    >
                                                        {currentAction === 'delete' ? (
                                                            <RotateCw className="w-3 h-3 motion-safe:animate-spin text-error" />
                                                        ) : (
                                                            <Trash2 className="w-3 h-3" />
                                                        )}
                                                    </button>
                                                )}
                                            </div>
                                        </div>

                                        {/* Bound Account / Email Section: Compact py-1 px-2.5 rounded-md */}
                                        <div className={cn(
                                            "py-1 px-2.5 rounded-md bg-base-200/80 dark:bg-base-200/90 border border-base-300 mb-2.5 flex items-center justify-between gap-1.5 transition-all duration-200",
                                            "group-hover:border-warning/30 dark:group-hover:bg-base-100"
                                        )}>
                                            <div className="flex items-center gap-1.5 min-w-0 flex-1">
                                                <Mail className="w-3 h-3 text-base-content/50 group-hover:text-warning transition-colors shrink-0" />
                                                <span className="text-[10px] text-base-content/60 font-medium shrink-0">
                                                    Account:
                                                </span>
                                                {displayEmail ? (
                                                    <span
                                                        className={cn(
                                                            "px-1.5 py-0.5 rounded-md text-[11px] font-semibold font-mono border flex items-center gap-1 min-w-0 shadow-2xs transition-colors",
                                                            theme.emailPill,
                                                            "group-hover:text-warning"
                                                        )}
                                                        title={displayEmail}
                                                    >
                                                        <span className={cn("w-1.5 h-1.5 rounded-full shrink-0", theme.dot)} />
                                                        <span className="truncate">{displayEmail}</span>
                                                    </span>
                                                ) : (
                                                    <span className="text-[11px] text-base-content/50 italic">
                                                        Unassigned
                                                    </span>
                                                )}
                                            </div>
                                            {boundAccount && (
                                                <TierBadge
                                                    tier={boundAccount.quota?.subscription_tier}
                                                    size="xs"
                                                    className="shrink-0"
                                                />
                                            )}
                                        </div>

                                        {/* Colorful Gemini Quota & Weekly Progress Bar */}
                                        <div className="mb-2.5">
                                            {(geminiModel || weeklyQuota) ? (
                                                <div className="p-2.5 rounded-md bg-base-200/90 border border-base-300/70 dark:border-base-300 space-y-2">
                                                    {geminiModel && (
                                                        <div className="space-y-1">
                                                            <div className="flex items-center justify-between text-xs">
                                                                <div className="flex items-center gap-1.5 min-w-0">
                                                                    <Gemini.Color className="w-3.5 h-3.5 shrink-0" />
                                                                    <span className="font-semibold text-base-content text-xs truncate">
                                                                        {geminiModel.display_name || geminiModel.name || 'Gemini 3.1 Pro'} (4H)
                                                                    </span>
                                                                </div>
                                                            </div>
                                                            <QuotaProgressBar
                                                                percentage={geminiModel.percentage}
                                                                resetTime={geminiModel.reset_time}
                                                            />
                                                        </div>
                                                    )}
                                                    {weeklyQuota && (
                                                        <div className={cn("space-y-1", geminiModel && "pt-1.5 border-t border-base-300/60")}>
                                                            <div className="flex items-center justify-between text-xs">
                                                                <span className="text-[11px] font-medium text-base-content/70 dark:text-base-content/80">
                                                                    Weekly Quota
                                                                </span>
                                                            </div>
                                                            <QuotaProgressBar
                                                                isWeekly
                                                                percentage={weeklyQuota.percentage}
                                                                resetTime={weeklyQuota.resetTime}
                                                            />
                                                        </div>
                                                    )}
                                                </div>
                                            ) : boundAccount ? (
                                                <div className="p-2 rounded-md bg-base-200/60 dark:bg-base-200/90 border border-base-300 flex items-center justify-between text-xs text-base-content/60">
                                                    <div className="flex items-center gap-1.5">
                                                        <Gemini.Color className="w-3.5 h-3.5 shrink-0 opacity-70" />
                                                        <span className="text-[11px] text-base-content/60">Quota not synced</span>
                                                    </div>
                                                    <button
                                                        onClick={() => refreshQuota(boundAccount.id)}
                                                        className="text-primary hover:underline gap-1 text-[11px] cursor-pointer flex items-center"
                                                    >
                                                        <ArrowLeftRight className="w-3 h-3 text-primary" />
                                                        <span>Sync</span>
                                                    </button>
                                                </div>
                                            ) : (
                                                <div className="p-2 rounded-md bg-base-200/40 dark:bg-base-200/60 border border-dashed border-base-300 flex items-center justify-between text-xs text-base-content/50">
                                                    <div className="flex items-center gap-1.5">
                                                        <Gemini.Color className="w-3.5 h-3.5 shrink-0 opacity-40 grayscale" />
                                                        <span className="text-[10px] italic">No profile bound</span>
                                                    </div>
                                                    <span className="text-[10px] text-base-content/50">Launch to assign</span>
                                                </div>
                                            )}
                                        </div>

                                        {/* Status and Profile details: Compact badge layout */}
                                        <div className="space-y-1.5 py-2 border-t border-base-300 dark:border-base-300/80 text-xs">
                                            <div className="flex items-center justify-between gap-1.5 flex-wrap">
                                                {/* Status Badge */}
                                                {inst.is_running ? (
                                                    <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-[5px] text-[10px] font-semibold bg-success/10 text-success border border-success/20 shadow-2xs">
                                                        <span className="w-1.5 h-1.5 rounded-full bg-success motion-safe:animate-pulse" />
                                                        Running {inst.pid ? `(${inst.pid})` : ''}
                                                    </span>
                                                ) : (
                                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-medium bg-base-300/10 text-base-content/70 dark:text-base-content/60 border border-base-300/20">
                                                        <span className="w-1.5 h-1.5 rounded-full bg-base-300" />
                                                        Idle
                                                    </span>
                                                )}

                                                {/* Profile ID Monospace Pill */}
                                                <span
                                                    className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono font-medium bg-base-200 text-base-content/70 dark:text-base-content/80 border border-base-300 truncate max-w-[120px]"
                                                    title={`Profile ID: ${inst.config.id}`}
                                                >
                                                    <span className="text-base-content/50">ID:</span>
                                                    <span className="truncate">{inst.config.id}</span>
                                                </span>
                                            </div>

                                            {/* Path with clean folder badge and copy action */}
                                            <div className="flex items-center justify-between gap-1.5 px-2 py-1 rounded-[5px] bg-base-200/80 dark:bg-base-200/70 border border-base-300/70 dark:border-base-300 text-[10px] group transition-colors hover:border-base-300 dark:hover:border-primary/30">
                                                <div className="flex items-center gap-1 min-w-0 flex-1 text-base-content/60" title={inst.config.data_dir}>
                                                    <Folder className="w-3 h-3 shrink-0 text-primary/80" />
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
                                                    className="opacity-60 group-hover:opacity-100 p-0.5 rounded-[5px] hover:bg-base-300 text-base-content/60 dark:text-base-content/80 transition-opacity cursor-pointer shrink-0"
                                                    title="Copy directory path"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                </button>
                                            </div>

                                            {/* Executable path badge (renders for custom instances AND default instance) */}
                                            {effectiveExePath ? (
                                                <div className="flex items-center justify-between gap-1.5 px-2 py-1 rounded-[5px] bg-base-200/70 dark:bg-base-200/60 border border-base-300/80 dark:border-base-300 text-[10px] group/exe transition-colors">
                                                    <div className="flex items-center gap-1 min-w-0 flex-1 text-base-content/80" title={effectiveExePath}>
                                                        <Cpu className="w-3 h-3 shrink-0 text-info" />
                                                        <span className="truncate font-mono" title={effectiveExePath}>{truncatePath(effectiveExePath)}</span>
                                                    </div>
                                                    <button
                                                        type="button"
                                                        onClick={async (e) => {
                                                            e.stopPropagation();
                                                            try {
                                                                await navigator.clipboard.writeText(effectiveExePath);
                                                                showToast('Executable path copied to clipboard', 'info');
                                                            } catch {
                                                                showToast('Failed to copy executable path', 'error');
                                                            }
                                                        }}
                                                        className="opacity-60 group-hover/exe:opacity-100 p-0.5 rounded-[5px] hover:bg-base-300 text-base-content/60 dark:text-base-content/80 transition-opacity cursor-pointer shrink-0"
                                                        title="Copy executable path"
                                                    >
                                                        <Copy className="w-3 h-3" />
                                                    </button>
                                                </div>
                                            ) : null}
                                        </div>

                                        {/* Active / Recent Projects Section (hidden in compact density) */}
                                        {cardDensity !== 'compact' && (() => {
                                            const instanceProjects = projectTreeNodes.filter((node) => isNodeOwnedByInstance(node, inst.config));

                                            const sortedProjects = [...instanceProjects].sort((a, b) => {
                                                const isARunning = Boolean(inst.is_running) && Boolean(a.is_running);
                                                const isBRunning = Boolean(inst.is_running) && Boolean(b.is_running);
                                                if (isARunning) {
                                                    if (!isBRunning) return -1;
                                                }
                                                if (isBRunning) {
                                                    if (!isARunning) return 1;
                                                }
                                                const aLatest = Math.max(0, ...(a.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
                                                const bLatest = Math.max(0, ...(b.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
                                                if (bLatest !== aLatest) return bLatest - aLatest;
                                                return a.repo_name.localeCompare(b.repo_name);
                                            });

                                            const maxP = config?.instance_card_max_projects || 3;
                                            const displayedProjects = sortedProjects.slice(0, maxP);

                                            return (
                                                <div className="mt-2 p-2 rounded-[5px] bg-base-200/80 border border-base-300/70 dark:border-base-300 text-xs">
                                                    <div className="flex items-center justify-between text-[11px] font-semibold text-base-content/80 mb-1.5 pb-1 border-b border-base-300/60">
                                                        <span className="flex items-center gap-1">
                                                            <Folder className="w-3 h-3 text-primary" />
                                                            <span>Active / Recent Projects</span>
                                                        </span>
                                                        <span className="px-1.5 py-0.2 rounded-[4px] text-[10px] font-mono bg-base-300/70 dark:bg-base-300 text-base-content/70 dark:text-base-content/80">
                                                            {instanceProjects.length}
                                                        </span>
                                                    </div>

                                                    {displayedProjects.length > 0 ? (
                                                        <div className="space-y-1">
                                                            {displayedProjects.map((proj) => {
                                                                const isProjRunning = Boolean(inst.is_running) && Boolean(proj.is_running);
                                                                const totalTurns = proj.conversations?.reduce((sum, c) => sum + Math.max(c.step_count || 1, 1), 0) || 0;

                                                                return (
                                                                    <div
                                                                        key={proj.project_id}
                                                                        onDoubleClick={() => setPromptTreeInstance({
                                                                            id: inst.config.id,
                                                                            name: inst.config.name,
                                                                            projectId: proj.project_id,
                                                                            seqNum: inst.config.seq_num,
                                                                            executablePath: inst.config.executable_path,
                                                                        })}
                                                                        className="flex items-center justify-between gap-1.5 px-1.5 py-1 rounded-[5px] bg-base-100 dark:bg-base-200 hover:bg-primary/10 dark:hover:bg-base-300 border border-base-300/50 dark:border-base-300/60 transition-colors cursor-pointer group/proj"
                                                                        title="Double-click to open in Prompt Tree"
                                                                    >
                                                                        <div className="flex items-center gap-1.5 min-w-0 flex-1">
                                                                            <Folder className="w-3 h-3 shrink-0 text-base-content/50 group-hover/proj:text-primary transition-colors" />
                                                                            <span className="truncate font-medium text-[11px] text-base-content" title={proj.repo_name}>
                                                                                {proj.repo_name}
                                                                            </span>
                                                                        </div>
                                                                        <div className="flex items-center gap-1 shrink-0">
                                                                            {isProjRunning && (
                                                                                <span className="px-1 py-0.2 rounded-[4px] text-[9px] font-bold bg-info/15 text-info border border-info/30 flex items-center gap-0.5">
                                                                                    <span className="w-1 h-1 rounded-full bg-info motion-safe:animate-pulse" />
                                                                                    RUNNING
                                                                                </span>
                                                                            )}
                                                                            <span className="px-1.5 py-0.2 rounded-[4px] text-[9px] font-mono text-base-content/60 bg-base-200 border border-base-300/50 dark:border-base-300">
                                                                                {totalTurns} turns
                                                                            </span>
                                                                        </div>
                                                                    </div>
                                                                );
                                                            })}
                                                        </div>
                                                    ) : (
                                                        <div className="text-[10px] text-base-content/50 italic py-1 text-center">
                                                            No recent projects
                                                        </div>
                                                    )}
                                                </div>
                                            );
                                        })()}
                                    </div>

                                    {/* Card Actions Toolbar: 2 clean structured rows with 5-6px radius */}
                                    <div className="pt-2.5 border-t border-base-300 dark:border-base-300/80 mt-2 space-y-1.5">
                                        {/* Row 1: Primary Actions Capsule (Launch/Stop, Switch Account, Fast-Forward, Sync PID) */}
                                        <div className="flex items-center rounded-[5px] overflow-hidden bg-base-200 border border-base-300/80 dark:border-base-300 p-0.5 divide-x divide-base-300 shadow-2xs w-full">
                                            {inst.is_running ? (
                                                <div className="flex-1 flex items-center divide-x divide-base-300">
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => handleStop(inst.config.id)}
                                                        className="flex-1 flex items-center justify-center py-1 text-error hover:bg-error/10 rounded-l-[4px] transition-colors cursor-pointer disabled:opacity-50"
                                                        title="Stop Instance"
                                                    >
                                                        {currentAction === 'stop' ? (
                                                            <RotateCw className="w-3.5 h-3.5 motion-safe:animate-spin text-error" />
                                                        ) : (
                                                            <Square className="w-3.5 h-3.5 fill-current" />
                                                        )}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        disabled={isBusy}
                                                        onClick={() => handleRestart(inst.config.id)}
                                                        className="flex-1 flex items-center justify-center py-1 text-warning hover:bg-warning/10 transition-colors cursor-pointer disabled:opacity-50"
                                                        title="Restart Instance on Current Account"
                                                    >
                                                        {currentAction === 'restart' ? (
                                                            <RotateCw className="w-3.5 h-3.5 motion-safe:animate-spin text-warning" />
                                                        ) : (
                                                            <RotateCcw className="w-3.5 h-3.5" />
                                                        )}
                                                    </button>
                                                </div>
                                            ) : (
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={() => handleLaunch(inst.config.id)}
                                                    className="flex-1 flex items-center justify-center py-1 text-info hover:bg-info/10 rounded-l-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                    title="Launch Instance"
                                                >
                                                    {currentAction === 'launch' ? (
                                                        <RotateCw className="w-3.5 h-3.5 motion-safe:animate-spin text-info" />
                                                    ) : (
                                                        <Play className="w-3.5 h-3.5 fill-current" />
                                                    )}
                                                </button>
                                            )}

                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => setSwitchTargetInstance(inst)}
                                                className="flex-1 flex items-center justify-center py-1 text-info hover:bg-base-300 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                title="Switch Account"
                                            >
                                                {currentAction === 'switch' ? (
                                                    <RotateCw className="w-3.5 h-3.5 motion-safe:animate-spin text-info" />
                                                ) : (
                                                    <ArrowRightLeft className="w-3.5 h-3.5" />
                                                )}
                                            </button>

                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => handleFastForward(inst.config.id)}
                                                className="flex-1 flex items-center justify-center py-1 text-warning hover:bg-base-300 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                title="Fast Forward to Next Best"
                                            >
                                                {currentAction === 'fast-forward' ? (
                                                    <RotateCw className="w-3.5 h-3.5 motion-safe:animate-spin text-warning" />
                                                ) : (
                                                    <FastForward className="w-3.5 h-3.5" />
                                                )}
                                            </button>

                                            <button
                                                type="button"
                                                disabled={isBusy || Boolean(syncingInstanceIds[inst.config.id])}
                                                onClick={() => handleSync(inst.config.id)}
                                                className="flex-1 flex items-center justify-center py-1 text-info hover:bg-base-300 rounded-r-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                title="Sync PID and Quota"
                                            >
                                                <Cpu className={cn("w-3.5 h-3.5 text-info", (currentAction === 'sync' || syncingInstanceIds[inst.config.id]) && "motion-safe:animate-pulse")} />
                                            </button>
                                        </div>

                                        {/* Row 2: Secondary Actions Capsule (Prompts Tree, Settings, Clone Profile, Audit, More Popover) */}
                                        <div className="flex items-center rounded-[5px] overflow-hidden bg-base-200 border border-base-300/80 dark:border-base-300 p-0.5 divide-x divide-base-300 shadow-2xs w-full relative">
                                            {/* Slot 1: Prompts Tree */}
                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => setPromptTreeInstance({ id: inst.config.id, name: inst.config.name, seqNum: inst.config.seq_num, executablePath: inst.config.executable_path })}
                                                className={cn(
                                                    "flex-1 flex items-center justify-center py-1 transition-colors cursor-pointer relative rounded-l-[4px] disabled:opacity-50 disabled:cursor-not-allowed",
                                                    hasActiveTask
                                                        ? "bg-info/10 text-info hover:bg-info/10"
                                                        : "text-base-content/70 dark:text-base-content/60 hover:text-info hover:bg-base-300"
                                                )}
                                                title={hasActiveTask ? "Prompt Tree (Active Task Running)" : "Prompt Tree"}
                                            >
                                                <Layers className="w-3.5 h-3.5" />
                                                {hasActiveTask && (
                                                    <span className="absolute top-0.5 right-0.5 w-1.5 h-1.5 rounded-full bg-info motion-safe:animate-pulse" />
                                                )}
                                            </button>

                                            {/* Slot 2: Settings */}
                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => {
                                                    setSettingsModalTarget(inst);
                                                    setIsSettingsModalOpen(true);
                                                }}
                                                className="flex-1 flex items-center justify-center py-1 text-base-content/70 dark:text-base-content/60 hover:text-primary hover:bg-base-300 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                title="Settings & Sync"
                                            >
                                                <SlidersHorizontal className="w-3.5 h-3.5" />
                                            </button>

                                            {/* Slot 3: Clone Profile */}
                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => {
                                                    setCopyTargetId(inst.config.id);
                                                    setCopyInstanceName(`${inst.config.name} Copy`);
                                                    setCopyProjects(true);
                                                }}
                                                className="flex-1 flex items-center justify-center py-1 text-base-content/70 dark:text-base-content/60 hover:text-accent hover:bg-base-300 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                title="Clone Profile"
                                            >
                                                <Copy className="w-3.5 h-3.5" />
                                            </button>

                                            {/* Slot 4: Audit Trail */}
                                            <button
                                                type="button"
                                                disabled={isBusy}
                                                onClick={() => setAuditModalInstance({
                                                    id: inst.config.id,
                                                    name: inst.config.name,
                                                    sequence_name: inst.config.seq_num ? `Instance #${inst.config.seq_num}` : undefined
                                                })}
                                                className="flex-1 flex items-center justify-center py-1 text-base-content/70 dark:text-base-content/60 hover:text-warning hover:bg-base-300 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                                title="Audit Trail"
                                            >
                                                <History className="w-3.5 h-3.5" />
                                            </button>

                                            {/* Slot 5: More Popover Trigger */}
                                            <div className="flex-1 relative">
                                                <button
                                                    type="button"
                                                    disabled={isBusy}
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setCardMoreId(cardMoreId === inst.config.id ? null : inst.config.id);
                                                    }}
                                                    className={cn(
                                                        "w-full flex items-center justify-center py-1 rounded-r-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                                                        cardMoreId === inst.config.id
                                                            ? "bg-base-300 text-base-content"
                                                            : "text-base-content/70 dark:text-base-content/60 hover:bg-base-300"
                                                    )}
                                                    title="More Actions (Clone Executable, Wipe, Delete)"
                                                >
                                                    <MoreHorizontal className="w-3.5 h-3.5" />
                                                </button>

                                                {/* More Popover Dropdown */}
                                                {cardMoreId === inst.config.id && (
                                                    <div
                                                        className="absolute right-0 bottom-full mb-1 z-30 min-w-[170px] rounded-[5px] border border-base-300 bg-base-100 dark:bg-base-200 py-1 text-base-content shadow-xl text-xs"
                                                        onClick={(e) => e.stopPropagation()}
                                                    >
                                                        <button
                                                            type="button"
                                                            disabled={isBusy}
                                                            onClick={() => {
                                                                setCardMoreId(null);
                                                                handleCloneExecutable(inst.config.id);
                                                            }}
                                                            className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-base-200 dark:hover:bg-base-300 text-accent cursor-pointer disabled:opacity-50"
                                                        >
                                                            <Cpu className="w-3.5 h-3.5" />
                                                            <span>Clone Executable</span>
                                                        </button>
                                                        <button
                                                            type="button"
                                                            disabled={inst.is_running || isBusy}
                                                            onClick={() => {
                                                                setCardMoreId(null);
                                                                handleWipeSession(inst.config.id);
                                                            }}
                                                            className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-base-200 dark:hover:bg-base-300 text-warning cursor-pointer disabled:opacity-40"
                                                        >
                                                            <KeyRound className="w-3.5 h-3.5 text-warning" />
                                                            <span>Wipe Credentials</span>
                                                        </button>
                                                        {!inst.config.is_default && (
                                                            <button
                                                                type="button"
                                                                disabled={inst.is_running || isBusy || deletingId === inst.config.id}
                                                                onClick={() => {
                                                                    setCardMoreId(null);
                                                                    handleDelete(inst.config.id);
                                                                }}
                                                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-error/10 text-error cursor-pointer disabled:opacity-40"
                                                            >
                                                                <Trash2 className="w-3.5 h-3.5" />
                                                                <span>Delete Profile</span>
                                                            </button>
                                                        )}
                                                    </div>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                {/* Bottom Accent Line with CSS3 Hover Animation */}
                                <div className="relative w-full h-[3px] overflow-hidden rounded-b-[5px]">
                                    <div
                                        className={cn(
                                            "absolute inset-0 bg-gradient-to-r transition-all duration-300 ease-out transform",
                                            theme.accentBar,
                                            isActive
                                                ? "opacity-60 scale-x-100 group-hover:opacity-100"
                                                : "opacity-0 scale-x-95 group-hover:opacity-100 group-hover:scale-x-100"
                                        )}
                                    />
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
                        className="bg-base-100 dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-base-300"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="flex items-center justify-between pb-3 border-b border-base-300 mb-4">
                            <div className="flex items-center gap-2.5">
                                <Laptop className="w-5 h-5 text-primary" />
                                <h3 className="font-bold text-base text-base-content">
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
                                className="btn btn-ghost btn-xs btn-circle text-base-content/50 hover:text-base-content/70 dark:hover:text-base-content/80 cursor-pointer"
                                title={t('common.close', 'Close')}
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>
                        <p className="text-xs text-base-content/60 mb-4">
                            Creates an isolated Antigravity profile folder with its own SQLite token store, extensions, and configuration.
                        </p>
                        <label className="block text-xs font-semibold text-base-content/80 mb-1.5">
                            Profile Name
                        </label>
                        <input
                            type="text"
                            placeholder={t('instances.name_placeholder', 'Profile name (e.g., Personal, Client Work)')}
                            value={newInstanceName}
                            onChange={(e) => setNewInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                            className="input w-full bg-base-200 dark:bg-base-100 border border-base-300 rounded-xl mb-4 text-sm"
                            autoFocus
                        />
                        <label className="block text-xs font-semibold text-base-content/80 mb-1.5">
                            Initial Account (Optional)
                        </label>
                        <select
                            value={newInstanceBoundAccount}
                            onChange={(e) => setNewInstanceBoundAccount(e.target.value)}
                            className="select select-sm w-full bg-base-200 dark:bg-base-100 border border-base-300 rounded-xl mb-4 text-xs"
                        >
                            <option value="">Auto-assign next available account</option>
                            {accounts.map(acc => (
                                <option key={acc.id} value={acc.id}>
                                    {acc.email} ({acc.quota?.subscription_tier || 'FREE'})
                                </option>
                            ))}
                        </select>

                        <label className="block text-xs font-semibold text-base-content/80 mb-1.5">
                            Clone Settings & Extensions From (Optional)
                        </label>
                        <select
                            value={newInstanceFromInstance}
                            onChange={(e) => setNewInstanceFromInstance(e.target.value)}
                            className="select select-sm w-full bg-base-200 dark:bg-base-100 border border-base-300 rounded-xl mb-4 text-xs"
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
                            <label htmlFor="launch-immediately-check" className="text-xs text-base-content/80 cursor-pointer select-none">
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
                                className="btn btn-ghost btn-sm text-base-content/70 dark:text-base-content/60 rounded-[5px]"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleCreate}
                                disabled={!newInstanceName.trim()}
                                className="btn btn-primary btn-sm rounded-[5px]"
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
                    <div className="bg-base-100 dark:bg-base-200 rounded-2xl p-6 w-full max-w-lg shadow-2xl border border-base-300 max-h-[85vh] flex flex-col">
                        <div className="flex items-center justify-between pb-3 border-b border-base-300">
                            <div className="flex items-center gap-2.5">
                                <ArrowRightLeft className="w-5 h-5 text-primary" />
                                <div>
                                    <h3 className="font-bold text-base text-base-content">
                                        Switch Account for {switchTargetInstance.config.name}
                                    </h3>
                                    <p className="text-xs text-base-content/60 mt-0.5">
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
                                <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-base-content/50" />
                                <input
                                    type="text"
                                    value={accountSearchQuery}
                                    onChange={(e) => setAccountSearchQuery(e.target.value)}
                                    placeholder="Filter accounts by email or ID..."
                                    className="w-full pl-8 pr-2.5 py-1 text-xs bg-base-200 dark:bg-base-100 border border-base-300 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary text-base-content"
                                />
                            </div>
                        </div>

                        <div className="overflow-y-auto py-2 space-y-2 flex-1 my-1 pr-1">
                            {accounts.filter(a => {
                                const q = accountSearchQuery.toLowerCase().trim();
                                if (!q) return true;
                                return a.email.toLowerCase().includes(q) || a.id.toLowerCase().includes(q);
                            }).length === 0 ? (
                                <p className="text-xs text-base-content/50 text-center py-4">No accounts matching filter.</p>
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
                                                    ? "bg-primary/10 border-primary/30"
                                                    : "bg-base-200/60 dark:bg-base-100/60 border-base-300 hover:border-base-300"
                                            )}
                                        >
                                            <div className="min-w-0 flex-1">
                                                <div className="flex items-center gap-2 mb-1">
                                                    <span className="font-mono text-xs font-semibold text-base-content truncate">
                                                        {acc.email}
                                                    </span>
                                                    <span className={cn(
                                                        "text-[9px] font-bold px-1.5 py-0.5 rounded",
                                                        tier.includes('ULTRA') ? "bg-accent/10 text-accent" :
                                                        tier.includes('PRO') ? "bg-primary/10 text-primary" :
                                                        "bg-base-200 text-base-content/70 dark:bg-base-300 dark:text-base-content/80"
                                                    )}>
                                                        {tier}
                                                    </span>
                                                    {isCurrent ? (
                                                        <span className="text-[10px] text-primary font-semibold">
                                                            Active
                                                        </span>
                                                    ) : null}
                                                </div>
                                                <div className="flex items-center gap-2 text-[11px] text-base-content/60">
                                                    <span>Quota: {pct}%</span>
                                                    <div className="w-24 h-1.5 bg-base-300 rounded-full overflow-hidden">
                                                        <div
                                                            className={cn(
                                                                "h-full rounded-full",
                                                                pct >= 50 ? "bg-success" : pct >= 20 ? "bg-warning" : "bg-error"
                                                            )}
                                                            style={{ width: `${pct}%` }}
                                                        />
                                                    </div>
                                                </div>
                                            </div>

                                            <button
                                                disabled={isCurrent || isLoading || Boolean(actionState[switchTargetInstance.config.id])}
                                                onClick={async () => {
                                                    const targetId = switchTargetInstance.config.id;
                                                    setActionState((prev) => ({ ...prev, [targetId]: 'switch' }));
                                                    setActionError(null);
                                                    try {
                                                        await switchAccountToInstance(acc.id, targetId);
                                                        showToast(`Switched ${switchTargetInstance.config.name} to ${acc.email}`, 'success');
                                                        setSwitchTargetInstance(null);
                                                        setAccountSearchQuery('');
                                                    } catch (e: any) {
                                                        setActionError(e?.toString() || 'Failed to switch account');
                                                    } finally {
                                                        setActionState((prev) => ({ ...prev, [targetId]: null }));
                                                    }
                                                }}
                                                className={cn(
                                                    "btn btn-xs rounded-[5px]",
                                                    isCurrent ? "btn-disabled opacity-50" : "btn-primary"
                                                )}
                                            >
                                                {actionState[switchTargetInstance.config.id] === 'switch' ? (
                                                    <span className="flex items-center gap-1">
                                                        <RotateCw className="w-3 h-3 motion-safe:animate-spin" />
                                                        <span>Switching...</span>
                                                    </span>
                                                ) : isCurrent ? 'Current' : 'Select'}
                                            </button>
                                        </div>
                                    );
                                })
                            )}
                        </div>

                        <div className="flex justify-end pt-3 border-t border-base-300">
                            <button
                                onClick={() => {
                                    setSwitchTargetInstance(null);
                                    setAccountSearchQuery('');
                                }}
                                className="btn btn-ghost btn-sm text-base-content/70 dark:text-base-content/60 rounded-[5px]"
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
                    className="fixed inset-0 bg-black/60 backdrop-blur-md flex items-center justify-center z-50 p-4"
                    onClick={() => {
                        setCopyTargetId(null);
                        setCopyInstanceName('');
                    }}
                >
                    <div
                        className="bg-base-100 dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-base-300 ring-1 ring-black/5 dark:ring-white/5 transition-all"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="flex items-center justify-between pb-3.5 border-b border-base-300 mb-4">
                            <div className="flex items-center gap-2.5">
                                <div className="p-2.5 rounded-xl bg-primary/10 text-primary border border-primary/30 shadow-xs">
                                    <Copy className="w-5 h-5" />
                                </div>
                                <div>
                                    <h3 className="font-bold text-sm text-base-content tracking-tight">
                                        {t('instances.copy_modal_title', 'Duplicate / Clone Profile')}
                                    </h3>
                                    <p className="text-[11px] text-base-content/60 mt-0.5">
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
                                className="p-1.5 rounded-lg text-base-content/50 hover:text-base-content/70 dark:text-base-content/60 dark:hover:text-base-content hover:bg-base-200 transition-colors cursor-pointer"
                                title={t('common.close', 'Close')}
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>

                        <label className="block text-xs font-semibold text-base-content/80 dark:text-base-content mb-1.5">
                            {t('instances.copy_name_label', 'New Profile Name')}
                        </label>
                        <input
                            type="text"
                            placeholder={t('instances.copy_placeholder', 'New profile name')}
                            value={copyInstanceName}
                            onChange={(e) => setCopyInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCopy()}
                            className="w-full px-4 py-2.5 bg-base-200 dark:bg-base-100 border border-base-300 text-base-content rounded-xl mb-4 text-xs font-medium focus:outline-none focus:ring-2 focus:ring-primary/40 focus:border-primary transition-all shadow-inner placeholder-base-content/40"
                            autoFocus
                        />

                        {/* Scope Selection Cards */}
                        <div className="mb-4 space-y-2">
                            <span className="block text-[11px] font-bold text-base-content/60 uppercase tracking-wider mb-2">
                                {t('instances.clone_mode_label', 'Duplication Scope / Clone Type')}
                            </span>
                            <div className="grid grid-cols-1 gap-2.5">
                                <div
                                    onClick={() => setCloneMode('full')}
                                    className={cn(
                                        "p-3.5 rounded-xl border-2 transition-all cursor-pointer flex items-start gap-3",
                                        cloneMode === 'full'
                                            ? "border-primary bg-primary/10 ring-2 ring-primary/20 shadow-xs"
                                            : "border-base-300 bg-base-200/50 dark:bg-base-200 hover:bg-base-200/60"
                                    )}
                                >
                                    <div className="mt-0.5 shrink-0">
                                        <div className={cn(
                                            "w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors",
                                            cloneMode === 'full'
                                                ? "border-primary bg-primary"
                                                : "border-base-300 bg-transparent"
                                        )}>
                                            {cloneMode === 'full' && <div className="w-1.5 h-1.5 rounded-full bg-base-100 dark:bg-base-200" />}
                                        </div>
                                    </div>
                                    <div className="min-w-0 flex-1">
                                        <div className="font-bold text-xs text-base-content flex items-center justify-between gap-1.5">
                                            <span className="flex items-center gap-1.5">
                                                <Layers className="w-3.5 h-3.5 text-primary shrink-0" />
                                                <span>{t('instances.clone_mode_full', 'IDE Copy (Full Environment & Sessions)')}</span>
                                            </span>
                                            <span className="px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-info/10 text-info border border-info/30">
                                                Full
                                            </span>
                                        </div>
                                        <p className="text-[11px] text-base-content/70 dark:text-base-content/80 leading-relaxed mt-1">
                                            {t('instances.clone_mode_full_desc', 'Clones complete isolated environment, sessions, extensions, cache, and state.')}
                                        </p>
                                    </div>
                                </div>

                                <div
                                    onClick={() => setCloneMode('profile')}
                                    className={cn(
                                        "p-3.5 rounded-xl border-2 transition-all cursor-pointer flex items-start gap-3",
                                        cloneMode === 'profile'
                                            ? "border-primary bg-primary/10 ring-2 ring-primary/20 shadow-xs"
                                            : "border-base-300 bg-base-200/50 dark:bg-base-200 hover:bg-base-200/60"
                                    )}
                                >
                                    <div className="mt-0.5 shrink-0">
                                        <div className={cn(
                                            "w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors",
                                            cloneMode === 'profile'
                                                ? "border-primary bg-primary"
                                                : "border-base-300 bg-transparent"
                                        )}>
                                            {cloneMode === 'profile' && <div className="w-1.5 h-1.5 rounded-full bg-base-100 dark:bg-base-200" />}
                                        </div>
                                    </div>
                                    <div className="min-w-0 flex-1">
                                        <div className="font-bold text-xs text-base-content flex items-center justify-between gap-1.5">
                                            <span className="flex items-center gap-1.5">
                                                <Sliders className="w-3.5 h-3.5 text-primary shrink-0" />
                                                <span>{t('instances.clone_mode_profile', 'Profile Copy (Preferences & Snippets)')}</span>
                                            </span>
                                            <span className="px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-base-200 text-base-content/80 border border-base-300">
                                                Preferences
                                            </span>
                                        </div>
                                        <p className="text-[11px] text-base-content/70 dark:text-base-content/80 leading-relaxed mt-1">
                                            {t('instances.clone_mode_profile_desc', 'Copies only User preferences, keybindings, and snippets without bulky runtime session state.')}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Copy Workspace Projects & Folders Option */}
                        <div
                            onClick={() => setCopyProjects(!copyProjects)}
                            className={cn(
                                "mb-5 p-3.5 rounded-xl border transition-all flex items-center justify-between gap-3 cursor-pointer",
                                copyProjects
                                    ? "border-primary/70 bg-primary/10 ring-1 ring-primary/20 shadow-xs"
                                    : "border-base-300 bg-base-200/50 dark:bg-base-200 hover:border-base-300"
                            )}
                        >
                            <div className="min-w-0 pr-2">
                                <label
                                    htmlFor="instances-page-copy-projects"
                                    onClick={(e) => e.stopPropagation()}
                                    className="font-bold text-xs text-base-content cursor-pointer flex items-center gap-1.5"
                                >
                                    <FolderSync className="w-3.5 h-3.5 text-success shrink-0" />
                                    <span>{t('instances.copy_projects_label', 'Copy Workspace Projects & Folders')}</span>
                                </label>
                                <p className="text-[11px] text-base-content/70 dark:text-base-content/80 mt-1 leading-relaxed">
                                    {t('instances.copy_projects_desc', 'Duplicate opened workspaces, project states, and recent folder paths into the new profile.')}
                                </p>
                            </div>
                            <div
                                className={cn(
                                    "w-5 h-5 rounded-md border flex items-center justify-center transition-all duration-200 shrink-0 shadow-xs",
                                    copyProjects
                                        ? "border-primary bg-primary text-primary-content"
                                        : "border-base-300 bg-base-100"
                                )}
                            >
                                {copyProjects && <Check className="w-3.5 h-3.5 stroke-[3]" />}
                            </div>
                        </div>

                        <div className="flex justify-end items-center gap-2.5 pt-3 border-t border-base-300">
                            <button
                                type="button"
                                onClick={() => {
                                    setCopyTargetId(null);
                                    setCopyInstanceName('');
                                }}
                                className="px-4 py-2.5 rounded-[5px] text-xs font-semibold text-base-content/70 dark:text-base-content/80 border border-base-300 hover:bg-base-200 transition-all duration-200 active:scale-95 cursor-pointer"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleCopy}
                                disabled={!copyInstanceName.trim()}
                                className="px-5 py-2.5 bg-primary hover:bg-[var(--primary-hover)] text-primary-content font-bold text-xs rounded-[5px] shadow-lg transition-all duration-200 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
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
                        className="bg-base-100 dark:bg-base-200 rounded-2xl p-6 w-full max-w-md shadow-2xl border border-base-300"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="flex items-center justify-between pb-3 border-b border-base-300 mb-4">
                            <div className="flex items-center gap-2.5">
                                <Pencil className="w-5 h-5 text-primary" />
                                <h3 className="font-bold text-base text-base-content">
                                    {t('instances.edit_modal_title', 'Rename Profile')}
                                </h3>
                            </div>
                            <button
                                type="button"
                                onClick={() => {
                                    setEditTargetId(null);
                                    setEditInstanceName('');
                                }}
                                className="btn btn-ghost btn-xs btn-circle text-base-content/50 hover:text-base-content/70 dark:hover:text-base-content/80 cursor-pointer"
                                title={t('common.close', 'Close')}
                            >
                                <X className="w-4 h-4" />
                            </button>
                        </div>
                        <p className="text-xs text-base-content/60 mb-4">
                            {t('instances.edit_modal_desc', 'Update display name for this isolated instance profile.')}
                        </p>
                        <input
                            type="text"
                            placeholder={t('instances.edit_placeholder', 'New profile name')}
                            value={editInstanceName}
                            onChange={(e) => setEditInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleEdit()}
                            className="input w-full bg-base-200 dark:bg-base-100 border border-base-300 rounded-xl mb-5 text-sm"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2.5">
                            <button
                                onClick={() => setEditTargetId(null)}
                                className="btn btn-ghost btn-sm text-base-content/70 dark:text-base-content/60 rounded-[5px]"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                onClick={handleEdit}
                                disabled={!editInstanceName.trim()}
                                className="btn btn-primary btn-sm rounded-[5px]"
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
                onClose={() => {
                    setPromptTreeInstance(null);
                    fetchRunningTasks();
                }}
                instanceId={promptTreeInstance?.id || ''}
                instanceName={promptTreeInstance?.name || ''}
                sequenceNumber={promptTreeInstance?.seqNum}
                executablePath={promptTreeInstance?.executablePath}
                initialSelectedProjectId={promptTreeInstance?.projectId}
            />

            {/* Instance Audit Trail & Switch History Modal */}
            <InstanceAuditTrailModal
                isOpen={Boolean(auditModalInstance)}
                onClose={() => setAuditModalInstance(null)}
                instance={auditModalInstance}
            />

            {/* Bespoke Delete Confirmation Modal */}
            <ModalDialog
                isOpen={Boolean(deleteModalTarget)}
                title={t('instances.delete_title', 'Delete Profile')}
                type="confirm"
                isDestructive={true}
                isLoading={deletingId === deleteModalTarget?.config.id}
                onConfirm={handleConfirmDelete}
                onCancel={() => setDeleteModalTarget(null)}
                confirmText={t('common.delete', 'Delete')}
                cancelText={t('common.cancel', 'Cancel')}
            >
                {deleteModalTarget && (
                    <div className="space-y-3">
                        <div className="p-3 rounded-xl bg-error/10 border border-error/30 text-xs text-error">
                            {t(
                                'instances.delete_warning',
                                'Are you sure you want to delete this profile? This will permanently delete the isolated profile data directory and all settings.'
                            )}
                        </div>
                        <div className="bg-base-200 rounded-xl p-3 border border-base-300/70 dark:border-base-300 space-y-1.5 text-xs">
                            <div className="flex items-center justify-between">
                                <span className="text-base-content/60 font-medium">{t('instances.name', 'Profile Name')}:</span>
                                <span className="font-bold text-base-content flex items-center gap-1.5">
                                    <span className="px-1.5 py-0.5 rounded-[5px] text-[10px] font-black bg-primary/15 text-primary border border-primary/25">
                                        #{deleteModalTarget.config.seq_num ?? 0}
                                    </span>
                                    {deleteModalTarget.config.name}
                                </span>
                            </div>
                            <div className="flex flex-col gap-1 pt-1.5 border-t border-base-300/50 dark:border-base-300/60">
                                <span className="text-base-content/60 font-medium">{t('instances.data_dir', 'Data Directory')}:</span>
                                <span className="font-mono text-[11px] text-base-content/80 break-all bg-base-100 dark:bg-base-200 p-1.5 rounded-lg border border-base-300">
                                    {deleteModalTarget.config.data_dir}
                                </span>
                            </div>
                        </div>
                    </div>
                )}
            </ModalDialog>

            {/* Bespoke Wipe Session Confirmation Modal */}
            <ModalDialog
                isOpen={Boolean(wipeModalTarget)}
                title={t('instances.wipe_title', 'Wipe Credentials')}
                type="confirm"
                isDestructive={true}
                isLoading={actionState[wipeModalTarget?.config.id || ''] === 'wipe'}
                onConfirm={handleConfirmWipe}
                onCancel={() => setWipeModalTarget(null)}
                confirmText={t('instances.wipe_confirm', 'Wipe Credentials')}
                cancelText={t('common.cancel', 'Cancel')}
            >
                {wipeModalTarget && (
                    <div className="space-y-3">
                        <div className="p-3 rounded-xl bg-warning/10 border border-warning/30 text-xs text-warning">
                            {t(
                                'instances.wipe_warning',
                                'This will clear all saved account credentials and session tokens from this profile without deleting its workspace or configurations.'
                            )}
                        </div>
                        <div className="bg-base-200 rounded-xl p-3 border border-base-300/70 dark:border-base-300 space-y-1.5 text-xs">
                            <div className="flex items-center justify-between">
                                <span className="text-base-content/60 font-medium">{t('instances.name', 'Profile Name')}:</span>
                                <span className="font-bold text-base-content flex items-center gap-1.5">
                                    <span className="px-1.5 py-0.5 rounded-[5px] text-[10px] font-black bg-primary/15 text-primary border border-primary/25">
                                        #{wipeModalTarget.config.seq_num ?? 0}
                                    </span>
                                    {wipeModalTarget.config.name}
                                </span>
                            </div>
                            {wipeModalTarget.config.bound_email && (
                                <div className="flex items-center justify-between pt-1.5 border-t border-base-300/50 dark:border-base-300/60">
                                    <span className="text-base-content/60 font-medium">{t('instances.bound_account', 'Bound Account')}:</span>
                                    <span className="font-mono text-[11px] text-warning font-semibold">
                                        {wipeModalTarget.config.bound_email}
                                    </span>
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </ModalDialog>
        </div>
        </div>
    );
}
