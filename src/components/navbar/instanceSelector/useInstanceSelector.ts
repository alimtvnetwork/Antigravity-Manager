import { useState, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { useConfigStore } from '../../../stores/useConfigStore';
import { useErrorStore } from '../../../stores/error-store';
import { isTauri } from '../../../utils/env';
import { request as invoke } from '../../../utils/request';
import { showToast } from '../../common/ToastContainer';
import type { InstanceSelectorApi, CreateMode, CloneMode } from './instanceSelectorTypes';

export type { InstanceSelectorApi, CreateMode, CloneMode };

function errorMessage(e: unknown): string {
    if (e instanceof Error) return e.message;
    return String(e);
}

function reportActionError(source: string, triggerAction: string, e: unknown, context?: Record<string, unknown>) {
    const captured = useErrorStore.getState().captureError(e, {
        source,
        triggerComponent: 'InstanceSelector',
        triggerAction,
        context,
    });
    useErrorStore.getState().openErrorModal(captured);
}


export function useInstanceSelector(): InstanceSelectorApi {
    const { t } = useTranslation();
    const {
        instances,
        activeInstanceId,
        fetchInstances,
        setActiveInstance,
        setDefaultInstance,
        createInstance,
        copyInstance,
        renameInstance,
        deleteInstance,
        launchInstance,
        closeInstance,
        exportInstancesJson,
        importInstancesJson,
        smartPlayInstance,
        smartRotateProfileAccount,
    } = useInstanceStore();

    const config = useConfigStore((state) => state.config);
    const fastForwardShortcut = config?.auto_profile_switcher?.fast_forward_shortcut || 'Ctrl+Shift+F';

    const [isOpen, setIsOpen] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [isCopyOpen, setIsCopyOpen] = useState(false);
    const [isEditOpen, setIsEditOpen] = useState(false);
    const [isDeleteOpen, setIsDeleteOpen] = useState(false);

    const [newInstanceName, setNewInstanceName] = useState('');
    const [createMode, setCreateMode] = useState<CreateMode>('clone-default');
    const [copyInstanceName, setCopyInstanceName] = useState('');
    const [copyTargetId, setCopyTargetId] = useState<string | null>(null);
    const [cloneMode, setCloneMode] = useState<CloneMode>('full');
    const [copyProjects, setCopyProjects] = useState<boolean>(true);
    const [editInstanceName, setEditInstanceName] = useState('');
    const [editTargetId, setEditTargetId] = useState<string | null>(null);
    const [deleteTarget, setDeleteTarget] = useState<InstanceStatus | null>(null);
    const [launchingId, setLaunchingId] = useState<string | null>(null);
    const [isRotating, setIsRotating] = useState(false);
    const [isIoOpen, setIsIoOpen] = useState(false);

    const dropdownRef = useRef<HTMLDivElement>(null);
    const fileInputRef = useRef<HTMLInputElement>(null);
    const activeItemRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (isOpen) {
            const frameId = requestAnimationFrame(() => {
                const timer = setTimeout(() => {
                    activeItemRef.current?.scrollIntoView({ behavior: 'smooth', block: 'center' });
                }, 80);
                return () => clearTimeout(timer);
            });
            return () => cancelAnimationFrame(frameId);
        }
    }, [isOpen, activeInstanceId]);

    useEffect(() => {
        const canRun = isTauri();
        if (!canRun) return;
        fetchInstances();
        const interval = setInterval(fetchInstances, 4000);
        return () => clearInterval(interval);
    }, [fetchInstances]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            const el = dropdownRef.current;
            if (!el) return;
            const clickedInside = el.contains(event.target as Node);
            if (!clickedInside) {
                setIsOpen(false);
                setIsIoOpen(false);
            }
        };
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsOpen(false);
                setIsIoOpen(false);
                setIsCreateOpen(false);
                setIsCopyOpen(false);
                setIsEditOpen(false);
                setIsDeleteOpen(false);
            }
        };
        const handleOtherDropdownOpen = (e: Event) => {
            const customEvent = e as CustomEvent<{ source?: string }>;
            if (customEvent.detail?.source !== 'instance-selector') {
                setIsOpen(false);
                setIsIoOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        document.addEventListener('keydown', handleKeyDown);
        window.addEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        };
    }, []);

    const activeInstance = instances.find((i) => i.config.id === activeInstanceId) || instances[0];

    const handleCreate = async () => {
        const trimmed = newInstanceName.trim();
        if (!trimmed) return;
        try {
            const created =
                createMode === 'clone-default'
                    ? await createInstance(trimmed, undefined, 'default')
                    : await createInstance(trimmed);
            await setActiveInstance(created.id);
            setNewInstanceName('');
            setIsCreateOpen(false);
            const toast =
                createMode === 'clone-default'
                    ? t('instances.cloned_default_toast', 'Cloned the default IDE')
                    : t('instances.created_toast', 'New instance profile created');
            showToast(toast, 'success');
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleCreate', e, { newInstanceName: trimmed });
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        }
    };

    const handleCopy = async () => {
        const trimmed = copyInstanceName.trim();
        if (!trimmed) return;
        const target =
            (copyTargetId ? instances.find((i) => i.config.id === copyTargetId) : null) || activeInstance || instances[0];
        if (!target) return;
        try {
            const copied = await copyInstance(target.config.id, trimmed, cloneMode, copyProjects);
            await setActiveInstance(copied.id);
            await fetchInstances(true);
            setCopyInstanceName('');
            setCopyTargetId(null);
            setIsCopyOpen(false);
            showToast(t('instances.copied_toast', 'Instance profile duplicated'), 'success');
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleCopy', e, {
                sourceInstanceId: target.config.id,
                copyInstanceName: trimmed,
                cloneMode,
                copyProjects,
            });
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        }
    };

    const handleSetDefault = async (targetId: string) => {
        try {
            await setDefaultInstance(targetId);
            showToast(t('instances.set_default_toast', 'Default profile updated successfully'), 'success');
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleSetDefault', e, { targetId });
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        }
    };

    const handleEdit = async () => {
        if (!editTargetId) return;
        const trimmed = editInstanceName.trim();
        if (!trimmed) return;
        try {
            await renameInstance(editTargetId, trimmed);
            setEditInstanceName('');
            setEditTargetId(null);
            setIsEditOpen(false);
            showToast(t('instances.renamed_toast', 'Profile renamed successfully'), 'success');
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleEdit', e, { editTargetId, editInstanceName: trimmed });
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        }
    };

    const handleDelete = async () => {
        if (!deleteTarget) return;
        const targetId = deleteTarget.config.id;
        const isDefault = Boolean(deleteTarget.config.is_default) || targetId === 'default';
        if (isDefault) {
            showToast(t('instances.cannot_delete_default', 'Cannot delete default profile'), 'warning');
            setIsDeleteOpen(false);
            return;
        }
        try {
            await deleteInstance(targetId);
            const wasActive = activeInstanceId === targetId;
            if (wasActive) {
                await setActiveInstance('default');
            }
            setDeleteTarget(null);
            setIsDeleteOpen(false);
            showToast(t('instances.deleted_toast', 'Profile deleted successfully'), 'success');
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleDelete', e, { targetId });
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        }
    };

    const handleSmartPlay = async (targetId?: string) => {
        const instId = targetId || activeInstance?.config.id || 'default';
        setLaunchingId(instId);
        try {
            const res = await smartPlayInstance(instId);
            showToast(
                t('instances.smart_play_toast', `Smart Play launched ${res.instanceName} with ${res.accountEmail}`),
                'success'
            );
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleSmartPlay', e, { targetId: instId });
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        } finally {
            setLaunchingId(null);
        }
    };

    const handleToggleLaunch = async (instanceId: string, isRunning: boolean) => {
        setLaunchingId(instanceId);
        try {
            if (isRunning) {
                await closeInstance(instanceId);
                showToast(t('instances.closed_toast', 'Instance window closed'), 'info');
            } else {
                await launchInstance(instanceId);
                showToast(t('instances.launched_toast', 'Instance launched successfully'), 'success');
            }
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleToggleLaunch', e, { instanceId, isRunning });
            showToast(t('instances.launch_error', 'Failed to launch instance: ') + errorMessage(e), 'error');
        } finally {
            setLaunchingId(null);
        }
    };

    const handleExportProfiles = async () => {
        try {
            const jsonStr = await exportInstancesJson();
            const dateStr = new Date().toISOString().slice(0, 10);
            const fileName = `antigravity_profiles_${dateStr}.json`;
            const tauriMode = isTauri();
            if (tauriMode) {
                const { save } = await import('@tauri-apps/plugin-dialog');
                const filePath = await save({
                    defaultPath: fileName,
                    filters: [{ name: 'JSON', extensions: ['json'] }],
                });
                if (filePath) {
                    await invoke('write_text_file', { path: filePath, content: jsonStr });
                    showToast(t('instances.export_success', 'Profiles exported successfully'), 'success');
                }
            } else {
                const blob = new Blob([jsonStr], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = fileName;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                showToast(t('instances.export_success', 'Profiles exported successfully'), 'success');
            }
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleExportProfiles', e);
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        }
    };

    const handleImportProfiles = async () => {
        try {
            const tauriMode = isTauri();
            if (tauriMode) {
                const { open } = await import('@tauri-apps/plugin-dialog');
                const selected = await open({
                    multiple: false,
                    filters: [{ name: 'JSON', extensions: ['json'] }],
                });
                if (selected) {
                    const filePath = typeof selected === 'string' ? selected : selected[0];
                    const content = await invoke<string>('read_text_file', { path: filePath });
                    const configs = await importInstancesJson(content);
                    showToast(t('instances.import_success', `Successfully imported ${configs.length} profiles`), 'success');
                }
            } else {
                fileInputRef.current?.click();
            }
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleImportProfiles', e);
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        }
    };

    const handleFileInput = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        try {
            const content = await file.text();
            const configs = await importInstancesJson(content);
            showToast(t('instances.import_success', `Successfully imported ${configs.length} profiles`), 'success');
        } catch (err: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleFileInput', err);
            showToast(`${t('common.error')}: ${errorMessage(err)}`, 'error');
        } finally {
            e.target.value = '';
        }
    };

    const handleSmartRotate = async (sourceId?: string) => {
        if (isRotating) return;
        setIsRotating(true);
        try {
            const targetId = sourceId || activeInstance?.config.id || 'default';
            const result = await smartRotateProfileAccount(targetId);
            const resumeText =
                (result.resumedProjectsCount ?? 0) > 0
                    ? ` · Auto-resumed ${result.resumedProjectsCount} project(s) (<1h)`
                    : '';
            showToast(
                t(
                    'instances.smart_switched_toast',
                    `Transferred ${result.instanceName} to ${result.accountEmail}${resumeText}`
                ),
                'success'
            );
            setIsOpen(false);
        } catch (e: unknown) {
            reportActionError('InstanceSelector.tsx', 'handleSmartRotate', e, {
                sourceId,
                activeInstanceId: activeInstance?.config.id,
                instanceName: activeInstance?.config.name,
            });
            showToast(`${t('common.error')}: ${errorMessage(e)}`, 'error');
        } finally {
            setIsRotating(false);
        }
    };

    const queryTrimmed = searchQuery.trim().toLowerCase();
    const hasQuery = queryTrimmed.length > 0;
    const filteredInstances = instances.filter((inst) => {
        if (!hasQuery) return true;
        const matchName = inst.config.name.toLowerCase().includes(queryTrimmed);
        const email = inst.config.bound_email || '';
        const matchEmail = email.toLowerCase().includes(queryTrimmed);
        return matchName || matchEmail;
    });

    const openCreate = () => {
        setNewInstanceName('');
        setCreateMode('clone-default');
        setIsCreateOpen(true);
    };

    const openCopy = (targetId: string, name: string) => {
        setCopyTargetId(targetId);
        setCopyInstanceName(`${name} (copy)`);
        setCloneMode('full');
        setCopyProjects(true);
        setIsCopyOpen(true);
    };

    const openEdit = (targetId: string, name: string) => {
        setEditTargetId(targetId);
        setEditInstanceName(name);
        setIsEditOpen(true);
    };

    const openDelete = (target: InstanceStatus) => {
        setDeleteTarget(target);
        setIsDeleteOpen(true);
    };

    const handleSelectInstance = async (instId: string, seqNum: number, name: string) => {
        await setActiveInstance(instId);
        await fetchInstances(true);
        showToast(t('instances.switched_toast', `Active profile switched to #${seqNum} (${name})`), 'success');
        setIsOpen(false);
    };

    const closeDelete = () => {
        setDeleteTarget(null);
        setIsDeleteOpen(false);
    };

    return {
        t,
        instances,
        activeInstance,
        activeInstanceId,
        fastForwardShortcut,
        isOpen,
        setIsOpen,
        searchQuery,
        setSearchQuery,
        isCreateOpen,
        setIsCreateOpen,
        isCopyOpen,
        setIsCopyOpen,
        isEditOpen,
        setIsEditOpen,
        isDeleteOpen,
        setIsDeleteOpen,
        newInstanceName,
        setNewInstanceName,
        createMode,
        setCreateMode,
        copyInstanceName,
        setCopyInstanceName,
        copyTargetId,
        setCopyTargetId,
        cloneMode,
        setCloneMode,
        copyProjects,
        setCopyProjects,
        editInstanceName,
        setEditInstanceName,
        editTargetId,
        launchingId,
        isRotating,
        isIoOpen,
        setIsIoOpen,
        deleteTarget,
        dropdownRef,
        fileInputRef,
        activeItemRef,
        filteredInstances,
        handleCreate,
        handleCopy,
        handleSetDefault,
        handleEdit,
        handleDelete,
        handleSmartPlay,
        handleToggleLaunch,
        handleExportProfiles,
        handleImportProfiles,
        handleFileInput,
        handleSmartRotate,
        openCreate,
        openCopy,
        openEdit,
        openDelete,
        handleSelectInstance,
        closeDelete,
    };
}
