import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';
import { copyToClipboard } from '../utils/clipboard';

export interface SshGitmapStatus {
    gitmap_installed: boolean;
    gitmap_version: string | null;
    gitmap_path: string | null;
    install_command: string;
    ssh_config_path: string;
    connections_json_path: string;
    rsa_public_key_pem: string | null;
    rsa_key_fingerprint?: string | null;
    total_nodes: number;
}

export interface SshNodeRecord {
    id?: string;
    alias: string;
    ip_address: string;
    host?: string;
    port: number;
    username: string;
    os: string;
    auth_method: string;
    key_path: string | null;
    has_encrypted_password: boolean;
    has_password?: boolean;
    encrypted_password_preview?: string;
    encryption_algo?: string;
    source?: string;
    last_status?: string;
}

export interface SaveSshNodeRsaPayload {
    alias: string;
    ip_address: string;
    port: number;
    username: string;
    os?: string;
    key_path?: string;
    plaintext_password?: string | null;
}

export interface ClusterInstanceBalanceItem {
    machine_alias: string;
    ip_address: string;
    machine_ip?: string;
    is_local: boolean;
    online: boolean;
    instance_id: string;
    instance_name: string;
    running: boolean;
    pid?: number | null;
    bound_email: string | null;
    tier?: string | null;
    quota_4h_pct?: number | null;
    quota_weekly_pct?: number | null;
    last_updated?: string;
    error?: string | null;
    error_message?: string | null;
    latency_ms?: number | null;
    status_label?: string;
    active_model?: string;
}

export interface RemotePromptDispatchPayload {
    node_alias: string;
    instance_target: string;
    instance_name?: string;
    prompt_text: string;
    conversation_id?: string | null;
}

export interface RemotePromptDispatchResult {
    node_alias: string;
    instance_target: string;
    instance_name?: string;
    success: boolean;
    command_executed: string;
    stdout: string;
    stderr: string;
    duration_ms: number;
    message?: string;
    dispatch_route?: string;
}

export interface NodeSyncStats {
    total: number;
    matched: number;
    updated: number;
    inserted: number;
    unchanged: number;
}

export interface MeshDeploySummary {
    gathered_keys?: Array<{
        source_node: string;
        key_name: string;
        public_key: string;
    }>;
    node_reports?: Array<{
        alias: string;
        ip_address: string;
        username: string;
        os: string;
        online: boolean;
        keys_deployed: number;
        batch_auth_verified: boolean;
        status: string;
        detail: string;
    }>;
    local_files_updated?: number;
    dry_run?: boolean;
    message?: string;
}

export type SystemExportTarget = 'accounts' | 'instances' | 'balance' | 'nodes' | 'prompts' | 'cluster' | 'all';
export type SystemExportFormat = 'csv' | 'json';

function computeRsaFingerprintPreview(pem: string | null | undefined): string {
    if (!pem) return 'RSA-2048 Vault Key (Auto-initialized on first save)';
    const cleaned = pem
        .replace(/-----BEGIN [^-]+-----/g, '')
        .replace(/-----END [^-]+-----/g, '')
        .replace(/\s+/g, '');
    if (cleaned.length <= 24) return `RSA-2048 (${cleaned})`;
    return `RSA-2048 SHA256:${cleaned.slice(0, 12)}...${cleaned.slice(-12)}`;
}

function escapeCsvCell(value: unknown): string {
    if (value === null || value === undefined) return '';
    const str = String(value);
    if (str.includes('"') || str.includes(',') || str.includes('\n') || str.includes('\r')) {
        return `"${str.replace(/"/g, '""')}"`;
    }
    return str;
}

