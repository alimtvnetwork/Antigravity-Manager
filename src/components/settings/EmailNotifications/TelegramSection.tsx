import { useEffect, useState } from 'react';
import { copyToClipboard } from '../../../utils/clipboard';
import { EmailNotificationsApi } from './useEmailNotifications';
import { showToast } from '../../common/ToastContainer';
import {
    sshService,
    type ClusterInstanceBalanceItem,
    type SshNodeRecord,
    type SystemExportFormat,
    type SystemExportTarget,
} from '../../../services/sshService';
import { useErrorStore } from '../../../stores/error-store';
import {
    Check,
    CheckCircle2,
    Copy,
    Cpu,
    Download,
    Eye,
    EyeOff,
    FileSpreadsheet,
    Layers,
    Loader2,
    MessageSquare,
    RefreshCw,
    Send,
    Sparkles,
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

const TELEGRAM_POWER_COMMAND_CATEGORIES: PowerCommandCategory[] = [
    {
        id: 'prompting',
        title: 'Easy Machine & Instance Prompting',
        badgeClass:
            'border-sky-200 dark:border-sky-800/70 bg-sky-50/70 dark:bg-sky-950/30 text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/50',
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
            'border-emerald-200 dark:border-emerald-800/70 bg-emerald-50/70 dark:bg-emerald-950/30 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100 dark:hover:bg-emerald-900/50',
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
            'border-purple-200 dark:border-purple-800/70 bg-purple-50/70 dark:bg-purple-950/30 text-purple-700 dark:text-purple-300 hover:bg-purple-100 dark:hover:bg-purple-900/50',
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
            'border-amber-200 dark:border-amber-800/70 bg-amber-50/70 dark:bg-amber-950/30 text-amber-700 dark:text-amber-300 hover:bg-amber-100 dark:hover:bg-amber-900/50',
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

export function TelegramSection(props: EmailNotificationsApi) {
    const {
        telegramConfig,
        setTelegramConfig,
        telegramStatus,
        isTestingTelegram,
        isDetectingChatId,
        isSavingTelegram,
        isSendingTelegramPing,
        telegramBotUsername,
        showBotToken,
        setShowBotToken,
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
                    source: 'TelegramSection.loadSshNodes',
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
        <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
                <div>
                    <h3 className="text-sm sm:text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                        <MessageSquare className="w-4 h-4 text-sky-500" />
                        Telegram Bot &amp; Alert Notifications
                    </h3>
                    <p className="text-xs text-gray-500 dark:text-slate-400">
                        Receive real-time push alerts, account switch logs, CSV/JSON file exports, and multi-machine remote command executions directly on Telegram.
                    </p>
                </div>

                <div className="flex flex-wrap items-center gap-2">
                    {telegramStatus?.is_running ? (
                        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                            Active &amp; Polling
                        </span>
                    ) : (
                        <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-gray-100 dark:bg-slate-800 text-gray-600 dark:text-slate-400 border border-gray-200 dark:border-slate-700">
                            Inactive
                        </span>
                    )}

                    <div className="inline-flex items-center rounded-full border border-sky-200 dark:border-sky-900/50 bg-sky-50/50 dark:bg-sky-950/20 p-0.5 shadow-xs">
                        <button
                            type="button"
                            onClick={() => {
                                const cmd = `agm telegram connect "${telegramConfig?.bot_token || '<BOT_TOKEN>'}"${telegramConfig?.allowed_chat_id ? ` ${telegramConfig.allowed_chat_id}` : ''}`;
                                handleCopyCommand('cli-connect', cmd);
                            }}
                            className="px-2.5 py-1 text-xs font-medium rounded-l-full text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/40 transition-all flex items-center gap-1.5 cursor-pointer"
                            title="Copy AGM CLI one-command connect string"
                        >
                            <Terminal className="w-3.5 h-3.5" />
                            <span>{copiedChip === 'cli-connect' ? 'Copied!' : 'Copy CLI Connect'}</span>
                        </button>
                        <div className="h-3.5 w-px bg-sky-200 dark:bg-sky-800" />
                        <button
                            type="button"
                            onClick={() => {
                                const cmd = `.\\03-ai-scripts\\telegram-bot-helper.ps1 -BotToken "${telegramConfig?.bot_token || 'TOKEN'}" -ChatId "${telegramConfig?.allowed_chat_id || 'CHAT_ID'}"`;
                                handleCopyCommand('ps-helper', cmd);
                            }}
                            className="px-2.5 py-1 text-xs font-medium rounded-r-full text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/40 transition-all flex items-center gap-1.5 cursor-pointer"
                            title="Copy PowerShell step-by-step helper command"
                        >
                            <Copy className="w-3.5 h-3.5" />
                            <span>{copiedChip === 'ps-helper' ? 'Copied!' : 'Copy PS Helper Script'}</span>
                        </button>
                    </div>
                </div>
            </div>

            <div className="space-y-4">
                {/* Bot Token & Allowed Chat ID (Horizontally Aligned) */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5 items-end">
                    <div className="space-y-1.5">
                        <label className="text-xs font-medium text-gray-700 dark:text-slate-300 flex items-center justify-between h-5">
                            <span>Telegram Bot Token</span>
                            {telegramBotUsername && (
                                <span className="text-[11px] text-sky-600 dark:text-sky-400 font-semibold">
                                    @{telegramBotUsername} Verified
                                </span>
                            )}
                        </label>
                        <div className="relative">
                            <input
                                type={showBotToken ? 'text' : 'password'}
                                value={telegramConfig?.bot_token || ''}
                                onChange={(e) =>
                                    setTelegramConfig((prev) =>
                                        prev
                                            ? { ...prev, bot_token: e.target.value }
                                            : {
                                                  bot_token: e.target.value,
                                                  allowed_chat_id: null,
                                                  is_enabled: false,
                                                  poll_interval_secs: 5,
                                              }
                                    )
                                }
                                placeholder="123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ"
                                className="w-full h-9 px-3 py-2 pr-10 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-gray-50 dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-sky-500 font-mono"
                            />
                            <button
                                type="button"
                                onClick={() => setShowBotToken(!showBotToken)}
                                className="absolute right-2.5 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-slate-300 p-0.5"
                            >
                                {showBotToken ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                            </button>
                        </div>
                    </div>

                    <div className="space-y-1.5">
                        <label className="text-xs font-medium text-gray-700 dark:text-slate-300 flex items-center justify-between h-5">
                            <span>Allowed Chat ID (Numeric — Auto-detectable)</span>
                            <button
                                type="button"
                                onClick={handleDetectTelegramChatId}
                                disabled={isDetectingChatId}
                                className="text-[11px] font-semibold text-sky-600 dark:text-sky-400 hover:underline flex items-center gap-1 cursor-pointer disabled:opacity-60"
                                title="Send /ping to your bot in Telegram, then click here to auto-fill Chat ID"
                            >
                                {isDetectingChatId ? (
                                    <Loader2 className="w-3 h-3 animate-spin" />
                                ) : (
                                    <Sparkles className="w-3 h-3" />
                                )}
                                <span>Auto-Detect from /ping</span>
                            </button>
                        </label>
                        <div className="flex items-center gap-2">
                            <input
                                type="number"
                                value={telegramConfig?.allowed_chat_id ?? ''}
                                onChange={(e) => {
                                    const val = e.target.value.trim();
                                    const parsed = val ? parseInt(val, 10) : null;
                                    setTelegramConfig((prev) =>
                                        prev
                                            ? { ...prev, allowed_chat_id: isNaN(parsed as any) ? null : parsed }
                                            : {
                                                  bot_token: '',
                                                  allowed_chat_id: isNaN(parsed as any) ? null : parsed,
                                                  is_enabled: false,
                                                  poll_interval_secs: 5,
                                              }
                                    );
                                }}
                                placeholder="Leave blank & send /ping to auto-bind, or click Auto-Detect"
                                className="w-full h-9 px-3 py-2 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-gray-50 dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-sky-500 font-mono"
                            />
                        </div>
                    </div>
                </div>

                {/* Enable Toggle & Polling Interval */}
                <div className="flex flex-wrap items-center justify-between gap-4 p-3 rounded-lg bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-750">
                    <label className="flex items-center gap-2.5 cursor-pointer">
                        <input
                            type="checkbox"
                            checked={telegramConfig?.is_enabled || false}
                            onChange={(e) =>
                                setTelegramConfig((prev) =>
                                    prev
                                        ? { ...prev, is_enabled: e.target.checked }
                                        : {
                                              bot_token: '',
                                              allowed_chat_id: null,
                                              is_enabled: e.target.checked,
                                              poll_interval_secs: 5,
                                          }
                                )
                            }
                            className="w-4 h-4 rounded text-sky-600 focus:ring-sky-500 border-gray-300 dark:border-slate-600"
                        />
                        <div>
                            <span className="text-xs font-semibold text-gray-900 dark:text-slate-100">
                                Enable Telegram Bot Notifications &amp; Inbound Daemon
                            </span>
                            <p className="text-[11px] text-gray-500 dark:text-slate-400">
                                Polls for commands and sends instant alerts when accounts switch or quota drops.
                            </p>
                        </div>
                    </label>

                    <div className="flex items-center gap-2">
                        <span className="text-xs text-gray-600 dark:text-slate-400">Poll Interval:</span>
                        <input
                            type="number"
                            min={2}
                            max={60}
                            value={telegramConfig?.poll_interval_secs || 5}
                            onChange={(e) => {
                                const val = parseInt(e.target.value, 10) || 5;
                                setTelegramConfig((prev) => (prev ? { ...prev, poll_interval_secs: val } : null));
                            }}
                            className="w-16 px-2 py-1 text-xs border border-gray-200 dark:border-slate-700 rounded bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 text-center font-mono"
                        />
                        <span className="text-xs text-gray-500">sec</span>
                    </div>
                </div>

                {/* System Update Telegram Alert Toggle */}
                <div className="flex items-center justify-between p-3 rounded-lg bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-750">
                    <label className="flex items-center gap-2.5 cursor-pointer">
                        <input
                            type="checkbox"
                            checked={telegramConfig?.notify_on_system_update ?? true}
                            onChange={(e) =>
                                setTelegramConfig((prev) =>
                                    prev
                                        ? { ...prev, notify_on_system_update: e.target.checked }
                                        : {
                                              bot_token: '',
                                              allowed_chat_id: null,
                                              is_enabled: false,
                                              poll_interval_secs: 5,
                                              notify_on_system_update: e.target.checked,
                                          }
                                )
                            }
                            className="w-4 h-4 rounded text-sky-600 focus:ring-sky-500 border-gray-300 dark:border-slate-600"
                        />
                        <div>
                            <span className="text-xs font-semibold text-gray-900 dark:text-slate-100">
                                Notify via Telegram on System Update
                            </span>
                            <p className="text-[11px] text-gray-500 dark:text-slate-400">
                                Send update confirmation and release telemetry to Telegram chat upon version upgrades (default enabled).
                            </p>
                        </div>
                    </label>
                </div>

                {/* Quick Interactive Tools: Prompt Dispatcher + CSV/JSON Export + Live Balance Check */}
                <div className="p-3.5 rounded-xl bg-slate-50/90 dark:bg-slate-800/50 border border-slate-200/80 dark:border-slate-700/70 space-y-3.5">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <div className="flex items-center gap-2">
                            <Cpu className="w-4 h-4 text-sky-500" />
                            <span className="text-xs font-semibold text-gray-900 dark:text-slate-100">
                                Quick Prompt Dispatcher (Machine → Instance → Prompt)
                            </span>
                        </div>
                        <span className="text-[11px] font-mono text-sky-600 dark:text-sky-400">
                            {formattedSlashPrompt}
                        </span>
                    </div>

                    <div className="flex flex-col lg:flex-row items-stretch lg:items-center gap-2">
                        <div className="inline-flex items-center rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 px-3 py-1 shadow-xs">
                            <span className="text-[11px] font-semibold text-gray-400 mr-1.5">Machine:</span>
                            <select
                                value={quickMachine}
                                onChange={(e) => setQuickMachine(e.target.value)}
                                className="text-xs font-mono text-gray-900 dark:text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none pr-2 cursor-pointer"
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
                            <div className="h-4 w-px bg-gray-200 dark:bg-slate-700 mx-2" />
                            <span className="text-[11px] font-semibold text-gray-400 mr-1.5">Instance:</span>
                            <input
                                type="text"
                                value={quickInstance}
                                onChange={(e) => setQuickInstance(e.target.value)}
                                placeholder="default"
                                className="w-24 text-xs font-mono text-gray-900 dark:text-slate-100 bg-transparent border-0 focus:ring-0 focus:outline-none"
                            />
                        </div>

                        <input
                            type="text"
                            value={quickPrompt}
                            onChange={(e) => setQuickPrompt(e.target.value)}
                            placeholder="Write unit tests, Fix build, Audit code..."
                            className="flex-1 h-8 px-3 text-xs rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 font-mono focus:outline-none focus:ring-2 focus:ring-sky-500"
                        />

                        <div className="inline-flex items-center rounded-full border border-sky-200 dark:border-sky-800 bg-white dark:bg-slate-900 p-0.5 self-start lg:self-auto">
                            <button
                                type="button"
                                onClick={() => handleCopyCommand('quick-slash-prompt', formattedSlashPrompt)}
                                className="px-3 py-1 text-xs font-medium text-gray-700 dark:text-slate-200 hover:bg-gray-50 dark:hover:bg-slate-800 rounded-l-full flex items-center gap-1.5 cursor-pointer"
                            >
                                {copiedChip === 'quick-slash-prompt' ? (
                                    <Check className="w-3.5 h-3.5 text-emerald-500" />
                                ) : (
                                    <Copy className="w-3.5 h-3.5 text-gray-400" />
                                )}
                                <span>
                                    {copiedChip === 'quick-slash-prompt' ? 'Copied!' : 'Copy /prompt Command'}
                                </span>
                            </button>
                            <div className="h-4 w-px bg-gray-200 dark:bg-slate-700" />
                            <button
                                type="button"
                                onClick={handleQuickSendPrompt}
                                disabled={isDispatchingPrompt}
                                className="px-3.5 py-1 text-xs font-semibold bg-sky-600 hover:bg-sky-700 text-white rounded-r-full flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
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

                    {/* CSV / JSON Export Bar & Account Balance Quick Check */}
                    <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-gray-200/70 dark:border-slate-700/60">
                        <div className="flex flex-wrap items-center gap-1.5">
                            <span className="text-[11px] font-semibold text-gray-500 dark:text-slate-400 flex items-center gap-1 mr-1">
                                <FileSpreadsheet className="w-3.5 h-3.5 text-purple-500" />
                                CSV / JSON Export:
                            </span>
                            <div className="inline-flex flex-wrap items-center rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-0.5 shadow-xs">
                                {(
                                    [
                                        { label: 'Accounts CSV', target: 'accounts', format: 'csv' },
                                        { label: 'Instances CSV', target: 'instances', format: 'csv' },
                                        { label: 'Balances CSV', target: 'balance', format: 'csv' },
                                        { label: 'SSH Nodes CSV', target: 'nodes', format: 'csv' },
                                        { label: 'Prompts JSON', target: 'prompts', format: 'json' },
                                    ] as const
                                ).map((btn, i, arr) => {
                                    const key = `${btn.target}-${btn.format}-dl`;
                                    return (
                                        <div key={key} className="inline-flex items-center">
                                            <button
                                                type="button"
                                                onClick={() => handleQuickExport(btn.target, btn.format, false)}
                                                disabled={exportingTarget === key}
                                                className={`px-2.5 py-1 text-[11px] font-medium text-gray-700 dark:text-slate-200 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors flex items-center gap-1 cursor-pointer disabled:opacity-50 ${
                                                    i === 0
                                                        ? 'rounded-l-full'
                                                        : i === arr.length - 1
                                                          ? 'rounded-r-full'
                                                          : ''
                                                }`}
                                            >
                                                {exportingTarget === key ? (
                                                    <Loader2 className="w-3 h-3 animate-spin text-purple-500" />
                                                ) : (
                                                    <Download className="w-3 h-3 text-purple-500" />
                                                )}
                                                <span>{btn.label}</span>
                                            </button>
                                            {i < arr.length - 1 && (
                                                <div className="h-3.5 w-px bg-gray-200 dark:bg-slate-700" />
                                            )}
                                        </div>
                                    );
                                })}
                            </div>
                        </div>

                        <div className="inline-flex items-center rounded-full border border-emerald-200 dark:border-emerald-800/70 bg-white dark:bg-slate-900 p-0.5 shadow-xs">
                            <button
                                type="button"
                                onClick={() => handleQuickCheckBalances(false)}
                                disabled={isCheckingBalances}
                                className="px-3 py-1 text-[11px] font-semibold text-emerald-700 dark:text-emerald-300 hover:bg-emerald-50 dark:hover:bg-emerald-950/40 rounded-l-full flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                            >
                                {isCheckingBalances ? (
                                    <Loader2 className="w-3 h-3 animate-spin" />
                                ) : (
                                    <Layers className="w-3 h-3 text-emerald-500" />
                                )}
                                <span>Account Balance Quick Check</span>
                            </button>
                            <div className="h-3.5 w-px bg-gray-200 dark:bg-slate-700" />
                            <button
                                type="button"
                                onClick={() => handleQuickCheckBalances(true)}
                                disabled={isCheckingBalances}
                                className="px-3 py-1 text-[11px] font-semibold text-sky-700 dark:text-sky-300 hover:bg-sky-50 dark:hover:bg-sky-950/40 rounded-r-full flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                            >
                                <Send className="w-3 h-3 text-sky-500" />
                                <span>Push Balance CSV to Telegram</span>
                            </button>
                        </div>
                    </div>

                    {/* Inline Live Balance Summary Table when toggled */}
                    {showBalancePanel && (
                        <div className="pt-2 space-y-1.5">
                            <div className="flex items-center justify-between text-[11px]">
                                <span className="font-semibold text-gray-700 dark:text-slate-200">
                                    Live Instance &amp; Account Balance Snapshot ({clusterBalances.length})
                                </span>
                                <button
                                    type="button"
                                    onClick={() => setShowBalancePanel(false)}
                                    className="text-gray-400 hover:text-gray-600 dark:hover:text-slate-200 cursor-pointer"
                                >
                                    Hide
                                </button>
                            </div>
                            <div className="overflow-x-auto rounded-lg border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900">
                                <table className="w-full text-left border-collapse text-[11px]">
                                    <thead>
                                        <tr className="bg-gray-50 dark:bg-slate-800 text-gray-600 dark:text-slate-300 border-b border-gray-200 dark:border-slate-700">
                                            <th className="py-1.5 px-2.5">Machine</th>
                                            <th className="py-1.5 px-2.5">Instance</th>
                                            <th className="py-1.5 px-2.5">Account</th>
                                            <th className="py-1.5 px-2.5">4h Credit</th>
                                            <th className="py-1.5 px-2.5">Weekly Credit</th>
                                        </tr>
                                    </thead>
                                    <tbody className="divide-y divide-gray-100 dark:divide-slate-800">
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

                {/* Interactive Click-to-Copy Power Commands & Examples Hub */}
                <div className="p-3.5 rounded-xl bg-sky-50/40 dark:bg-sky-950/20 border border-sky-100 dark:border-sky-900/40 space-y-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <div className="font-semibold text-xs text-sky-900 dark:text-sky-200 flex items-center gap-1.5">
                            <Terminal className="w-3.5 h-3.5 text-sky-500" />
                            <span>Click-to-Copy Power Commands &amp; Examples (Telegram Bot &amp; AGM / GitMap CLI)</span>
                        </div>
                        <span className="text-[11px] text-sky-700 dark:text-sky-400">
                            Click any command pill to copy immediately
                        </span>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                        {TELEGRAM_POWER_COMMAND_CATEGORIES.map((cat) => (
                            <div
                                key={cat.id}
                                className="p-2.5 rounded-lg bg-white/80 dark:bg-slate-900/70 border border-gray-100 dark:border-slate-800 space-y-2"
                            >
                                <div className="text-[11px] font-semibold text-gray-800 dark:text-slate-200">
                                    {cat.title}
                                </div>
                                <div className="flex flex-wrap gap-1.5">
                                    {cat.commands.map((cmd) => {
                                        const chipKey = `${cat.id}:${cmd.label}`;
                                        const isCopied = copiedChip === chipKey;
                                        return (
                                            <button
                                                key={chipKey}
                                                type="button"
                                                onClick={() => handleCopyCommand(chipKey, cmd.payload)}
                                                title={`${cmd.hint} — Click to copy: ${cmd.payload}`}
                                                className={`inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-[10px] font-mono border transition-all cursor-pointer ${cat.badgeClass}`}
                                            >
                                                {isCopied ? (
                                                    <Check className="w-3 h-3 text-emerald-500 shrink-0" />
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

                {/* Action Buttons */}
                <div className="flex flex-wrap items-center justify-between gap-2 pt-1 border-t border-gray-100 dark:border-slate-800">
                    <div className="text-[11px] text-gray-500 dark:text-slate-400">
                        Send <code className="font-semibold text-sky-600 dark:text-sky-400">/ping</code> to your bot in Telegram, then click{' '}
                        <span className="font-semibold text-sky-600 dark:text-sky-400">Auto-Detect Chat ID</span>.
                    </div>

                    <div className="flex flex-wrap items-center gap-2">
                        <button
                            type="button"
                            onClick={handleTestTelegram}
                            disabled={isTestingTelegram}
                            className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                        >
                            {isTestingTelegram ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <RefreshCw className="w-3.5 h-3.5" />
                            )}
                            Test Bot Token
                        </button>
                        <button
                            type="button"
                            onClick={handleDetectTelegramChatId}
                            disabled={isDetectingChatId}
                            className="px-3 py-1.5 text-xs font-medium border border-sky-200 dark:border-sky-800 bg-sky-50/60 dark:bg-sky-950/30 hover:bg-sky-100 dark:hover:bg-sky-900/50 text-sky-700 dark:text-sky-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                        >
                            {isDetectingChatId ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Sparkles className="w-3.5 h-3.5" />
                            )}
                            Auto-Detect Chat ID
                        </button>
                        <button
                            type="button"
                            onClick={handleSendTelegramPing}
                            disabled={isSendingTelegramPing}
                            className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                        >
                            {isSendingTelegramPing ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Send className="w-3.5 h-3.5" />
                            )}
                            Send Test Alert
                        </button>
                        <button
                            type="button"
                            onClick={handleSaveTelegram}
                            disabled={isSavingTelegram}
                            className="px-4 py-1.5 bg-sky-600 hover:bg-sky-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                        >
                            {isSavingTelegram ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <CheckCircle2 className="w-3.5 h-3.5" />
                            )}
                            Save Telegram Settings
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
