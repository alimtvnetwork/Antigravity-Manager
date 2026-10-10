import type { TFunction } from 'i18next';
import type {
    AppConfig,
    ProxyConfig,
    StickySessionConfig,
    ExperimentalConfig,
} from '../../types/config';

export interface ProxyStatus {
    running: boolean;
    port: number;
    base_url: string;
    active_accounts: number;
}

export interface CustomPreset {
    id: string;
    name: string;
    description: string;
    mappings: Record<string, string>;
}

export type MenuTab = 'settings' | 'cli' | 'protocols';

export type CloudflaredMode = 'quick' | 'auth';

export interface CloudflaredStatus {
    installed: boolean;
    version?: string;
    running: boolean;
    url?: string;
    error?: string;
}

export interface CloudflaredControls {
    cfStatus: CloudflaredStatus;
    cfLoading: boolean;
    cfMode: CloudflaredMode;
    cfToken: string;
    cfUseHttp2: boolean;
    setCfMode: (mode: CloudflaredMode) => void;
    setCfToken: (token: string) => void;
    setCfUseHttp2: (useHttp2: boolean) => void;
    loadCfStatus: () => Promise<void>;
    handleCfInstall: () => Promise<void>;
    handleCfToggle: (enable: boolean) => Promise<void>;
    handleCfCopyUrl: () => Promise<void>;
}

export interface ProxyModelInfo {
    id: string;
    name: string;
    group: string;
    icon: React.ReactNode;
    desc: string;
}

export type ProtocolKind = 'openai' | 'anthropic' | 'gemini';

export interface AvailableAccount {
    id: string;
    email: string;
}

export interface FixedAccountControls {
    preferredAccountId: string | null;
    availableAccounts: AvailableAccount[];
    handleSetPreferredAccount: (accountId: string | null) => Promise<void>;
}

export interface PresetManager {
    defaultPresets: CustomPreset[];
    presetOptions: CustomPreset[];
    customPresets: CustomPreset[];
    selectedPreset: string;
    setSelectedPreset: (id: string) => void;
    newPresetName: string;
    setNewPresetName: (name: string) => void;
    isPresetManagerOpen: boolean;
    setIsPresetManagerOpen: (open: boolean) => void;
    handleSaveCurrentAsPreset: () => void;
    handleDeletePreset: (id: string) => void;
    handleApplyPresets: () => Promise<void>;
}

export interface CredentialEditing {
    isEditingApiKey: boolean;
    tempApiKey: string;
    setTempApiKey: (value: string) => void;
    handleEditApiKey: () => void;
    handleSaveApiKey: () => void;
    handleCancelEditApiKey: () => void;
    isEditingAdminPassword: boolean;
    tempAdminPassword: string;
    setTempAdminPassword: (value: string) => void;
    handleEditAdminPassword: () => void;
    handleSaveAdminPassword: () => void;
    handleCancelEditAdminPassword: () => void;
    handleGenerateApiKey: () => void;
    executeGenerateApiKey: () => Promise<void>;
    isRegenerateKeyConfirmOpen: boolean;
    setIsRegenerateKeyConfirmOpen: (open: boolean) => void;
}

export type ProxyConfigUpdater = (updates: Partial<ProxyConfig>) => void;
export type SchedulingConfigUpdater = (updates: Partial<StickySessionConfig>) => void;
export type ExperimentalConfigUpdater = (updates: Partial<ExperimentalConfig>) => void;
export type SaveConfigFn = (newConfig: AppConfig) => Promise<void>;

/**
 * Shared base props for ApiProxy sub-sections.
 * Each section takes what it needs from this shape.
 */
export interface SectionBaseProps {
    config: AppConfig;
    t: TFunction;
    updateProxyConfig: ProxyConfigUpdater;
    saveConfig: SaveConfigFn;
    copied: string | null;
    copyToClipboardHandler: (text: string, label: string) => void;
}
