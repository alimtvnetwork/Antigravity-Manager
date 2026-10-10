export interface CliSyncCardProps {
    proxyUrl: string;
    apiKey: string;
    className?: string;
}

export type CliAppType =
    | 'Claude'
    | 'Codex'
    | 'JeikCode'
    | 'GrokBuild'
    | 'Hermes'
    | 'OpenClaw'
    | 'Gemini'
    | 'OpenCode'
    | 'Droid';

export interface CliStatus {
    installed: boolean;
    version: string | null;
    is_synced: boolean;
    has_backup: boolean;
    current_base_url: string | null;
    files: string[];
    synced_count?: number;
}

export interface ViewingConfig {
    app: CliAppType;
    content: string;
    fileName: string;
    allFiles: string[];
}

/** CLI apps that require the `/v1` path suffix on the proxy URL. */
export const V1_PATH_APPS: ReadonlySet<CliAppType> = new Set<CliAppType>([
    'Codex',
    'OpenCode',
    'JeikCode',
    'GrokBuild',
    'Hermes',
    'OpenClaw',
]);

/** CLI apps with a dedicated sync-status backend command. */
export const DEDICATED_STATUS_COMMAND: Record<string, string> = {
    Droid: 'get_droid_sync_status',
    OpenCode: 'get_opencode_sync_status',
    Hermes: 'get_hermes_sync_status',
    OpenClaw: 'get_openclaw_sync_status',
};

export const ALL_CLI_APPS: CliAppType[] = [
    'Claude',
    'Codex',
    'JeikCode',
    'GrokBuild',
    'Hermes',
    'OpenClaw',
    'Gemini',
    'OpenCode',
    'Droid',
];

export const DEFAULT_SELECTED_MODELS: Record<CliAppType, string> = {
    Claude: 'claude-3-5-sonnet-latest',
    Codex: 'gpt-4o',
    JeikCode: 'gemini-3.8-flash-high',
    GrokBuild: 'gemini-3.8-flash',
    Gemini: 'gemini-1.5-pro',
    OpenCode: '',
    Droid: '',
    Hermes: '',
    OpenClaw: '',
};
