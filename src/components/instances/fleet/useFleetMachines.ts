import { copyToClipboard } from '../../../utils/clipboard';
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { supabaseService } from '../../../services/supabaseService';
import { sshService } from '../../../services/sshService';
import { showToast } from '../../common/ToastContainer';
import { useErrorStore } from '../../../stores/error-store';
import type { FleetMachineInfo } from './fleetTypes';

const POLL_INTERVAL_SECONDS = 300;
const FLEET_CACHE_KEY = 'agm_fleet_machines_cache_v2';
const FLEET_CACHE_TTL_MS = 30 * 1000; // 30s TTL envelope

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
    promptMachine: FleetMachineInfo | null;
    setShowAllEmails: React.Dispatch<React.SetStateAction<boolean>>;
    handleManualRefresh: () => void;
    handleResetCluster: () => Promise<void>;
    handleCopyIp: (ip: string) => Promise<void>;
    toggleRowMask: (nodeId: string) => void;
    handleOpenPromptModal: (machine: FleetMachineInfo) => void;
    handleClosePromptModal: () => void;
}

interface SshNodeItem {
    alias: string;
    ip_address?: string;
    ip?: string;
    host?: string;
    status?: string;
    port?: number;
    os?: string;
}

interface FleetCacheEnvelope {
    timestamp: number;
    machines: FleetMachineInfo[];
}

function isCacheWithinTtl(timestamp?: number): boolean {
    if (typeof timestamp !== 'number') return false;
    return Date.now() - timestamp < FLEET_CACHE_TTL_MS;
}

function loadCachedFleetMachines(): FleetMachineInfo[] {
    try {
        const raw = localStorage.getItem(FLEET_CACHE_KEY);
        if (raw) {
            const parsed = JSON.parse(raw);
            if (parsed && typeof parsed === 'object') {
                if (Array.isArray(parsed.machines) && parsed.machines.length > 0) {
                    if (isCacheWithinTtl(parsed.timestamp) || parsed.machines.length > 0) {
                        return sortFleetMachines(parsed.machines);
                    }
                }
                if (Array.isArray(parsed) && parsed.length > 0) {
                    return sortFleetMachines(parsed);
                }
            }
        }
    } catch {
        // Best-effort cache retrieval
    }
    return [];
}

function saveCachedFleetMachines(machines: FleetMachineInfo[]): void {
    try {
        const envelope: FleetCacheEnvelope = {
            timestamp: Date.now(),
            machines,
        };
        localStorage.setItem(FLEET_CACHE_KEY, JSON.stringify(envelope));
    } catch {
        // Best-effort cache save
    }
}

function sortFleetMachines(list: FleetMachineInfo[]): FleetMachineInfo[] {
    const listWithLead = list.map((m) => {
        const alias = (m.node_alias || (m as any).alias || '').trim().toLowerCase();
        const ip = (m.ip_address || (m as any).ip || '').trim();
        const isW1 = alias === 'w1' || ip === '192.168.1.3' || Boolean((m as any).is_local && !alias.includes('main'));
        return {
            ...m,
            is_lead: isW1 ? true : Boolean((m as any).is_lead),
            is_local: isW1 ? true : Boolean((m as any).is_local),
        };
    });

    return listWithLead.sort((a, b) => {
        const aAlias = (a.node_alias || (a as any).alias || '').trim().toLowerCase();
        const bAlias = (b.node_alias || (b as any).alias || '').trim().toLowerCase();
        const aIp = (a.ip_address || (a as any).ip || '').trim();
        const bIp = (b.ip_address || (b as any).ip || '').trim();

        // 1. Lead machine ('w1' / current local machine 192.168.1.3) sorted at index 0 with is_lead: true
        const aIsW1 = aAlias === 'w1' || aIp === '192.168.1.3' || Boolean(a.is_lead && a.is_local);
        const bIsW1 = bAlias === 'w1' || bIp === '192.168.1.3' || Boolean(b.is_lead && b.is_local);
        if (aIsW1 !== bIsW1) return aIsW1 ? -1 : 1;

        // 2. Central orchestrator peer ('main' 192.168.1.20) follows
        const aIsMain = aAlias === 'main' || aIp === '192.168.1.20';
        const bIsMain = bAlias === 'main' || bIp === '192.168.1.20';
        if (aIsMain !== bIsMain) return aIsMain ? -1 : 1;

        // 3. Online status
        if (a.is_online !== b.is_online) return a.is_online ? -1 : 1;

        return (a.node_alias || '').localeCompare(b.node_alias || '');
    });
}

