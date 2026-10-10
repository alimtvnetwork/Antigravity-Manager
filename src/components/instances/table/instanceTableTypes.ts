import type { InstanceStatus } from '../../../services/instanceService';

export type InstanceActionType =
    | 'launch'
    | 'stop'
    | 'restart'
    | 'switch'
    | 'fast-forward'
    | 'wipe'
    | 'delete'
    | 'sync'
    | null;

export interface InstanceStatusWithDataDir extends InstanceStatus {
    data_dir?: string;
}

export interface InstanceTableProps {
    instances: InstanceStatus[];
    activeInstanceId?: string | null;
    searchQuery?: string;
    actionState?: Record<string, InstanceActionType>;
    onLaunch: (id: string) => void;
    onStop: (id: string) => void;
    onRestart: (id: string) => void;
    onSwitch: (id: string) => void;
    onFastForward: (id: string) => void;
    onAudit?: (id: string, name: string) => void;
    onSync?: (id: string) => void;
    syncingInstanceIds?: Record<string, boolean>;
    onSettings: (id: string) => void;
    onClone: (id: string, name: string) => void;
    onDelete: (id: string) => void;
    onOpenPromptTree: (instanceId: string) => void;
    onSetActive: (id: string) => void;
    onSetDefault?: (id: string) => void;
}
