import { useEffect, useState } from 'react';
import { SupabaseSyncApi } from './useSupabaseSync';
import { useErrorStore } from '../../../stores/error-store';
import { telegramService } from '../../../services/telegramService';
import {
    sshService,
    type ClusterInstanceBalanceItem,
    type SshNodeRecord,
    type SystemExportFormat,
    type SystemExportTarget,
} from '../../../services/sshService';
import { copyToClipboard } from '../../../utils/clipboard';
import { showToast } from '../../common/ToastContainer';
import {
    Check,
    CheckCircle2,
    Copy,
    Cpu,
    Download,
    FileSpreadsheet,
    HelpCircle,
    Layers,
    Loader2,
    RefreshCw,
    Send,
    Terminal,
} from 'lucide-react';

interface PowerCommandCategory {
    id: string;
    title: string;
    badgeClass: string;
    commands: Array<{
        label: string;
        payload: string;
        hint: string;
    }>;
}

const SUPABASE_TELEGRAM_COMMAND_CATEGORIES: PowerCommandCategory[] = [
    {
        id: 'prompting',
        title: 'Easy Machine & Instance Prompting',
        badgeClass:
            'border-sky-500/30 bg-sky-500/10 text-sky-300 hover:bg-sky-500/20',
        commands: [
            {
                label: '/prompt default Write unit tests',
                payload: '/prompt default Write unit tests',
                hint: 'Inject prompt directly into local default instance',
            },
            {
                label: '/prompt W3:default Fix build',
                payload: '/prompt W3:default Fix build',
                hint: 'Target remote machine W3 and instance default via colon syntax',
            },
            {
                label: '/prompt --node vm-1 --instance work Audit code',
                payload: '/prompt --node vm-1 --instance work Audit code',
                hint: 'Explicitly target remote SSH node and instance flags',
            },
            {
                label: '/prompt <machine>:<instance> <text>',
                payload: '/prompt vm-1:default Continue current task and run tests',
                hint: 'Template for machine:instance prompt delegation',
            },
            {
                label: '/target W3 default',
                payload: '/target W3 default',
                hint: 'Lock default machine and instance for subsequent chat messages',
            },
            {
                label: '/send',
                payload: '/send',
                hint: 'Open interactive machine -> instance -> conversation picker in Telegram',
            },
            {
                label: 'agm prompt --node <alias> --instance <inst> "<text>"',
                payload: 'agm prompt --node vm-1 --instance default "Verify build"',
                hint: 'Terminal CLI remote SSH & instance prompt injection',
            },
        ],
    },
    {
        id: 'balance',
        title: 'Account Balance & Quota Checking',
        badgeClass:
            'border-emerald-500/30 bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/20',
        commands: [
            {
                label: '/balance',
                payload: '/balance',
                hint: 'Multi-machine & local per-instance 4h and weekly quota report',
            },
            {
                label: '/credits',
                payload: '/credits',
                hint: 'Alias for /balance showing active account credit headroom',
            },
            {
                label: '/quota',
                payload: '/quota',
                hint: 'Alias for /balance with standby pool rotation candidates',
            },
            {
                label: '/balance --csv',
                payload: '/balance --csv',
                hint: 'Receive live account & instance quota table as a CSV document',
            },
            {
                label: 'agm balance --csv',
                payload: 'agm balance --csv',
                hint: 'CLI account balance & quota output in RFC 4180 CSV format',
            },
            {
                label: 'agm ssh balance',
                payload: 'agm ssh balance',
                hint: 'Poll all SSH fleet nodes for active instances and 4h/weekly credits',
            },
        ],
    },
    {
        id: 'csv-export',
        title: 'CSV & File Format Output',
        badgeClass:
            'border-purple-500/30 bg-purple-500/10 text-purple-300 hover:bg-purple-500/20',
        commands: [
            {
                label: '/export accounts csv',
                payload: '/export accounts csv',
                hint: 'Download all accounts and dual-window balances as CSV attachment',
            },
            {
                label: '/export instances csv',
                payload: '/export instances csv',
                hint: 'Download all instances, PIDs, and bound accounts as CSV',
            },
            {
                label: '/export balance csv',
                payload: '/export balance csv',
                hint: 'Download unified cluster balance report as CSV',
            },
            {
                label: '/export nodes csv',
                payload: '/export nodes csv',
                hint: 'Download configured SSH nodes inventory as CSV',
            },
            {
                label: '/export prompts json',
                payload: '/export prompts json',
                hint: 'Export active & backed-up prompt queue in JSON format',
            },
            {
                label: '/export csv',
                payload: '/export csv',
                hint: 'Full system telemetry CSV bundle',
            },
            {
                label: '/accounts --csv',
                payload: '/accounts --csv',
                hint: 'Flag shorthand to export accounts as CSV file',
            },
            {
                label: '/instances --csv',
                payload: '/instances --csv',
                hint: 'Flag shorthand to export instances as CSV file',
            },
            {
                label: 'agm accounts --csv',
                payload: 'agm accounts --csv',
                hint: 'CLI export of accounts table in CSV format',
            },
            {
                label: 'agm instances --csv',
                payload: 'agm instances --csv',
                hint: 'CLI export of instances table in CSV format',
            },
        ],
    },
    {
        id: 'gitmap-ssh',
        title: 'GitMap Multi-Machine & SSH Control',
        badgeClass:
            'border-amber-500/30 bg-amber-500/10 text-amber-300 hover:bg-amber-500/20',
        commands: [
            {
                label: '/help',
                payload: '/help',
                hint: 'Full categorized Telegram command manual with copyable examples',
            },
            {
                label: '/nodes',
                payload: '/nodes',
                hint: 'Show all registered SSH & GitMap cluster nodes',
            },
            {
                label: '/ssh nodes',
                payload: '/ssh nodes',
                hint: 'List configured SSH fleet nodes and RSA vault status',
            },
            {
                label: '/ssh import-gitmap',
                payload: '/ssh import-gitmap',
                hint: 'Import SSH node inventory from GitMap into AGM',
            },
            {
                label: '/ssh balance',
                payload: '/ssh balance',
                hint: 'Query multi-machine account balances across all SSH nodes',
            },
            {
                label: '/ssh exec vm-1 agm status',
                payload: '/ssh exec vm-1 agm status',
                hint: 'Execute AGM CLI command on remote SSH node vm-1',
            },
            {
                label: '/gitmap cluster status',
                payload: '/gitmap cluster status',
                hint: 'Query GitMap multi-machine cluster topology and sync status',
            },
        ],
    },
];

