import { copyToClipboard } from '../../../utils/clipboard';
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { supabaseService } from '../../../services/supabaseService';
import { sshService } from '../../../services/sshService';
import { showToast } from '../../common/ToastContainer';
import { useErrorStore } from '../../../stores/error-store';
import type { FleetMachineInfo } from './fleetTypes';

const POLL_INTERVAL_SECONDS = 300;

export interface FleetMachinesState {
    machines: FleetMachineInfo[];
    isLoading: boolean;
    isRefreshing: boolean;
    isResettingCluster: boolean;
    isSyncEnabled: boolean;
    showAllEmails: boolean;
    unmaskedRows: Record<string, boolean>;
    copiedIp: string | null;
    countdown: number;
    setShowAllEmails: React.Dispatch<React.SetStateAction<boolean>>;
    handleManualRefresh: () => void;
    handleResetCluster: () => Promise<void>;
    handleCopyIp: (ip: string) => Promise<void>;
    toggleRowMask: (nodeId: string) => void;
}

interface SshNodeItem {
    alias: string;
    ip: string;
    status?: string;
    port?: number;
    os?: string;
}

function sortFleetMachines(list: FleetMachineInfo[]): FleetMachineInfo[] {
    return [...list].sort((a, b) => {
        const aAlias = (a.node_alias || (a as any).alias || '').trim().toLowerCase();
        const bAlias = (b.node_alias || (b as any).alias || '').trim().toLowerCase();

        // 1. Lead machine ('main') has top primacy
        const aIsMain = aAlias === 'main' || Boolean((a as any).is_lead);
        const bIsMain = bAlias === 'main' || Boolean((b as any).is_lead);
        if (aIsMain !== bIsMain) return aIsMain ? -1 : 1;

        // 2. Local worker machine ('w1' / local) follows
        const aIsLocal = Boolean((a as any).is_local || aAlias === 'w1' || aAlias.startsWith('local'));
        const bIsLocal = Boolean((b as any).is_local || bAlias === 'w1' || bAlias.startsWith('local'));
        if (aIsLocal !== bIsLocal) return aIsLocal ? -1 : 1;

        // 3. Online status
        if (a.is_online !== b.is_online) return a.is_online ? -1 : 1;

        return (a.node_alias || '').localeCompare(b.node_alias || '');
    });
}

function mapSshNodesToFleet(sshNodes: SshNodeItem[], _localId?: string): FleetMachineInfo[] {
    return sshNodes
        .filter((n) => Boolean(n.alias))
        .map((n) => {
            let alias = n.alias.trim();
            let ip = n.ip?.trim() || '';

            // Fix inverted or mislabeled SSH host alias between main and w1
            if (alias.toLowerCase() === 'main' && ip === '192.168.1.3') {
                ip = '192.168.1.20';
            } else if (alias.toLowerCase() === 'w1' && ip === '192.168.1.20') {
                alias = 'main';
            }

            const aliasLower = alias.toLowerCase();
            const isLead = aliasLower === 'main';
            const isLocal =
                aliasLower === 'w1' ||
                aliasLower.startsWith('local');

            return {
                node_id: `ssh_${alias}`,
                node_alias: alias,
                ip_address: ip,
                is_online: n.status === 'online' || n.status === 'ready' || isLocal || isLead,
                last_heartbeat_timestamp: Math.floor(Date.now() / 1000),
                uptime_seconds: 0,
                in_flight_prompts_count: 0,
                is_local: isLocal,
                is_lead: isLead,
                source: isLocal ? 'local' : isLead ? 'lead' : 'ssh-mesh',
                active_instances: [
                    {
                        profile_id: `${alias}:default`,
                        profile_name: `${alias}:default`,
                        is_running: true,
                    },
                ],
                bound_emails: [],
                git_repo: 'Antigravity-Manager',
                git_link: 'gitmap / Antigravity-Manager',
            } as FleetMachineInfo;
        });
}

