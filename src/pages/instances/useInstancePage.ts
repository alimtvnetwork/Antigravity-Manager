import { useState, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { useErrorStore } from '../../stores/error-store';
import { useAccountStore } from '../../stores/useAccountStore';
import { useConfigStore } from '../../stores/useConfigStore';
import {
    type InstanceStatus,
    getInstanceCardDensity,
    setInstanceCardDensity,
    getAutoSwitcherDaemonStatus,
    type AutoSwitcherDaemonStatus,
} from '../../services/instanceService';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../../utils/env';
import { showToast } from '../../components/common/ToastContainer';
import type { AgmProjectTreeNode } from '../../components/instances/PromptTreeViewModal';
import { isGhostConversation } from '../../components/instances/PromptTreeViewModal';
import { isNodeOwnedByInstance } from './instancePageUtils';
import type { InstanceActionType } from './instancePageUtils';
import type { InstancePageApi } from './instancePageTypes';
import { createPageActions } from './instancePageActions';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { useErrorStore } from '../../stores/error-store';

function actionErrorMessage(e: unknown): string {
    return e?.toString?.() || 'Operation failed';
}


export function useInstancePage(): InstancePageApi {
    const { t } = useTranslation();
    const {
        instances,
        activeInstanceId,
        switcherStatus,
        isLoading,
        error: storeError,
        fetchInstances,
        fetchSwitcherStatus,
        fetchRunningTasks,
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

    const { accounts, currentAccount, fetchAccounts, refreshQuota } = useAccountStore();
    const { config } = useConfigStore();

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
    const [auditModalInstance, setAuditModalInstance] = useState<InstancePageApi['auditModalInstance']>(null);
    const [actionState, setActionState] = useState<Record<string, InstanceActionType>>({});
    const [deleteModalTarget, setDeleteModalTarget] = useState<InstanceStatus | null>(null);
    const [wipeModalTarget, setWipeModalTarget] = useState<InstanceStatus | null>(null);
    const activeCardRef = useRef<HTMLDivElement>(null);

    const [viewMode, setViewMode] = useState<'card' | 'list'>(() => {
        return (localStorage.getItem('agm_instance_view_mode') as 'card' | 'list') || 'card';
    });
    const [cardDensity, setCardDensity] = useState<'normal' | 'compact'>(() => getInstanceCardDensity());
    const [promptTreeInstance, setPromptTreeInstance] = useState<InstancePageApi['promptTreeInstance']>(null);
    const [cardMoreId, setCardMoreId] = useState<string | null>(null);

    const [runningTreeNodes, setRunningTreeNodes] = useState<AgmProjectTreeNode[]>([]);
    const [projectTreeNodes, setProjectTreeNodes] = useState<AgmProjectTreeNode[]>([]);
    const [daemonStatus, setDaemonStatus] = useState<AutoSwitcherDaemonStatus | null>(null);
    const [daemonCountdown, setDaemonCountdown] = useState<number>(0);

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

    const fetchRunningTasks = async () => {
        try {
            const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                maxWords: 50,
                onlyRunning: false,
                force: false,
            });
            if (Array.isArray(data)) {
                setProjectTreeNodes(data);
                const running = data.filter((node) => Boolean(node.is_running));
                setRunningTreeNodes(running);
            }
        } catch (e: unknown) {
            useErrorStore.getState().trackWarning(e, { source: 'Instances.fetchRunningTasks', triggerAction: 'get_project_conversation_tree' });
        }
    };

    const fetchDaemonTelemetry = async () => {
        try {
            const status = await getAutoSwitcherDaemonStatus();
            setDaemonStatus(status);
            if (status.next_check_in_seconds !== undefined) {
                setDaemonCountdown(Math.max(0, status.next_check_in_seconds));
            }
        } catch (e: unknown) {
            useErrorStore.getState().trackWarning(e, { source: 'Instances.fetchDaemonTelemetry', triggerAction: 'get_auto_switcher_daemon_status' });
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
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [fetchInstances, fetchSwitcherStatus, fetchAccounts]);

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
        const q = searchQuery.trim().toLowerCase();
        return (
            inst.config.name.toLowerCase().includes(q) ||
            (inst.config.bound_email || '').toLowerCase().includes(q) ||
            inst.config.id.toLowerCase().includes(q)
        );
    });

    const runningCount = instances.filter((i) => i.is_running).length;

    const hasActiveTaskFor = (inst: InstanceStatus): boolean => {
        return (
            Boolean(inst.is_running) &&
            runningTreeNodes.some((node) => {
                const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
                const hasGenuineRunningConv = (node.conversations || []).some(
                    (c) =>
                        Boolean(c.is_running) &&
                        !isGhostConversation(c) &&
                        !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
                );
                return isInstanceMatch && Boolean(node.is_running) && hasGenuineRunningConv;
            })
        );
    };

    const pageActions = createPageActions({
        t,
        instances,
        actionState,
        newInstanceName,
        newInstanceBoundAccount,
        newInstanceFromInstance,
        newInstanceLaunchImmediately,
        copyTargetId,
        copyInstanceName,
        cloneMode,
        copyProjects,
        editTargetId,
        editInstanceName,
        deleteModalTarget,
        wipeModalTarget,
        setActionError,
        setActionState,
        setDeletingId,
        setIsSyncingAll,
        setSyncingInstanceIds,
        setNewInstanceName,
        setNewInstanceBoundAccount,
        setNewInstanceFromInstance,
        setNewInstanceLaunchImmediately,
        setCopyInstanceName,
        setCopyTargetId,
        setEditInstanceName,
        setEditTargetId,
        setDeleteModalTarget,
        setWipeModalTarget,
        setSwitchTargetInstance,
        setAuditModalInstance,
        setSettingsModalTarget,
        setIsSettingsModalOpen,
        setCopyProjects,
        setPromptTreeInstance,
        fetchInstances,
        createInstance,
        copyInstance,
        renameInstance,
        deleteInstance,
        wipeSession,
        launchInstance,
        stopInstance,
        restartInstance,
        fastForwardInstance,
        syncInstance,
        syncAllInstances,
        cloneInstanceExecutable,
        cleanAndRestartWorkspace,
        toggleAutoSwitcher,
        triggerManualRotation,
        setActiveInstance,
        getSwitcherStatus: () => useInstanceStore.getState().switcherStatus,
    });


    return {
        t,
        instances,
        activeInstanceId,
        switcherStatus,
        isLoading,
        storeError,
        accounts,
        currentAccount,
        config,
        searchQuery,
        setSearchQuery,
        viewMode,
        cardDensity,
        filteredInstances,
        runningCount,
        runningTreeNodes,
        projectTreeNodes,
        daemonStatus,
        daemonCountdown,
        actionError,
        setActionError,
        actionState,
        deletingId,
        isSyncingAll,
        syncingInstanceIds,
        activeCardRef,
        cardMoreId,
        setCardMoreId,
        promptTreeInstance,
        setPromptTreeInstance,
        isCreateOpen,
        setIsCreateOpen,
        newInstanceName,
        setNewInstanceName,
        newInstanceBoundAccount,
        setNewInstanceBoundAccount,
        newInstanceFromInstance,
        setNewInstanceFromInstance,
        newInstanceLaunchImmediately,
        setNewInstanceLaunchImmediately,
        switchTargetInstance,
        setSwitchTargetInstance,
        accountSearchQuery,
        setAccountSearchQuery,
        copyTargetId,
        setCopyTargetId,
        copyInstanceName,
        setCopyInstanceName,
        cloneMode,
        setCloneMode,
        copyProjects,
        setCopyProjects,
        isSettingsModalOpen,
        setIsSettingsModalOpen,
        settingsModalTarget,
        setSettingsModalTarget,
        editTargetId,
        setEditTargetId,
        editInstanceName,
        setEditInstanceName,
        auditModalInstance,
        setAuditModalInstance,
        deleteModalTarget,
        setDeleteModalTarget,
        wipeModalTarget,
        setWipeModalTarget,
        fetchInstances,
        fetchSwitcherStatus,
        fetchRunningTasks,
        handleSetViewMode,
        handleSetCardDensity,
        handleCreate: pageActions.handleCreate,
        handleInitDefault: pageActions.handleInitDefault,
        handleCopy: pageActions.handleCopy,
        handleEdit: pageActions.handleEdit,
        handleDelete: pageActions.handleDelete,
        handleConfirmDelete: pageActions.handleConfirmDelete,
        handleWipeSession: pageActions.handleWipeSession,
        handleConfirmWipe: pageActions.handleConfirmWipe,
        handleLaunch: pageActions.handleLaunch,
        handleStop: pageActions.handleStop,
        handleRestart: pageActions.handleRestart,
        handleFastForward: pageActions.handleFastForward,
        handleSync: pageActions.handleSync,
        handleCloneExecutable: pageActions.handleCloneExecutable,
        handleSyncAll: pageActions.handleSyncAll,
        handleCleanRestart: pageActions.handleCleanRestart,
        handleToggleAutoSwitcher: pageActions.handleToggleAutoSwitcher,
        handleTriggerRotation: pageActions.handleTriggerRotation,
        openSwitchDialog: pageActions.openSwitchDialog,
        openAuditDialog: pageActions.openAuditDialog,
        openSettingsDialog: pageActions.openSettingsDialog,
        openCopyDialog: pageActions.openCopyDialog,
        openPromptTree: pageActions.openPromptTree,
        setActiveInstance,
        setDefaultInstance,
        switchAccountToInstance,
        hasActiveTaskFor,
        refreshQuota,
    };
}