export function TelegramSection(props: SupabaseSyncApi) {
    const {
        telegramConfig,
        setTelegramConfig,
        telegramStatus,
        isTestingTelegram,
        telegramBotUsername,
        isSavingTelegram,
        isSendingPing,
        setIsTgGuideOpen,
        handleSaveTelegram,
        handleTestTelegram,
        handleDetectTelegramChatId,
        handleSendTelegramPing,
    } = props;

    const [copiedChip, setCopiedChip] = useState<string | null>(null);
    const [sshNodes, setSshNodes] = useState<SshNodeRecord[]>([]);
    const [quickMachine, setQuickMachine] = useState<string>('local');
    const [quickInstance, setQuickInstance] = useState<string>('default');
    const [quickPrompt, setQuickPrompt] = useState<string>('');
    const [isDispatchingPrompt, setIsDispatchingPrompt] = useState<boolean>(false);
    const [exportingTarget, setExportingTarget] = useState<string | null>(null);
    const [clusterBalances, setClusterBalances] = useState<ClusterInstanceBalanceItem[]>([]);
    const [isCheckingBalances, setIsCheckingBalances] = useState<boolean>(false);
    const [showBalancePanel, setShowBalancePanel] = useState<boolean>(false);

    useEffect(() => {
        sshService
            .listSshNodes()
            .then((list) => setSshNodes(list))
            .catch((err) => {
                useErrorStore.getState().trackWarning(err, {
                    source: 'SupabaseSync.TelegramSection.loadSshNodes',
                    triggerAction: 'list_ssh_nodes',
                });
            });
    }, []);

    const handleCopyCommand = async (chipKey: string, payload: string) => {
        const ok = await copyToClipboard(payload);
        if (ok) {
            setCopiedChip(chipKey);
            showToast(`Copied: ${payload}`, 'info');
            setTimeout(() => {
                setCopiedChip((prev) => (prev === chipKey ? null : prev));
            }, 1600);
        }
    };

    const handleQuickSendPrompt = async () => {
        const trimmedPrompt = quickPrompt.trim();
        if (!trimmedPrompt) {
            showToast('Please enter a prompt to send to the selected machine & instance', 'warning');
            return;
        }
        setIsDispatchingPrompt(true);
        try {
            const res = await sshService.sendRemoteSshPrompt({
                node_alias: quickMachine.trim() || 'local',
                instance_target: quickInstance.trim() || 'default',
                prompt_text: trimmedPrompt,
            });
            if (res.success) {
                showToast(`Prompt sent to ${res.node_alias}:${res.instance_target}`, 'success');
            } else {
                showToast(
                    `Prompt dispatched to ${res.node_alias}:${res.instance_target} with warnings`,
                    'warning'
                );
            }
        } catch (err: any) {
            showToast(`Failed to send prompt: ${err?.message || err}`, 'error');
        } finally {
            setIsDispatchingPrompt(false);
        }
    };

    const handleQuickExport = async (
        target: SystemExportTarget,
        format: SystemExportFormat,
        sendToTelegram: boolean = false
    ) => {
        const key = `${target}-${format}-${sendToTelegram ? 'tg' : 'dl'}`;
        setExportingTarget(key);
        try {
            const data = await sshService.exportSystemDataCsv({
                target,
                format,
                sendToTelegram,
            });
            const ext = format === 'json' ? 'json' : 'csv';
            await sshService.triggerDownloadAndCopy(data, `agm-${target}.${ext}`, format);
            showToast(
                sendToTelegram
                    ? `Exported ${target.toUpperCase()} (${ext.toUpperCase()}) to Telegram & clipboard`
                    : `Exported ${target.toUpperCase()} (${ext.toUpperCase()}) — downloaded & copied to clipboard`,
                'success'
            );
        } catch (err: any) {
            showToast(`Export failed: ${err?.message || err}`, 'error');
        } finally {
            setExportingTarget(null);
        }
    };

    const handleQuickCheckBalances = async (sendToTelegram: boolean = false) => {
        setIsCheckingBalances(true);
        setShowBalancePanel(true);
        try {
            const items = await sshService.getClusterInstancesAndBalances();
            setClusterBalances(items);
            if (sendToTelegram) {
                await sshService.exportSystemDataCsv({
                    target: 'balance',
                    format: 'csv',
                    sendToTelegram: true,
                });
                showToast('Live account balances fetched and pushed to Telegram!', 'success');
            } else {
                showToast(`Loaded live balances across ${items.length} instance(s)`, 'info');
            }
        } catch (err: any) {
            showToast(`Balance check failed: ${err?.message || err}`, 'error');
        } finally {
            setIsCheckingBalances(false);
        }
    };

    const formattedSlashPrompt =
        quickMachine === 'local'
            ? `/prompt ${quickInstance || 'default'} ${quickPrompt || 'Write unit tests'}`
            : `/prompt ${quickMachine}:${quickInstance || 'default'} ${quickPrompt || 'Fix build'}`;

    return (
        <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-4">
            <div className="flex items-center justify-between">
                <div>
                    <div className="flex items-center gap-2">
                        <h4 className="font-semibold text-white flex items-center gap-2">
                            <Send className="w-4 h-4 text-sky-400" />
                            Telegram Inbound Bot Integration
                        </h4>
                        <button
                            type="button"
                            onClick={() => setIsTgGuideOpen(true)}
                            className="px-2 py-0.5 rounded-md text-[11px] font-medium bg-sky-500/10 text-sky-400 hover:bg-sky-500/20 border border-sky-500/20 transition flex items-center gap-1 cursor-pointer"
                            title="Step-by-step instructions to create Telegram Bot"
                        >
                            <HelpCircle className="w-3 h-3" />
                            <span>Create Bot Guide</span>
                        </button>
                    </div>
                    <p className="text-xs text-gray-400 mt-0.5">
                        Query cluster snapshots, check per-instance account balances, export CSV/JSON reports, and prompt machines directly from Telegram.
                    </p>
                </div>
                {telegramConfig && (
                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            checked={telegramConfig.is_enabled}
                            onChange={(e) => {
                                const updated = { ...telegramConfig, is_enabled: e.target.checked };
                                setTelegramConfig(updated);
                                telegramService.saveConfig(updated).catch((err) => {
                                    // Tracked in the error module; toggle already applied locally, persisted on next save.
                                    useErrorStore.getState().trackWarning(err, {
                                        source: 'SupabaseSyncSettings.telegramToggle',
                                        triggerAction: 'save_telegram_config',
                                    });
                                });
                            }}
                            className="sr-only peer"
                        />
                        <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-sky-600"></div>
                    </label>
                )}
            </div>

            {telegramConfig && (
                <div className="space-y-3.5 pt-2 border-t border-slate-800/80">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <div className="flex flex-col">
                            <label className="block text-xs font-medium text-gray-300 mb-1.5 h-4 leading-4 truncate">
                                Telegram Bot Token
                            </label>
                            <input
                                type="password"
                                placeholder="123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ"
                                value={telegramConfig.bot_token}
                                onChange={(e) =>
                                    setTelegramConfig({ ...telegramConfig, bot_token: e.target.value })
                                }
                                className="w-full h-9 px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-sky-500 text-xs font-mono"
                            />
                        </div>

                        <div className="flex flex-col">
                            <div className="flex items-center justify-between mb-1.5 h-4">
                                <label className="block text-xs font-medium text-gray-300 leading-4 truncate">
                                    Allowed Chat ID (Auto-Detected from /ping or /start)
                                </label>
                                <button
                                    type="button"
                                    onClick={handleDetectTelegramChatId}
                                    disabled={isTestingTelegram || !telegramConfig.bot_token.trim()}
                                    className="text-[11px] font-medium text-sky-400 hover:text-sky-300 disabled:opacity-40 cursor-pointer"
                                >
                                    Auto-Detect Chat ID
                                </button>
                            </div>
                            <input
                                type="number"
                                placeholder="Send /ping to your bot, then click Auto-Detect"
                                value={telegramConfig.allowed_chat_id ?? ''}
                                onChange={(e) =>
                                    setTelegramConfig({
                                        ...telegramConfig,
                                        allowed_chat_id: e.target.value ? Number(e.target.value) : null,
                                    })
                                }
                                className="w-full h-9 px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-sky-500 text-xs font-mono"
                            />
                        </div>
                    </div>

                    <div className="flex items-center justify-between p-2.5 rounded-lg bg-slate-900/60 border border-slate-800">
                        <label className="flex items-center gap-2 cursor-pointer">
                            <input
                                type="checkbox"
                                checked={telegramConfig.notify_on_system_update ?? true}
                                onChange={(e) => {
                                    const updated = {
                                        ...telegramConfig,
                                        notify_on_system_update: e.target.checked,
                                    };
                                    setTelegramConfig(updated);
                                    telegramService.saveConfig(updated).catch((err) => {
                                        useErrorStore.getState().trackWarning(err, {
                                            source: 'SupabaseSyncSettings.telegramToggle',
                                            triggerAction: 'save_telegram_config',
                                        });
                                    });
                                }}
                                className="w-3.5 h-3.5 rounded text-sky-600 focus:ring-sky-500 border-slate-700 bg-slate-950"
                            />
                            <span className="text-xs text-gray-300">
                                Notify via Telegram on System Update (default enabled)
                            </span>
                        </label>
                    </div>

                    {/* Quick Interactive Tools: Prompt Dispatcher + CSV/JSON Export + Balance Check */}
                    <div className="p-3 rounded-xl bg-slate-950/70 border border-slate-800 space-y-3">
                        <div className="flex flex-wrap items-center justify-between gap-2">
                            <div className="flex items-center gap-2">
                                <Cpu className="w-3.5 h-3.5 text-sky-400" />
                                <span className="text-xs font-semibold text-slate-200">
                                    Quick Prompt Dispatcher (Machine → Instance → Prompt)
                                </span>
                            </div>
                            <span className="text-[11px] font-mono text-sky-400">
                                {formattedSlashPrompt}
                            </span>
                        </div>

                        <div className="flex flex-col lg:flex-row items-stretch lg:items-center gap-2">
                            <div className="inline-flex items-center rounded-full border border-slate-700 bg-slate-900 px-3 py-1">
                                <span className="text-[11px] font-semibold text-slate-400 mr-1.5">Machine:</span>
                                <select
                                    value={quickMachine}
                                    onChange={(e) => setQuickMachine(e.target.value)}
                                    className="text-xs font-mono text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none pr-2 cursor-pointer"
                                >
                                    <option value="local">local</option>
                                    <option value="W3">W3</option>
                                    <option value="vm-1">vm-1</option>
                                    {sshNodes
                                        .filter((n) => n.alias !== 'W3' && n.alias !== 'vm-1')
                                        .map((n) => (
                                            <option key={n.alias} value={n.alias}>
                                                {n.alias} ({n.ip_address})
                                            </option>
                                        ))}
                                </select>
                                <div className="h-4 w-px bg-slate-700 mx-2" />
                                <span className="text-[11px] font-semibold text-slate-400 mr-1.5">Instance:</span>
                                <input
                                    type="text"
                                    value={quickInstance}
                                    onChange={(e) => setQuickInstance(e.target.value)}
                                    placeholder="default"
                                    className="w-24 text-xs font-mono text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none"
                                />
                            </div>

                            <input
                                type="text"
                                value={quickPrompt}
                                onChange={(e) => setQuickPrompt(e.target.value)}
                                placeholder="Write unit tests, Fix build, Audit code..."
                                className="flex-1 h-8 px-3 text-xs rounded-full border border-slate-700 bg-slate-900 text-slate-100 font-mono focus:outline-none focus:border-sky-500"
                            />

                            <div className="inline-flex items-center rounded-full border border-sky-500/30 bg-slate-900 p-0.5 self-start lg:self-auto">
                                <button
                                    type="button"
                                    onClick={() => handleCopyCommand('sb-quick-prompt', formattedSlashPrompt)}
                                    className="px-3 py-1 text-xs font-medium text-slate-200 hover:bg-slate-800 rounded-l-full flex items-center gap-1.5 cursor-pointer"
                                >
                                    {copiedChip === 'sb-quick-prompt' ? (
                                        <Check className="w-3.5 h-3.5 text-emerald-400" />
                                    ) : (
                                        <Copy className="w-3.5 h-3.5 text-slate-400" />
                                    )}
                                    <span>
                                        {copiedChip === 'sb-quick-prompt' ? 'Copied!' : 'Copy /prompt Command'}
                                    </span>
                                </button>
                                <div className="h-4 w-px bg-slate-700" />
                                <button
                                    type="button"
                                    onClick={handleQuickSendPrompt}
                                    disabled={isDispatchingPrompt}
                                    className="px-3.5 py-1 text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white rounded-r-full flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                                >
                                    {isDispatchingPrompt ? (
                                        <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                    ) : (
                                        <Send className="w-3.5 h-3.5" />
                                    )}
                                    <span>Send Prompt Now</span>
                                </button>
                            </div>
                        </div>

                        <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-slate-800">
                            <div className="flex flex-wrap items-center gap-1.5">
                                <span className="text-[11px] font-semibold text-slate-400 flex items-center gap-1 mr-1">
                                    <FileSpreadsheet className="w-3.5 h-3.5 text-purple-400" />
                                    CSV / JSON Export:
                                </span>
                                <div className="inline-flex flex-wrap items-center rounded-full border border-slate-700 bg-slate-900 p-0.5">
                                    {(
                                        [
                                            { label: 'Accounts CSV', target: 'accounts', format: 'csv' },
                                            { label: 'Instances CSV', target: 'instances', format: 'csv' },
                                            { label: 'Balances CSV', target: 'balance', format: 'csv' },
                                            { label: 'SSH Nodes CSV', target: 'nodes', format: 'csv' },
                                        ] as const
                                    ).map((btn, i, arr) => {
                                        const key = `${btn.target}-${btn.format}-dl`;
                                        return (
                                            <div key={key} className="inline-flex items-center">
                                                <button
                                                    type="button"
                                                    onClick={() => handleQuickExport(btn.target, btn.format, false)}
                                                    disabled={exportingTarget === key}
                                                    className={`px-2.5 py-1 text-[11px] font-medium text-slate-200 hover:bg-slate-800 transition-colors flex items-center gap-1 cursor-pointer disabled:opacity-50 ${
                                                        i === 0
                                                            ? 'rounded-l-full'
                                                            : i === arr.length - 1
                                                              ? 'rounded-r-full'
                                                              : ''
                                                    }`}
                                                >
                                                    {exportingTarget === key ? (
                                                        <Loader2 className="w-3 h-3 animate-spin text-purple-400" />
                                                    ) : (
                                                        <Download className="w-3 h-3 text-purple-400" />
                                                    )}
                                                    <span>{btn.label}</span>
                                                </button>
                                                {i < arr.length - 1 && <div className="h-3.5 w-px bg-slate-700" />}
                                            </div>
                                        );
                                    })}
                                </div>
                            </div>

                            <button
                                type="button"
                                onClick={() => handleQuickCheckBalances(false)}
                                disabled={isCheckingBalances}
                                className="px-3 py-1 rounded-full border border-emerald-500/30 bg-emerald-500/10 hover:bg-emerald-500/20 text-[11px] font-semibold text-emerald-300 flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                            >
                                {isCheckingBalances ? (
                                    <Loader2 className="w-3 h-3 animate-spin" />
                                ) : (
                                    <Layers className="w-3 h-3 text-emerald-400" />
                                )}
                                <span>Account Balance Quick Check</span>
                            </button>
                        </div>

                        {showBalancePanel && (
                            <div className="pt-1 space-y-1">
                                <div className="flex items-center justify-between text-[11px] text-slate-300">
                                    <span className="font-semibold">
                                        Live Instance &amp; Account Balances ({clusterBalances.length})
                                    </span>
                                    <button
                                        type="button"
                                        onClick={() => setShowBalancePanel(false)}
                                        className="text-slate-400 hover:text-white cursor-pointer"
                                    >
                                        Hide
                                    </button>
                                </div>
                                <div className="overflow-x-auto rounded-lg border border-slate-800 bg-slate-900">
                                    <table className="w-full text-left border-collapse text-[11px] text-slate-200">
                                        <thead>
                                            <tr className="bg-slate-800/80 text-slate-300 border-b border-slate-700">
                                                <th className="py-1.5 px-2.5">Machine</th>
                                                <th className="py-1.5 px-2.5">Instance</th>
                                                <th className="py-1.5 px-2.5">Account</th>
                                                <th className="py-1.5 px-2.5">4h Credit</th>
                                                <th className="py-1.5 px-2.5">Weekly Credit</th>
                                            </tr>
                                        </thead>
                                        <tbody className="divide-y divide-slate-800">
                                            {clusterBalances.map((item, idx) => (
                                                <tr key={`${item.machine_alias}-${item.instance_id}-${idx}`}>
                                                    <td className="py-1.5 px-2.5 font-mono font-semibold">
                                                        {item.machine_alias}
                                                    </td>
                                                    <td className="py-1.5 px-2.5 font-mono">{item.instance_name}</td>
                                                    <td className="py-1.5 px-2.5 font-mono">
                                                        {item.bound_email || 'Unbound'}
                                                    </td>
                                                    <td className="py-1.5 px-2.5 font-mono">
                                                        {typeof item.quota_4h_pct === 'number'
                                                            ? `${item.quota_4h_pct.toFixed(1)}%`
                                                            : '—'}
                                                    </td>
                                                    <td className="py-1.5 px-2.5 font-mono">
                                                        {typeof item.quota_weekly_pct === 'number'
                                                            ? `${item.quota_weekly_pct.toFixed(1)}%`
                                                            : '—'}
                                                    </td>
                                                </tr>
                                            ))}
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        )}
                    </div>

                    {/* Categorized Click-to-Copy Power Commands & Examples */}
                    <div className="p-3 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-2.5">
                        <div className="flex items-center justify-between">
                            <span className="text-xs font-semibold text-gray-200 flex items-center gap-1.5">
                                <Terminal className="w-3.5 h-3.5 text-sky-400" />
                                Click-to-Copy Power Commands &amp; Examples
                            </span>
                            <span className="text-[11px] text-slate-400">Click any pill to copy</span>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5">
                            {SUPABASE_TELEGRAM_COMMAND_CATEGORIES.map((cat) => (
                                <div
                                    key={cat.id}
                                    className="p-2.5 rounded-lg bg-slate-900/80 border border-slate-800 space-y-1.5"
                                >
                                    <div className="text-[11px] font-semibold text-slate-200">{cat.title}</div>
                                    <div className="flex flex-wrap gap-1.5">
                                        {cat.commands.map((cmd) => {
                                            const chipKey = `sb:${cat.id}:${cmd.label}`;
                                            const isCopied = copiedChip === chipKey;
                                            return (
                                                <button
                                                    key={chipKey}
                                                    type="button"
                                                    onClick={() => handleCopyCommand(chipKey, cmd.payload)}
                                                    title={`${cmd.hint} — Click to copy: ${cmd.payload}`}
                                                    className={`inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-mono border transition-all cursor-pointer ${cat.badgeClass}`}
                                                >
                                                    {isCopied ? (
                                                        <Check className="w-3 h-3 text-emerald-400 shrink-0" />
                                                    ) : (
                                                        <Copy className="w-2.5 h-2.5 opacity-70 shrink-0" />
                                                    )}
                                                    <span>{isCopied ? 'Copied!' : cmd.label}</span>
                                                </button>
                                            );
                                        })}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>

                    <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
                        <div className="flex items-center gap-2">
                            {telegramBotUsername && (
                                <span className="text-xs text-sky-400 font-mono bg-sky-500/10 px-2 py-0.5 rounded border border-sky-500/20 flex items-center gap-1">
                                    <CheckCircle2 className="w-3.5 h-3.5 text-sky-400" />
                                    @{telegramBotUsername}
                                </span>
                            )}
                            {telegramStatus?.is_running && (
                                <span className="text-[10px] text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20 font-mono">
                                    Polling Active
                                </span>
                            )}
                        </div>

                        <div className="flex items-center gap-2">
                            <button
                                onClick={handleTestTelegram}
                                disabled={isTestingTelegram}
                                className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs flex items-center gap-1.5 transition cursor-pointer"
                            >
                                {isTestingTelegram ? (
                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                ) : (
                                    <RefreshCw className="w-3.5 h-3.5 text-sky-400" />
                                )}
                                Test Bot &amp; Detect ID
                            </button>
                            <button
                                onClick={handleSendTelegramPing}
                                disabled={isSendingPing || !telegramConfig.allowed_chat_id}
                                className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs flex items-center gap-1.5 transition disabled:opacity-50 cursor-pointer"
                            >
                                {isSendingPing ? (
                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                ) : (
                                    <Send className="w-3.5 h-3.5 text-sky-400" />
                                )}
                                Send Ping
                            </button>
                            <button
                                onClick={handleSaveTelegram}
                                disabled={isSavingTelegram}
                                className="px-3 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white text-xs font-medium transition cursor-pointer"
                            >
                                Save Telegram
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
