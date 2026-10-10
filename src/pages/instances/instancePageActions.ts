import { showToast } from '../../components/common/ToastContainer';
import type { InstanceStatus } from '../../services/instanceService';
import type { InstanceActionType } from './instancePageUtils';

function actionErrorMessage(e: unknown): string {
    return (e as { toString?: () => string })?.toString?.() || 'Operation failed';
}

export interface PageActionContext {
    t: (key: string, fallback?: string) => string;
    instances: InstanceStatus[];
    actionState: Record<string, InstanceActionType>;
    newInstanceName: string;
    newInstanceBoundAccount: string;
    newInstanceFromInstance: string;
    newInstanceLaunchImmediately: boolean;
    copyTargetId: string | null;
    copyInstanceName: string;
    cloneMode: 'full' | 'profile';
    copyProjects: boolean;
    editTargetId: string | null;
    editInstanceName: string;
    deleteModalTarget: InstanceStatus | null;
    wipeModalTarget: InstanceStatus | null;
    setActionError: (v: string | null) => void;
    setActionState: React.Dispatch<React.SetStateAction<Record<string, InstanceActionType>>>;
    setDeletingId: (v: string | null) => void;
    setIsSyncingAll: (v: boolean) => void;
    setSyncingInstanceIds: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
    setNewInstanceName: (v: string) => void;
    setNewInstanceBoundAccount: (v: string) => void;
    setNewInstanceFromInstance: (v: string) => void;
    setNewInstanceLaunchImmediately: (v: boolean) => void;
    setCopyInstanceName: (v: string) => void;
    setCopyTargetId: (v: string | null) => void;
    setEditInstanceName: (v: string) => void;
    setEditTargetId: (v: string | null) => void;
    setDeleteModalTarget: (v: InstanceStatus | null) => void;
    setWipeModalTarget: (v: InstanceStatus | null) => void;
    setSwitchTargetInstance: (v: InstanceStatus | null) => void;
    setAuditModalInstance: (v: { id: string; name: string; sequence_name?: string } | null) => void;
    setSettingsModalTarget: (v: InstanceStatus | null) => void;
    setIsSettingsModalOpen: (v: boolean) => void;
    setCopyProjects: (v: boolean) => void;
    setPromptTreeInstance: (v: { id: string; name: string; projectId?: string; seqNum?: number; executablePath?: string } | null) => void;
    fetchInstances: (force?: boolean) => Promise<void>;
    createInstance: (name: string, boundAccountId?: string, fromInstanceId?: string) => Promise<{ id: string; name: string }>;
    copyInstance: (id: string, name: string, mode: 'full' | 'profile', copyProjects: boolean) => Promise<{ id: string }>;
    renameInstance: (id: string, name: string) => Promise<void>;
    deleteInstance: (id: string) => Promise<void>;
    wipeSession: (id: string) => Promise<void>;
    launchInstance: (id: string) => Promise<void>;
    stopInstance: (id: string) => Promise<void>;
    restartInstance: (id: string) => Promise<void>;
    fastForwardInstance: (id: string) => Promise<string>;
    syncInstance: (id: string) => Promise<void>;
    syncAllInstances: () => Promise<void>;
    cloneInstanceExecutable: (id: string) => Promise<string>;
    cleanAndRestartWorkspace: () => Promise<string>;
    toggleAutoSwitcher: () => Promise<void>;
    triggerManualRotation: () => Promise<void>;
    setActiveInstance: (id: string) => Promise<void>;
    getSwitcherStatus: () => unknown;
}

export interface PageActions {
    handleCreate: () => Promise<void>;
    handleInitDefault: () => Promise<void>;
    handleCopy: () => Promise<void>;
    handleEdit: () => Promise<void>;
    handleDelete: (id: string) => void;
    handleConfirmDelete: () => Promise<void>;
    handleWipeSession: (id: string) => void;
    handleConfirmWipe: () => Promise<void>;
    handleLaunch: (id: string) => Promise<void>;
    handleStop: (id: string) => Promise<void>;
    handleRestart: (id: string) => Promise<void>;
    handleFastForward: (id: string) => Promise<void>;
    handleSync: (id: string) => Promise<void>;
    handleCloneExecutable: (id: string) => Promise<void>;
    handleSyncAll: () => Promise<void>;
    handleCleanRestart: () => Promise<void>;
    handleToggleAutoSwitcher: () => Promise<void>;
    handleTriggerRotation: () => Promise<void>;
    openSwitchDialog: (id: string) => void;
    openAuditDialog: (id: string, name: string) => void;
    openSettingsDialog: (id: string) => void;
    openCopyDialog: (id: string, name: string) => void;
    openPromptTree: (id: string) => void;
}