export const sshService = {
    async getSshGitmapStatus(): Promise<SshGitmapStatus> {
        try {
            const status = await invoke<SshGitmapStatus>('get_ssh_gitmap_status');
            return {
                ...status,
                rsa_key_fingerprint:
                    status.rsa_key_fingerprint || computeRsaFingerprintPreview(status.rsa_public_key_pem),
            };
        } catch (primaryError) {
            useErrorStore.getState().trackWarning(primaryError, {
                source: 'sshService.getSshGitmapStatus',
                triggerAction: 'get_ssh_gitmap_status',
            });

            try {
                const installed = await invoke<boolean>('check_gitmap_installed');
                return {
                    gitmap_installed: Boolean(installed),
                    gitmap_version: installed ? 'detected' : null,
                    gitmap_path: installed ? 'gitmap' : null,
                    install_command: 'agm ssh install-gitmap',
                    ssh_config_path: '~/.ssh/config',
                    connections_json_path: '~/.agm/ssh/ssh_nodes.json',
                    rsa_public_key_pem: null,
                    rsa_key_fingerprint: 'RSA-2048 Vault (~/.agm/ssh/rsa_vault_key.pub)',
                    total_nodes: 0,
                };
            } catch (fallbackError) {
                useErrorStore.getState().trackWarning(fallbackError, {
                    source: 'sshService.getSshGitmapStatus.fallback',
                    triggerAction: 'check_gitmap_installed',
                });
                return {
                    gitmap_installed: false,
                    gitmap_version: null,
                    gitmap_path: null,
                    install_command: 'agm ssh install-gitmap',
                    ssh_config_path: '~/.ssh/config',
                    connections_json_path: '~/.agm/ssh/ssh_nodes.json',
                    rsa_public_key_pem: null,
                    rsa_key_fingerprint: 'RSA-2048 Vault (~/.agm/ssh/rsa_vault_key.pub)',
                    total_nodes: 0,
                };
            }
        }
    },

    async installGitmapPowershell(openWindow: boolean = true): Promise<string> {
        try {
            const res = await invoke<string | { message?: string }>('install_gitmap_powershell', {
                openWindow,
                open_window: openWindow,
            });
            if (typeof res === 'string') return res;
            return res?.message || 'Launched PowerShell installer for GitMap.';
        } catch (primaryError) {
            useErrorStore.getState().trackWarning(primaryError, {
                source: 'sshService.installGitmapPowershell',
                triggerAction: 'install_gitmap_powershell',
            });
            try {
                const fallback = await invoke<{ message?: string; raw_output?: string } | string>(
                    'install_gitmap_cli',
                    { openWindow, open_window: openWindow }
                );
                if (typeof fallback === 'string') return fallback;
                return fallback?.message || fallback?.raw_output || 'Launched GitMap CLI installer.';
            } catch (secondaryError) {
                useErrorStore.getState().captureError(secondaryError, {
                    source: 'sshService.installGitmapPowershell',
                    triggerAction: 'install_gitmap_cli',
                });
                throw secondaryError;
            }
        }
    },

    async listSshNodes(): Promise<SshNodeRecord[]> {
        try {
            const nodes = await invoke<SshNodeRecord[]>('list_ssh_nodes');
            return Array.isArray(nodes)
                ? nodes.map((n) => ({
                      ...n,
                      ip_address: n.ip_address || (n as any).host || '',
                      host: (n as any).host || n.ip_address || '',
                      has_encrypted_password:
                          n.has_encrypted_password !== undefined
                              ? Boolean(n.has_encrypted_password)
                              : Boolean((n as any).has_password),
                  }))
                : [];
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'sshService.listSshNodes',
                triggerAction: 'list_ssh_nodes',
            });
            return [];
        }
    },

    async saveSshNodeRsa(payload: SaveSshNodeRsaPayload): Promise<SshNodeRecord> {
        try {
            return await invoke<SshNodeRecord>('save_ssh_node_rsa', {
                payload,
                ...payload,
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'sshService.saveSshNodeRsa',
                triggerAction: 'save_ssh_node_rsa',
            });
            throw error;
        }
    },

    async deleteSshNode(alias: string): Promise<boolean> {
        try {
            return await invoke<boolean>('delete_ssh_node', { alias });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'sshService.deleteSshNode',
                triggerAction: 'delete_ssh_node',
            });
            throw error;
        }
    },

    async importSshNodesFromGitmap(): Promise<NodeSyncStats> {
        try {
            const stats = await invoke<NodeSyncStats>('import_ssh_nodes_from_gitmap');
            return stats || { total: 0, matched: 0, updated: 0, inserted: 0, unchanged: 0 };
        } catch (primaryError) {
            useErrorStore.getState().trackWarning(primaryError, {
                source: 'sshService.importSshNodesFromGitmap',
                triggerAction: 'import_ssh_nodes_from_gitmap',
            });
            try {
                const fallback = await invoke<any>('import_gitmap_ssh_nodes');
                const stats = fallback?.stats || fallback;
                return {
                    total: Number(stats?.total ?? 0),
                    matched: Number(stats?.matched ?? 0),
                    updated: Number(stats?.updated ?? 0),
                    inserted: Number(stats?.inserted ?? 0),
                    unchanged: Number(stats?.unchanged ?? 0),
                };
            } catch (secondaryError) {
                useErrorStore.getState().captureError(secondaryError, {
                    source: 'sshService.importSshNodesFromGitmap',
                    triggerAction: 'import_gitmap_ssh_nodes',
                });
                throw secondaryError;
            }
        }
    },

    async resetSshCluster(): Promise<SshNodeRecord[]> {
        try {
            return await invoke<SshNodeRecord[]>('reset_ssh_cluster');
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: 'sshService.resetSshCluster',
                triggerAction: 'reset_ssh_cluster',
            });
            throw err;
        }
    },

    async resetClusterNodes(): Promise<SshNodeRecord[]> {
        return await this.resetSshCluster();
    },

    async deployEncryptedSshBundle(targetAliases?: string[]): Promise<MeshDeploySummary> {
        try {
            return await invoke<MeshDeploySummary>('deploy_encrypted_ssh_bundle', {
                targetAliases: targetAliases ?? null,
                target_aliases: targetAliases ?? null,
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'sshService.deployEncryptedSshBundle',
                triggerAction: 'deploy_encrypted_ssh_bundle',
            });
            throw error;
        }
    },

    async getClusterInstancesAndBalances(): Promise<ClusterInstanceBalanceItem[]> {
        try {
            const items = await invoke<ClusterInstanceBalanceItem[]>('get_cluster_instances_and_balances');
            if (Array.isArray(items)) {
                return items.map((raw) => {
                    const item = { ...raw };
                    item.ip_address = item.ip_address || (item as any).machine_ip || '';
                    if (item.online === undefined) {
                        item.online = Boolean(item.running);
                    }
                    if (item.bound_email && typeof item.bound_email === 'string') {
                        item.bound_email = item.bound_email
                            .replace(/\$\{adminUser\}/g, 'Administrator')
                            .replace(/\$\{linuxUser\}/g, 'root');
                    }
                    return item;
                });
            }
            return [];
        } catch (primaryError) {
            useErrorStore.getState().trackWarning(primaryError, {
                source: 'sshService.getClusterInstancesAndBalances',
                triggerAction: 'get_cluster_instances_and_balances',
            });

            // Fallback: aggregate local instances and accounts so local telemetry remains visible
            try {
                const [instancesRes, accountsRes] = await Promise.all([
                    invoke<any[]>('list_instances'),
                    invoke<any>('list_accounts'),
                ]);
                const instances = Array.isArray(instancesRes) ? instancesRes : [];
                const accounts: any[] = Array.isArray(accountsRes)
                    ? accountsRes
                    : Array.isArray(accountsRes?.accounts)
                      ? accountsRes.accounts
                      : [];

                const accountByEmail = new Map<string, any>();
                const accountById = new Map<string, any>();
                for (const acc of accounts) {
                    if (acc?.email) accountByEmail.set(String(acc.email).toLowerCase(), acc);
                    if (acc?.id) accountById.set(String(acc.id), acc);
                }

                const nowIso = new Date().toISOString();
                return instances.map((inst): ClusterInstanceBalanceItem => {
                    const cfg = inst?.config || inst || {};
                    let boundEmail = cfg.bound_email || null;
                    if (boundEmail && typeof boundEmail === 'string') {
                        boundEmail = boundEmail
                            .replace(/\$\{adminUser\}/g, 'Administrator')
                            .replace(/\$\{linuxUser\}/g, 'root');
                    }
                    const boundAcc =
                        (cfg.bound_account_id && accountById.get(String(cfg.bound_account_id))) ||
                        (boundEmail && accountByEmail.get(String(boundEmail).toLowerCase())) ||
                        null;
                    const models: any[] = boundAcc?.quota?.models || [];
                    const firstModel = models[0] || null;
                    const pct4h =
                        typeof firstModel?.percentage === 'number' ? firstModel.percentage : null;
                    const pctWeekly =
                        typeof firstModel?.weekly_percentage === 'number'
                            ? firstModel.weekly_percentage
                            : pct4h;

                    return {
                        machine_alias: 'local',
                        ip_address: '127.0.0.1',
                        machine_ip: '127.0.0.1',
                        is_local: true,
                        online: true,
                        instance_id: String(cfg.id || 'default'),
                        instance_name: String(cfg.name || cfg.id || 'default'),
                        running: Boolean(inst?.is_running),
                        pid: typeof inst?.pid === 'number' ? inst.pid : null,
                        bound_email: boundEmail,
                        tier: boundAcc?.quota?.subscription_tier || null,
                        quota_4h_pct: pct4h,
                        quota_weekly_pct: pctWeekly,
                        last_updated: nowIso,
                        error: null,
                        error_message: null,
                        latency_ms: 0,
                    };
                });
            } catch (fallbackError) {
                useErrorStore.getState().captureError(fallbackError, {
                    source: 'sshService.getClusterInstancesAndBalances.fallback',
                    triggerAction: 'list_instances',
                });
                throw fallbackError;
            }
        }
    },

    async sendRemoteSshPrompt(
        payload: RemotePromptDispatchPayload
    ): Promise<RemotePromptDispatchResult> {
        try {
            return await invoke<RemotePromptDispatchResult>('send_remote_ssh_prompt', {
                payload,
                ...payload,
            });
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'sshService.sendRemoteSshPrompt',
                triggerAction: 'send_remote_ssh_prompt',
            });
            throw error;
        }
    },

    async testSshNodeConnection(alias: string): Promise<RemotePromptDispatchResult> {
        try {
            const result = await invoke<{
                alias?: string;
                target?: string;
                stdout?: string;
                stderr?: string;
                duration_ms?: number | string;
                success?: boolean;
            }>('test_ssh_node_connection', { idOrAlias: alias, id_or_alias: alias });
            const duration = Number(result?.duration_ms ?? 0);
            return {
                node_alias: result?.alias || alias,
                instance_target: result?.target || alias,
                success: Boolean(result?.success),
                command_executed: '',
                stdout: result?.stdout || '',
                stderr: result?.stderr || '',
                duration_ms: Number.isFinite(duration) ? duration : 0,
            };
        } catch (error) {
            useErrorStore.getState().captureError(error, {
                source: 'sshService.testSshNodeConnection',
                triggerAction: 'test_ssh_node_connection',
            });
            throw error;
        }
    },

    async exportSystemDataCsv(options: {
        target: SystemExportTarget;
        format?: SystemExportFormat;
        sendToTelegram?: boolean;
    }): Promise<string> {
        const format = options.format || 'csv';
        const sendToTelegram = Boolean(options.sendToTelegram);
        try {
            return await invoke<string>('export_system_data_csv', {
                kind: options.target,
                target: options.target,
                format,
                sendToTelegram,
                send_to_telegram: sendToTelegram,
            });
        } catch (primaryError) {
            useErrorStore.getState().trackWarning(primaryError, {
                source: 'sshService.exportSystemDataCsv',
                triggerAction: 'export_system_data_csv',
            });

            // Client-side structured fallback for immediate export availability
            try {
                if (options.target === 'nodes') {
                    const nodes = await this.listSshNodes();
                    if (format === 'json') {
                        return JSON.stringify(nodes, null, 2);
                    }
                    const header = 'alias,ip_address,port,username,os,auth_method,has_encrypted_password';
                    const rows = nodes.map((n) =>
                        [
                            n.alias,
                            n.ip_address,
                            n.port,
                            n.username,
                            n.os,
                            n.auth_method,
                            n.has_encrypted_password,
                        ]
                            .map(escapeCsvCell)
                            .join(',')
                    );
                    return [header, ...rows].join('\n');
                }

                const cluster = await this.getClusterInstancesAndBalances();
                if (format === 'json') {
                    return JSON.stringify(cluster, null, 2);
                }
                const header =
                    'machine_alias,ip_address,instance_id,instance_name,running,pid,bound_email,tier,quota_4h_pct,quota_weekly_pct,last_updated';
                const rows = cluster.map((item) =>
                    [
                        item.machine_alias,
                        item.ip_address,
                        item.instance_id,
                        item.instance_name,
                        item.running,
                        item.pid ?? '',
                        item.bound_email ?? '',
                        item.tier ?? '',
                        item.quota_4h_pct ?? '',
                        item.quota_weekly_pct ?? '',
                        item.last_updated,
                    ]
                        .map(escapeCsvCell)
                        .join(',')
                );
                return [header, ...rows].join('\n');
            } catch (fallbackError) {
                useErrorStore.getState().captureError(fallbackError, {
                    source: 'sshService.exportSystemDataCsv.fallback',
                    triggerAction: 'export_system_data_csv',
                });
                throw fallbackError;
            }
        }
    },

    async triggerDownloadAndCopy(
        content: string,
        filename: string,
        format: SystemExportFormat = 'csv'
    ): Promise<void> {
        await copyToClipboard(content);
        try {
            if (typeof window !== 'undefined' && typeof document !== 'undefined') {
                const mimeType =
                    format === 'json' ? 'application/json;charset=utf-8' : 'text/csv;charset=utf-8';
                const blob = new Blob([content], { type: mimeType });
                const url = URL.createObjectURL(blob);
                const anchor = document.createElement('a');
                anchor.href = url;
                anchor.download = filename;
                anchor.style.display = 'none';
                document.body.appendChild(anchor);
                anchor.click();
                document.body.removeChild(anchor);
                URL.revokeObjectURL(url);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'sshService.triggerDownloadAndCopy',
                triggerAction: 'browser_blob_download',
            });
        }
    },
};
