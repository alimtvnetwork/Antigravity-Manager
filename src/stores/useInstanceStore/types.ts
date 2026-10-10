import type {
    InstanceConfig,
    InstanceStatus,
    AutoProfileSwitcherConfig,
    AutoSwitcherStatus,
} from '../../services/instanceService';

interface InstanceState {
    instances: InstanceStatus[];
    activeInstanceId: string;
    switcherStatus: AutoSwitcherStatus | null;
    isLoading: boolean;
    error: string | null;

    fetchInstances: (silent?: boolean) => Promise<void>;
    fetchSwitcherStatus: () => Promise<void>;
    updateSwitcherConfig: (config: AutoProfileSwitcherConfig) => Promise<void>;
    triggerManualRotation: () => Promise<string>;
    createInstance: (name: string, boundAccountId?: string, fromInstanceId?: string) => Promise<InstanceConfig>;
    copyInstance: (sourceId: string, targetName: string, cloneMode?: string, copyProjects?: boolean) => Promise<InstanceConfig>;
    renameInstance: (instanceId: string, newName: string) => Promise<InstanceConfig>;
    deleteInstance: (instanceId: string) => Promise<void>;
    wipeSession: (instanceId: string) => Promise<void>;
    launchInstance: (instanceId: string) => Promise<void>;
    cloneInstanceExecutable: (instanceId: string) => Promise<string>;
    setInstanceExecutable: (instanceId: string, executablePath?: string) => Promise<void>;
    closeInstance: (instanceId: string) => Promise<void>;
    stopInstance: (instanceId: string) => Promise<void>;
    restartInstance: (instanceId: string) => Promise<InstanceStatus>;
    fastForwardInstance: (instanceId: string) => Promise<string>;
    toggleAutoSwitcher: () => Promise<void>;
    setActiveInstance: (instanceId: string) => Promise<void>;
    setDefaultInstance: (instanceId: string) => Promise<void>;
    switchAccountToInstance: (accountId: string, instanceId?: string) => Promise<void>;
    exportInstancesJson: () => Promise<string>;
    importInstancesJson: (jsonContent: string) => Promise<InstanceConfig[]>;
    smartPlayInstance: (instanceId?: string) => Promise<{ accountEmail: string; instanceName: string }>;
    rotateToNextBestProfile: (sourceInstanceId?: string) => Promise<InstanceStatus>;
    smartRotateProfileAccount: (targetInstanceId?: string) => Promise<{
        accountEmail: string;
        instanceName: string;
        daysUntilRefill: number;
        resumedProjectsCount?: number;
        skippedProjectsCount?: number;
    }>;
    cleanAndRestartWorkspace: () => Promise<string>;
    resumeRecentProjectPrompts: (instanceId?: string) => Promise<instanceService.AutoResumeResult>;
    syncInstance: (instanceId: string) => Promise<void>;
    syncAllInstances: () => Promise<void>;
}
