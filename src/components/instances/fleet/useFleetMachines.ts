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
        const aLocal = Boolean((a as { is_local?: boolean }).is_local || a.node_alias === 'w1');
        const bLocal = Boolean((b as { is_local?: boolean }).is_local || b.node_alias === 'w1');
        if (aLocal !== bLocal) return bLocal ? 1 : -1;
        return (a.node_alias || '').localeCompare(b.node_alias || '');
    });
}

function mapSshNodesToFleet(sshNodes: SshNodeItem[], _localId?: string): FleetMachineInfo[] {
    return sshNodes
        .filter((n) => Boolean(n.alias))
        .map((n) => {
            const isLocal =
                n.alias.toLowerCase() === 'w1' ||
                n.alias.toLowerCase().startsWith('local');
            return {
                node_id: `ssh_${n.alias}`,
                node_alias: n.alias,
                ip_address: n.ip,
                is_online: n.status === 'online' || n.status === 'ready' || isLocal,
                last_heartbeat_timestamp: Math.floor(Date.now() / 1000),
                uptime_seconds: 0,
                in_flight_prompts_count: 0,
                is_local: isLocal,
                source: isLocal ? 'local' : 'ssh-mesh',
                active_instances: [
                    {
                        profile_id: `${n.alias}:default`,
                        profile_name: `${n.alias}:default`,
                        is_running: true,
                    },
                ],
                bound_emails: [],
            };
        });
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

                setIsSyncEnabled(syncActive || effectiveMachines.length > 0);
                setMachines(sortFleetMachines(effectiveMachines));
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

                    setIsSyncEnabled(syncActive || effectiveMachines.length > 0);
                    setMachines(sortFleetMachines(effectiveMachines));
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
