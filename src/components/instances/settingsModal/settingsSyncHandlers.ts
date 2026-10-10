import { showToast } from '../../common/ToastContainer';
import * as instanceService from '../../../services/instanceService';
import type { InstanceClipboardBuffer } from './instanceSettingsTypes';
import { errorMessage, type SettingsCore } from './useInstanceSettings';

function makeBuffer(srcId: string, srcName: string, hasSettings: boolean, hasWorkspaces: boolean): InstanceClipboardBuffer {
    return {
        sourceId: srcId,
        sourceName: srcName,
        hasSettings,
        hasWorkspaces,
        copiedAt: Date.now(),
    };
}

export interface SyncHandlers {
    handleCopySettings: () => Promise<void>;
    handleCopyProjects: () => Promise<void>;
    handleCopyBothDirect: (sourceId: string) => Promise<void>;
    handleMoveSettings: () => Promise<void>;
    handleMoveProjects: () => Promise<void>;
    handleMoveBothDirect: (sourceId: string) => Promise<void>;
    handlePasteFromBuffer: (mode: 'both' | 'settings' | 'workspaces') => Promise<void>;
}

export function createSyncHandlers(core: SettingsCore): SyncHandlers {
    const {
        t,
        instances,
        onInstancesUpdated,
        selectedTargetId,
        copySettingsSourceId,
        copyProjectsSourceId,
        replicationSourceId,
        clipboardBuffer,
        setIsOperating,
        loadSettings,
        persistClipboardBuffer,
    } = core;
    void t;

    const handleCopySettings = async () => {
        if (!copySettingsSourceId || !selectedTargetId) return;
        setIsOperating(true);
        try {
            await instanceService.copyInstanceSettings(copySettingsSourceId, selectedTargetId);
            const src = instances.find((i) => i.config.id === copySettingsSourceId);
            if (src) {
                persistClipboardBuffer(makeBuffer(src.config.id, src.config.name, true, false), 'InstanceSettingsModal.copySettings');
            }
            showToast(`Settings synchronized from source to ${selectedTargetId}`, 'success');
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Failed to copy settings: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    const handleCopyProjects = async () => {
        if (!copyProjectsSourceId || !selectedTargetId) return;
        setIsOperating(true);
        try {
            const copiedCount = await instanceService.copyInstanceProjects(copyProjectsSourceId, selectedTargetId);
            const src = instances.find((i) => i.config.id === copyProjectsSourceId);
            if (src) {
                persistClipboardBuffer(makeBuffer(src.config.id, src.config.name, false, true), 'InstanceSettingsModal.copyWorkspaces');
            }
            showToast(`Successfully copied ${copiedCount} workspace folder(s) & project states`, 'success');
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Failed to copy projects: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    const handleCopyBothDirect = async (sourceId: string) => {
        if (!sourceId || !selectedTargetId) return;
        if (sourceId === selectedTargetId) {
            showToast('Source and target profiles are identical', 'warning');
            return;
        }
        setIsOperating(true);
        try {
            const src = instances.find((i) => i.config.id === sourceId);
            if (src) {
                persistClipboardBuffer(makeBuffer(src.config.id, src.config.name, true, true), 'InstanceSettingsModal.copySettingsAndWorkspaces');
            }
            const { projectsCount } = await instanceService.copyInstanceBoth(sourceId, selectedTargetId);
            showToast(`Successfully copied settings & ${projectsCount} workspace folder(s) to target profile`, 'success');
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Failed to copy both: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    const handleMoveSettings = async () => {
        const sourceId = copySettingsSourceId || replicationSourceId;
        if (!sourceId || !selectedTargetId) return;
        if (sourceId === selectedTargetId) {
            showToast('Source and target profiles are identical', 'warning');
            return;
        }
        setIsOperating(true);
        try {
            await instanceService.copyInstanceSettings(sourceId, selectedTargetId);
            await instanceService.enforceDefaultSettings(sourceId);
            showToast('Settings moved to target profile (source reset to baseline)', 'success');
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Failed to move settings: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    const handleMoveProjects = async () => {
        const sourceId = copyProjectsSourceId || replicationSourceId;
        if (!sourceId || !selectedTargetId) return;
        if (sourceId === selectedTargetId) {
            showToast('Source and target profiles are identical', 'warning');
            return;
        }
        setIsOperating(true);
        try {
            const count = await instanceService.copyInstanceProjects(sourceId, selectedTargetId);
            await instanceService.wipeInstanceSession(sourceId);
            showToast(`Moved ${count} workspace folder(s) to target profile (source wiped)`, 'success');
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Failed to move projects: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    const handleMoveBothDirect = async (sourceId: string) => {
        if (!sourceId || !selectedTargetId) return;
        if (sourceId === selectedTargetId) {
            showToast('Source and target profiles are identical', 'warning');
            return;
        }
        setIsOperating(true);
        try {
            const { projectsCount } = await instanceService.copyInstanceBoth(sourceId, selectedTargetId);
            await instanceService.enforceDefaultSettings(sourceId);
            await instanceService.wipeInstanceSession(sourceId);
            showToast(`Moved settings & ${projectsCount} workspace folder(s) to target`, 'success');
            await loadSettings(selectedTargetId);
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Failed to move both: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

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
        } catch (err: unknown) {
            showToast(`Paste failed: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    return {
        handleCopySettings,
        handleCopyProjects,
        handleCopyBothDirect,
        handleMoveSettings,
        handleMoveProjects,
        handleMoveBothDirect,
        handlePasteFromBuffer,
    };
}
