import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
    X,
    MessageSquare,
    Copy,
    Check,
    Terminal,
    ExternalLink,
    Sparkles,
    ShieldCheck,
    ArrowRight,
    Bot,
    Key,
    Hash,
} from 'lucide-react';
import { copyToClipboard } from '../../utils/clipboard';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import { telegramService } from '../../services/telegramService';

interface TelegramGuideModalProps {
    isOpen: boolean;
    onClose: () => void;
    botUsername?: string | null;
    isBotRunning?: boolean;
}

interface CliCommand {
    cmd: string;
    desc: string;
    category: string;
}

const CLI_COMMANDS: CliCommand[] = [
    {
        cmd: 'agm telegram status',
        desc: 'Inspect Telegram daemon connectivity, polling latency, and webhook health.',
        category: 'Diagnostics',
    },
    {
        cmd: 'agm telegram send "Hello from AGM CLI"',
        desc: 'Send an immediate test alert or message to your authorized chat.',
        category: 'Outbound',
    },
    {
        cmd: 'agm telegram listen',
        desc: 'Launch continuous background listener daemon to receive and execute remote commands.',
        category: 'Daemon',
    },
    {
        cmd: 'agm telegram setup',
        desc: 'Interactive step-by-step terminal wizard to configure bot token and chat ID.',
        category: 'Configuration',
    },
    {
        cmd: 'agm telegram test',
        desc: 'Validate bot token against Telegram getMe API and retrieve bot username.',
        category: 'Validation',
    },
    {
        cmd: 'agm telegram chat-id',
        desc: 'Auto-detect your authorized chat ID by querying recent bot update events.',
        category: 'Discovery',
    },
    {
        cmd: 'agm prompt --instance default "Run tests and verify build"',
        desc: 'Directly dispatch an execution prompt to a local or cluster instance via CLI.',
        category: 'Remote Dispatch',
    },
    {
        cmd: 'agm failed-commands',
        desc: 'Inspect failed or unrecognized CLI commands with hit count & suggestion telemetry.',
        category: 'Telemetry',
    },
];

