import { useEffect, useState, useCallback } from 'react';
import {
    Check,
    CheckCircle2,
    Copy,
    Cpu,
    Download,
    Eye,
    EyeOff,
    KeyRound,
    Layers,
    Loader2,
    Lock,
    Network,
    Play,
    Plus,
    RefreshCw,
    Send,
    Server,
    ShieldCheck,
    Terminal,
    Trash2,
    UploadCloud,
    AlertTriangle,
    RotateCcw,
} from 'lucide-react';
import {
    sshService,
    type ClusterInstanceBalanceItem,
    type RemotePromptDispatchResult,
    type SshGitmapStatus,
    type SshNodeRecord,
} from '../../../services/sshService';
import { copyToClipboard } from '../../../utils/clipboard';
import { showToast } from '../../common/ToastContainer';
import { useErrorStore } from '../../../stores/error-store';
import { maskEmailAddress } from '../../instances/fleet/fleetFormatUtils';

const toast = {
    success: (msg: string) => showToast(msg, 'success'),
    error: (msg: string) => showToast(msg, 'error'),
    info: (msg: string) => showToast(msg, 'info'),
    warning: (msg: string) => showToast(msg, 'warning'),
};

function isAdminIdentifier(val?: string | null): boolean {
    if (!val) return false;
    const lower = val.toLowerCase().trim();
    return (
        lower.includes('administrator') ||
        lower.includes('root') ||
        lower.includes('${adminuser}') ||
        lower.includes('${linuxuser}') ||
        lower.includes(':') ||
        lower.startsWith('password')
    );
}

function sanitizeAdminText(text?: string | null): string {
    if (!text) return '';
    return text
        .replace(/\$\{adminUser\}|\$\{linuxUser\}/gi, '')
        .replace(/\b(administrator|root)\b/gi, '')
        .trim();
}

