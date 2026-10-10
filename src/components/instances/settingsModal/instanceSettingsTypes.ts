import type { InstanceStatus } from '../../../services/instanceService';

export interface InstanceClipboardBuffer {
    sourceId: string;
    sourceName: string;
    hasSettings: boolean;
    hasWorkspaces: boolean;
    copiedAt: number;
}

export interface InstanceSettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
    targetInstance?: InstanceStatus | null;
    instances: InstanceStatus[];
    onInstancesUpdated?: () => Promise<void> | void;
}

export interface InstanceSettingsApi {
    selectedTargetId: string;
    copySettingsSourceId: string;
    setCopySettingsSourceId: (v: string) => void;
    copyProjectsSourceId: string;
    setCopyProjectsSourceId: (v: string) => void;
    replicationSourceId: string;
    setReplicationSourceId: (v: string) => void;
    clipboardBuffer: InstanceClipboardBuffer | null;
    isTurboMode: boolean;
    isAlwaysProceed: boolean;
    jsonContent: string;
    setJsonContent: (v: string) => void;
    isJsonExpanded: boolean;
    setIsJsonExpanded: (v: boolean) => void;
    isLoading: boolean;
    isOperating: boolean;
    canUndo: boolean;
    canRedo: boolean;
    fileInputRef: React.RefObject<HTMLInputElement | null>;
    currentInstanceObj?: InstanceStatus;
    candidateSources: InstanceStatus[];
    currentSourceObj?: InstanceStatus;
    currentTargetObj?: InstanceStatus;
    handleTargetChange: (id: string) => void;
    handleSetTurboMode: (applyToAll: boolean) => Promise<void>;
    handleSetPlanReview: (applyToAll: boolean, value: boolean) => Promise<void>;
    handleEnforceDefaults: (applyToAll: boolean) => Promise<void>;
    handleCopySettings: () => Promise<void>;
    handleCopyProjects: () => Promise<void>;
    handleCopyBothDirect: (sourceId: string) => Promise<void>;
    handleMoveSettings: () => Promise<void>;
    handleMoveProjects: () => Promise<void>;
    handleMoveBothDirect: (sourceId: string) => Promise<void>;
    handlePasteFromBuffer: (mode: 'both' | 'settings' | 'workspaces') => Promise<void>;
    handleCopyToClipboard: () => Promise<void>;
    handlePasteFromClipboard: () => Promise<void>;
    handleApplyJsonText: () => Promise<void>;
    handleUndo: () => Promise<void>;
    handleRedo: () => Promise<void>;
    handleExportJsonFile: () => Promise<void>;
    handleImportJsonFile: (e: React.ChangeEvent<HTMLInputElement>) => Promise<void>;
    persistClipboardBuffer: (buffer: InstanceClipboardBuffer | null, source: string) => void;
}
