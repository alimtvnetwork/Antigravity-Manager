import { useState, useEffect, useRef } from 'react';
import {
    X,
    Copy,
    ClipboardPaste,
    Download,
    Upload,
    RotateCcw,
    RotateCw,
    Sliders,
    Zap,
    CheckCircle2,
    FolderSync,
    FileCode,
    SlidersHorizontal,
    Sparkles,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { isTauri } from '../../utils/env';
import { cn } from '../../utils/cn';
import { showToast } from '../common/ToastContainer';
import * as instanceService from '../../services/instanceService';
import type { InstanceStatus } from '../../services/instanceService';

interface InstanceClipboardBuffer {
    sourceId: string;
    sourceName: string;
    hasSettings: boolean;
    hasWorkspaces: boolean;
    copiedAt: number;
}

interface InstanceSettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
    targetInstance?: InstanceStatus | null;
    instances: InstanceStatus[];
    onInstancesUpdated?: () => Promise<void> | void;
}

export function InstanceSettingsModal({
    isOpen,
    onClose,
    targetInstance,
    instances,
    onInstancesUpdated,
}: InstanceSettingsModalProps) {
    const { t } = useTranslation();

    const [selectedTargetId, setSelectedTargetId] = useState<string>('default');
    const [copySettingsSourceId, setCopySettingsSourceId] = useState<string>('');
    const [copyProjectsSourceId, setCopyProjectsSourceId] = useState<string>('');
    const [replicationSourceId, setReplicationSourceId] = useState<string>('');
    const [clipboardBuffer, setClipboardBuffer] = useState<InstanceClipboardBuffer | null>(() => {
        try {
            const raw = localStorage.getItem('agm_instance_clipboard_buffer');
            return raw ? JSON.parse(raw) : null;
        } catch {
            return null;
        }
    });

    // Settings state
    const [isTurboMode, setIsTurboMode] = useState<boolean>(true);
    const [isAlwaysProceed, setIsAlwaysProceed] = useState<boolean>(true);

    // JSON editor & history stack
    const [jsonContent, setJsonContent] = useState<string>('');
    const [isJsonExpanded, setIsJsonExpanded] = useState<boolean>(false);
    const [history, setHistory] = useState<string[]>([]);
    const [historyIndex, setHistoryIndex] = useState<number>(-1);

    // Loading states
    const [isLoading, setIsLoading] = useState<boolean>(false);
    const [isOperating, setIsOperating] = useState<boolean>(false);

    const fileInputRef = useRef<HTMLInputElement>(null);

    // Sync selected target when modal opens or targetInstance changes
    useEffect(() => {
        if (isOpen) {
            const initialId = targetInstance?.config.id || (instances[0]?.config.id ?? 'default');
            setSelectedTargetId(initialId);

            // Default source instances for copy operations
            const otherInstances = instances.filter(i => i.config.id !== initialId);
            if (otherInstances.length > 0) {
                setCopySettingsSourceId(otherInstances[0].config.id);
                setCopyProjectsSourceId(otherInstances[0].config.id);
                setReplicationSourceId(otherInstances[0].config.id);
            } else {
                setCopySettingsSourceId('');
                setCopyProjectsSourceId('');
                setReplicationSourceId('');
            }

            // Sync clipboard buffer from localStorage
            try {
                const raw = localStorage.getItem('agm_instance_clipboard_buffer');
                if (raw) {
                    setClipboardBuffer(JSON.parse(raw));
                }
            } catch {}

            loadSettings(initialId);
        }
    }, [isOpen, targetInstance, instances]);

    // Handle escape key
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen) {
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    const loadSettings = async (instanceId: string) => {
        if (!instanceId) return;
        setIsLoading(true);
        try {
            const rawJson = await instanceService.exportInstanceSettings(instanceId);
            let formatted = rawJson;
            try {
                const parsed = JSON.parse(rawJson);
                formatted = JSON.stringify(parsed, null, 2);

                // Detect turbo mode and plan review states from exported config
                if (typeof parsed['antigravity.turboMode'] === 'boolean') {
                    setIsTurboMode(parsed['antigravity.turboMode']);
                }
                if (typeof parsed['antigravity.planReviewAlwaysProceed'] === 'boolean') {
                    setIsAlwaysProceed(parsed['antigravity.planReviewAlwaysProceed']);
                }
            } catch {
                // Keep raw string if parse fails
            }

            setJsonContent(formatted);
            // Push initial snapshot into history if empty
            setHistory([formatted]);
            setHistoryIndex(0);
        } catch (err: any) {
            console.warn('Failed to load instance settings:', err);
            setJsonContent('{\n  "antigravity.turboMode": true,\n  "antigravity.planReviewAlwaysProceed": true\n}');
        } finally {
            setIsLoading(false);
        }
    };

    const pushHistory = (newContent: string) => {
        const updatedHistory = history.slice(0, historyIndex + 1);
        updatedHistory.push(newContent);
        setHistory(updatedHistory);
        setHistoryIndex(updatedHistory.length - 1);
    };

    const handleTargetChange = (newTargetId: string) => {
        setSelectedTargetId(newTargetId);
        const otherInstances = instances.filter(i => i.config.id !== newTargetId);
        if (otherInstances.length > 0) {
            if (copySettingsSourceId === newTargetId) {
                setCopySettingsSourceId(otherInstances[0].config.id);
            }
            if (copyProjectsSourceId === newTargetId) {
                setCopyProjectsSourceId(otherInstances[0].config.id);
            }
            if (replicationSourceId === newTargetId) {
                setReplicationSourceId(otherInstances[0].config.id);
            }
        }
        loadSettings(newTargetId);
    };

    // Toggle Turbo Mode
    const handleSetTurboMode = async (applyToAll: boolean) => {
        setIsOperating(true);
        try {
            const nextValue = !isTurboMode;
            const target = applyToAll ? undefined : selectedTargetId;
            const updatedCount = await instanceService.setInstanceTurboMode(target, nextValue);
            setIsTurboMode(nextValue);

            const toastMsg = applyToAll
                ? `Turbo Mode ${nextValue ? 'enabled' : 'disabled'} across all ${updatedCount} instances`
                : `Turbo Mode ${nextValue ? 'enabled' : 'disabled'} for target profile`;
            showToast(toastMsg, 'success');

            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Failed to update Turbo Mode: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Toggle Plan Review
    const handleSetPlanReview = async (applyToAll: boolean, alwaysProceedValue: boolean) => {
        setIsOperating(true);
        try {
            const target = applyToAll ? undefined : selectedTargetId;
            const updatedCount = await instanceService.setInstancePlanReview(target, alwaysProceedValue);
            setIsAlwaysProceed(alwaysProceedValue);

            const toastMsg = applyToAll
                ? `Plan Review set to '${alwaysProceedValue ? 'Always Proceed' : 'Ask Permission'}' across ${updatedCount} instances`
                : `Plan Review set to '${alwaysProceedValue ? 'Always Proceed' : 'Ask Permission'}' for target profile`;
            showToast(toastMsg, 'success');

            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Failed to update Plan Review: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Enforce Defaults
    const handleEnforceDefaults = async (applyToAll: boolean) => {
        setIsOperating(true);
        try {
            const target = applyToAll ? undefined : selectedTargetId;
            const count = await instanceService.enforceDefaultSettings(target);
            showToast(
                applyToAll
                    ? `Default baseline settings enforced across all ${count} instances`
                    : `Default baseline settings enforced for target profile`,
                'success'
            );
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Failed to enforce defaults: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Copy Settings From Another Instance
    const handleCopySettings = async () => {
        if (!copySettingsSourceId || !selectedTargetId) return;
        setIsOperating(true);
        try {
            await instanceService.copyInstanceSettings(copySettingsSourceId, selectedTargetId);
            const src = instances.find(i => i.config.id === copySettingsSourceId);
            if (src) {
                const buf: InstanceClipboardBuffer = {
                    sourceId: src.config.id,
                    sourceName: src.config.name,
                    hasSettings: true,
                    hasWorkspaces: false,
                    copiedAt: Date.now(),
                };
                try {
                    localStorage.setItem('agm_instance_clipboard_buffer', JSON.stringify(buf));
                    setClipboardBuffer(buf);
                } catch {}
            }
            showToast(`Settings synchronized from source to ${selectedTargetId}`, 'success');
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Failed to copy settings: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Copy Projects/Folders From Another Instance
    const handleCopyProjects = async () => {
        if (!copyProjectsSourceId || !selectedTargetId) return;
        setIsOperating(true);
        try {
            const copiedCount = await instanceService.copyInstanceProjects(copyProjectsSourceId, selectedTargetId);
            const src = instances.find(i => i.config.id === copyProjectsSourceId);
            if (src) {
                const buf: InstanceClipboardBuffer = {
                    sourceId: src.config.id,
                    sourceName: src.config.name,
                    hasSettings: false,
                    hasWorkspaces: true,
                    copiedAt: Date.now(),
                };
                try {
                    localStorage.setItem('agm_instance_clipboard_buffer', JSON.stringify(buf));
                    setClipboardBuffer(buf);
                } catch {}
            }
            showToast(`Successfully copied ${copiedCount} workspace folder(s) & project states`, 'success');
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Failed to copy projects: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Copy Both Settings & Workspaces to Target and persist in localStorage buffer
    const handleCopyBothDirect = async (sourceId: string) => {
        if (!sourceId || !selectedTargetId) return;
        if (sourceId === selectedTargetId) {
            showToast('Source and target profiles are identical', 'warning');
            return;
        }
        setIsOperating(true);
        try {
            const src = instances.find(i => i.config.id === sourceId);
            if (src) {
                const buf: InstanceClipboardBuffer = {
                    sourceId: src.config.id,
                    sourceName: src.config.name,
                    hasSettings: true,
                    hasWorkspaces: true,
                    copiedAt: Date.now(),
                };
                try {
                    localStorage.setItem('agm_instance_clipboard_buffer', JSON.stringify(buf));
                    setClipboardBuffer(buf);
                } catch {}
            }

            const { projectsCount } = await instanceService.copyInstanceBoth(sourceId, selectedTargetId);
            showToast(`Successfully copied settings & ${projectsCount} workspace folder(s) to target profile`, 'success');
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Failed to copy both: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Split paste from buffer
    const handlePasteFromBuffer = async (mode: 'both' | 'settings' | 'workspaces') => {
        if (!clipboardBuffer || !selectedTargetId) {
            showToast('Replication buffer is empty', 'warning');
            return;
        }
        if (clipboardBuffer.sourceId === selectedTargetId) {
            showToast('Source and target profiles are identical', 'warning');
            return;
        }
        setIsOperating(true);
        try {
            let msg = '';
            if (mode === 'both' || mode === 'settings') {
                await instanceService.copyInstanceSettings(clipboardBuffer.sourceId, selectedTargetId);
                msg += 'Settings applied. ';
            }
            if (mode === 'both' || mode === 'workspaces') {
                const count = await instanceService.copyInstanceProjects(clipboardBuffer.sourceId, selectedTargetId);
                msg += `${count} workspace folder(s) replicated.`;
            }
            showToast(msg || 'Pasted from replication buffer successfully', 'success');
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Paste failed: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Clipboard Copy
    const handleCopyToClipboard = async () => {
        try {
            await navigator.clipboard.writeText(jsonContent);
            showToast('Settings JSON copied to clipboard', 'info');
        } catch {
            showToast('Failed to copy to clipboard', 'error');
        }
    };

    // Clipboard Paste & Apply
    const handlePasteFromClipboard = async () => {
        try {
            const text = await navigator.clipboard.readText();
            if (!text.trim()) {
                showToast('Clipboard is empty', 'warning');
                return;
            }
            // Validate JSON
            const parsed = JSON.parse(text);
            const formatted = JSON.stringify(parsed, null, 2);

            setIsOperating(true);
            await instanceService.importInstanceSettings(selectedTargetId, formatted);
            pushHistory(formatted);
            setJsonContent(formatted);
            showToast('Settings pasted from clipboard and applied successfully', 'success');
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Invalid JSON in clipboard or apply failed: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Apply JSON text area
    const handleApplyJsonText = async () => {
        try {
            const parsed = JSON.parse(jsonContent);
            const formatted = JSON.stringify(parsed, null, 2);
            setIsOperating(true);
            await instanceService.importInstanceSettings(selectedTargetId, formatted);
            pushHistory(formatted);
            showToast('Settings JSON applied successfully', 'success');
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Invalid JSON: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Undo
    const handleUndo = async () => {
        const canUndo = historyIndex > 0;
        if (!canUndo) return;
        const targetSnapshot = history[historyIndex - 1];
        setIsOperating(true);
        try {
            await instanceService.importInstanceSettings(selectedTargetId, targetSnapshot);
            setHistoryIndex(historyIndex - 1);
            setJsonContent(targetSnapshot);
            showToast('Reverted to previous settings snapshot (Undo)', 'info');
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Undo failed: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Redo
    const handleRedo = async () => {
        const canRedo = historyIndex < history.length - 1;
        if (!canRedo) return;
        const targetSnapshot = history[historyIndex + 1];
        setIsOperating(true);
        try {
            await instanceService.importInstanceSettings(selectedTargetId, targetSnapshot);
            setHistoryIndex(historyIndex + 1);
            setJsonContent(targetSnapshot);
            showToast('Reapplied settings snapshot (Redo)', 'info');
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Redo failed: ${err?.message || err}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    // Export JSON File
    const handleExportJsonFile = async () => {
        try {
            const fileName = `antigravity_${selectedTargetId}_settings.json`;
            if (isTauri()) {
                const { save } = await import('@tauri-apps/plugin-dialog');
                const filePath = await save({
                    defaultPath: fileName,
                    filters: [{ name: 'JSON', extensions: ['json'] }],
                });
                if (filePath) {
                    const { writeTextFile } = await import('@tauri-apps/plugin-fs');
                    await writeTextFile(filePath, jsonContent);
                    showToast('Settings exported to file', 'success');
                }
            } else {
                const blob = new Blob([jsonContent], { type: 'application/json' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = fileName;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
                showToast('Settings exported to file', 'success');
            }
        } catch (err: any) {
            showToast(`Export failed: ${err?.message || err}`, 'error');
        }
    };

    // Import JSON File
    const handleImportJsonFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        try {
            const text = await file.text();
            const parsed = JSON.parse(text);
            const formatted = JSON.stringify(parsed, null, 2);

            setIsOperating(true);
            await instanceService.importInstanceSettings(selectedTargetId, formatted);
            pushHistory(formatted);
            setJsonContent(formatted);
            showToast('Imported and applied settings from file', 'success');
            onInstancesUpdated?.();
        } catch (err: any) {
            showToast(`Failed to parse/import settings file: ${err?.message || err}`, 'error');
        } finally {
            e.target.value = '';
            setIsOperating(false);
        }
    };

    if (!isOpen) return null;

    const canUndo = historyIndex > 0;
    const canRedo = historyIndex < history.length - 1;
    const currentInstanceObj = instances.find(i => i.config.id === selectedTargetId);
    const candidateSources = instances.filter(i => i.config.id !== selectedTargetId);

    return (
        <div
            className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={onClose}
        >
            <input
                ref={fileInputRef}
                type="file"
                accept=".json,application/json"
                style={{ display: 'none' }}
                onChange={handleImportJsonFile}
            />

            <div
                className="bg-white dark:bg-[#0c2438] rounded-2xl w-full max-w-2xl max-h-[90vh] shadow-2xl border border-gray-200 dark:border-[#15334d] flex flex-col overflow-hidden text-xs text-gray-900 dark:text-gray-100 animate-in fade-in zoom-in-95"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Modal Header */}
                <div className="flex items-center justify-between px-5 py-3.5 border-b border-gray-100 dark:border-[#15334d] bg-gray-50/70 dark:bg-[#091e30]">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-cyan-400">
                            <SlidersHorizontal className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                                <span>{t('instances.settings_modal_title', 'Instance Settings & Deep Sync')}</span>
                                {isLoading && <RotateCw className="w-3.5 h-3.5 animate-spin text-blue-500" />}
                            </h2>
                            <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                {t('instances.settings_modal_desc', 'Synchronize Turbo Mode, Plan Review, themes, projects, and JSON configurations across profiles')}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Target Instance Bar */}
                <div className="px-5 py-2.5 bg-blue-50/40 dark:bg-[#081a28] border-b border-gray-100 dark:border-[#15334d] flex items-center justify-between gap-3 flex-wrap">
                    <div className="flex items-center gap-2 flex-1 min-w-0">
                        <span className="font-semibold text-gray-700 dark:text-gray-300 shrink-0">Target Profile:</span>
                        <select
                            value={selectedTargetId}
                            onChange={(e) => handleTargetChange(e.target.value)}
                            className="min-w-0 flex-1 truncate bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] rounded-lg px-2.5 py-1 text-xs font-bold text-blue-700 dark:text-cyan-300 focus:outline-none focus:ring-1 focus:ring-blue-500 cursor-pointer"
                        >
                            {instances.map((inst, idx) => {
                                const seq = inst.config.seq_num ?? idx + 1;
                                const idSuffix = inst.config.id.startsWith('antigravity-')
                                    ? inst.config.id
                                    : `antigravity-${inst.config.id}`;
                                const isDefault = inst.config.is_default || inst.config.id === 'default';
                                return (
                                    <option key={inst.config.id} value={inst.config.id}>
                                        #{seq} {inst.config.name} ({idSuffix}){isDefault ? ' [Default]' : ''}
                                    </option>
                                );
                            })}
                        </select>
                    </div>

                    <div className="flex items-center gap-1.5 text-[11px] text-gray-500 dark:text-gray-400 font-mono shrink-0">
                        <span className="px-2 py-0.5 rounded bg-gray-100 dark:bg-[#15334d] border border-gray-200 dark:border-[#1e466b]">
                            ID: {selectedTargetId}
                        </span>
                        {currentInstanceObj?.is_running && (
                            <span className="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-bold border border-emerald-500/20">
                                Running
                            </span>
                        )}
                    </div>
                </div>

                {/* Scrollable Content */}
                <div className="flex-1 overflow-y-auto overflow-x-hidden p-5 space-y-4">
                    {/* 1. Common Settings Quick Toggles Grid */}
                    <div>
                        <span className="block text-[11px] font-bold text-gray-400 uppercase tracking-wider mb-2">
                            Common Automation Settings
                        </span>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                            {/* Turbo Mode Card */}
                            <div className="p-3.5 rounded-xl border border-gray-200 dark:border-[#15334d] bg-gray-50/60 dark:bg-[#071a27]/60 flex flex-col justify-between space-y-2.5">
                                <div className="flex items-start justify-between gap-2">
                                    <div className="flex items-center gap-2">
                                        <div className="p-1.5 rounded-lg bg-amber-500/15 text-amber-600 dark:text-amber-400 shrink-0">
                                            <Zap className="w-4 h-4 fill-current" />
                                        </div>
                                        <div>
                                            <div className="font-bold text-xs text-gray-900 dark:text-gray-100">
                                                Turbo Mode
                                            </div>
                                            <div className="text-[10px] text-gray-500 dark:text-gray-400">
                                                Auto-confirm safe edits
                                            </div>
                                        </div>
                                    </div>
                                    <span className={cn(
                                        "px-2 py-0.5 rounded text-[10px] font-bold uppercase shrink-0",
                                        isTurboMode
                                            ? "bg-amber-500/20 text-amber-700 dark:text-amber-300 border border-amber-500/30"
                                            : "bg-gray-200 dark:bg-slate-800 text-gray-600 dark:text-gray-400"
                                    )}>
                                        {isTurboMode ? 'ON' : 'OFF'}
                                    </span>
                                </div>

                                {/* Segmented Switch Capsule */}
                                <div className="flex items-center rounded-lg bg-gray-100 dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs">
                                    <button
                                        type="button"
                                        disabled={isOperating}
                                        onClick={() => handleSetTurboMode(false)}
                                        title={isTurboMode ? "Click to disable Auto-Confirm (Turbo Mode)" : "Click to enable Auto-Confirm (Turbo Mode)"}
                                        className={cn(
                                            "flex-1 py-1 px-2.5 rounded-md text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5",
                                            isTurboMode
                                                ? "bg-amber-600 text-white shadow-xs"
                                                : "text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d]"
                                        )}
                                    >
                                        <Zap className="w-3.5 h-3.5 fill-current" />
                                        <span>Auto-Confirm: {isTurboMode ? 'ON' : 'OFF'}</span>
                                    </button>
                                    <button
                                        type="button"
                                        disabled={isOperating}
                                        onClick={() => handleSetTurboMode(true)}
                                        className="py-1 px-2.5 text-xs font-medium text-blue-600 dark:text-cyan-400 hover:bg-gray-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer shrink-0 flex items-center gap-1"
                                        title="Apply this Auto-Confirm state across all instances"
                                    >
                                        <Sliders className="w-3 h-3" />
                                        <span>Apply All</span>
                                    </button>
                                </div>
                            </div>

                            {/* Plan Review Card */}
                            <div className="p-3.5 rounded-xl border border-gray-200 dark:border-[#15334d] bg-gray-50/60 dark:bg-[#071a27]/60 flex flex-col justify-between space-y-2.5">
                                <div className="flex items-start justify-between gap-2">
                                    <div className="flex items-center gap-2">
                                        <div className="p-1.5 rounded-lg bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 shrink-0">
                                            <CheckCircle2 className="w-4 h-4" />
                                        </div>
                                        <div>
                                            <div className="font-bold text-xs text-gray-900 dark:text-gray-100">
                                                Plan Review
                                            </div>
                                            <div className="text-[10px] text-gray-500 dark:text-gray-400">
                                                Proceed vs require prompt
                                            </div>
                                        </div>
                                    </div>
                                    <span className={cn(
                                        "px-2 py-0.5 rounded text-[10px] font-bold shrink-0",
                                        isAlwaysProceed
                                            ? "bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30"
                                            : "bg-blue-500/20 text-blue-700 dark:text-blue-300 border border-blue-500/30"
                                    )}>
                                        {isAlwaysProceed ? 'Always Proceed' : 'Ask Permission'}
                                    </span>
                                </div>

                                {/* Segmented Switch Capsule */}
                                <div className="flex items-center rounded-lg bg-gray-100 dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs">
                                    <button
                                        type="button"
                                        disabled={isOperating}
                                        onClick={() => handleSetPlanReview(false, !isAlwaysProceed)}
                                        title={isAlwaysProceed ? "Click to require permission before executing plans" : "Click to always proceed without asking"}
                                        className={cn(
                                            "flex-1 py-1 px-2.5 rounded-md text-xs font-semibold transition-all cursor-pointer flex items-center justify-center gap-1.5",
                                            isAlwaysProceed
                                                ? "bg-emerald-600 text-white shadow-xs"
                                                : "bg-blue-600 text-white shadow-xs"
                                        )}
                                    >
                                        <CheckCircle2 className="w-3.5 h-3.5" />
                                        <span>{isAlwaysProceed ? 'Always Proceed' : 'Ask Permission'}</span>
                                    </button>
                                    <button
                                        type="button"
                                        disabled={isOperating}
                                        onClick={() => handleSetPlanReview(true, isAlwaysProceed)}
                                        className="py-1 px-2.5 text-xs font-medium text-blue-600 dark:text-cyan-400 hover:bg-gray-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer shrink-0 flex items-center gap-1"
                                        title="Apply current Plan Review setting across all instances"
                                    >
                                        <Sliders className="w-3 h-3" />
                                        <span>Apply All</span>
                                    </button>
                                </div>
                            </div>
                        </div>

                        {/* Enforce Baseline Defaults Action */}
                        <div className="mt-3 p-3 rounded-xl border border-gray-200 dark:border-[#15334d] bg-gray-50/40 dark:bg-[#071a27]/40 flex items-center justify-between gap-3 flex-wrap">
                            <div className="flex items-center gap-2">
                                <Sparkles className="w-4 h-4 text-indigo-500 shrink-0" />
                                <div>
                                    <div className="font-semibold text-xs text-gray-900 dark:text-gray-100">
                                        Default Baseline Alignment
                                    </div>
                                    <div className="text-[10px] text-gray-500 dark:text-gray-400">
                                        Reset and propagate standard baseline settings & themes from default profile
                                    </div>
                                </div>
                            </div>
                            <div className="flex items-center rounded-lg bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs shrink-0">
                                <button
                                    type="button"
                                    disabled={isOperating}
                                    onClick={() => handleEnforceDefaults(false)}
                                    title="Enforce baseline settings onto target profile"
                                    className="px-2.5 py-1 text-xs font-medium text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-[#15334d] rounded-l-md transition-colors cursor-pointer flex items-center gap-1"
                                >
                                    <Sparkles className="w-3 h-3 text-indigo-500" />
                                    <span>Enforce Target</span>
                                </button>
                                <button
                                    type="button"
                                    disabled={isOperating}
                                    onClick={() => handleEnforceDefaults(true)}
                                    title="Enforce baseline settings across all profiles"
                                    className="px-2.5 py-1 text-xs font-semibold text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-950/40 rounded-r-md transition-colors cursor-pointer flex items-center gap-1"
                                >
                                    <Sliders className="w-3 h-3" />
                                    <span>Enforce All</span>
                                </button>
                            </div>
                        </div>
                    </div>

                    {/* 2. Cross-Instance Synchronization */}
                    <div className="space-y-3">
                        <div className="flex items-center justify-between gap-2 flex-wrap">
                            <span className="block text-[11px] font-bold text-gray-400 uppercase tracking-wider">
                                Cross-Instance Replication
                            </span>

                            {/* Buffer status indicator */}
                            {clipboardBuffer ? (
                                <div className="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/25 text-[10px] font-mono text-emerald-600 dark:text-emerald-400">
                                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                    <span>
                                        Buffer: {clipboardBuffer.sourceName} (
                                        {clipboardBuffer.hasSettings && clipboardBuffer.hasWorkspaces
                                            ? 'Both'
                                            : clipboardBuffer.hasSettings
                                            ? 'Settings'
                                            : 'Workspaces'}
                                        )
                                    </span>
                                </div>
                            ) : (
                                <span className="text-[10px] text-gray-400 italic font-mono">
                                    Buffer empty
                                </span>
                            )}
                        </div>

                        {/* Master Replication Bar */}
                        <div className="rounded-xl bg-slate-50 dark:bg-[#071a27]/80 border border-gray-200 dark:border-[#15334d] p-3 flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3">
                            <div className="flex items-center gap-2 flex-1 min-w-0">
                                <span className="text-xs font-semibold text-gray-700 dark:text-gray-300 shrink-0">Source:</span>
                                <select
                                    value={replicationSourceId}
                                    onChange={(e) => setReplicationSourceId(e.target.value)}
                                    className="min-w-0 flex-1 truncate bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] rounded-lg px-2.5 py-1 text-xs font-bold text-blue-700 dark:text-cyan-300 focus:outline-none focus:ring-1 focus:ring-blue-500 cursor-pointer"
                                >
                                    {candidateSources.map((inst, idx) => {
                                        const seq = inst.config.seq_num ?? idx + 1;
                                        const idSuffix = inst.config.id.startsWith('antigravity-')
                                            ? inst.config.id
                                            : `antigravity-${inst.config.id}`;
                                        return (
                                            <option key={inst.config.id} value={inst.config.id}>
                                                #{seq} {inst.config.name} ({idSuffix})
                                            </option>
                                        );
                                    })}
                                </select>
                            </div>

                            <div className="flex items-center gap-2 shrink-0 flex-wrap">
                                {/* Copy Both Action Button */}
                                <button
                                    type="button"
                                    disabled={isOperating || !replicationSourceId}
                                    onClick={() => handleCopyBothDirect(replicationSourceId)}
                                    className="shrink-0 whitespace-nowrap px-3 py-1.5 rounded-lg text-xs font-bold bg-indigo-600 hover:bg-indigo-700 text-white shadow-xs transition-colors cursor-pointer disabled:opacity-50 flex items-center gap-1.5"
                                    title="Copy both settings and workspaces directly into target and update buffer"
                                >
                                    <Copy className="w-3.5 h-3.5" />
                                    <span>Copy Both</span>
                                </button>

                                {/* Split Paste Segmented Pill Controls */}
                                <div className="inline-flex rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-0.5 divide-x divide-emerald-500/20 shadow-2xs">
                                    <button
                                        type="button"
                                        disabled={isOperating || !clipboardBuffer}
                                        onClick={() => handlePasteFromBuffer('both')}
                                        className="px-2.5 py-1 text-xs font-bold text-emerald-700 dark:text-emerald-300 hover:bg-emerald-600 hover:text-white transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1.5 rounded-l-md"
                                        title="Paste both settings & workspaces from buffer into target"
                                    >
                                        <ClipboardPaste className="w-3.5 h-3.5" />
                                        <span>Paste Both</span>
                                    </button>
                                    <button
                                        type="button"
                                        disabled={isOperating || !clipboardBuffer}
                                        onClick={() => handlePasteFromBuffer('settings')}
                                        className="px-2 py-1 text-xs font-medium text-emerald-700 dark:text-emerald-300 hover:bg-emerald-600 hover:text-white transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1"
                                        title="Paste settings only from buffer into target"
                                    >
                                        <Sliders className="w-3 h-3" />
                                        <span>Settings</span>
                                    </button>
                                    <button
                                        type="button"
                                        disabled={isOperating || !clipboardBuffer}
                                        onClick={() => handlePasteFromBuffer('workspaces')}
                                        className="px-2 py-1 text-xs font-medium text-emerald-700 dark:text-emerald-300 hover:bg-emerald-600 hover:text-white transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1 rounded-r-md"
                                        title="Paste workspaces and folders only from buffer into target"
                                    >
                                        <FolderSync className="w-3 h-3" />
                                        <span>Folders</span>
                                    </button>
                                </div>
                            </div>
                        </div>

                        {/* Individual Cards for Settings and Workspaces */}
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                            {/* Copy Settings Card */}
                            <div className="rounded-xl bg-slate-50 dark:bg-[#071a27]/80 border border-gray-200 dark:border-[#15334d] p-3 space-y-2.5 flex flex-col justify-between">
                                <div>
                                    <div className="flex items-center justify-between gap-2 mb-1.5">
                                        <div className="flex items-center gap-2 min-w-0">
                                            <Sliders className="w-4 h-4 text-blue-500 shrink-0" />
                                            <span className="font-bold text-xs text-gray-900 dark:text-gray-100 truncate">
                                                Settings & Themes
                                            </span>
                                        </div>
                                        <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20 shrink-0">
                                            settings.json
                                        </span>
                                    </div>
                                    <p className="text-[10px] text-gray-500 dark:text-gray-400 line-clamp-1" title="Copies color themes, policies, and Antigravity preferences into target">
                                        Copies color themes, policies, and Antigravity preferences.
                                    </p>
                                </div>
                                <div className="space-y-2 pt-1">
                                    <select
                                        value={copySettingsSourceId}
                                        onChange={(e) => setCopySettingsSourceId(e.target.value)}
                                        className="w-full truncate bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] rounded-lg px-2.5 py-1.5 text-xs text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-1 focus:ring-blue-500 cursor-pointer"
                                    >
                                        {candidateSources.map((inst, idx) => {
                                            const seq = inst.config.seq_num ?? idx + 1;
                                            const idSuffix = inst.config.id.startsWith('antigravity-')
                                                ? inst.config.id
                                                : `antigravity-${inst.config.id}`;
                                            return (
                                                <option key={inst.config.id} value={inst.config.id}>
                                                    #{seq} {inst.config.name} ({idSuffix})
                                                </option>
                                            );
                                        })}
                                    </select>
                                    <button
                                        type="button"
                                        disabled={isOperating || !copySettingsSourceId}
                                        onClick={handleCopySettings}
                                        className="w-full px-3 py-1.5 rounded-lg text-xs font-semibold bg-blue-600 hover:bg-blue-700 text-white shadow-xs transition-colors cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                                        title="Copy settings and themes from source to target profile"
                                    >
                                        <Sliders className="w-3 h-3" />
                                        <span>Copy Settings Now</span>
                                    </button>
                                </div>
                            </div>

                            {/* Copy Projects/Workspaces Card */}
                            <div className="rounded-xl bg-slate-50 dark:bg-[#071a27]/80 border border-gray-200 dark:border-[#15334d] p-3 space-y-2.5 flex flex-col justify-between">
                                <div>
                                    <div className="flex items-center justify-between gap-2 mb-1.5">
                                        <div className="flex items-center gap-2 min-w-0">
                                            <FolderSync className="w-4 h-4 text-teal-500 shrink-0" />
                                            <span className="font-bold text-xs text-gray-900 dark:text-gray-100 truncate">
                                                Workspaces & Folders
                                            </span>
                                        </div>
                                        <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-teal-500/10 text-teal-600 dark:text-teal-400 border border-teal-500/20 shrink-0">
                                            workspaceStorage
                                        </span>
                                    </div>
                                    <p className="text-[10px] text-gray-500 dark:text-gray-400 line-clamp-1" title="Copies open projects, workspace storage, and recent folder state">
                                        Copies open projects, workspace storage, and recent folder state.
                                    </p>
                                </div>
                                <div className="space-y-2 pt-1">
                                    <select
                                        value={copyProjectsSourceId}
                                        onChange={(e) => setCopyProjectsSourceId(e.target.value)}
                                        className="w-full truncate bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] rounded-lg px-2.5 py-1.5 text-xs text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-1 focus:ring-blue-500 cursor-pointer"
                                    >
                                        {candidateSources.map((inst, idx) => {
                                            const seq = inst.config.seq_num ?? idx + 1;
                                            const idSuffix = inst.config.id.startsWith('antigravity-')
                                                ? inst.config.id
                                                : `antigravity-${inst.config.id}`;
                                            return (
                                                <option key={inst.config.id} value={inst.config.id}>
                                                    #{seq} {inst.config.name} ({idSuffix})
                                                </option>
                                            );
                                        })}
                                    </select>
                                    <button
                                        type="button"
                                        disabled={isOperating || !copyProjectsSourceId}
                                        onClick={handleCopyProjects}
                                        className="w-full px-3 py-1.5 rounded-lg text-xs font-semibold bg-teal-600 hover:bg-teal-700 text-white shadow-xs transition-colors cursor-pointer disabled:opacity-50 flex items-center justify-center gap-1.5"
                                        title="Copy workspaces and project folders from source to target profile"
                                    >
                                        <FolderSync className="w-3 h-3" />
                                        <span>Copy Folders Now</span>
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* 3. Clipboard, Undo/Redo & JSON Tools */}
                    <div className="border border-gray-200 dark:border-[#15334d] rounded-xl p-3.5 bg-gray-50/50 dark:bg-[#071a27]/50 space-y-3">
                        <div className="flex items-center justify-between flex-wrap gap-2">
                            <span className="text-[11px] font-bold text-gray-400 uppercase tracking-wider flex items-center gap-1.5">
                                <FileCode className="w-3.5 h-3.5" />
                                <span>Clipboard & JSON Editor</span>
                            </span>

                            {/* Undo / Redo buttons */}
                            <div className="flex items-center gap-1">
                                <button
                                    type="button"
                                    disabled={!canUndo || isOperating}
                                    onClick={handleUndo}
                                    className="p-1 px-2 rounded-lg text-xs font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d] disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1 transition-colors cursor-pointer"
                                    title="Undo last applied JSON settings change"
                                >
                                    <RotateCcw className="w-3 h-3" />
                                    <span>Undo</span>
                                </button>
                                <button
                                    type="button"
                                    disabled={!canRedo || isOperating}
                                    onClick={handleRedo}
                                    className="p-1 px-2 rounded-lg text-xs font-medium text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#15334d] disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-1 transition-colors cursor-pointer"
                                    title="Redo previously undone change"
                                >
                                    <RotateCw className="w-3 h-3" />
                                    <span>Redo</span>
                                </button>
                            </div>
                        </div>

                        {/* Group JSON tools into a contiguous segmented pill capsule ([Copy | Paste | Export | Import | Raw Editor]) */}
                        <div className="flex items-center justify-between flex-wrap gap-2">
                            <div className="inline-flex items-center rounded-lg bg-white dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d] p-0.5 divide-x divide-gray-200 dark:divide-[#15334d] shadow-2xs">
                                <button
                                    type="button"
                                    onClick={handleCopyToClipboard}
                                    className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] rounded-l-md flex items-center gap-1.5 transition-colors cursor-pointer"
                                    title="Copy settings JSON to clipboard"
                                >
                                    <Copy className="w-3.5 h-3.5 text-blue-500" />
                                    <span>Copy</span>
                                </button>

                                <button
                                    type="button"
                                    disabled={isOperating}
                                    onClick={handlePasteFromClipboard}
                                    className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] flex items-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50"
                                    title="Paste settings JSON from clipboard and apply"
                                >
                                    <ClipboardPaste className="w-3.5 h-3.5 text-emerald-500" />
                                    <span>Paste</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={handleExportJsonFile}
                                    className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] flex items-center gap-1.5 transition-colors cursor-pointer"
                                    title="Export settings to JSON file"
                                >
                                    <Download className="w-3.5 h-3.5 text-indigo-500" />
                                    <span>Export</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={() => fileInputRef.current?.click()}
                                    className="px-2.5 py-1.5 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d] flex items-center gap-1.5 transition-colors cursor-pointer"
                                    title="Import settings from JSON file"
                                >
                                    <Upload className="w-3.5 h-3.5 text-purple-500" />
                                    <span>Import</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={() => setIsJsonExpanded(!isJsonExpanded)}
                                    className={cn(
                                        "px-2.5 py-1.5 text-xs font-medium rounded-r-md flex items-center gap-1.5 transition-colors cursor-pointer",
                                        isJsonExpanded
                                            ? "bg-blue-50 dark:bg-blue-900/40 text-blue-600 dark:text-cyan-300 font-semibold"
                                            : "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#15334d]"
                                    )}
                                    title={isJsonExpanded ? "Hide Raw JSON Editor" : "Show Raw JSON Editor"}
                                >
                                    <FileCode className="w-3.5 h-3.5 text-amber-500" />
                                    <span>Raw Editor</span>
                                </button>
                            </div>
                        </div>

                        {/* Expandable Raw JSON Area */}
                        {isJsonExpanded && (
                            <div className="space-y-2 pt-2 animate-in fade-in">
                                <textarea
                                    value={jsonContent}
                                    onChange={(e) => setJsonContent(e.target.value)}
                                    rows={8}
                                    className="w-full p-2.5 rounded-xl font-mono text-[11px] bg-white dark:bg-[#051421] border border-gray-200 dark:border-[#15334d] text-gray-900 dark:text-emerald-300 focus:outline-none focus:ring-1 focus:ring-blue-500"
                                    placeholder="{\n  // settings.json contents\n}"
                                />
                                <div className="flex justify-end gap-2">
                                    <button
                                        type="button"
                                        disabled={isOperating}
                                        onClick={handleApplyJsonText}
                                        className="px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-700 text-white font-semibold text-xs shadow-xs transition-colors cursor-pointer"
                                    >
                                        Apply JSON to Target
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-between px-5 py-3 border-t border-gray-100 dark:border-[#15334d] bg-gray-50/70 dark:bg-[#091e30]">
                    <span className="text-[11px] text-gray-400">
                        Targeting: <strong className="text-gray-700 dark:text-gray-200">{currentInstanceObj?.config.name || selectedTargetId}</strong>
                    </span>
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-1.5 rounded-xl text-xs font-semibold text-gray-600 dark:text-gray-300 border border-gray-200 dark:border-[#15334d] hover:bg-gray-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                    >
                        {t('common.done', 'Done')}
                    </button>
                </div>
            </div>
        </div>
    );
}