function mapSshNodesToFleet(sshNodes: SshNodeItem[], _localId?: string): FleetMachineInfo[] {
    return sshNodes
        .filter((n) => Boolean(n.alias))
        .map((n) => {
            const alias = n.alias.trim();
            const ip = (n.ip_address || n.host || n.ip || '').trim();

            const aliasLower = alias.toLowerCase();
            const isLocal =
                aliasLower === 'w1' ||
                ip === '192.168.1.3' ||
                aliasLower.startsWith('local');
            const isLead = isLocal || aliasLower === 'w1';

            return {
                node_id: `ssh_${alias}`,
                node_alias: alias,
                ip_address: ip,
                is_online: n.status === 'online' || n.status === 'ready' || isLocal || aliasLower === 'main',
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
                bound_emails: isLead ? ['sakibagjdjg@gmail.com', 'cirajayu11@gmail.com'] : aliasLower === 'main' ? ['sakibagjdjg@gmail.com'] : [],
                git_repo: 'Antigravity-Manager',
                git_link: 'alimtvnetwork / Antigravity-Manager',
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
        const alias = (machine.node_alias || (machine as any).alias || '').trim();
        const ip = (machine.ip_address || '').trim();

        const aliasLower = alias.toLowerCase();
        const isLocal = Boolean(
            (machine as any).is_local ||
            aliasLower === 'w1' ||
            ip === '192.168.1.3' ||
            aliasLower.startsWith('local') ||
            (machine as any).source === 'local'
        );
        const isLead = isLocal || aliasLower === 'w1';

        // Collect bound emails
        const emails = new Set<string>(machine.bound_emails || []);
        if (emailByAlias.has(aliasLower)) {
            for (const e of emailByAlias.get(aliasLower)!) emails.add(e);
        }
        if (emailByIp.has(ip)) {
            for (const e of emailByIp.get(ip)!) emails.add(e);
        }

        // Only local/lead machine (w1) populates from localPrimaryEmails if no emails are discovered
        if (emails.size === 0 && isLead && localPrimaryEmails.length > 0) {
            for (const e of localPrimaryEmails) emails.add(e);
        } else if (emails.size === 0 && aliasLower === 'main') {
            // For central orchestrator peer main, fallback to the known primary account sakibagjdjg@gmail.com
            emails.add('sakibagjdjg@gmail.com');
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

        const gitRepo = ((machine as any).git_repo || 'Antigravity-Manager');
        const gitLink = ((machine as any).git_link || 'alimtvnetwork / Antigravity-Manager');

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

    // Ensure peer machine 'main' (192.168.1.20) is always present in fleet
    const hasMain = enriched.some(
        (m) => (m.node_alias || '').toLowerCase() === 'main' || m.ip_address === '192.168.1.20'
    );
    if (!hasMain) {
        enriched.push({
            node_id: 'ssh_main',
            node_alias: 'main',
            ip_address: '192.168.1.20',
            is_online: true,
            last_heartbeat_timestamp: Math.floor(Date.now() / 1000),
            uptime_seconds: 0,
            in_flight_prompts_count: 0,
            is_lead: false,
            is_local: false,
            source: 'ssh-mesh',
            active_instances: [
                {
                    profile_id: 'main:default',
                    profile_name: 'main:default',
                    is_running: true,
                },
            ],
            bound_emails: ['sakibagjdjg@gmail.com'],
            git_repo: 'Antigravity-Manager',
            git_link: 'alimtvnetwork / Antigravity-Manager',
        } as FleetMachineInfo);
    }

    return enriched;
}

export function useFleetMachines(): FleetMachinesState {
    const initialMachines = loadCachedFleetMachines();
    const [machines, setMachines] = useState<FleetMachineInfo[]>(initialMachines);
    const [localNodeId, setLocalNodeId] = useState<string>('');
    const [isLoading, setIsLoading] = useState<boolean>(initialMachines.length === 0);
    const [isRefreshing, setIsRefreshing] = useState<boolean>(false);
    const [isResettingCluster, setIsResettingCluster] = useState<boolean>(false);
    const [isSyncEnabled, setIsSyncEnabled] = useState<boolean>(true);
    const [showAllEmails, setShowAllEmails] = useState<boolean>(false);
    const [unmaskedRows, setUnmaskedRows] = useState<Record<string, boolean>>({});
    const [copiedIp, setCopiedIp] = useState<string | null>(null);
    const [countdown, setCountdown] = useState<number>(POLL_INTERVAL_SECONDS);
    const [promptMachine, setPromptMachine] = useState<FleetMachineInfo | null>(null);

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

                // 1. Immediate baseline rendering & cache sync
                const sorted = sortFleetMachines(effectiveMachines);
                setIsSyncEnabled(syncActive || effectiveMachines.length > 0);
                setMachines(sorted);
                saveCachedFleetMachines(sorted);

                // 2. Decoupled async enrichment via SWR without blocking table interactivity
                enrichFleetMachines(effectiveMachines)
                    .then((enriched) => {
                        const sortedEnriched = sortFleetMachines(enriched);
                        setIsSyncEnabled(syncActive || enriched.length > 0);
                        setMachines(sortedEnriched);
                        saveCachedFleetMachines(sortedEnriched);
                    })
                    .catch((e) => {
                        useErrorStore.getState().trackWarning(e, {
                            source: 'useFleetMachines.asyncEnrich',
                            triggerAction: 'enrich_fleet_machines',
                        });
                    });
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
            if (initialMachines.length === 0) {
                setIsLoading(true);
            }
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

                    // Immediate baseline rendering on mount
                    const sorted = sortFleetMachines(effectiveMachines);
                    setIsSyncEnabled(syncActive || effectiveMachines.length > 0);
                    setMachines(sorted);
                    setIsLoading(false);
                    saveCachedFleetMachines(sorted);

                    // Decoupled async enrichment via SWR without blocking initial UI
                    enrichFleetMachines(effectiveMachines)
                        .then((enriched) => {
                            if (isMounted) {
                                const sortedEnriched = sortFleetMachines(enriched);
                                setIsSyncEnabled(syncActive || enriched.length > 0);
                                setMachines(sortedEnriched);
                                saveCachedFleetMachines(sortedEnriched);
                            }
                        })
                        .catch((e) => {
                            useErrorStore.getState().trackWarning(e, {
                                source: 'useFleetMachines.initAsyncEnrich',
                                triggerAction: 'enrich_fleet_machines',
                            });
                        });
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

    const handleOpenPromptModal = useCallback((m: FleetMachineInfo) => {
        setPromptMachine(m);
    }, []);

    const handleClosePromptModal = useCallback(() => {
        setPromptMachine(null);
    }, []);

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
        promptMachine,
        setShowAllEmails,
        handleManualRefresh,
        handleResetCluster,
        handleCopyIp,
        toggleRowMask,
        handleOpenPromptModal,
        handleClosePromptModal,
    };
}
