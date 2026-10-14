/**
 * Audit recorder — zustand store holding recording state and the
 * cached entry list for the panel. The global listener hook
 * (`src/hooks/useAuditRecorder.ts`) reads `isRecording` via `getState()`
 * so events never trigger re-renders.
 */
import { create } from 'zustand';
import { useErrorStore } from '../../stores/error-store';
import { showToast } from '../common/ToastContainer';
import {
    auditStart,
    auditStop,
    auditIsRecording,
    auditList,
    auditClear,
} from './audit-client';
import type { AuditActor, AuditEntry, AuditResult } from './audit-types';

export type AuditActorFilter = 'all' | AuditActor;
export type AuditResultFilter = 'all' | AuditResult;

interface AuditStoreState {
    isRecording: boolean;
    recordingSince: number | null;
    entries: AuditEntry[];
    isLoading: boolean;
    isPanelOpen: boolean;
    actorFilter: AuditActorFilter;
    resultFilter: AuditResultFilter;

    /** Reconcile with backend state (called once when the hook mounts). */
    syncRecordingState: () => Promise<void>;
    startRecording: () => Promise<void>;
    stopRecording: () => Promise<void>;
    toggleRecording: () => Promise<void>;
    refreshEntries: () => Promise<void>;
    clearEntries: () => Promise<void>;
    setPanelOpen: (open: boolean) => void;
    setActorFilter: (filter: AuditActorFilter) => void;
    setResultFilter: (filter: AuditResultFilter) => void;
}

export const useAuditStore = create<AuditStoreState>((set, get) => ({
    isRecording: false,
    recordingSince: null,
    entries: [],
    isLoading: false,
    isPanelOpen: false,
    actorFilter: 'all',
    resultFilter: 'all',

    syncRecordingState: async () => {
        try {
            const recording = await auditIsRecording();
            set({
                isRecording: recording,
                recordingSince: recording ? Date.now() : get().recordingSince,
            });
        } catch (error) {
            // auditIsRecording already routed the failure into the error
            // module; the store simply keeps its last known state.
            useErrorStore.getState().trackWarning(error, {
                source: 'audit/audit-store',
                triggerAction: 'syncRecordingState',
            });
        }
    },

    startRecording: async () => {
        const ok = await auditStart();
        if (ok) {
            set({ isRecording: true, recordingSince: Date.now() });
            showToast('Audit recording started', 'success');
        } else {
            showToast('Could not start audit recording', 'error');
        }
    },

    stopRecording: async () => {
        const ok = await auditStop();
        if (ok) {
            set({ isRecording: false });
            showToast('Audit recording stopped', 'info');
            // Pull the final entries so the panel shows the full session.
            await get().refreshEntries();
        } else {
            showToast('Could not stop audit recording', 'error');
        }
    },

    toggleRecording: async () => {
        if (get().isRecording) {
            await get().stopRecording();
        } else {
            await get().startRecording();
        }
    },

    refreshEntries: async () => {
        set({ isLoading: true });
        try {
            const entries = await auditList();
            // null = IPC failure, already tracked; keep stale list rather
            // than wiping it.
            if (entries !== null) {
                set({ entries });
            }
        } finally {
            set({ isLoading: false });
        }
    },

    clearEntries: async () => {
        const ok = await auditClear();
        if (ok) {
            set({ entries: [] });
            showToast('Audit entries cleared', 'info');
        } else {
            showToast('Could not clear audit entries', 'error');
        }
    },

    setPanelOpen: (open: boolean) => {
        set({ isPanelOpen: open });
        if (open) {
            // Fire-and-forget refresh; failures are tracked inside refreshEntries.
            void get().refreshEntries().catch((error) => {
                useErrorStore.getState().trackWarning(error, {
                    source: 'audit/audit-store',
                    triggerAction: 'setPanelOpen.refreshEntries',
                });
            });
        }
    },

    setActorFilter: (filter: AuditActorFilter) => set({ actorFilter: filter }),
    setResultFilter: (filter: AuditResultFilter) => set({ resultFilter: filter }),
}));