async function enrichFleetMachines(machines: FleetMachineInfo[]): Promise<FleetMachineInfo[]> {
    let clusterItems: any[] = [];
    try {
        clusterItems = await sshService.getClusterInstancesAndBalances();
    } catch (e) {
        useErrorStore.getState().trackWarning(e, {
            source: 'useFleetMachines.enrichFleetMachines',
            triggerAction: 'get_cluster_instances_and_balances',
        });
    }

    let localAccounts: any[] = [];
    let currentAccountEmail: string | null = null;
    try {
        const [accRes, currAccRes] = await Promise.allSettled([
            invoke<any>('list_accounts'),
            invoke<any>('get_current_account'),
        ]);
        if (accRes.status === 'fulfilled' && accRes.value) {
            const val = accRes.value;
            localAccounts = Array.isArray(val) ? val : Array.isArray(val?.accounts) ? val.accounts : [];
        }
        if (currAccRes.status === 'fulfilled' && currAccRes.value) {
            currentAccountEmail = currAccRes.value?.email || null;
        }
    } catch {
        // Best effort
    }

    const emailByAlias = new Map<string, Set<string>>();
    const emailByIp = new Map<string, Set<string>>();
    const instancesByAlias = new Map<string, any[]>();

    if (Array.isArray(clusterItems)) {
        for (const item of clusterItems) {
            const aliasKey = (item.machine_alias || '').trim().toLowerCase();
            const ipKey = (item.machine_ip || item.ip_address || '').trim();
            const email = (item.bound_email || '').trim();

            if (email && email !== '(unbound)' && !email.toLowerCase().includes('admin') && !email.toLowerCase().includes('root')) {
                if (aliasKey) {
                    if (!emailByAlias.has(aliasKey)) emailByAlias.set(aliasKey, new Set());
                    emailByAlias.get(aliasKey)!.add(email);
                }
                if (ipKey) {
                    if (!emailByIp.has(ipKey)) emailByIp.set(ipKey, new Set());
                    emailByIp.get(ipKey)!.add(email);
                }
            }

            if (aliasKey && item.instance_name) {
                if (!instancesByAlias.has(aliasKey)) instancesByAlias.set(aliasKey, []);
                const list = instancesByAlias.get(aliasKey)!;
                if (!list.some((i: any) => i.profile_name === item.instance_name || i.profile_id === item.instance_id)) {
                    list.push({
                        profile_id: item.instance_id || `${aliasKey}:${item.instance_name}`,
                        profile_name: item.instance_name,
                        is_running: Boolean(item.running),
                        bound_account_email: email && email !== '(unbound)' ? email : undefined,
                    });
                }
            }
        }
    }

    const localPrimaryEmails: string[] = [];
    if (currentAccountEmail) {
        localPrimaryEmails.push(currentAccountEmail);
    }
    for (const acc of localAccounts) {
        if (acc?.email && !localPrimaryEmails.includes(acc.email)) {
            localPrimaryEmails.push(acc.email);
        }
    }

    let enriched = machines.map((machine) => {
        let alias = (machine.node_alias || (machine as any).alias || '').trim();
        let ip = (machine.ip_address || '').trim();

        // Fix inverted or mislabeled SSH host alias between main and w1
        if (alias.toLowerCase() === 'main' && ip === '192.168.1.3') {
            ip = '192.168.1.20';
        } else if (alias.toLowerCase() === 'w1' && ip === '192.168.1.20') {
            alias = 'main';
        }

        const aliasLower = alias.toLowerCase();
        const isLead = aliasLower === 'main';
        const isLocal = Boolean(
            (machine as any).is_local ||
            aliasLower === 'w1' ||
            aliasLower.startsWith('local') ||
            (machine as any).source === 'local'
        );

        // Collect bound emails
        const emails = new Set<string>(machine.bound_emails || []);
        if (emailByAlias.has(aliasLower)) {
            for (const e of emailByAlias.get(aliasLower)!) emails.add(e);
        }
        if (emailByIp.has(ip)) {
            for (const e of emailByIp.get(ip)!) emails.add(e);
        }

        // If lead (main) or local (w1) has no emails, populate from local primary accounts
        if (emails.size === 0 && (isLead || isLocal) && localPrimaryEmails.length > 0) {
            for (const e of localPrimaryEmails) emails.add(e);
        }

        // Active instances
        let activeInstances = machine.active_instances || [];
        if (instancesByAlias.has(aliasLower)) {
            const extra = instancesByAlias.get(aliasLower)!;
            if (activeInstances.length === 0) {
                activeInstances = extra;
            } else {
                for (const ex of extra) {
                    if (!activeInstances.some((i: any) => i.profile_name === ex.profile_name)) {
                        activeInstances.push(ex);
                    }
                }
            }
        }

        const gitRepo = (machine as any).git_repo || 'Antigravity-Manager';
        const gitLink = (machine as any).git_link || `gitmap / ${gitRepo}`;

        return {
            ...machine,
            node_alias: alias,
            alias,
            ip_address: ip,
            is_local: isLocal,
            is_lead: isLead,
            bound_emails: Array.from(emails),
            active_instances: activeInstances,
            git_repo: gitRepo,
            git_link: gitLink,
        } as FleetMachineInfo;
    });

    // Ensure lead machine 'main' (192.168.1.20) is always present in fleet
    const hasMain = enriched.some(
        (m) => (m.node_alias || '').toLowerCase() === 'main' || m.ip_address === '192.168.1.20'
    );
    if (!hasMain) {
        enriched.unshift({
            node_id: 'ssh_main',
            node_alias: 'main',
            ip_address: '192.168.1.20',
            is_online: true,
            last_heartbeat_timestamp: Math.floor(Date.now() / 1000),
            uptime_seconds: 0,
            in_flight_prompts_count: 0,
            is_lead: true,
            is_local: false,
            source: 'lead',
            active_instances: [
                {
                    profile_id: 'main:default',
                    profile_name: 'main:default',
                    is_running: true,
                },
            ],
            bound_emails: localPrimaryEmails.slice(0, 1),
            git_repo: 'Antigravity-Manager',
            git_link: 'gitmap / Antigravity-Manager',
        } as FleetMachineInfo);
    }

    return enriched;
}

