import { showToast } from '../../common/ToastContainer';
import * as instanceService from '../../../services/instanceService';
import { isTauri } from '../../../utils/env';
import { errorMessage, type SettingsCore } from './useInstanceSettings';

export interface JsonHandlers {
    handleCopyToClipboard: () => Promise<void>;
    handlePasteFromClipboard: () => Promise<void>;
    handleApplyJsonText: () => Promise<void>;
    handleUndo: () => Promise<void>;
    handleRedo: () => Promise<void>;
    handleExportJsonFile: () => Promise<void>;
    handleImportJsonFile: (e: React.ChangeEvent<HTMLInputElement>) => Promise<void>;
}

export function createJsonHandlers(core: SettingsCore): JsonHandlers {
    const {
        instances,
        onInstancesUpdated,
        selectedTargetId,
        jsonContent,
        setJsonContent,
        history,
        historyIndex,
        setHistoryIndex,
        setIsOperating,
        pushHistory,
    } = core;
    void instances;

    const handleCopyToClipboard = async () => {
        try {
            await navigator.clipboard.writeText(jsonContent);
            showToast('Settings JSON copied to clipboard', 'info');
        } catch {
            showToast('Failed to copy to clipboard', 'error');
        }
    };

    const handlePasteFromClipboard = async () => {
        try {
            const text = await navigator.clipboard.readText();
            if (!text.trim()) {
                showToast('Clipboard is empty', 'warning');
                return;
            }
            const parsed = JSON.parse(text) as unknown;
            const formatted = JSON.stringify(parsed, null, 2);

            setIsOperating(true);
            await instanceService.importInstanceSettings(selectedTargetId, formatted);
            pushHistory(formatted);
            setJsonContent(formatted);
            showToast('Settings pasted from clipboard and applied successfully', 'success');
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Invalid JSON in clipboard or apply failed: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

    const handleApplyJsonText = async () => {
        try {
            const parsed = JSON.parse(jsonContent) as unknown;
            const formatted = JSON.stringify(parsed, null, 2);
            setIsOperating(true);
            await instanceService.importInstanceSettings(selectedTargetId, formatted);
            pushHistory(formatted);
            showToast('Settings JSON applied successfully', 'success');
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Invalid JSON: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

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
        } catch (err: unknown) {
            showToast(`Undo failed: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

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
        } catch (err: unknown) {
            showToast(`Redo failed: ${errorMessage(err)}`, 'error');
        } finally {
            setIsOperating(false);
        }
    };

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
        } catch (err: unknown) {
            showToast(`Export failed: ${errorMessage(err)}`, 'error');
        }
    };

    const handleImportJsonFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        try {
            const text = await file.text();
            const parsed = JSON.parse(text) as unknown;
            const formatted = JSON.stringify(parsed, null, 2);

            setIsOperating(true);
            await instanceService.importInstanceSettings(selectedTargetId, formatted);
            pushHistory(formatted);
            setJsonContent(formatted);
            showToast('Imported and applied settings from file', 'success');
            onInstancesUpdated?.();
        } catch (err: unknown) {
            showToast(`Failed to parse/import settings file: ${errorMessage(err)}`, 'error');
        } finally {
            e.target.value = '';
            setIsOperating(false);
        }
    };

    return {
        handleCopyToClipboard,
        handlePasteFromClipboard,
        handleApplyJsonText,
        handleUndo,
        handleRedo,
        handleExportJsonFile,
        handleImportJsonFile,
    };
}
