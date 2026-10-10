import type { Dispatch, SetStateAction, MouseEvent } from 'react';
import type { getProfileInfo, OpencodeProviderSummary } from '../../utils/opencodeProfiles';

export interface ManagedApiKey {
    id: string;
    key: string;
    name: string;
    baseUrl: string;
    createdAt: number;
    lastUsedAt: number;
    lastStatus?: 'ok' | 'bad' | 'unknown';
    lastRemaining?: string;
    models?: string[];
}

export interface UsageSummary {
    remaining: string;
    used: string;
    todayRequests: string;
    todayTokens: string;
    totalRequests: string;
    totalTokens: string;
    unit: string;
    isValid: boolean;
}

export type CliSyncApp = 'Codex' | 'Claude' | 'Gemini';

export type OpenCodeProfileInfo = ReturnType<typeof getProfileInfo>;

export interface UseApiKeyFunResult {
    apiKey: string;
    baseUrl: string;
    showApiKey: boolean;
    setShowApiKey: Dispatch<SetStateAction<boolean>>;
    querying: boolean;
    usage: UsageSummary | null;
    models: string[];
    queryError: string | null;
    modelsError: string | null;
    syncingKey: string | null;
    managedKeys: ManagedApiKey[];
    editingId: string | null;
    setEditingId: Dispatch<SetStateAction<string | null>>;
    editNameValue: string;
    setEditNameValue: Dispatch<SetStateAction<string>>;
    runQuery: (keyToQuery: string, urlToQuery: string) => Promise<void>;
    fetchOpencodeProviders: () => Promise<OpencodeProviderSummary[]>;
    getModelsForKey: (targetKey: string, targetUrl?: string) => string[] | undefined;
    profileInfo: (key: string, url: string, modelIds?: string[]) => OpenCodeProfileInfo;
    handleCopy: (text: string) => Promise<void>;
    handleSyncCli: (app: CliSyncApp) => Promise<void>;
    handleToggleOpenCodeProfile: (targetKey: string, targetUrl: string, explicitModels?: string[]) => Promise<void>;
    handleSyncOpenCode: () => Promise<void>;
    handleDeleteKey: (id: string, e: MouseEvent) => void;
    startRename: (item: ManagedApiKey, e: MouseEvent) => void;
    saveRename: (id: string) => void;
    handleApiKeyChange: (value: string) => void;
    handleSelectKey: (item: ManagedApiKey) => void;
}
