import { useState, useEffect, useRef } from 'react';
import {
    ChevronDown,
    Copy,
    Laptop,
    Pencil,
    Play,
    FastForward,
    Square,
    Trash2,
    Search,
    Download,
    Upload,
    AlertTriangle,
    Star,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useInstanceStore } from '../../stores/useInstanceStore';
import { useAccountStore } from '../../stores/useAccountStore';
import { useConfigStore } from '../../stores/useConfigStore';
import { useErrorStore } from '../../stores/error-store';
import { cn } from '../../utils/cn';
import { isTauri } from '../../utils/env';
import { request as invoke } from '../../utils/request';
import { showToast } from '../common/ToastContainer';
import { type InstanceStatus } from '../../services/instanceService';

export function InstanceSelector() {
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

    const config = useConfigStore(state => state.config);
    const fastForwardShortcut = config?.auto_profile_switcher?.fast_forward_shortcut || 'Ctrl+Shift+F';

    const currentAccount = useAccountStore(state => state.currentAccount);
    const accounts = useAccountStore(state => state.accounts);

    const [isOpen, setIsOpen] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [isCopyOpen, setIsCopyOpen] = useState(false);
    const [isEditOpen, setIsEditOpen] = useState(false);
    const [isDeleteOpen, setIsDeleteOpen] = useState(false);

    const [newInstanceName, setNewInstanceName] = useState('');
    const [copyInstanceName, setCopyInstanceName] = useState('');
    const [copyTargetId, setCopyTargetId] = useState<string | null>(null);
    const [cloneMode, setCloneMode] = useState<'full' | 'profile'>('full');
    const [editInstanceName, setEditInstanceName] = useState('');
    const [editTargetId, setEditTargetId] = useState<string | null>(null);
    const [deleteTarget, setDeleteTarget] = useState<InstanceStatus | null>(null);
    const [launchingId, setLaunchingId] = useState<string | null>(null);
    const [isRotating, setIsRotating] = useState(false);
    const [isIoOpen, setIsIoOpen] = useState(false);

    const dropdownRef = useRef<HTMLDivElement>(null);
    const fileInputRef = useRef<HTMLInputElement>(null);

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
        const handleOtherDropdownOpen = (e: Event) => {
            const customEvent = e as CustomEvent<{ source?: string }>;
            if (customEvent.detail?.source !== 'instance-selector') {
                setIsOpen(false);
                setIsIoOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        window.addEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            window.removeEventListener('agm:dropdown-open', handleOtherDropdownOpen);
        };
    }, []);

    const activeInstance = instances.find(i => i.config.id === activeInstanceId) || instances[0];

    const handleCreate = async () => {
        const trimmed = newInstanceName.trim();
        if (!trimmed) return;
        try {
            const created = await createInstance(trimmed);
            await setActiveInstance(created.id);
            setNewInstanceName('');
            setIsCreateOpen(false);
            showToast(t('instances.created_toast', 'New instance profile created'), 'success');
        } catch (e: any) {
            console.error('Failed to create instance:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleCreate',
                context: { newInstanceName: trimmed },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
        }
    };

    const handleCopy = async () => {
        const trimmed = copyInstanceName.trim();
        if (!trimmed) return;
        const target = (copyTargetId ? instances.find(i => i.config.id === copyTargetId) : null) || activeInstance || instances[0];
        if (!target) return;
        try {
            const copied = await copyInstance(target.config.id, trimmed, cloneMode);
            await setActiveInstance(copied.id);
            setCopyInstanceName('');
            setCopyTargetId(null);
            setIsCopyOpen(false);
            showToast(t('instances.copied_toast', 'Instance profile duplicated'), 'success');
        } catch (e: any) {
            console.error('Failed to copy instance:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleCopy',
                context: { sourceInstanceId: target.config.id, copyInstanceName: trimmed, cloneMode },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
        }
    };

    const handleSetDefault = async (targetId: string) => {
        try {
            await setDefaultInstance(targetId);
            showToast(t('instances.set_default_toast', 'Default profile updated successfully'), 'success');
        } catch (e: any) {
            console.error('Failed to set default instance:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleSetDefault',
                context: { targetId },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
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
        } catch (e: any) {
            console.error('Failed to rename instance:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleEdit',
                context: { editTargetId, editInstanceName: trimmed },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
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
        } catch (e: any) {
            console.error('Failed to delete instance:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleDelete',
                context: { targetId },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
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
        } catch (e: any) {
            console.error('Smart play failed:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleSmartPlay',
                context: { targetId: instId },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
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
        } catch (e: any) {
            console.error('Failed to toggle instance:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleToggleLaunch',
                context: { instanceId, isRunning },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(t('instances.launch_error', 'Failed to launch instance: ') + (e?.message || e), 'error');
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
        } catch (e: any) {
            console.error('Failed to export profiles:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleExportProfiles',
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
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
        } catch (e: any) {
            console.error('Failed to import profiles:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleImportProfiles',
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
        }
    };

    const handleFileInput = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        try {
            const content = await file.text();
            const configs = await importInstancesJson(content);
            showToast(t('instances.import_success', `Successfully imported ${configs.length} profiles`), 'success');
        } catch (err: any) {
            console.error('Failed to import file:', err);
            const captured = useErrorStore.getState().captureError(err, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector',
                triggerAction: 'handleFileInput',
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${err?.message || err}`, 'error');
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
            const resumeText = (result.resumedProjectsCount ?? 0) > 0
                ? ` · Auto-resumed ${result.resumedProjectsCount} project(s) (<1h)`
                : '';
            showToast(
                t('instances.smart_switched_toast', `Transferred ${result.instanceName} to ${result.accountEmail}${resumeText}`),
                'success'
            );
            setIsOpen(false);
        } catch (e: any) {
            console.error('Failed to smart transfer profile account:', e);
            const captured = useErrorStore.getState().captureError(e, {
                source: 'InstanceSelector.tsx',
                triggerComponent: 'InstanceSelector.TransferButton',
                triggerAction: 'handleSmartRotate',
                context: {
                    sourceId,
                    activeInstanceId: activeInstance?.config.id,
                    instanceName: activeInstance?.config.name,
                },
            });
            useErrorStore.getState().openErrorModal(captured);
            showToast(`${t('common.error')}: ${e?.message || e}`, 'error');
        } finally {
            setIsRotating(false);
        }
    };

    const queryTrimmed = searchQuery.trim().toLowerCase();
    const hasQuery = queryTrimmed.length > 0;
    const filteredInstances = instances.filter(inst => {
        if (!hasQuery) return true;
        const matchName = inst.config.name.toLowerCase().includes(queryTrimmed);
        const email = inst.config.bound_email || '';
        const matchEmail = email.toLowerCase().includes(queryTrimmed);
        return matchName || matchEmail;
    });

    const isAvailable = isTauri();
    if (!isAvailable) return null;

    const isActiveRunning = Boolean(activeInstance?.is_running);

    return (
        <div className="relative flex items-center gap-1 shrink-0" ref={dropdownRef}>
            <input
                ref={fileInputRef}
                type="file"
                accept=".json,application/json"
                style={{ display: 'none' }}
                onChange={handleFileInput}
            />

            {/* 1. Profile Dropdown Trigger */}
            <button
                type="button"
                onClick={() => {
                    const next = !isOpen;
                    setIsOpen(next);
                    if (next) {
                        window.dispatchEvent(new CustomEvent('agm:dropdown-open', { detail: { source: 'instance-selector' } }));
                    }
                }}
                className="flex items-center gap-1.5 md:gap-2 px-2.5 md:px-3 py-1.5 rounded-lg text-xs font-medium bg-gray-100 dark:bg-slate-800 hover:bg-gray-200 dark:hover:bg-slate-700 transition-colors border border-gray-200/60 dark:border-slate-700 shrink-0 cursor-pointer"
                title={t('instances.selector_tooltip', 'Select active Antigravity instance')}
            >
                <span
                    className={`w-2 h-2 rounded-full shrink-0 ${isActiveRunning ? 'bg-emerald-500 animate-pulse' : 'bg-gray-400'}`}
                />
                <span className="truncate max-w-[95px] md:max-w-[130px] text-gray-800 dark:text-gray-200 font-medium">
                    {activeInstance ? `#${instances.findIndex(i => i.config.id === activeInstance.config.id) + 1} ${activeInstance.config.name}` : 'Default'}
                </span>
                <ChevronDown className="w-3.5 h-3.5 text-gray-500 shrink-0" />
            </button>

            {/* 2. Top Action Bar: Fast-Forward only (Smart Switch) */}
            <button
                type="button"
                disabled={isRotating}
                onClick={(e) => {
                    e.stopPropagation();
                    handleSmartRotate(activeInstance?.config.id);
                }}
                className={`flex p-1.5 rounded-lg text-white shadow-xs transition-colors duration-150 shrink-0 items-center justify-center cursor-pointer ${
                    isRotating
                        ? 'bg-blue-400 dark:bg-blue-800 cursor-not-allowed opacity-80'
                        : 'bg-blue-600 hover:bg-blue-700 active:bg-blue-800'
                }`}
                title={t(
                    'instances.smart_switch_shortcut_tooltip',
                    `Fast-Forward (${fastForwardShortcut}): Close running process, pick account with longest refill runway, and switch profile`
                )}
            >
                <FastForward className={`w-3.5 h-3.5 fill-current ${isRotating ? 'animate-spin' : ''}`} />
            </button>

            {/* Dropdown Menu Popup (Strictly Above Page Content) */}
            {isOpen && (
                <div
                    className="absolute top-full right-0 mt-1.5 w-96 max-w-[calc(100vw-24px)] md:w-[420px] rounded-xl shadow-2xl bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 py-2 z-[9999] animate-in fade-in zoom-in-95"
                    style={{ isolation: 'isolate' }}
                >
                    {/* Dropdown Header Bar with Duplicate and Combined Import / Export Dropdown */}
                    <div className="flex items-center justify-between px-3 py-1 border-b border-gray-100 dark:border-slate-800 pb-1.5">
                        <span className="text-[11px] font-semibold text-gray-400 uppercase tracking-wider">
                            {t('instances.header_title', 'INSTANCES / PROFILES')}
                        </span>
                        <div className="flex items-center gap-1 relative">
                            <button
                                type="button"
                                onClick={() => {
                                    const target = activeInstance || instances[0];
                                    if (target) {
                                        setCopyTargetId(target.config.id);
                                        setCopyInstanceName(`${target.config.name} Copy`);
                                        setCloneMode('full');
                                        setIsCopyOpen(true);
                                    }
                                }}
                                className="p-1 rounded-md text-gray-500 hover:text-indigo-600 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
                                title={t('instances.duplicate_active', 'Duplicate Active Profile')}
                            >
                                <Copy className="w-3.5 h-3.5" />
                            </button>

                            {/* Combined Import / Export Icon Dropdown */}
                            <div className="relative">
                                <button
                                    type="button"
                                    onClick={() => setIsIoOpen(!isIoOpen)}
                                    className="px-1.5 py-1 rounded-md text-gray-500 hover:text-blue-600 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer flex items-center gap-0.5"
                                    title="Import / Export Profiles (JSON)"
                                >
                                    <Download className="w-3.5 h-3.5" />
                                    <ChevronDown className="w-2.5 h-2.5" />
                                </button>
                                {isIoOpen && (
                                    <div className="absolute right-0 mt-1 w-44 rounded-lg bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 shadow-xl py-1 z-[9999] text-xs">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsIoOpen(false);
                                                handleImportProfiles();
                                            }}
                                            className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
                                        >
                                            <Upload className="w-3.5 h-3.5 text-emerald-500" />
                                            <span>{t('instances.import_json', 'Import Profiles (JSON)')}</span>
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsIoOpen(false);
                                                handleExportProfiles();
                                            }}
                                            className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
                                        >
                                            <Download className="w-3.5 h-3.5 text-blue-500" />
                                            <span>{t('instances.export_json', 'Export Profiles (JSON)')}</span>
                                        </button>
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* Profile Search Input Box */}
                    <div className="px-2.5 pt-2 pb-1">
                        <div className="relative">
                            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
                            <input
                                type="text"
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                placeholder={t('instances.search_placeholder', 'Search profiles...')}
                                className="w-full pl-8 pr-2.5 py-1 text-xs bg-gray-50 dark:bg-slate-800 border border-gray-200 dark:border-slate-700 rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500 text-gray-900 dark:text-slate-100"
                            />
                        </div>
                    </div>

                    {/* Filtered Profiles List */}
                    <div className="max-h-60 overflow-y-auto py-1">
                        {filteredInstances.length === 0 ? (
                            <div className="px-3 py-4 text-center text-xs text-gray-400">
                                {t('instances.no_profiles_found', 'No profiles found')}
                            </div>
                        ) : (
                            filteredInstances.map((inst) => {
                                const isSelected = inst.config.id === activeInstanceId;
                                const isDefault = Boolean(inst.config.is_default);
                                const isRunning = Boolean(inst.is_running);
                                const linkedAccount = accounts.find(a => a.id === inst.config.bound_account_id || (inst.config.bound_email && a.email === inst.config.bound_email));
                                const displayEmail = inst.config.bound_email || linkedAccount?.email || (isSelected ? currentAccount?.email : undefined);
                                const seqNum = instances.findIndex(i => i.config.id === inst.config.id) + 1;

                                return (
                                    <div
                                        key={inst.config.id}
                                        className={cn(
                                            "w-full group flex items-center justify-between px-3 py-2 text-xs text-left transition-all duration-150 border-l-4",
                                            isSelected
                                                ? "bg-amber-500/15 dark:bg-blue-950/80 border-l-amber-400 dark:border-l-amber-400 text-amber-950 dark:text-blue-200 font-medium shadow-xs"
                                                : "border-l-transparent text-gray-700 dark:text-gray-300 hover:bg-amber-500/10 dark:hover:bg-blue-900/40 hover:border-l-amber-400/80"
                                        )}
                                    >
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setActiveInstance(inst.config.id);
                                                setIsOpen(false);
                                            }}
                                            className="flex items-center gap-2 truncate flex-1 text-left cursor-pointer min-w-0 pr-2"
                                        >
                                            <span
                                                className={`w-2 h-2 rounded-full shrink-0 ${
                                                    isRunning ? 'bg-emerald-500 animate-pulse' : 'bg-gray-400'
                                                }`}
                                            />
                                            <div className="flex flex-col truncate min-w-0">
                                                <div className="flex items-center gap-1.5 truncate">
                                                    <span className="px-1.5 py-0.5 rounded text-[10px] font-black bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 shrink-0">
                                                        #{seqNum}
                                                    </span>
                                                    <span className="truncate font-semibold">{inst.config.name}</span>
                                                    {isDefault && (
                                                        <span className="px-1 py-0.2 rounded text-[8px] font-bold bg-indigo-500/15 text-indigo-700 dark:text-indigo-300 border border-indigo-400/30 shrink-0">
                                                            DEFAULT
                                                        </span>
                                                    )}
                                                    {isSelected && (
                                                        <span className="px-1 py-0.2 rounded text-[8px] font-bold bg-amber-500/20 text-amber-700 dark:text-amber-300 dark:bg-amber-400/10 border border-amber-400/30 shrink-0">
                                                            ACTIVE
                                                        </span>
                                                    )}
                                                </div>
                                                {displayEmail ? (
                                                    <span className="text-[10px] text-gray-500 dark:text-gray-400 truncate font-mono">
                                                        {displayEmail}
                                                    </span>
                                                ) : (
                                                    <span className="text-[10px] text-gray-400/60 dark:text-gray-500/60 italic truncate">
                                                        {t('instances.unlinked', 'No account linked')}
                                                    </span>
                                                )}
                                            </div>
                                        </button>

                                        {/* Action Buttons: Launch, Smart Switch, Duplicate, Rename, Set Default, Delete */}
                                        <div className="flex items-center gap-1 shrink-0 ml-auto">
                                            {/* 1: Item Launch / Stop */}
                                            <button
                                                type="button"
                                                disabled={launchingId === inst.config.id}
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    handleToggleLaunch(inst.config.id, isRunning);
                                                }}
                                                className={cn(
                                                    "w-6 h-6 p-1 rounded transition-colors cursor-pointer flex items-center justify-center",
                                                    isRunning
                                                        ? "text-red-500 hover:bg-red-100 dark:hover:bg-red-900/40"
                                                        : "text-emerald-600 hover:bg-emerald-100 dark:hover:bg-emerald-900/40"
                                                )}
                                                title={
                                                    isRunning
                                                        ? t('instances.close_title', 'Close window')
                                                        : t('instances.launch_title', 'Run profile')
                                                }
                                            >
                                                {isRunning ? (
                                                    <Square className="w-3 h-3 fill-current" />
                                                ) : (
                                                    <Play className="w-3 h-3 fill-current" />
                                                )}
                                            </button>

                                            {/* 2: Item Smart Switch */}
                                            <button
                                                type="button"
                                                disabled={isRotating}
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    handleSmartRotate(inst.config.id);
                                                }}
                                                className="w-6 h-6 p-1 rounded text-blue-600 hover:text-blue-700 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors cursor-pointer flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
                                                title={t('instances.smart_switch_tooltip', 'Smart Switch: Close process, pick account with longest refill runway, and switch')}
                                            >
                                                <FastForward className={`w-3 h-3 fill-current ${isRotating ? 'animate-spin' : ''}`} />
                                            </button>

                                            {/* 3: Item Rename (Pencil) */}
                                            <button
                                                type="button"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setEditTargetId(inst.config.id);
                                                    setEditInstanceName(inst.config.name);
                                                    setIsEditOpen(true);
                                                }}
                                                className="w-6 h-6 p-1 rounded text-gray-400 hover:text-blue-600 hover:bg-gray-200/60 dark:hover:bg-slate-800 transition-colors opacity-70 group-hover:opacity-100 cursor-pointer flex items-center justify-center"
                                                title={t('instances.edit_title', 'Rename profile')}
                                            >
                                                <Pencil className="w-3 h-3" />
                                            </button>

                                            {/* 4: Item Duplicate / Clone (Copy) */}
                                            <button
                                                type="button"
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setCopyTargetId(inst.config.id);
                                                    setCopyInstanceName(`${inst.config.name} Copy`);
                                                    setCloneMode('full');
                                                    setIsCopyOpen(true);
                                                }}
                                                className="w-6 h-6 p-1 rounded text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 transition-colors opacity-70 group-hover:opacity-100 cursor-pointer flex items-center justify-center"
                                                title={t('instances.duplicate_title', 'Duplicate profile')}
                                            >
                                                <Copy className="w-3 h-3" />
                                            </button>

                                            {/* 5: Set as Default Toggle */}
                                            <button
                                                type="button"
                                                disabled={isDefault}
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    if (!isDefault) {
                                                        handleSetDefault(inst.config.id);
                                                    }
                                                }}
                                                className={cn(
                                                    "w-6 h-6 p-1 rounded transition-colors flex items-center justify-center",
                                                    isDefault
                                                        ? "text-amber-500 cursor-default"
                                                        : "text-gray-400 hover:text-amber-500 hover:bg-amber-50 dark:hover:bg-amber-950/30 cursor-pointer opacity-70 group-hover:opacity-100"
                                                )}
                                                title={
                                                    isDefault
                                                        ? t('instances.is_default_title', 'Default profile')
                                                        : t('instances.set_default_title', 'Set as default profile')
                                                }
                                            >
                                                <Star className={cn("w-3 h-3", isDefault ? "fill-amber-400 text-amber-500" : "")} />
                                            </button>

                                            {/* 6: Item Delete (Hidden for Default profile) */}
                                            {isDefault ? (
                                                <span className="w-6 h-6" aria-hidden="true" />
                                            ) : (
                                                <button
                                                    type="button"
                                                    onClick={(e) => {
                                                        e.stopPropagation();
                                                        setDeleteTarget(inst);
                                                        setIsDeleteOpen(true);
                                                    }}
                                                    className="w-6 h-6 p-1 rounded text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 transition-colors opacity-70 group-hover:opacity-100 cursor-pointer flex items-center justify-center"
                                                    title={t('instances.delete_title', 'Delete profile')}
                                                >
                                                    <Trash2 className="w-3 h-3" />
                                                </button>
                                            )}
                                        </div>
                                    </div>
                                );
                            })
                        )}
                    </div>

                    {/* Footer Bar: Smart Play / Run Selected Profile */}
                    <div className="p-2 border-t border-gray-100 dark:border-slate-800">
                        <button
                            type="button"
                            disabled={launchingId === activeInstance?.config.id}
                            onClick={() => {
                                if (isActiveRunning) {
                                    handleToggleLaunch(activeInstance.config.id, true);
                                } else {
                                    handleSmartPlay(activeInstance?.config.id);
                                }
                            }}
                            className={`w-full py-1.5 px-3 rounded-lg text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors cursor-pointer ${
                                isActiveRunning
                                    ? 'bg-red-50 dark:bg-red-950/30 text-red-600 hover:bg-red-100 dark:hover:bg-red-900/40 border border-red-200 dark:border-red-900/40'
                                    : 'bg-emerald-600 hover:bg-emerald-700 active:bg-emerald-800 hover:brightness-105 text-white shadow-xs'
                            }`}
                        >
                            {isActiveRunning ? (
                                <>
                                    <Square className="w-3.5 h-3.5 fill-current" />
                                    <span>{t('instances.close_active', 'Close Active Instance')}</span>
                                </>
                            ) : (
                                <>
                                    <Play className="w-3.5 h-3.5 fill-current" />
                                    <span>{t('instances.smart_play_active', 'Smart Play Selected Profile')}</span>
                                </>
                            )}
                        </button>
                    </div>
                </div>
            )}

            {/* Create Instance Modal */}
            {isCreateOpen && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-[99999] p-4">
                    <div className="bg-white dark:bg-slate-900 rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-slate-800">
                        <div className="flex items-center gap-2 mb-3">
                            <Laptop className="w-5 h-5 text-blue-600" />
                            <h3 className="font-bold text-sm text-gray-900 dark:text-slate-100">
                                {t('instances.create_modal_title', 'Create New Profile')}
                            </h3>
                        </div>
                        <input
                            type="text"
                            placeholder={t('instances.name_placeholder', 'Profile name (e.g., Work, Client B)')}
                            value={newInstanceName}
                            onChange={(e) => setNewInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                            className="input input-sm w-full bg-gray-50 dark:bg-slate-800 border border-gray-200 dark:border-slate-700 text-gray-900 dark:text-slate-100 rounded-lg mb-4 text-xs"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2">
                            <button
                                type="button"
                                onClick={() => setIsCreateOpen(false)}
                                className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleCreate}
                                disabled={!newInstanceName.trim()}
                                className="btn btn-primary btn-xs"
                            >
                                {t('common.create', 'Create')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Duplicate Instance Modal with Full Directory Copy Toggle */}
            {isCopyOpen && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-[99999] p-4">
                    <div className="bg-white dark:bg-slate-900 rounded-2xl p-5 w-full max-w-md shadow-2xl border border-gray-100 dark:border-slate-800">
                        <div className="flex items-center gap-2 mb-3">
                            <Copy className="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
                            <h3 className="font-bold text-sm text-gray-900 dark:text-slate-100">
                                {t('instances.copy_modal_title', 'Duplicate / Clone Profile')}
                            </h3>
                        </div>

                        <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                            {t('instances.copy_name_label', 'New Profile Name')}
                        </label>
                        <input
                            type="text"
                            placeholder={t('instances.copy_placeholder', 'New profile name')}
                            value={copyInstanceName}
                            onChange={(e) => setCopyInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleCopy()}
                            className="input input-sm w-full bg-gray-50 dark:bg-slate-800 border border-gray-200 dark:border-slate-700 text-gray-900 dark:text-slate-100 rounded-lg mb-3 text-xs"
                            autoFocus
                        />

                        {/* Clone Mode Selection */}
                        <div className="mb-4 bg-gray-50 dark:bg-slate-800/80 p-3 rounded-xl border border-gray-200/60 dark:border-slate-700">
                            <span className="block text-[11px] font-semibold text-gray-500 dark:text-gray-400 mb-2 uppercase">
                                {t('instances.clone_mode_label', 'Duplication Scope / Clone Type')}
                            </span>
                            <div className="flex flex-col gap-2">
                                <label className="flex items-start gap-2 cursor-pointer text-xs">
                                    <input
                                        type="radio"
                                        name="clone_mode"
                                        value="full"
                                        checked={cloneMode === 'full'}
                                        onChange={() => setCloneMode('full')}
                                        className="radio radio-xs radio-primary mt-0.5"
                                    />
                                    <div>
                                        <div className="font-semibold text-gray-800 dark:text-gray-200">
                                            {t('instances.clone_mode_full', 'IDE Copy (Full Environment & Sessions)')}
                                        </div>
                                        <div className="text-[11px] text-gray-500 dark:text-gray-400 leading-tight">
                                            {t('instances.clone_mode_full_desc', 'Clones complete isolated environment, sessions, extensions, cache, and state.')}
                                        </div>
                                    </div>
                                </label>

                                <label className="flex items-start gap-2 cursor-pointer text-xs">
                                    <input
                                        type="radio"
                                        name="clone_mode"
                                        value="profile"
                                        checked={cloneMode === 'profile'}
                                        onChange={() => setCloneMode('profile')}
                                        className="radio radio-xs radio-primary mt-0.5"
                                    />
                                    <div>
                                        <div className="font-semibold text-gray-800 dark:text-gray-200">
                                            {t('instances.clone_mode_profile', 'Profile Copy (Preferences & Snippets)')}
                                        </div>
                                        <div className="text-[11px] text-gray-500 dark:text-gray-400 leading-tight">
                                            {t('instances.clone_mode_profile_desc', 'Copies only User preferences, keybindings, and snippets without bulky runtime session state.')}
                                        </div>
                                    </div>
                                </label>
                            </div>
                        </div>

                        <div className="flex justify-end gap-2">
                            <button
                                type="button"
                                onClick={() => setIsCopyOpen(false)}
                                className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleCopy}
                                disabled={!copyInstanceName.trim()}
                                className="btn btn-primary btn-xs"
                            >
                                {t('instances.duplicate', 'Duplicate')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Edit / Rename Instance Modal */}
            {isEditOpen && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-[99999] p-4">
                    <div className="bg-white dark:bg-slate-900 rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-slate-800">
                        <div className="flex items-center gap-2 mb-3">
                            <Pencil className="w-5 h-5 text-blue-600" />
                            <h3 className="font-bold text-sm text-gray-900 dark:text-slate-100">
                                {t('instances.edit_modal_title', 'Rename Profile')}
                            </h3>
                        </div>
                        <input
                            type="text"
                            placeholder={t('instances.edit_placeholder', 'Profile name')}
                            value={editInstanceName}
                            onChange={(e) => setEditInstanceName(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleEdit()}
                            className="input input-sm w-full bg-gray-50 dark:bg-slate-800 border border-gray-200 dark:border-slate-700 text-gray-900 dark:text-slate-100 rounded-lg mb-4 text-xs"
                            autoFocus
                        />
                        <div className="flex justify-end gap-2">
                            <button
                                type="button"
                                onClick={() => setIsEditOpen(false)}
                                className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleEdit}
                                disabled={!editInstanceName.trim()}
                                className="btn btn-primary btn-xs"
                            >
                                {t('common.save', 'Save')}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Delete Instance Modal */}
            {isDeleteOpen && deleteTarget && (
                <div className="fixed inset-0 bg-black/40 backdrop-blur-xs flex items-center justify-center z-[99999] p-4">
                    <div className="bg-white dark:bg-slate-900 rounded-2xl p-5 w-full max-w-sm shadow-2xl border border-gray-100 dark:border-slate-800">
                        <div className="flex items-center gap-2 mb-3 text-red-600">
                            <AlertTriangle className="w-5 h-5" />
                            <h3 className="font-bold text-sm text-gray-900 dark:text-slate-100">
                                {t('instances.delete_modal_title', 'Delete Profile')}
                            </h3>
                        </div>
                        <p className="text-xs text-gray-600 dark:text-gray-300 mb-4 leading-relaxed">
                            {t(
                                'instances.delete_confirm_desc',
                                `Are you sure you want to delete profile "${deleteTarget.config.name}"? All isolated data and workspace sessions will be deleted.`
                            )}
                        </p>
                        <div className="flex justify-end gap-2">
                            <button
                                type="button"
                                onClick={() => {
                                    setDeleteTarget(null);
                                    setIsDeleteOpen(false);
                                }}
                                className="btn btn-ghost btn-xs text-gray-600 dark:text-gray-400"
                            >
                                {t('common.cancel', 'Cancel')}
                            </button>
                            <button
                                type="button"
                                onClick={handleDelete}
                                className="btn btn-error btn-xs text-white"
                            >
                                {t('common.delete', 'Delete')}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