export function createPageActions(ctx: PageActionContext): PageActions {
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
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to create instance');
            }
        };

        const handleInitDefault = async () => {
        setActionError(null);
        try {
            await createInstance('Default');
        } catch (e: unknown) {
            setActionError(actionErrorMessage(e) || 'Failed to initialize default profile');
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
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to copy instance');
            }
        };

        const handleEdit = async () => {
            if (!editTargetId || !editInstanceName.trim()) return;
            setActionError(null);
            try {
                await renameInstance(editTargetId, editInstanceName.trim());
                setEditInstanceName('');
                setEditTargetId(null);
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to rename instance');
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
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to delete instance');
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
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to wipe session');
            } finally {
                setActionState((prev) => ({ ...prev, [targetId]: null }));
                setWipeModalTarget(null);
            }
        };

        const runGuardedAction = async (
            id: string,
            action: Exclude<InstanceActionType, null>,
            fn: () => Promise<void>,
            successToast?: string
        ) => {
            if (actionState[id]) return;
            setActionState((prev) => ({ ...prev, [id]: action }));
            setActionError(null);
            try {
                await fn();
                if (successToast) showToast(successToast, 'success');
            } catch (e: unknown) {
                const msg = actionErrorMessage(e);
                setActionError(msg);
                if (action === 'restart') showToast(msg, 'error');
            } finally {
                setActionState((prev) => ({ ...prev, [id]: null }));
            }
        };

        const handleLaunch = (id: string) => runGuardedAction(id, 'launch', () => launchInstance(id));
        const handleStop = (id: string) => runGuardedAction(id, 'stop', () => stopInstance(id));
        const handleRestart = (id: string) =>
            runGuardedAction(id, 'restart', () => restartInstance(id), 'Instance restarted successfully on current account');
        const handleFastForward = (id: string) =>
            runGuardedAction(id, 'fast-forward', async () => {
                const msg = await fastForwardInstance(id);
                showToast(msg || 'Rotated to next best profile!', 'success');
            });

        const handleSync = async (id: string) => {
            if (actionState[id]) return;
            setActionState((prev) => ({ ...prev, [id]: 'sync' }));
            setSyncingInstanceIds((prev) => ({ ...prev, [id]: true }));
            setActionError(null);
            try {
                await syncInstance(id);
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to sync instance');
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
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to clone executable');
            }
        };

        const handleSyncAll = async () => {
            setIsSyncingAll(true);
            try {
                await syncAllInstances();
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to sync all instances and quotas');
            } finally {
                setIsSyncingAll(false);
            }
        };

        const handleCleanRestart = async () => {
            try {
                const msg = await cleanAndRestartWorkspace();
                showToast(msg || 'Stuck Electron processes cleared & Antigravity restarted!', 'success');
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e) || 'Failed to clean and restart workspace');
            }
        };

        const handleToggleAutoSwitcher = async () => {
            try {
                await toggleAutoSwitcher();
                const status = useInstanceStore.getState().switcherStatus;
                showToast(status?.is_running ? 'Auto-Switcher disabled' : 'Auto-Switcher enabled', 'success');
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e));
            }
        };

        const handleTriggerRotation = async () => {
            try {
                await triggerManualRotation();
            } catch (e: unknown) {
                setActionError(actionErrorMessage(e));
            }
        };

        const openSwitchDialog = (id: string) => {
            const target = instances.find((i) => i.config.id === id);
            if (target) setSwitchTargetInstance(target);
        };

        const openAuditDialog = (id: string, name: string) => {
            const target = instances.find((i) => i.config.id === id);
            setAuditModalInstance({
                id,
                name,
                sequence_name: target?.config.seq_num ? `Instance #${target.config.seq_num}` : undefined,
            });
        };

        const openSettingsDialog = (id: string) => {
            const target = instances.find((i) => i.config.id === id);
            setSettingsModalTarget(target || null);
            setIsSettingsModalOpen(true);
        };

        const openCopyDialog = (id: string, name: string) => {
            setCopyTargetId(id);
            setCopyInstanceName(`${name} Copy`);
            setCopyProjects(true);
        };

        const openPromptTree = (id: string) => {
            const target = instances.find((i) => i.config.id === id);
            setPromptTreeInstance({
                id,
                name: target?.config.name || id,
                seqNum: target?.config.seq_num,
                executablePath: target?.config.executable_path,
            });
        };
    return {
        handleCreate,
        handleInitDefault,
        handleCopy,
        handleEdit,
        handleDelete,
        handleConfirmDelete,
        handleWipeSession,
        handleConfirmWipe,
        handleLaunch,
        handleStop,
        handleRestart,
        handleFastForward,
        handleSync,
        handleCloneExecutable,
        handleSyncAll,
        handleCleanRestart,
        handleToggleAutoSwitcher,
        handleTriggerRotation,
        openSwitchDialog,
        openAuditDialog,
        openSettingsDialog,
        openCopyDialog,
        openPromptTree,
    };
}
