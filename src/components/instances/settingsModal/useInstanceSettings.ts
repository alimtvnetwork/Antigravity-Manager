import { useState, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { showToast } from '../../common/ToastContainer';
import * as instanceService from '../../../services/instanceService';
import { useErrorStore } from '../../../stores/error-store';
import type { InstanceStatus } from '../../../services/instanceService';
import type { InstanceClipboardBuffer, InstanceSettingsApi } from './instanceSettingsTypes';
import { createSyncHandlers } from './settingsSyncHandlers';
import { createJsonHandlers } from './settingsJsonHandlers';

export function errorMessage(e: unknown): string {
    if (e instanceof Error) return e.message;
    return String(e);
}

export interface SettingsCore {
    t: (key: string, fallback?: string) => string;
    instances: InstanceStatus[];
    onInstancesUpdated?: () => Promise<void> | void;
    selectedTargetId: string;
    copySettingsSourceId: string;
    setCopySettingsSourceId: (v: string) => void;
    copyProjectsSourceId: string;
    setCopyProjectsSourceId: (v: string) => void;
    replicationSourceId: string;
    setReplicationSourceId: (v: string) => void;
    clipboardBuffer: InstanceClipboardBuffer | null;
    setClipboardBuffer: (v: InstanceClipboardBuffer | null) => void;
    jsonContent: string;
    setJsonContent: (v: string) => void;
    history: string[];
    setHistory: (v: string[]) => void;
    historyIndex: number;
    setHistoryIndex: (v: number) => void;
    setIsTurboMode: (v: boolean) => void;
    setIsAlwaysProceed: (v: boolean) => void;
    setIsLoading: (v: boolean) => void;
    setIsOperating: (v: boolean) => void;
    isTurboMode: boolean;
    isAlwaysProceed: boolean;
    loadSettings: (instanceId: string) => Promise<void>;
    pushHistory: (content: string) => void;
    persistClipboardBuffer: (buffer: InstanceClipboardBuffer | null, source: string) => void;
}

export function useInstanceSettings(
    isOpen: boolean,
    onClose: () => void,
    targetInstance: InstanceStatus | null | undefined,
    instances: InstanceStatus[],
    onInstancesUpdated?: () => Promise<void> | void
): InstanceSettingsApi {
    const { t } = useTranslation();
    const [selectedTargetId, setSelectedTargetId] = useState<string>('default');
    const [copySettingsSourceId, setCopySettingsSourceId] = useState<string>('');
    const [copyProjectsSourceId, setCopyProjectsSourceId] = useState<string>('');
    const [replicationSourceId, setReplicationSourceId] = useState<string>('');
    const [clipboardBuffer, setClipboardBuffer] = useState<InstanceClipboardBuffer | null>(() => {
        try {
            const raw = localStorage.getItem('agm_instance_clipboard_buffer');
            return raw ? (JSON.parse(raw) as InstanceClipboardBuffer) : null;
        } catch {
            return null;
        }
    });
    const [isTurboMode, setIsTurboMode] = useState<boolean>(true);
    const [isAlwaysProceed, setIsAlwaysProceed] = useState<boolean>(true);

    const [jsonContent, setJsonContent] = useState<string>('');
    const [isJsonExpanded, setIsJsonExpanded] = useState<boolean>(false);
    const [history, setHistory] = useState<string[]>([]);
    const [historyIndex, setHistoryIndex] = useState<number>(-1);

    const [isLoading, setIsLoading] = useState<boolean>(false);
    const [isOperating, setIsOperating] = useState<boolean>(false);

    const fileInputRef = useRef<HTMLInputElement>(null);

    const persistClipboardBuffer = (buffer: InstanceClipboardBuffer | null, source: string) => {
        try {
            if (buffer) {
                localStorage.setItem('agm_instance_clipboard_buffer', JSON.stringify(buffer));
            } else {
                localStorage.removeItem('agm_instance_clipboard_buffer');
            }
            setClipboardBuffer(buffer);
        } catch (e) {
            // Best-effort clipboard persist; localStorage may throw in restricted contexts. Tracked.
            useErrorStore.getState().trackWarning(e, {
                source,
                triggerAction: 'persist_clipboard_buffer',
            });
        }
    };

    const loadSettings = async (instanceId: string) => {
        if (!instanceId) return;
        setIsLoading(true);
        try {
            const rawJson = await instanceService.exportInstanceSettings(instanceId);
            let formatted = rawJson;
            try {
                const parsed = JSON.parse(rawJson) as Record<string, unknown>;
                formatted = JSON.stringify(parsed, null, 2);
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
            setHistory([formatted]);
            setHistoryIndex(0);
        } catch (err: unknown) {
            // Tracked in the error module; safe default settings applied, user can still edit and save.
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceSettingsModal.loadSettings',
                triggerAction: 'load_instance_settings',
            });
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

    useEffect(() => {
        if (isOpen) {
            const initialId = targetInstance?.config.id || instances[0]?.config.id || 'default';
            setSelectedTargetId(initialId);

            const otherInstances = instances.filter((i) => i.config.id !== initialId);
            if (otherInstances.length > 0) {
                setCopySettingsSourceId(otherInstances[0].config.id);
                setCopyProjectsSourceId(otherInstances[0].config.id);
                setReplicationSourceId(otherInstances[0].config.id);
            } else {
                setCopySettingsSourceId('');
                setCopyProjectsSourceId('');
                setReplicationSourceId('');
            }

            try {
                const raw = localStorage.getItem('agm_instance_clipboard_buffer');
                if (raw) {
                    setClipboardBuffer(JSON.parse(raw) as InstanceClipboardBuffer);
                }
            } catch (e) {
                // Best-effort clipboard restore; localStorage may throw in restricted contexts. Tracked, buffer stays empty.
                useErrorStore.getState().trackWarning(e, {
                    source: 'InstanceSettingsModal.syncClipboardBuffer',
                    triggerAction: 'read_clipboard_buffer',
                });
            }

            loadSettings(initialId);
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [isOpen, targetInstance, instances]);

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen) {
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    const handleTargetChange = (newTargetId: string) => {
        setSelectedTargetId(newTargetId);
        const otherInstances = instances.filter((i) => i.config.id !== newTargetId);
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
        } catch (err: unknown) {
            showToast(`Failed to update Turbo Mode: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

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
        } catch (err: unknown) {
            showToast(`Failed to update Plan Review: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

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
        } catch (err: unknown) {
            showToast(`Failed to enforce defaults: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    const core: SettingsCore = {
        t,
        instances,
        onInstancesUpdated,
        selectedTargetId,
        copySettingsSourceId,
        setCopySettingsSourceId,
        copyProjectsSourceId,
        setCopyProjectsSourceId,
        replicationSourceId,
        setReplicationSourceId,
        clipboardBuffer,
        setClipboardBuffer,
        jsonContent,
        setJsonContent,
        history,
        setHistory,
        historyIndex,
        setHistoryIndex,
        setIsTurboMode,
        setIsAlwaysProceed,
        setIsLoading,
        setIsOperating,
        isTurboMode,
        isAlwaysProceed,
        loadSettings,
        pushHistory,
        persistClipboardBuffer,
    };

    const syncHandlers = createSyncHandlers(core);
    const jsonHandlers = createJsonHandlers(core);

    const canUndo = historyIndex > 0;
    const canRedo = historyIndex < history.length - 1;
    const currentInstanceObj = instances.find((i) => i.config.id === selectedTargetId);
    const candidateSources = instances.filter((i) => i.config.id !== selectedTargetId);
    const currentSourceObj = instances.find((i) => i.config.id === replicationSourceId) || candidateSources[0];

    return {
        selectedTargetId,
        copySettingsSourceId,
        setCopySettingsSourceId,
        copyProjectsSourceId,
        setCopyProjectsSourceId,
        replicationSourceId,
        setReplicationSourceId,
        clipboardBuffer,
        isTurboMode,
        isAlwaysProceed,
        jsonContent,
        setJsonContent,
        isJsonExpanded,
        setIsJsonExpanded,
        isLoading,
        isOperating,
        canUndo,
        canRedo,
        fileInputRef,
        currentInstanceObj,
        candidateSources,
        currentSourceObj,
        currentTargetObj: currentInstanceObj,
        handleTargetChange,
        handleSetTurboMode,
        handleSetPlanReview,
        handleEnforceDefaults,
        ...syncHandlers,
        ...jsonHandlers,
        persistClipboardBuffer,
    };
}
