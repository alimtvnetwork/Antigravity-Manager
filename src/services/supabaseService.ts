import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export interface SupabaseEndpoint {
    id: string;
    name: string;
    url: string;
    api_key: string;
    role: 'root' | 'secondary';
    is_enabled: boolean;
    prune_threshold_mb: number;
    priority: number;
}

export interface SupabaseConfig {
    endpoints: SupabaseEndpoint[];
    node_alias: string;
    is_sync_enabled: boolean;
    auto_prune_root_mb: number;
    auto_prune_secondary_mb: number;
    heartbeat_interval_secs: number;
}

export interface WorkspaceLease {
    account_id: string;
    node_id: string;
    node_alias: string;
    profile_name: string;
    leased_at: number;
    expires_at: number;
}

export interface LeaseResult {
    is_success: boolean;
    error_message?: string;
    owner_node_id?: string;
    owner_alias?: string;
    expires_at?: number;
}

export interface TableVerificationResult {
    endpoint_id: string;
    is_connected: boolean;
    verified_tables: string[];
    missing_tables: string[];
    error_message?: string;
}

export interface DataMigrationSummary {
    is_success: boolean;
    nodes_migrated: number;
    profiles_migrated: number;
    leases_migrated: number;
    commands_migrated: number;
    message: string;
}

export interface LocalNodeInfo {
    node_id: string;
    node_alias: string;
    ip_address: string;
    uptime_seconds: number;
}

export const supabaseService = {
    async getConfig(): Promise<SupabaseConfig> {
        try {
            return await invoke<SupabaseConfig>('get_supabase_config');
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            throw error;
        }
    },

    async saveConfig(config: SupabaseConfig): Promise<void> {
        try {
            await invoke('save_supabase_config', { config });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            throw error;
        }
    },

    async testEndpoint(endpoint: SupabaseEndpoint): Promise<boolean> {
        try {
            return await invoke<boolean>('test_supabase_endpoint', { endpoint });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            return false;
        }
    },

    async checkEndpointTables(endpoint: SupabaseEndpoint): Promise<TableVerificationResult> {
        try {
            return await invoke<TableVerificationResult>('check_supabase_endpoint_tables', { endpoint });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            return {
                endpoint_id: endpoint.id,
                is_connected: false,
                verified_tables: [],
                missing_tables: [],
                error_message: String(error),
            };
        }
    },

    async migrateData(sourceId: string, targetId: string): Promise<DataMigrationSummary> {
        try {
            return await invoke<DataMigrationSummary>('migrate_supabase_data', {
                sourceId,
                targetId,
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            return {
                is_success: false,
                nodes_migrated: 0,
                profiles_migrated: 0,
                leases_migrated: 0,
                commands_migrated: 0,
                message: String(error),
            };
        }
    },

    async getSchemaSql(role: 'root' | 'secondary'): Promise<string> {
        try {
            return await invoke<string>('get_supabase_schema_sql', { role });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            throw error;
        }
    },

    async exportConfig(formatType: 'json' | 'yaml', rounds: number = 4): Promise<string> {
        try {
            return await invoke<string>('export_supabase_config', {
                formatType,
                rounds,
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            throw error;
        }
    },

    async importConfig(content: string): Promise<SupabaseConfig> {
        try {
            return await invoke<SupabaseConfig>('import_supabase_config', { content });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'SupabaseService' });
            throw error;
        }
    },

    async acquireLease(accountId: string, profileName: string, ttlSecs: number = 90): Promise<LeaseResult> {
        try {
            return await invoke<LeaseResult>('acquire_account_lease', {
                accountId,
                profileName,
                ttlSecs,
            });
        } catch (error) {
            return {
                is_success: false,
                error_message: String(error),
            };
        }
    },

    async releaseLease(accountId: string): Promise<void> {
        try {
            await invoke('release_account_lease', { accountId });
        } catch (error) {
            console.error('Failed to release lease:', error);
        }
    },

    async listActiveLeases(): Promise<WorkspaceLease[]> {
        try {
            return await invoke<WorkspaceLease[]>('list_active_account_leases');
        } catch (error) {
            return [];
        }
    },

    async getLocalNodeInfo(): Promise<LocalNodeInfo> {
        try {
            return await invoke<LocalNodeInfo>('get_local_node_info');
        } catch (error) {
            return {
                node_id: 'unknown',
                node_alias: 'Node-Local',
                ip_address: '127.0.0.1',
                uptime_seconds: 0,
            };
        }
    },
};