export function SshGitmapSection() {
    const [status, setStatus] = useState<SshGitmapStatus | null>(null);
    const [nodes, setNodes] = useState<SshNodeRecord[]>([]);
    const [clusterBalances, setClusterBalances] = useState<ClusterInstanceBalanceItem[]>([]);

    const [isLoadingStatus, setIsLoadingStatus] = useState(false);
    const [isInstallingGitmap, setIsInstallingGitmap] = useState(false);
    const [isImportingGitmap, setIsImportingGitmap] = useState(false);
    const [isDeployingFleet, setIsDeployingFleet] = useState(false);
    const [isSavingNode, setIsSavingNode] = useState(false);
    const [isLoadingBalances, setIsLoadingBalances] = useState(false);
    const [testingNodeAlias, setTestingNodeAlias] = useState<string | null>(null);
    const [deletingNodeAlias, setDeletingNodeAlias] = useState<string | null>(null);
    const [isSendingPrompt, setIsSendingPrompt] = useState(false);
    const [isExportingCsv, setIsExportingCsv] = useState(false);

    const [copiedKey, setCopiedKey] = useState<string | null>(null);
    const [unmaskedEmailNodes, setUnmaskedEmailNodes] = useState<Record<string, boolean>>({});

    const toggleEmailMask = (key: string) => {
        setUnmaskedEmailNodes((prev) => ({ ...prev, [key]: !prev[key] }));
    };

    // SSH Node Form State (RSA-2048 Encrypted Password on First Login)
    const [alias, setAlias] = useState('');
    const [ipAddress, setIpAddress] = useState('');
    const [port, setPort] = useState<number>(22);
    const [username, setUsername] = useState('');
    const [osType, setOsType] = useState('windows');
    const [keyPath, setKeyPath] = useState('~/.ssh/id_ed25519');
    const [plaintextPassword, setPlaintextPassword] = useState('');
    const [showPassword, setShowPassword] = useState(false);

    // Remote Prompt Dispatcher State
    const [targetMachine, setTargetMachine] = useState('local');
    const [targetInstance, setTargetInstance] = useState('default');
    const [promptText, setPromptText] = useState('');
    const [lastPromptResult, setLastPromptResult] = useState<RemotePromptDispatchResult | null>(null);

    const handleCopyChip = async (key: string, text: string, label?: string) => {
        const ok = await copyToClipboard(text);
        if (ok) {
            setCopiedKey(key);
            showToast(`Copied: ${label || text}`, 'info');
            setTimeout(() => {
                setCopiedKey((prev) => (prev === key ? null : prev));
            }, 1800);
        }
    };

    const loadAllSshData = useCallback(async () => {
        setIsLoadingStatus(true);
        try {
            const [st, nodeList] = await Promise.all([
                sshService.getSshGitmapStatus(),
                sshService.listSshNodes(),
            ]);
            setStatus(st);
            setNodes(nodeList);
            if (nodeList.length > 0 && targetMachine === 'local') {
                setTargetMachine(nodeList[0].alias);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.loadAllSshData',
                triggerAction: 'load_ssh_status_and_nodes',
            });
        } finally {
            setIsLoadingStatus(false);
        }
    }, [targetMachine]);

    const handleRefreshClusterBalances = useCallback(async () => {
        setIsLoadingBalances(true);
        try {
            const res = await sshService.getClusterInstancesAndBalances();
            setClusterBalances(res);
            const onlineCount = res.filter((x) => x.running || x.online).length;
            const offlineCount = res.length - onlineCount;
            toast.success(`Checked ${res.length} cluster nodes (${onlineCount} online, ${offlineCount} offline)`);
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleRefreshClusterBalances',
                triggerAction: 'query_cluster_balances',
            });
            showToast(`Failed to query multi-machine balances: ${err?.message || err}`, 'error');
        } finally {
            setIsLoadingBalances(false);
        }
    }, []);

    useEffect(() => {
        loadAllSshData();
        handleRefreshClusterBalances();
    }, [loadAllSshData, handleRefreshClusterBalances]);

    const handleInstallGitmapPowershell = async () => {
        setIsInstallingGitmap(true);
        try {
            const msg = await sshService.installGitmapPowershell(true);
            showToast(msg || 'Opened PowerShell to install GitMap CLI', 'success');
            await loadAllSshData();
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleInstallGitmapPowershell',
                triggerAction: 'install_gitmap_powershell',
            });
            showToast(`GitMap installer failed: ${err?.message || err}`, 'error');
        } finally {
            setIsInstallingGitmap(false);
        }
    };

    const handleImportFromGitmap = async () => {
        setIsImportingGitmap(true);
        try {
            const stats = await sshService.importSshNodesFromGitmap();
            showToast(
                `Imported SSH nodes from GitMap (total: ${stats.total}, inserted: ${stats.inserted}, updated: ${stats.updated})`,
                'success'
            );
            await loadAllSshData();
            await handleRefreshClusterBalances();
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleImportFromGitmap',
                triggerAction: 'import_ssh_from_gitmap',
            });
            showToast(`Failed to import SSH nodes from GitMap: ${err?.message || err}`, 'error');
        } finally {
            setIsImportingGitmap(false);
        }
    };

    const handleDeployEncryptedFleet = async () => {
        setIsDeployingFleet(true);
        try {
            const summary = await sshService.deployEncryptedSshBundle();
            const count = summary?.node_reports?.length ?? nodes.length;
            showToast(
                summary?.message ||
                    `Deployed RSA-2048 encrypted SSH credentials bundle across ${count} fleet node(s).`,
                'success'
            );
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleDeployEncryptedFleet',
                triggerAction: 'deploy_encrypted_fleet',
            });
            showToast(`Failed to deploy encrypted credentials: ${err?.message || err}`, 'error');
        } finally {
            setIsDeployingFleet(false);
        }
    };

    const handleSaveNodeRsa = async (e: React.FormEvent) => {
        e.preventDefault();
        const trimmedAlias = alias.trim();
        const trimmedIp = ipAddress.trim();
        const trimmedUser = username.trim();

        if (!trimmedAlias || !trimmedIp || !trimmedUser) {
            showToast('Please provide Machine Alias, IP / Hostname, and Username', 'warning');
            return;
        }

        setIsSavingNode(true);
        try {
            const saved = await sshService.saveSshNodeRsa({
                alias: trimmedAlias,
                ip_address: trimmedIp,
                port: Number(port) || 22,
                username: trimmedUser,
                os: osType,
                key_path: keyPath.trim() || undefined,
                plaintext_password: plaintextPassword ? plaintextPassword : null,
            });
            // Immediately wipe plaintext password from component state
            setPlaintextPassword('');
            showToast(
                `SSH node "${saved.alias}" saved with RSA-2048 encrypted vault credentials.`,
                'success'
            );
            setAlias('');
            setIpAddress('');
            await loadAllSshData();
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleSaveNodeRsa',
                triggerAction: 'save_ssh_node_rsa',
            });
            showToast(`Failed to save SSH node: ${err?.message || err}`, 'error');
        } finally {
            setIsSavingNode(false);
        }
    };

    const handleTestNode = async (nodeAlias: string) => {
        setTestingNodeAlias(nodeAlias);
        try {
            const res = await sshService.testSshNodeConnection(nodeAlias);
            setLastPromptResult(res);
            if (res.success) {
                showToast(`SSH connection to "${nodeAlias}" verified (${res.duration_ms}ms)`, 'success');
            } else {
                const reason = res.stderr?.trim() || res.stdout?.trim() || 'Connection failed or node unreachable';
                showToast(`SSH test on "${nodeAlias}" failed: ${reason}`, 'error');
            }
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleTestNode',
                triggerAction: 'test_ssh_node',
            });
            showToast(`SSH test failed for "${nodeAlias}": ${err?.message || err}`, 'error');
        } finally {
            setTestingNodeAlias(null);
        }
    };

    const handleDeleteNode = async (nodeAlias: string) => {
        setDeletingNodeAlias(nodeAlias);
        try {
            await sshService.deleteSshNode(nodeAlias);
            showToast(`Removed SSH node "${nodeAlias}"`, 'info');
            await loadAllSshData();
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleDeleteNode',
                triggerAction: 'delete_ssh_node',
            });
            showToast(`Failed to delete SSH node "${nodeAlias}": ${err?.message || err}`, 'error');
        } finally {
            setDeletingNodeAlias(null);
        }
    };

    const handleDispatchRemotePrompt = async (
        overrideMachine?: string,
        overrideInstance?: string,
        overridePrompt?: string
    ) => {
        const machine = (overrideMachine ?? targetMachine).trim() || 'local';
        const instance = (overrideInstance ?? targetInstance).trim() || 'default';
        const text = (overridePrompt ?? promptText).trim();

        if (!text) {
            showToast('Please enter a prompt instruction to send to the target instance', 'warning');
            return;
        }

        setIsSendingPrompt(true);
        try {
            const res = await sshService.sendRemoteSshPrompt({
                node_alias: machine,
                instance_target: instance,
                prompt_text: text,
            });
            setLastPromptResult(res);
            if (res.success) {
                showToast(`Dispatched prompt to ${machine}:${instance}`, 'success');
            } else {
                showToast(`Prompt dispatch to ${machine}:${instance} reported stderr`, 'warning');
            }
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleDispatchRemotePrompt',
                triggerAction: 'dispatch_remote_prompt',
            });
            showToast(`Failed to send prompt to ${machine}:${instance}: ${err?.message || err}`, 'error');
        } finally {
            setIsSendingPrompt(false);
        }
    };

    const handleExportClusterCsv = async () => {
        setIsExportingCsv(true);
        try {
            const csv = await sshService.exportSystemDataCsv({
                target: 'balance',
                format: 'csv',
                sendToTelegram: false,
            });
            await sshService.triggerDownloadAndCopy(csv, 'agm-cluster-balances.csv', 'csv');
            showToast('Cluster instances & balances CSV downloaded and copied to clipboard', 'success');
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SshGitmapSection.handleExportClusterCsv',
                triggerAction: 'export_cluster_csv',
            });
            showToast(`Failed to export CSV: ${err?.message || err}`, 'error');
        } finally {
            setIsExportingCsv(false);
        }
    };

    const installCmd = status?.install_command || 'agm ssh install-gitmap';

    return (
        <div
            id="ssh-gitmap-fleet-section"
            className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800 space-y-5"
        >
            {/* 1. Header & Fleet Action Segmented Capsule */}
            <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-3">
                <div>
                    <div className="flex flex-wrap items-center gap-2">
                        <h3 className="text-sm sm:text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Server className="w-4 h-4 text-indigo-500" />
                            SSH &amp; GitMap Multi-Machine Fleet Control
                        </h3>
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[11px] font-semibold bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800/70">
                            <Lock className="w-3 h-3" />
                            🔒 RSA-2048 Encrypted Vault (Zero Plaintext Storage)
                        </span>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-slate-400 mt-0.5">
                        Import SSH topologies from GitMap, encrypt first-time login passwords with RSA-2048 asymmetric keys, inspect multi-machine account credits, and delegate prompts across remote instances.
                    </p>
                </div>

                {/* Contiguous Segmented Pill Capsule for Fleet Toolbar Actions */}
                <div className="inline-flex items-center rounded-full bg-slate-900/90 dark:bg-slate-900/90 border border-slate-700/80 p-0.5 shadow-sm divide-x divide-slate-700/60 backdrop-blur-sm self-start lg:self-auto">
                    <button
                        type="button"
                        onClick={handleImportFromGitmap}
                        disabled={isImportingGitmap}
                        className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-l-full text-xs font-medium text-slate-300 hover:text-white hover:bg-slate-800/80 active:scale-95 transition-all disabled:opacity-50 cursor-pointer"
                        title="Import SSH nodes from GitMap inventory and ~/.ssh/config"
                    >
                        {isImportingGitmap ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin text-indigo-400" />
                        ) : (
                            <Download className="w-3.5 h-3.5 text-indigo-400" />
                        )}
                        <span>Import SSH from GitMap</span>
                    </button>
                    <button
                        type="button"
                        onClick={handleDeployEncryptedFleet}
                        disabled={isDeployingFleet}
                        className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-slate-300 hover:text-white hover:bg-slate-800/80 active:scale-95 transition-all disabled:opacity-50 cursor-pointer"
                        title="Deploy RSA-2048 encrypted SSH credentials bundle to authorized fleet machines"
                    >
                        {isDeployingFleet ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin text-emerald-400" />
                        ) : (
                            <UploadCloud className="w-3.5 h-3.5 text-emerald-400" />
                        )}
                        <span>Deploy Encrypted Credentials to Fleet</span>
                    </button>
                    <button
                        type="button"
                        onClick={handleRefreshClusterBalances}
                        disabled={isLoadingBalances}
                        className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-r-full text-xs font-medium text-sky-400 hover:text-sky-300 hover:bg-slate-800/80 active:scale-95 transition-all disabled:opacity-50 cursor-pointer"
                        title="Poll local and SSH nodes for active instances and 4h/weekly account balances"
                    >
                        {isLoadingBalances ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin text-sky-400" />
                        ) : (
                            <RefreshCw className="w-3.5 h-3.5 text-sky-400" />
                        )}
                        <span>Check Cluster Balances</span>
                    </button>
                </div>
            </div>

            {/* 2. GitMap Detection & One-Click PowerShell Installer Banner */}
            <div
                className={`p-3.5 rounded-xl border transition-colors ${
                    status?.gitmap_installed
                        ? 'bg-emerald-50/40 dark:bg-emerald-950/20 border-emerald-200/70 dark:border-emerald-900/50'
                        : 'bg-amber-50/60 dark:bg-amber-950/25 border-amber-200 dark:border-amber-800/60'
                }`}
            >
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-3">
                    <div className="space-y-1.5">
                        <div className="flex flex-wrap items-center gap-2">
                            {status?.gitmap_installed ? (
                                <span
                                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-500/15 text-emerald-400 dark:text-emerald-300 border border-emerald-500/30"
                                    title={`GitMap Binary: ${status.gitmap_path || 'gitmap'}`}
                                >
                                    <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                                    GitMap Installed ({status.gitmap_version || 'v6.x'})
                                </span>
                            ) : (
                                <span
                                    className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-amber-500/15 text-amber-400 dark:text-amber-300 border border-amber-500/30"
                                    title="GitMap CLI not found on PATH or default local locations"
                                >
                                    <AlertTriangle className="w-3.5 h-3.5 text-amber-400" />
                                    GitMap Not Detected
                                </span>
                            )}

                            <span
                                className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-purple-500/15 text-purple-400 dark:text-purple-300 border border-purple-500/30"
                                title={status?.rsa_key_fingerprint ? `RSA Fingerprint: ${status.rsa_key_fingerprint}` : 'RSA-2048 Vault Key Active'}
                            >
                                <KeyRound className="w-3.5 h-3.5 text-purple-400" />
                                RSA-2048 Vault Key Active
                            </span>

                            <span
                                className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-slate-700/50 text-slate-300 border border-slate-600/50"
                                title={`Managed SSH Config: ${status?.ssh_config_path || '~/.ssh/config'}`}
                            >
                                <Server className="w-3.5 h-3.5 text-slate-400" />
                                {nodes.length} Nodes Configured
                            </span>

                            {isLoadingStatus && (
                                <Loader2 className="w-3.5 h-3.5 animate-spin text-gray-400" />
                            )}
                        </div>

                        <details className="text-[11px] text-gray-500 dark:text-slate-400 cursor-pointer group">
                            <summary className="hover:text-gray-700 dark:hover:text-slate-300 transition-colors list-none flex items-center gap-1">
                                <span className="underline decoration-dotted">Inspect paths &amp; environment</span>
                                <span className="text-[9px] opacity-70 group-open:rotate-180 transition-transform">▼</span>
                            </summary>
                            <div className="mt-1 pl-2 text-[10px] font-mono text-gray-500 dark:text-slate-400 border-l border-slate-700 space-y-0.5">
                                <div>Binary: {status?.gitmap_path || 'gitmap'}</div>
                                <div>SSH Config: {status?.ssh_config_path || '~/.ssh/config'}</div>
                                {status?.rsa_key_fingerprint && <div>Fingerprint: {status.rsa_key_fingerprint}</div>}
                            </div>
                        </details>
                    </div>

                    <div className="inline-flex items-center rounded-full bg-slate-900/90 border border-slate-700/80 p-0.5 shadow-sm divide-x divide-slate-700/60 self-start md:self-auto">
                        <button
                            type="button"
                            onClick={() => handleCopyChip('gitmap-install-cmd', installCmd, 'GitMap install command')}
                            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-l-full text-xs font-medium text-slate-300 hover:text-white hover:bg-slate-800/80 active:scale-95 transition-all cursor-pointer"
                            title={`Copy: ${installCmd}`}
                        >
                            {copiedKey === 'gitmap-install-cmd' ? (
                                <Check className="w-3.5 h-3.5 text-emerald-400" />
                            ) : (
                                <Copy className="w-3.5 h-3.5 text-slate-400" />
                            )}
                            <span>{copiedKey === 'gitmap-install-cmd' ? 'Copied Command!' : 'Copy Install Command'}</span>
                        </button>
                        <button
                            type="button"
                            onClick={handleInstallGitmapPowershell}
                            disabled={isInstallingGitmap}
                            className="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-r-full text-xs font-semibold text-cyan-400 hover:text-cyan-300 hover:bg-slate-800/80 active:scale-95 transition-all cursor-pointer disabled:opacity-50"
                            title="Spawn PowerShell to install or update GitMap automatically"
                        >
                            {isInstallingGitmap ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin text-cyan-400" />
                            ) : (
                                <Terminal className="w-3.5 h-3.5 text-cyan-400" />
                            )}
                            <span>Open PowerShell &amp; Install GitMap</span>
                        </button>
                    </div>
                </div>
            </div>

            {/* 3. SSH Node Configuration Form with Immediate RSA-2048 Password Encryption */}
            <form
                onSubmit={handleSaveNodeRsa}
                className="p-4 rounded-xl bg-gray-50/70 dark:bg-slate-800/50 border border-gray-200/80 dark:border-slate-700/70 space-y-3.5"
            >
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                        <ShieldCheck className="w-4 h-4 text-emerald-500" />
                        <h4 className="text-xs sm:text-sm font-semibold text-gray-900 dark:text-slate-100">
                            Add / Update SSH Machine Node (RSA-2048 Password Encryption)
                        </h4>
                    </div>
                    <span className="text-[11px] text-emerald-700 dark:text-emerald-400 font-medium">
                        Password is asked on first login, encrypted immediately via RSA-2048 (ag_rsa_v1_), and never stored as plaintext.
                    </span>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-6 gap-3">
                    <div>
                        <label className="block text-[11px] font-medium text-gray-700 dark:text-slate-300 mb-1">
                            Machine Alias *
                        </label>
                        <input
                            type="text"
                            value={alias}
                            onChange={(e) => setAlias(e.target.value)}
                            placeholder="W3 or vm-prod-01"
                            className="w-full h-8 px-2.5 text-xs rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        />
                    </div>

                    <div>
                        <label className="block text-[11px] font-medium text-gray-700 dark:text-slate-300 mb-1">
                            IP / Hostname *
                        </label>
                        <input
                            type="text"
                            value={ipAddress}
                            onChange={(e) => setIpAddress(e.target.value)}
                            placeholder="192.168.1.3"
                            className="w-full h-8 px-2.5 text-xs rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        />
                    </div>

                    <div>
                        <label className="block text-[11px] font-medium text-gray-700 dark:text-slate-300 mb-1">
                            Port &amp; OS
                        </label>
                        <div className="flex items-center rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 overflow-hidden h-8">
                            <input
                                type="number"
                                min={1}
                                max={65535}
                                value={port}
                                onChange={(e) => setPort(parseInt(e.target.value, 10) || 22)}
                                className="w-16 px-2 text-xs text-gray-900 dark:text-slate-100 font-mono bg-transparent border-0 focus:outline-none"
                            />
                            <div className="h-4 w-px bg-gray-200 dark:bg-slate-700" />
                            <select
                                value={osType}
                                onChange={(e) => setOsType(e.target.value)}
                                className="flex-1 px-2 text-xs text-gray-700 dark:text-slate-200 bg-transparent border-0 focus:ring-0 focus:outline-none"
                            >
                                <option value="windows">Windows</option>
                                <option value="linux">Linux</option>
                                <option value="macos">macOS</option>
                            </select>
                        </div>
                    </div>

                    <div>
                        <label className="block text-[11px] font-medium text-gray-700 dark:text-slate-300 mb-1">
                            SSH OS User *
                        </label>
                        <input
                            type="text"
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
                            placeholder="SSH user (e.g. deploy or agm-agent)"
                            className="w-full h-8 px-2.5 text-xs rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        />
                    </div>

                    <div>
                        <label className="block text-[11px] font-medium text-gray-700 dark:text-slate-300 mb-1">
                            SSH Key Path (Optional)
                        </label>
                        <input
                            type="text"
                            value={keyPath}
                            onChange={(e) => setKeyPath(e.target.value)}
                            placeholder="~/.ssh/id_ed25519"
                            className="w-full h-8 px-2.5 text-xs rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        />
                    </div>

                    <div>
                        <label className="block text-[11px] font-medium text-gray-700 dark:text-slate-300 mb-1">
                            First-Time Password (RSA-2048)
                        </label>
                        <div className="relative">
                            <input
                                type={showPassword ? 'text' : 'password'}
                                value={plaintextPassword}
                                onChange={(e) => setPlaintextPassword(e.target.value)}
                                placeholder="Encrypted before disk write"
                                className="w-full h-8 px-2.5 pr-8 text-xs rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            />
                            <button
                                type="button"
                                onClick={() => setShowPassword(!showPassword)}
                                className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-slate-200 cursor-pointer"
                            >
                                {showPassword ? (
                                    <EyeOff className="w-3.5 h-3.5" />
                                ) : (
                                    <Eye className="w-3.5 h-3.5" />
                                )}
                            </button>
                        </div>
                    </div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
                    <div className="flex flex-wrap items-center gap-1.5 text-[11px] text-gray-500 dark:text-slate-400">
                        <span>CLI Equivalents:</span>
                        {[
                            { id: 'cli-import', cmd: 'agm ssh import-gitmap' },
                            { id: 'cli-set-pw', cmd: 'agm ssh set-password W3 "<password>"' },
                            { id: 'cli-nodes', cmd: 'agm ssh nodes' },
                        ].map((item) => (
                            <button
                                key={item.id}
                                type="button"
                                onClick={() => handleCopyChip(item.id, item.cmd)}
                                className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-700 dark:text-slate-300 font-mono text-[10px] hover:border-indigo-400 transition-colors cursor-pointer"
                            >
                                {copiedKey === item.id ? (
                                    <Check className="w-2.5 h-2.5 text-emerald-500" />
                                ) : (
                                    <Copy className="w-2.5 h-2.5 text-gray-400" />
                                )}
                                <span>{copiedKey === item.id ? 'Copied!' : item.cmd}</span>
                            </button>
                        ))}
                    </div>

                    <button
                        type="submit"
                        disabled={isSavingNode}
                        className="px-4 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-700 text-white text-xs font-semibold transition-colors shadow-xs flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                    >
                        {isSavingNode ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        ) : (
                            <Plus className="w-3.5 h-3.5" />
                        )}
                        <span>Save &amp; Encrypt Node (RSA-2048)</span>
                    </button>
                </div>
            </form>

            {/* 4. Configured SSH Nodes Table */}
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <h4 className="text-xs sm:text-sm font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-1.5">
                        <Network className="w-4 h-4 text-sky-500" />
                        Configured SSH Nodes ({nodes.length})
                    </h4>
                    <span className="text-[11px] text-gray-500 dark:text-slate-400">
                        Synced with ~/.ssh/config &amp; RSA-2048 credential vault
                    </span>
                </div>

                {nodes.length === 0 ? (
                    <div className="p-4 rounded-xl border border-dashed border-gray-200 dark:border-slate-800 text-center text-xs text-gray-500 dark:text-slate-400">
                        No remote SSH nodes registered yet. Add a machine above or click{' '}
                        <span className="font-semibold text-indigo-600 dark:text-indigo-400">
                            Import SSH from GitMap
                        </span>
                        .
                    </div>
                ) : (
                    <div className="overflow-x-auto rounded-xl border border-gray-200 dark:border-slate-800">
                        <table className="w-full text-left border-collapse text-xs">
                            <thead>
                                <tr className="bg-gray-50 dark:bg-slate-800/80 text-gray-600 dark:text-slate-300 border-b border-gray-200 dark:border-slate-800">
                                    <th className="py-2 px-3 font-semibold">Node &amp; Alias</th>
                                    <th className="py-2 px-3 font-semibold">IP / Host:Port</th>
                                    <th className="py-2 px-3 font-semibold">Instance Count &amp; Status</th>
                                    <th className="py-2 px-3 font-semibold">Bound Account Email(s)</th>
                                    <th className="py-2 px-3 font-semibold">Security &amp; Auth</th>
                                    <th className="py-2 px-3 font-semibold text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-gray-100 dark:divide-slate-800/70">
                                {nodes.map((node) => {
                                    const nodeInstances = clusterBalances.filter(
                                        (item) =>
                                            item.machine_alias.toLowerCase() === node.alias.toLowerCase() ||
                                            (Boolean(node.ip_address) && (item.machine_ip === node.ip_address || item.ip_address === node.ip_address))
                                    );
                                    const runningCount = nodeInstances.filter((item) => item.running).length;
                                    const totalCount = nodeInstances.length;
                                    const nodeEmails = Array.from(
                                        new Set(
                                            nodeInstances
                                                .map((item) => item.bound_email)
                                                .filter(
                                                    (email): email is string =>
                                                        typeof email === 'string' &&
                                                        email.trim().length > 0 &&
                                                        !isAdminIdentifier(email)
                                                )
                                        )
                                    );
                                    const cleanNodeAlias = sanitizeAdminText(node.alias) || 'Node';

                                    return (
                                        <tr
                                            key={node.alias}
                                            className="hover:bg-gray-50/60 dark:hover:bg-slate-800/40 transition-colors"
                                        >
                                            <td className="py-2 px-3 font-mono">
                                                <div className="flex items-center gap-1.5">
                                                    <span className="font-semibold text-gray-900 dark:text-slate-100" title={cleanNodeAlias}>
                                                        {cleanNodeAlias}
                                                    </span>
                                                    <span className="uppercase text-[9px] font-bold px-1.5 py-0.2 rounded bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300">
                                                        {node.os || 'linux'}
                                                    </span>
                                                </div>
                                            </td>
                                            <td className="py-2 px-3 font-mono text-gray-600 dark:text-slate-300">
                                                <div className="inline-flex items-center gap-1 font-mono text-[11px] text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800/40 px-1.5 py-0.5 rounded border border-slate-200 dark:border-slate-700/50" title={`Host: ${node.ip_address || (node as any).host || 'unknown'}:${node.port || 22}`}>
                                                    <span>
                                                        {(node.ip_address || (node as any).host || 'unknown')}:{node.port || 22}
                                                    </span>
                                                    {node.ip_address && (
                                                        <button
                                                            type="button"
                                                            onClick={() => handleCopyChip(`ip-${node.alias}`, node.ip_address, 'Node IP')}
                                                            className="p-0.5 rounded hover:bg-slate-200 dark:hover:bg-slate-700 text-slate-400 cursor-pointer"
                                                            title="Copy IP Address"
                                                        >
                                                            {copiedKey === `ip-${node.alias}` ? (
                                                                <Check className="w-2.5 h-2.5 text-emerald-500" />
                                                            ) : (
                                                                <Copy className="w-2.5 h-2.5" />
                                                            )}
                                                        </button>
                                                    )}
                                                </div>
                                            </td>
                                            <td className="py-2 px-3 font-mono">
                                                {totalCount > 0 ? (
                                                    <div className="flex flex-col gap-0.5">
                                                        <div className="flex items-center gap-1.5 flex-wrap">
                                                            <span className="font-semibold text-gray-800 dark:text-slate-200 text-xs">
                                                                {totalCount} Instance{totalCount === 1 ? '' : 's'}
                                                            </span>
                                                            {runningCount > 0 ? (
                                                                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
                                                                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                                                    {runningCount} Running
                                                                </span>
                                                            ) : (
                                                                <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-medium bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 border border-slate-200 dark:border-slate-700">
                                                                    All Idle
                                                                </span>
                                                            )}
                                                        </div>
                                                        {totalCount > runningCount && runningCount > 0 && (
                                                            <span className="text-[10px] text-gray-400 dark:text-slate-400">
                                                                {totalCount - runningCount} Idle
                                                            </span>
                                                        )}
                                                    </div>
                                                ) : (
                                                    <span className="text-gray-400 italic text-[11px]">
                                                        {isLoadingBalances ? 'Scanning...' : '0 instances (unverified)'}
                                                    </span>
                                                )}
                                            </td>
                                            <td className="py-2 px-3 font-mono text-gray-700 dark:text-slate-300">
                                                {nodeEmails.length > 0 ? (
                                                    <div className="flex items-center gap-1.5 flex-wrap">
                                                        <div className="flex flex-col gap-0.5">
                                                            {nodeEmails.map((em) => {
                                                                const isUnmasked = Boolean(unmaskedEmailNodes[node.alias]);
                                                                return (
                                                                    <span
                                                                        key={em}
                                                                        className="text-[11px] text-sky-600 dark:text-sky-300 font-mono font-medium"
                                                                        title={isUnmasked ? em : 'Masked email (click eye icon to toggle)'}
                                                                    >
                                                                        {isUnmasked ? em : maskEmailAddress(em)}
                                                                    </span>
                                                                );
                                                            })}
                                                        </div>
                                                        <button
                                                            type="button"
                                                            onClick={() => toggleEmailMask(node.alias)}
                                                            className="p-1 rounded-full hover:bg-slate-200 dark:hover:bg-slate-700 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors cursor-pointer"
                                                            title={
                                                                unmaskedEmailNodes[node.alias]
                                                                    ? 'Mask bound account emails for this node'
                                                                    : 'Reveal bound account emails for this node'
                                                            }
                                                        >
                                                            {unmaskedEmailNodes[node.alias] ? (
                                                                <EyeOff className="w-3 h-3 text-indigo-500" />
                                                            ) : (
                                                                <Eye className="w-3 h-3" />
                                                            )}
                                                        </button>
                                                    </div>
                                                ) : (
                                                    <span className="text-gray-400 italic text-[11px]">(unbound)</span>
                                                )}
                                            </td>
                                            <td className="py-2 px-3">
                                                {(node.has_encrypted_password || (node as any).has_password) ? (
                                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
                                                        <Lock className="w-2.5 h-2.5" />
                                                        RSA-2048 Vault
                                                    </span>
                                                ) : (
                                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 border border-sky-200 dark:border-sky-800">
                                                        <KeyRound className="w-2.5 h-2.5" />
                                                        SSH Key Auth
                                                    </span>
                                                )}
                                            </td>
                                        <td className="py-2 px-3 text-right">
                                            <div className="inline-flex items-center rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-0.5">
                                                <button
                                                    type="button"
                                                    onClick={() => handleTestNode(node.alias)}
                                                    disabled={testingNodeAlias === node.alias}
                                                    className="px-2.5 py-1 text-[11px] font-medium text-gray-700 dark:text-slate-200 hover:bg-gray-100 dark:hover:bg-slate-800 rounded-l-full flex items-center gap-1 cursor-pointer disabled:opacity-50"
                                                    title="Test SSH connectivity"
                                                >
                                                    {testingNodeAlias === node.alias ? (
                                                        <Loader2 className="w-3 h-3 animate-spin" />
                                                    ) : (
                                                        <Play className="w-3 h-3 text-emerald-500" />
                                                    )}
                                                    <span>Test Connection</span>
                                                </button>
                                                <div className="h-3.5 w-px bg-gray-200 dark:bg-slate-700" />
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        setTargetMachine(node.alias);
                                                        const matching = clusterBalances.filter(
                                                            (item) => item.machine_alias.toLowerCase() === node.alias.toLowerCase()
                                                        );
                                                        const cleanInst = matching.length > 0 ? (matching[0].instance_id || 'default') : 'default';
                                                        setTargetInstance(cleanInst);
                                                        showToast(`Selected ${node.alias}:${cleanInst} for prompting`, 'info');
                                                        const promptInput = document.getElementById('remote-prompt-input');
                                                        if (promptInput) {
                                                            promptInput.scrollIntoView({ behavior: 'smooth', block: 'center' });
                                                            promptInput.focus();
                                                        }
                                                    }}
                                                    className="px-2.5 py-1 text-[11px] font-medium text-sky-600 dark:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-950/40 flex items-center gap-1 cursor-pointer"
                                                    title="Target this machine in the Prompt Dispatcher below"
                                                >
                                                    <Send className="w-3 h-3" />
                                                    <span>Prompt Machine</span>
                                                </button>
                                                <div className="h-3.5 w-px bg-gray-200 dark:bg-slate-700" />
                                                <button
                                                    type="button"
                                                    onClick={() => handleDeleteNode(node.alias)}
                                                    disabled={deletingNodeAlias === node.alias}
                                                    className="px-2.5 py-1 text-[11px] font-medium text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-r-full flex items-center gap-1 cursor-pointer disabled:opacity-50"
                                                    title="Delete SSH node"
                                                >
                                                    {deletingNodeAlias === node.alias ? (
                                                        <Loader2 className="w-3 h-3 animate-spin" />
                                                    ) : (
                                                        <Trash2 className="w-3 h-3" />
                                                    )}
                                                    <span>Delete</span>
                                                </button>
                                            </div>
                                        </td>
                                    </tr>
                                );
                            })}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

            {/* 5. Multi-Machine Instance & Account Credit Matrix */}
            <div className="space-y-2.5">
                <div className="flex flex-wrap items-center justify-between gap-2">
                    <div>
                        <h4 className="text-xs sm:text-sm font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-1.5">
                            <Layers className="w-4 h-4 text-purple-500" />
                            Multi-Machine Instance &amp; Account Credit Matrix ({clusterBalances.length})
                        </h4>
                        <p className="text-[11px] text-gray-500 dark:text-slate-400">
                            Live overview of which instance on which machine is bound to which account, with 4-hour and weekly credit headroom.
                        </p>
                    </div>

                    <div className="inline-flex items-center rounded-full border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 p-0.5">
                        <button
                            type="button"
                            onClick={handleRefreshClusterBalances}
                            disabled={isLoadingBalances}
                            className="px-3 py-1 text-xs font-medium text-gray-700 dark:text-slate-200 hover:bg-white dark:hover:bg-slate-700 rounded-l-full flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                        >
                            {isLoadingBalances ? (
                                <Loader2 className="w-3 h-3 animate-spin" />
                            ) : (
                                <RefreshCw className="w-3 h-3 text-purple-500" />
                            )}
                            <span>Refresh Matrix</span>
                        </button>
                        <div className="h-3.5 w-px bg-gray-200 dark:bg-slate-700" />
                        <button
                            type="button"
                            onClick={handleExportClusterCsv}
                            disabled={isExportingCsv}
                            className="px-3 py-1 text-xs font-medium text-gray-700 dark:text-slate-200 hover:bg-white dark:hover:bg-slate-700 rounded-r-full flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                        >
                            {isExportingCsv ? (
                                <Loader2 className="w-3 h-3 animate-spin" />
                            ) : (
                                <Download className="w-3 h-3 text-emerald-500" />
                            )}
                            <span>Export CSV</span>
                        </button>
                    </div>
                </div>

                <div className="overflow-x-auto rounded-xl border border-gray-200 dark:border-slate-800">
                    <table className="w-full text-left border-collapse text-xs">
                        <thead>
                            <tr className="bg-gray-50 dark:bg-slate-800/80 text-gray-600 dark:text-slate-300 border-b border-gray-200 dark:border-slate-800">
                                <th className="py-2 px-3 font-semibold">Machine &amp; Node</th>
                                <th className="py-2 px-3 font-semibold">Instance &amp; Status</th>
                                <th className="py-2 px-3 font-semibold">Connected Google Account</th>
                                <th className="py-2 px-3 font-semibold">Subscription Tier</th>
                                <th className="py-2 px-3 font-semibold">4h Rolling Quota</th>
                                <th className="py-2 px-3 font-semibold">Weekly Quota</th>
                                <th className="py-2 px-3 font-semibold text-right">Action</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-gray-100 dark:divide-slate-800/70">
                            {clusterBalances.length === 0 ? (
                                <tr>
                                    <td
                                        colSpan={7}
                                        className="py-4 px-3 text-center text-gray-500 dark:text-slate-400"
                                    >
                                        {isLoadingBalances
                                            ? 'Polling local and remote SSH instances for account credit telemetry...'
                                            : 'No instances discovered yet. Click Refresh Matrix to scan.'}
                                    </td>
                                </tr>
                            ) : (
                                clusterBalances.map((item, idx) => {
                                    const conn = nodes.find(
                                        (n) => n.alias.toLowerCase() === item.machine_alias.toLowerCase()
                                    );
                                    const ip =
                                        item.ip_address ||
                                        (item as any).machine_ip ||
                                        conn?.ip_address ||
                                        '';
                                    const isOnline = Boolean(item.online || item.running);
                                    const nodeError =
                                        (item as any).error_message ||
                                        item.error ||
                                        'Connection timed out';

                                    const rawEmail = item.bound_email || '';
                                    const displayEmail = isAdminIdentifier(rawEmail) ? '' : rawEmail;
                                    const cleanMachineAlias = sanitizeAdminText(item.machine_alias) || 'local';

                                    const rawBadge = item.tier || (!item.is_local ? 'SSH-NODE' : '');
                                    const badgeText = rawBadge.toUpperCase();

                                    const q4h = item.quota_4h_pct;
                                    const qWeek = item.quota_weekly_pct;
                                    return (
                                        <tr
                                            key={`${item.machine_alias}-${item.instance_id}-${idx}`}
                                            className="hover:bg-gray-50/60 dark:hover:bg-slate-800/40 transition-colors"
                                        >
                                            <td className="py-2 px-3 font-mono">
                                                <div className="flex items-center gap-1.5 flex-wrap">
                                                    <span
                                                        className={`w-2 h-2 rounded-full ${
                                                            isOnline
                                                                ? 'bg-emerald-500 shadow-xs shadow-emerald-500/50'
                                                                : 'bg-rose-500 shadow-xs shadow-rose-500/50'
                                                        }`}
                                                        title={isOnline ? 'Node online & reachable' : nodeError}
                                                    />
                                                    <span className="font-semibold text-gray-900 dark:text-slate-100" title={cleanMachineAlias}>
                                                        {cleanMachineAlias}
                                                    </span>
                                                    {ip ? (
                                                        <div className="inline-flex items-center gap-1 font-mono text-[10px] text-slate-400 bg-slate-100 dark:bg-slate-800/40 px-1.5 py-0.2 rounded border border-slate-200 dark:border-slate-700/50" title={`Host IP: ${ip}`}>
                                                            <span>{ip}</span>
                                                            <button
                                                                type="button"
                                                                onClick={() => handleCopyChip(`mat-ip-${item.machine_alias}-${idx}`, ip, 'Node IP')}
                                                                className="p-0.5 rounded hover:bg-slate-200 dark:hover:bg-slate-700 text-slate-400 cursor-pointer"
                                                                title="Copy IP Address"
                                                            >
                                                                {copiedKey === `mat-ip-${item.machine_alias}-${idx}` ? (
                                                                    <Check className="w-2.5 h-2.5 text-emerald-500" />
                                                                ) : (
                                                                    <Copy className="w-2.5 h-2.5" />
                                                                )}
                                                            </button>
                                                        </div>
                                                    ) : null}
                                                </div>
                                            </td>
                                            <td className="py-2 px-3 font-mono">
                                                <div className="flex items-center gap-1.5 flex-wrap">
                                                    <span className="font-medium text-gray-800 dark:text-slate-200">
                                                        {item.instance_name}
                                                    </span>
                                                    {item.running ? (
                                                        <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
                                                            {typeof item.pid === 'number' ? `PID ${item.pid}` : 'Running'}
                                                        </span>
                                                    ) : isOnline ? (
                                                        <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-emerald-50/80 dark:bg-emerald-950/40 text-emerald-600 dark:text-emerald-400 border border-emerald-200/60 dark:border-emerald-800/60">
                                                            Reachable
                                                        </span>
                                                    ) : (
                                                        <span
                                                            title={nodeError}
                                                            className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-rose-50 dark:bg-rose-950/50 text-rose-700 dark:text-rose-300 border border-rose-200 dark:border-rose-800/80 cursor-help"
                                                        >
                                                            Offline
                                                        </span>
                                                    )}
                                                </div>
                                            </td>
                                            <td className="py-2 px-3 font-mono text-gray-900 dark:text-slate-100 font-medium">
                                                {displayEmail ? (
                                                    <div className="flex items-center gap-1.5 flex-wrap">
                                                        {(() => {
                                                            const maskKey = `inst-${item.machine_alias}-${item.instance_name || item.instance_id}`;
                                                            const isUnmasked = Boolean(unmaskedEmailNodes[maskKey] || unmaskedEmailNodes[item.machine_alias]);
                                                            return (
                                                                <>
                                                                    <span
                                                                        title={isUnmasked ? displayEmail : 'Masked email (click eye icon to toggle)'}
                                                                    >
                                                                        {isUnmasked ? displayEmail : maskEmailAddress(displayEmail)}
                                                                    </span>
                                                                    <button
                                                                        type="button"
                                                                        onClick={() => toggleEmailMask(maskKey)}
                                                                        className="p-0.5 rounded-full hover:bg-slate-200 dark:hover:bg-slate-700 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 transition-colors cursor-pointer"
                                                                        title={isUnmasked ? 'Mask email for this instance' : 'Reveal email for this instance'}
                                                                    >
                                                                        {isUnmasked ? (
                                                                            <EyeOff className="w-3 h-3 text-indigo-500" />
                                                                        ) : (
                                                                            <Eye className="w-3 h-3" />
                                                                        )}
                                                                    </button>
                                                                </>
                                                            );
                                                        })()}
                                                    </div>
                                                ) : (
                                                    <span className="text-gray-400 italic">Unbound</span>
                                                )}
                                            </td>
                                            <td className="py-2 px-3">
                                                {badgeText ? (
                                                    <span
                                                        className={`px-1.5 py-0.5 rounded text-[9px] font-bold uppercase border ${
                                                            badgeText === 'SSH-NODE'
                                                                ? 'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-300 border-indigo-200 dark:border-indigo-800'
                                                                : badgeText === 'VAULT'
                                                                  ? 'bg-purple-50 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 border-purple-200 dark:border-purple-800'
                                                                  : 'bg-sky-50 dark:bg-sky-950/60 text-sky-700 dark:text-sky-300 border border-sky-200 dark:border-sky-800'
                                                        }`}
                                                    >
                                                        {badgeText}
                                                    </span>
                                                ) : (
                                                    <span className="text-gray-400">—</span>
                                                )}
                                            </td>
                                            <td className="py-2 px-3 font-mono">
                                                {typeof q4h === 'number' ? (
                                                    <span
                                                        className={`inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold ${
                                                            q4h >= 50
                                                                ? 'bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300'
                                                                : q4h >= 20
                                                                  ? 'bg-amber-50 dark:bg-amber-950/50 text-amber-700 dark:text-amber-300'
                                                                  : 'bg-rose-50 dark:bg-rose-950/50 text-rose-700 dark:text-rose-300'
                                                        }`}
                                                    >
                                                        {q4h.toFixed(1)}%
                                                    </span>
                                                ) : (
                                                    <span className="text-gray-400">—</span>
                                                )}
                                            </td>
                                            <td className="py-2 px-3 font-mono">
                                                {typeof qWeek === 'number' ? (
                                                    <span
                                                        className={`inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-semibold ${
                                                            qWeek >= 50
                                                                ? 'bg-emerald-50 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300'
                                                                : qWeek >= 20
                                                                  ? 'bg-amber-50 dark:bg-amber-950/50 text-amber-700 dark:text-amber-300'
                                                                  : 'bg-rose-50 dark:bg-rose-950/50 text-rose-700 dark:text-rose-300'
                                                        }`}
                                                    >
                                                        {qWeek.toFixed(1)}%
                                                    </span>
                                                ) : (
                                                    <span className="text-gray-400">—</span>
                                                )}
                                            </td>
                                            <td className="py-2 px-3 text-right">
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        const cleanId = item.instance_id || 'default';
                                                        setTargetMachine(item.machine_alias);
                                                        setTargetInstance(cleanId);
                                                        if (promptText.trim()) {
                                                            handleDispatchRemotePrompt(
                                                                item.machine_alias,
                                                                cleanId,
                                                                promptText
                                                            );
                                                        } else {
                                                            showToast(
                                                                `Target locked to ${item.machine_alias}:${cleanId} (${displayEmail || 'unbound'}) — enter prompt below to send`,
                                                                'info'
                                                            );
                                                            const promptInput = document.getElementById('remote-prompt-input');
                                                            if (promptInput) {
                                                                promptInput.scrollIntoView({ behavior: 'smooth', block: 'center' });
                                                                promptInput.focus();
                                                            }
                                                        }
                                                    }}
                                                    className="px-2.5 py-1 rounded-full text-[11px] font-medium bg-indigo-50 dark:bg-indigo-950/50 hover:bg-indigo-100 dark:hover:bg-indigo-900/60 text-indigo-700 dark:text-indigo-300 border border-indigo-200 dark:border-indigo-800 inline-flex items-center gap-1 cursor-pointer"
                                                >
                                                    <Send className="w-3 h-3" />
                                                    <span>Prompt This Instance</span>
                                                </button>
                                            </td>
                                        </tr>
                                    );
                                })
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* 6. Remote Machine & Instance Prompt Dispatcher Bar */}
            <div className="p-3.5 rounded-xl bg-indigo-50/40 dark:bg-indigo-950/20 border border-indigo-100 dark:border-indigo-900/40 space-y-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                        <Cpu className="w-4 h-4 text-indigo-500" />
                        <span className="text-xs font-semibold text-gray-900 dark:text-slate-100">
                            Direct Multi-Machine &amp; Instance Prompt Delegation (Machine → Instance → Prompt)
                        </span>
                    </div>
                    <span className="text-[11px] font-mono text-indigo-600 dark:text-indigo-400">
                        /prompt {targetMachine}:{targetInstance} {promptText || '<prompt>'}
                    </span>
                </div>

                <div className="flex flex-col md:flex-row items-stretch md:items-center gap-2">
                    {/* Segmented Capsule for Machine + Instance Selection */}
                    <div className="inline-flex items-center rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 px-3 py-1 shadow-xs">
                        <span className="text-[11px] font-semibold text-gray-400 mr-1.5">Machine:</span>
                        <select
                            value={targetMachine}
                            onChange={(e) => {
                                const newMachine = e.target.value;
                                setTargetMachine(newMachine);
                                const matching = clusterBalances.filter(
                                    (item) => item.machine_alias.toLowerCase() === newMachine.toLowerCase()
                                );
                                if (matching.length > 0) {
                                    setTargetInstance(matching[0].instance_id || 'default');
                                } else {
                                    setTargetInstance('default');
                                }
                            }}
                            className="text-xs font-mono text-gray-900 dark:text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none pr-2 cursor-pointer"
                        >
                            <option value="local">local (This Machine)</option>
                            {nodes.map((n) => (
                                <option key={n.alias} value={n.alias}>
                                    {n.alias}{n.ip_address ? ` (${n.ip_address})` : ''}
                                </option>
                            ))}
                        </select>
                        <div className="h-4 w-px bg-gray-200 dark:bg-slate-700 mx-2" />
                        <span className="text-[11px] font-semibold text-gray-400 mr-1.5">Instance:</span>
                        <select
                            value={targetInstance}
                            onChange={(e) => setTargetInstance(e.target.value)}
                            className="text-xs font-mono text-gray-900 dark:text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none pr-2 cursor-pointer max-w-[280px]"
                        >
                            {clusterBalances
                                .filter((item) => item.machine_alias.toLowerCase() === targetMachine.toLowerCase())
                                .map((item) => {
                                    const cleanId = item.instance_id || 'default';
                                    const email = item.bound_email && !isAdminIdentifier(item.bound_email) ? item.bound_email : 'unbound';
                                    const statusText = item.running ? 'Running' : 'Idle';
                                    return (
                                        <option key={`${item.machine_alias}-${cleanId}`} value={cleanId}>
                                            {cleanId} — {email} ({statusText})
                                        </option>
                                    );
                                })}
                            {clusterBalances.filter((item) => item.machine_alias.toLowerCase() === targetMachine.toLowerCase()).length === 0 && (
                                <option value="default">default — unbound (Default Instance)</option>
                            )}
                        </select>
                    </div>

                    <button
                        type="button"
                        onClick={() => {
                            setTargetMachine('local');
                            setTargetInstance('default');
                            showToast('Reset target to local:default', 'info');
                        }}
                        disabled={targetMachine === 'local' && targetInstance === 'default'}
                        className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium text-slate-600 dark:text-slate-300 hover:text-indigo-600 dark:hover:text-indigo-400 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-700 rounded-full shadow-2xs hover:border-indigo-300 dark:hover:border-indigo-700 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shrink-0"
                        title="Reset target machine to local and instance to default"
                    >
                        <RotateCcw className="w-3.5 h-3.5" />
                        <span>Reset to Local</span>
                    </button>

                    <input
                        id="remote-prompt-input"
                        type="text"
                        value={promptText}
                        onChange={(e) => setPromptText(e.target.value)}
                        placeholder="Write prompt instruction to inject into target machine & instance..."
                        className="flex-1 h-9 px-3 text-xs rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    />

                    <div className="inline-flex items-center rounded-full border border-indigo-200 dark:border-indigo-800 bg-white dark:bg-slate-900 p-0.5 self-start md:self-auto">
                        <button
                            type="button"
                            onClick={() =>
                                handleCopyChip(
                                    'copy-remote-prompt-cmd',
                                    `/prompt ${targetMachine}:${targetInstance} ${promptText || 'Continue current task'}`
                                )
                            }
                            className="px-3 py-1.5 text-xs font-medium text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 rounded-l-full flex items-center gap-1.5 cursor-pointer"
                        >
                            {copiedKey === 'copy-remote-prompt-cmd' ? (
                                <Check className="w-3.5 h-3.5 text-emerald-500" />
                            ) : (
                                <Copy className="w-3.5 h-3.5 text-gray-400" />
                            )}
                            <span>{copiedKey === 'copy-remote-prompt-cmd' ? 'Copied!' : 'Copy /prompt'}</span>
                        </button>
                        <div className="h-4 w-px bg-gray-200 dark:bg-slate-700" />
                        <button
                            type="button"
                            onClick={() => handleDispatchRemotePrompt()}
                            disabled={isSendingPrompt}
                            className="px-3.5 py-1.5 text-xs font-semibold bg-indigo-600 hover:bg-indigo-700 text-white rounded-r-full flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                        >
                            {isSendingPrompt ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Send className="w-3.5 h-3.5" />
                            )}
                            <span>Send Prompt Now</span>
                        </button>
                    </div>
                </div>

                {lastPromptResult && (
                    <div className="p-2.5 rounded-lg bg-slate-950 text-slate-200 font-mono text-[11px] space-y-1 border border-slate-800">
                        <div className="flex items-center justify-between text-[10px] text-slate-400">
                            <span>
                                Target: {lastPromptResult.node_alias}:{lastPromptResult.instance_target} •{' '}
                                {lastPromptResult.duration_ms}ms •{' '}
                                {lastPromptResult.success ? 'SUCCESS' : 'ERROR'}
                            </span>
                            <button
                                type="button"
                                onClick={() => setLastPromptResult(null)}
                                className="hover:text-white cursor-pointer"
                            >
                                Dismiss
                            </button>
                        </div>
                        {lastPromptResult.stdout && (
                            <pre className="text-emerald-400 whitespace-pre-wrap overflow-x-auto">
                                {lastPromptResult.stdout}
                            </pre>
                        )}
                        {lastPromptResult.stderr && (
                            <pre className="text-rose-400 whitespace-pre-wrap overflow-x-auto">
                                {lastPromptResult.stderr}
                            </pre>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
