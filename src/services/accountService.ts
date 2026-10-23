import i18n from '../i18n';
import { Account, DeviceProfile, DeviceProfileVersion, QuotaData } from '../types/account';
import { request as invoke } from '../utils/request';

export async function updateAccountPriority(accountId: string, priority: number): Promise<void> {
    if (!Number.isInteger(priority) || priority < 1 || priority > 100) {
        throw new Error(i18n.t('accounts.priority_invalid'));
    }
    await invoke('update_account_priority', { accountId, priority });
}

// Check environment (optional)
function ensureTauriEnvironment() {
    // In Web mode, request is also a function, so no error should be thrown here.
    if (typeof invoke !== 'function') {
        throw new Error(i18n.t('common.tauri_api_not_loaded'));
    }
}

export async function listAccounts(): Promise<Account[]> {
    const response = await invoke<any>('list_accounts');
    // If the returned object format is { accounts: [...] }, then take its accounts attribute
    if (response && typeof response === 'object' && Array.isArray(response.accounts)) {
        return response.accounts;
    }
    // Otherwise, return the response content directly (assumed to be an array)
    return response || [];
}

export async function getCurrentAccount(): Promise<Account | null> {
    return await invoke('get_current_account');
}

export async function addAccount(email: string, refreshToken: string): Promise<Account> {
    return await invoke('add_account', { email, refreshToken });
}

export async function deleteAccount(accountId: string): Promise<void> {
    return await invoke('delete_account', { accountId });
}

export async function deleteAccounts(accountIds: string[]): Promise<void> {
    return await invoke('delete_accounts', { accountIds });
}

export async function switchAccount(accountId: string, targetIde?: string): Promise<void> {
    return await invoke('switch_account', { accountId, targetIde });
}

export async function fetchAccountQuota(accountId: string): Promise<QuotaData> {
    return await invoke('fetch_account_quota', { accountId });
}

export interface RefreshStats {
    total: number;
    success: number;
    failed: number;
    details: string[];
}

export async function refreshAllQuotas(): Promise<RefreshStats> {
    return await invoke('refresh_all_quotas');
}

// OAuth
export async function startOAuthLogin(oauthClientKey?: string): Promise<Account> {
    ensureTauriEnvironment();

    try {
        return await invoke('start_oauth_login', oauthClientKey ? { oauthClientKey } : undefined);
    } catch (error) {
        // Enhanced error messages
        if (typeof error === 'string') {
            // If it is a refresh_token missing error, leave it as is (detailed instructions included)
            if (error.includes('Refresh Token') || error.includes('refresh_token')) {
                throw error;
            }
            // Other errors add context
            throw i18n.t('accounts.add.oauth_error', { error });
        }
        throw error;
    }
}

export async function completeOAuthLogin(): Promise<Account> {
    ensureTauriEnvironment();
    try {
        return await invoke('complete_oauth_login');
    } catch (error) {
        if (typeof error === 'string') {
            if (error.includes('Refresh Token') || error.includes('refresh_token')) {
                throw error;
            }
            throw i18n.t('accounts.add.oauth_error', { error });
        }
        throw error;
    }
}

export async function cancelOAuthLogin(): Promise<void> {
    ensureTauriEnvironment();
    return await invoke('cancel_oauth_login');
}

export interface OAuthClientInfo {
    key: string;
    label: string;
    client_id: string;
    is_active: boolean;
    is_builtin: boolean;
}

export async function listOAuthClients(): Promise<OAuthClientInfo[]> {
    const res = await invoke<{ clients: OAuthClientInfo[] }>('list_oauth_clients');
    if (res && Array.isArray(res.clients)) return res.clients;
    return (res as unknown as OAuthClientInfo[]) || [];
}

export async function getActiveOAuthClient(): Promise<string> {
    const res = await invoke<{ client_key: string } | string>('get_active_oauth_client');
    if (typeof res === 'string') return res;
    return res?.client_key || '';
}

export async function setActiveOAuthClient(clientKey: string): Promise<void> {
    return await invoke('set_active_oauth_client', { clientKey });
}