export function useFleetMachines(): FleetMachinesState {
    const [machines, setMachines] = useState<FleetMachineInfo[]>([]);
    const [localNodeId, setLocalNodeId] = useState<string>('');
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
    const [isResettingCluster, setIsResettingCluster] = useState<boolean>(false);
    const [isSyncEnabled, setIsSyncEnabled] = useState<boolean>(true);
    const [showAllEmails, setShowAllEmails] = useState<boolean>(false);
    const [unmaskedRows, setUnmaskedRows] = useState<Record<string, boolean>>({});
    const [copiedIp, setCopiedIp] = useState<string | null>(null);
    const [countdown, setCountdown] = useState<number>(POLL_INTERVAL_SECONDS);

    const fetchFleetMachinesApi = useCallback(async (): Promise<FleetMachineInfo[]> => {
        const svc = supabaseService as unknown as {
            getFleetMachines?: () => Promise<FleetMachineInfo[]>;
        };
        if (typeof svc.getFleetMachines === 'function') {
            return await svc.getFleetMachines();
        }
        try {
            return await invoke<FleetMachineInfo[]>('get_fleet_machines');
        } catch (err) {
            // Tracked in the error module; empty list shown, retried on next refresh.
            useErrorStore.getState().trackWarning(err, {
                source: 'FleetMachinesTable.fetchFleetMachinesApi',
                endpoint: 'get_fleet_machines',
                triggerAction: 'fetch_fleet_machines',
            });
            return [];
        }
    }, []);

    const fetchMachines = useCallback(
        async (showLoadingSpinner: boolean = false) => {
            if (showLoadingSpinner) {
                setIsRefreshing(true);
            }
            try {
                const config = await supabaseService.getConfig().catch(() => null);
                const syncActive = Boolean(config?.is_sync_enabled);

                let effectiveLocalId = localNodeId;
                if (!effectiveLocalId) {
                    const localInfo = await supabaseService.getLocalNodeInfo().catch(() => null);
                    if (localInfo?.node_id) {
                        effectiveLocalId = localInfo.node_id;
                        setLocalNodeId(effectiveLocalId);
                    }
                }

                const data = await fetchFleetMachinesApi();
                let effectiveMachines = data;

                if (effectiveMachines.length === 0) {
                    try {
                        const sshNodes = await invoke<SshNodeItem[]>('list_ssh_nodes');
                        if (sshNodes && sshNodes.length > 0) {
                            effectiveMachines = mapSshNodesToFleet(sshNodes, effectiveLocalId);
                        }
                    } catch (e) {
                        useErrorStore.getState().trackWarning(e, {
                            source: 'FleetMachinesTable.fallback_ssh_nodes',
                            triggerAction: 'fetch_ssh_nodes',
                        });
                    }
                }

                const enriched = await enrichFleetMachines(effectiveMachines);
                setIsSyncEnabled(syncActive || enriched.length > 0);
                setMachines(sortFleetMachines(enriched));
            } catch (err) {
                // Tracked in the error module; stale list kept, retried on next refresh.
                useErrorStore.getState().trackWarning(err, {
                    source: 'FleetMachinesTable.fetchMachines',
                    triggerAction: 'fetch_machines',
                });
            } finally {
                setIsRefreshing(false);
            }
        },
        [fetchFleetMachinesApi, localNodeId]
    );

    // Initial load
    useEffect(() => {
        let isMounted = true;
        const init = async () => {
            setIsLoading(true);
            try {
                const [configRes, localNodeRes] = await Promise.allSettled([
                    supabaseService.getConfig(),
                    supabaseService.getLocalNodeInfo(),
                ]);

                let syncActive = false;
                if (configRes.status === 'fulfilled' && configRes.value) {
                    syncActive = Boolean(configRes.value.is_sync_enabled);
                    if (isMounted) setIsSyncEnabled(syncActive);
                }

                let localId = '';
                if (localNodeRes.status === 'fulfilled' && localNodeRes.value) {
                    localId = localNodeRes.value.node_id;
                    if (isMounted) setLocalNodeId(localId);
                }

                const data = await fetchFleetMachinesApi();
                if (isMounted) {
                    let effectiveMachines = data;

                    if (effectiveMachines.length === 0) {
                        try {
                            const sshNodes = await invoke<SshNodeItem[]>('list_ssh_nodes');
                            if (sshNodes && sshNodes.length > 0) {
                                effectiveMachines = mapSshNodesToFleet(sshNodes, localId);
                            }
                        } catch (e) {
                            useErrorStore.getState().trackWarning(e, {
                                source: 'FleetMachinesTable.init_fallback_ssh_nodes',
                                triggerAction: 'init_ssh_nodes',
                            });
                        }
                    }

                    const enriched = await enrichFleetMachines(effectiveMachines);
                    setIsSyncEnabled(syncActive || enriched.length > 0);
                    setMachines(sortFleetMachines(enriched));
                }
            } catch (err) {
                // Tracked in the error module; loading spinner cleared, empty state shown.
                useErrorStore.getState().trackWarning(err, {
                    source: 'FleetMachinesTable.init',
                    triggerAction: 'init_machines',
                });
            } finally {
                if (isMounted) {
                    setIsLoading(false);
                }
            }
        };

        init();
        return () => {
            isMounted = false;
        };
    }, [fetchFleetMachinesApi]);

    // Polling interval
    useEffect(() => {
        const timer = setInterval(() => {
            setCountdown((prev) => {
                if (prev <= 1) {
                    fetchMachines(false);
                    return POLL_INTERVAL_SECONDS;
                }
                return prev - 1;
            });
        }, 1000);

        return () => clearInterval(timer);
    }, [fetchMachines]);

    const handleManualRefresh = () => {
        setCountdown(POLL_INTERVAL_SECONDS);
        fetchMachines(true);
    };

    const handleResetCluster = async () => {
        setIsResettingCluster(true);
        try {
            await sshService.resetSshCluster();
            showToast('SSH cluster reset to canonical 6 nodes (w1, main, u1, w2, w3, w4)', 'success');
            await fetchMachines(true);
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: 'FleetMachinesTable.handleResetCluster',
                triggerAction: 'reset_ssh_cluster',
            });
            showToast('Failed to reset SSH cluster nodes', 'error');
        } finally {
            setIsResettingCluster(false);
        }
    };

    const handleCopyIp = async (ip: string) => {
        if (!ip) return;
        try {
            await copyToClipboard(ip);
            setCopiedIp(ip);
            setTimeout(() => {
                setCopiedIp(null);
            }, 2000);
        } catch (err) {
            // Tracked in the error module; copy feedback simply won't show, user can copy manually.
            useErrorStore.getState().trackWarning(err, {
                source: 'FleetMachinesTable.copyIp',
                triggerAction: 'copy_ip',
            });
        }
    };

    const toggleRowMask = (nodeId: string) => {
        setUnmaskedRows((prev) => ({
            ...prev,
            [nodeId]: !prev[nodeId],
        }));
    };

    return {
        machines,
        isLoading,
        isRefreshing,
        isResettingCluster,
        isSyncEnabled,
        showAllEmails,
        unmaskedRows,
        copiedIp,
        countdown,
        setShowAllEmails,
        handleManualRefresh,
        handleResetCluster,
        handleCopyIp,
        toggleRowMask,
    };
}