export function TelegramGuideModal({
    isOpen,
    onClose,
    botUsername,
    isBotRunning,
}: TelegramGuideModalProps) {
    const navigate = useNavigate();
    const [copiedCmd, setCopiedCmd] = useState<string | null>(null);
    const [liveRunning, setLiveRunning] = useState<boolean | null>(isBotRunning ?? null);
    const [liveUsername, setLiveUsername] = useState<string | null>(botUsername ?? null);

    useEffect(() => {
        if (!isOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    useEffect(() => {
        if (!isOpen) return;
        if (isBotRunning !== undefined) {
            setLiveRunning(isBotRunning);
            setLiveUsername(botUsername ?? null);
            return;
        }

        let isMounted = true;
        void (async () => {
            try {
                const [stat, cfg] = await Promise.all([
                    telegramService.getStatus().catch((err) => {
                        useErrorStore.getState().trackWarning(err, {
                            source: 'TelegramGuideModal.getStatus',
                            triggerAction: 'poll_telegram_status',
                        });
                        return null;
                    }),
                    telegramService.getConfig().catch((err) => {
                        useErrorStore.getState().trackWarning(err, {
                            source: 'TelegramGuideModal.getConfig',
                            triggerAction: 'get_telegram_config',
                        });
                        return null;
                    }),
                ]);

                if (isMounted) {
                    if (stat) setLiveRunning(stat.is_running);
                    if (cfg?.bot_token) {
                        telegramService.testBot(cfg.bot_token).then((username) => {
                            if (isMounted) setLiveUsername(username);
                        }).catch((err) => {
                            useErrorStore.getState().trackWarning(err, {
                                source: 'TelegramGuideModal.testBot',
                                triggerAction: 'test_telegram_bot',
                            });
                        });
                    }
                }
            } catch (err) {
                useErrorStore.getState().trackWarning(err, {
                    source: 'TelegramGuideModal.init',
                    triggerAction: 'init_guide_modal',
                });
            }
        })();

        return () => {
            isMounted = false;
        };
    }, [isOpen, isBotRunning, botUsername]);

    if (!isOpen) return null;

    const handleCopy = async (text: string, id: string) => {
        try {
            await copyToClipboard(text);
            setCopiedCmd(id);
            showToast('Command copied to clipboard', 'success');
            setTimeout(() => setCopiedCmd(null), 2000);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'TelegramGuideModal.handleCopy',
                triggerAction: 'copy_cli_command',
            });
        }
    };

    return (
        <div
            className="fixed inset-0 z-[99999] flex items-center justify-center bg-black/60 backdrop-blur-xs animate-in fade-in duration-200 p-4 sm:p-6"
            onClick={onClose}
        >
            <div
                className="relative w-full max-w-2xl max-h-[90vh] flex flex-col rounded-2xl bg-white dark:bg-[#171025] border border-slate-200 dark:border-[#40285e] shadow-2xl overflow-hidden"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="px-6 py-4 border-b border-slate-200 dark:border-[#40285e] flex items-center justify-between bg-slate-50/50 dark:bg-[#090611]/50 shrink-0">
                    <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-xl bg-sky-500/10 border border-sky-500/20 text-sky-500 flex items-center justify-center shrink-0">
                            <MessageSquare className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-base font-bold text-slate-900 dark:text-white flex items-center gap-2">
                                <span>Telegram Remote Control Setup</span>
                                {liveRunning ? (
                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30">
                                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                        {liveUsername ? `@${liveUsername}` : 'Bot Connected'}
                                    </span>
                                ) : (
                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-slate-500/15 text-slate-600 dark:text-slate-400 border border-slate-500/30">
                                        Setup Required
                                    </span>
                                )}
                            </h2>
                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                                Complete 3-step setup guide and recommended terminal CLI commands.
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#25173e] transition-colors cursor-pointer"
                        title="Close Guide (Esc)"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Body Content */}
                <div className="p-6 overflow-y-auto space-y-6 flex-1 text-xs">
                    {/* Step-by-Step Cards */}
                    <div className="space-y-3">
                        <h3 className="text-xs font-semibold text-slate-900 dark:text-slate-200 uppercase tracking-wider flex items-center gap-1.5">
                            <Sparkles className="w-3.5 h-3.5 text-sky-500" />
                            <span>Quick Setup Steps</span>
                        </h3>

                        {/* Step 1 */}
                        <div className="p-3.5 rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/70 dark:bg-[#090611]/60 space-y-1.5">
                            <div className="flex items-center justify-between">
                                <span className="font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
                                    <span className="w-5 h-5 rounded-full bg-sky-500/20 text-sky-600 dark:text-sky-400 font-bold text-[10px] flex items-center justify-center">1</span>
                                    <Bot className="w-3.5 h-3.5 text-sky-500" />
                                    <span>Create Telegram Bot via @BotFather</span>
                                </span>
                                <a
                                    href="https://t.me/botfather"
                                    target="_blank"
                                    rel="noreferrer"
                                    className="text-[11px] font-medium text-sky-600 dark:text-sky-400 hover:underline flex items-center gap-1"
                                >
                                    <span>Open @BotFather</span>
                                    <ExternalLink className="w-3 h-3" />
                                </a>
                            </div>
                            <p className="text-slate-600 dark:text-slate-400 leading-relaxed pl-7">
                                Open Telegram, start a chat with <code className="font-mono bg-white dark:bg-[#1f1633] px-1 py-0.5 rounded border border-slate-200 dark:border-[#40285e]">@BotFather</code>, send <code className="font-mono bg-white dark:bg-[#1f1633] px-1 py-0.5 rounded border border-slate-200 dark:border-[#40285e]">/newbot</code>, and choose a friendly display name and a username ending in <code className="font-mono">bot</code> (e.g. <code className="font-mono">my_agm_control_bot</code>).
                            </p>
                        </div>

                        {/* Step 2 */}
                        <div className="p-3.5 rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/70 dark:bg-[#090611]/60 space-y-1.5">
                            <div className="flex items-center justify-between">
                                <span className="font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
                                    <span className="w-5 h-5 rounded-full bg-sky-500/20 text-sky-600 dark:text-sky-400 font-bold text-[10px] flex items-center justify-center">2</span>
                                    <Key className="w-3.5 h-3.5 text-amber-500" />
                                    <span>Obtain Bot Token & Send Initial Message</span>
                                </span>
                            </div>
                            <p className="text-slate-600 dark:text-slate-400 leading-relaxed pl-7">
                                Copy the API token provided by BotFather (format: <code className="font-mono bg-white dark:bg-[#1f1633] px-1 py-0.5 rounded border border-slate-200 dark:border-[#40285e]">123456789:ABCdefGhI...</code>) into the Bot Token input on the Telegram page or run <code className="font-mono bg-white dark:bg-[#1f1633] px-1 py-0.5 rounded border border-slate-200 dark:border-[#40285e]">agm telegram setup</code>. Then open your new bot in Telegram and send <code className="font-mono bg-white dark:bg-[#1f1633] px-1 py-0.5 rounded border border-slate-200 dark:border-[#40285e]">/start</code> so it receives your chat ID.
                            </p>
                        </div>

                        {/* Step 3 */}
                        <div className="p-3.5 rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/70 dark:bg-[#090611]/60 space-y-1.5">
                            <div className="flex items-center justify-between">
                                <span className="font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-2">
                                    <span className="w-5 h-5 rounded-full bg-sky-500/20 text-sky-600 dark:text-sky-400 font-bold text-[10px] flex items-center justify-center">3</span>
                                    <Hash className="w-3.5 h-3.5 text-emerald-500" />
                                    <span>Auto-Detect Chat ID & Enable Listener</span>
                                </span>
                            </div>
                            <p className="text-slate-600 dark:text-slate-400 leading-relaxed pl-7">
                                Click the <strong>Auto-Detect Chat ID</strong> button or run <code className="font-mono bg-white dark:bg-[#1f1633] px-1 py-0.5 rounded border border-slate-200 dark:border-[#40285e]">agm telegram chat-id</code>. AGM queries recent bot updates to fetch your authorized numeric ID automatically. Toggle <strong>Enable Telegram Listener</strong> and save!
                            </p>
                        </div>
                    </div>

                    {/* Recommended AGM CLI Commands */}
                    <div className="space-y-3 pt-2">
                        <div className="flex items-center justify-between">
                            <h3 className="text-xs font-semibold text-slate-900 dark:text-slate-200 uppercase tracking-wider flex items-center gap-1.5">
                                <Terminal className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Recommended AGM Terminal CLI Commands</span>
                            </h3>
                            <span className="text-[10px] text-slate-400">Click any command to copy</span>
                        </div>

                        <div className="space-y-2">
                            {CLI_COMMANDS.map((item) => {
                                const isCopied = copiedCmd === item.cmd;
                                return (
                                    <div
                                        key={item.cmd}
                                        onClick={() => handleCopy(item.cmd, item.cmd)}
                                        className="group p-2.5 rounded-xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] hover:border-sky-500/40 hover:bg-sky-50/30 dark:hover:bg-[#1e1336] transition-all cursor-pointer flex items-center justify-between gap-3 shadow-2xs"
                                    >
                                        <div className="min-w-0 flex-1 space-y-0.5">
                                            <div className="flex items-center gap-2">
                                                <span className="px-1.5 py-0.2 rounded text-[9px] font-semibold bg-slate-100 dark:bg-[#25173e] text-slate-600 dark:text-slate-300 border border-slate-200 dark:border-[#40285e]">
                                                    {item.category}
                                                </span>
                                                <code className="font-mono text-xs font-bold text-slate-900 dark:text-white truncate">
                                                    {item.cmd}
                                                </code>
                                            </div>
                                            <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-tight">
                                                {item.desc}
                                            </p>
                                        </div>
                                        <button
                                            type="button"
                                            className="p-1.5 rounded-lg text-slate-400 group-hover:text-sky-500 hover:bg-slate-100 dark:hover:bg-[#201538] transition-colors shrink-0"
                                            title="Copy command"
                                        >
                                            {isCopied ? (
                                                <Check className="w-3.5 h-3.5 text-emerald-500" />
                                            ) : (
                                                <Copy className="w-3.5 h-3.5" />
                                            )}
                                        </button>
                                    </div>
                                );
                            })}
                        </div>
                    </div>
                </div>

                {/* Footer */}
                <div className="px-6 py-3 border-t border-slate-200 dark:border-[#40285e] flex items-center justify-between bg-slate-50/50 dark:bg-[#090611]/50 shrink-0">
                    <div className="flex items-center gap-2 text-[11px] text-slate-500 dark:text-slate-400">
                        <ShieldCheck className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
                        <span>Single-Chat Auth Protection: Encrypted in SQLite vault</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={() => {
                                onClose();
                                navigate('/telegram');
                            }}
                            className="px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white shadow-xs flex items-center gap-1.5 transition-colors cursor-pointer"
                        >
                            <span>Open Telegram Hub</span>
                            <ArrowRight className="w-3.5 h-3.5" />
                        </button>
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-4 py-1.5 rounded-lg text-xs font-semibold bg-slate-900 text-white dark:bg-white dark:text-slate-900 hover:opacity-90 transition-opacity cursor-pointer shadow-xs"
                        >
                            Got It
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