// import
export async function importV1Accounts(): Promise<Account[]> {
    return await invoke('import_v1_accounts');
}

export async function importFromDb(targetIde?: string): Promise<Account[]> {
    const res = await invoke<any>('import_from_db', { targetIde });
    if (Array.isArray(res)) return res;
    return res ? [res] : [];
}

export async function importFromCustomDb(path: string): Promise<Account> {
    return await invoke('import_custom_db', { path });
}

export async function syncAccountFromDb(): Promise<Account | null> {
    return await invoke('sync_account_from_db', { _suppressGlobalModal: true });
}

export async function toggleProxyStatus(accountId: string, enable: boolean, reason?: string): Promise<void> {
    return await invoke('toggle_proxy_status', { accountId, enable, reason });
}

/**
 * Reorder account list
 * @param accountIds Array of account IDs in new order
 */
export async function reorderAccounts(accountIds: string[]): Promise<void> {
    return await invoke('reorder_accounts', { accountIds });
}

// Device fingerprint related
export interface DeviceProfilesResponse {
    current_storage?: DeviceProfile;
    history?: DeviceProfileVersion[];
    baseline?: DeviceProfile;
}

export async function getDeviceProfiles(accountId: string): Promise<DeviceProfilesResponse> {
    return await invoke('get_device_profiles', { accountId });
}

export async function bindDeviceProfile(accountId: string, mode: 'capture' | 'generate'): Promise<DeviceProfile> {
    return await invoke('bind_device_profile', { accountId, mode });
}

export async function restoreOriginalDevice(): Promise<string> {
    return await invoke('restore_original_device');
}

export async function listDeviceVersions(accountId: string): Promise<DeviceProfilesResponse> {
    return await invoke('list_device_versions', { accountId });
}

export async function restoreDeviceVersion(accountId: string, versionId: string): Promise<DeviceProfile> {
    return await invoke('restore_device_version', { accountId, versionId });
}

export async function deleteDeviceVersion(accountId: string, versionId: string): Promise<void> {
    return await invoke('delete_device_version', { accountId, versionId });
}

export async function openDeviceFolder(): Promise<void> {
    return await invoke('open_device_folder');
}

export async function previewGenerateProfile(): Promise<DeviceProfile> {
    return await invoke('preview_generate_profile');
}

export async function bindDeviceProfileWithProfile(accountId: string, profile: DeviceProfile): Promise<DeviceProfile> {
    return await invoke('bind_device_profile_with_profile', { accountId, profile });
}

// Preheating related
export async function warmUpAllAccounts(): Promise<string> {
    return await invoke('warm_up_all_accounts');
}

export async function warmUpAccount(accountId: string): Promise<string> {
    return await invoke('warm_up_account', { accountId });
}

// Export account related
export interface ExportAccountItem {
    email: string;
    refresh_token: string;
}

export interface ExportAccountsResponse {
    accounts: ExportAccountItem[];
}

export async function exportAccounts(accountIds: string[]): Promise<ExportAccountsResponse> {
    return await invoke('export_accounts', { accountIds });
}

// Custom label related
export async function updateAccountLabel(accountId: string, label: string): Promise<void> {
    return await invoke('update_account_label', { accountId, label });
}

// Fleet Deployment via GitMap
export interface FleetDeployResult {
    success: boolean;
    message: string;
    raw_output: string;
    rawOutput?: string;
}

export async function checkGitmapAvailable(): Promise<boolean> {
    return await invoke('check_gitmap_available');
}

export async function deployAccountsToFleet(
    includeMain?: boolean,
    exceptNodes?: string[]
): Promise<FleetDeployResult> {
    return await invoke('deploy_accounts_to_fleet', { includeMain, exceptNodes });
}

export async function toggleAccountDisabled(
    accountId: string,
    disabled: boolean,
    reason?: string
): Promise<void> {
    return await invoke('toggle_account_disabled', { accountId, disabled, reason });
}

export async function setAccountsDisabled(
    accountIds: string[],
    disabled: boolean,
    reason?: string
): Promise<void> {
    return await invoke('set_accounts_disabled', { accountIds, disabled, reason });
}
