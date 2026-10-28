import { useEffect, useState, useCallback } from 'react';
import {
    MessageSquare,
    Bot,
    Key,
    Hash,
    Radio,
    RefreshCw,
    Send,
    Check,
    CheckCircle2,
    AlertCircle,
    HelpCircle,
    Eye,
    EyeOff,
    Copy,
    Save,
    Sparkles,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import {
    telegramService,
    type TelegramConfig,
    type TelegramWatcherStatus,
} from '../services/telegramService';
import { showToast } from '../components/common/ToastContainer';
import { useErrorStore } from '../stores/error-store';
import { copyToClipboard } from '../utils/clipboard';
import TelegramSettings from '../components/settings/TelegramSettings';
import { TelegramGuideModal } from '../components/telegram/TelegramGuideModal';

export default function Telegram() {
    const { t } = useTranslation();
    const [config, setConfig] = useState<TelegramConfig>({
        bot_token: '',
        allowed_chat_id: null,
        is_enabled: false,
        poll_interval_secs: 5,
        notify_on_system_update: true,
    });
    const [status, setStatus] = useState<TelegramWatcherStatus | null>(null);
    const [botUsername, setBotUsername] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isSaving, setIsSaving] = useState(false);
    const [isTesting, setIsTesting] = useState(false);
    const [isDetecting, setIsDetecting] = useState(false);
    const [isSendingTest, setIsSendingTest] = useState(false);
    const [showToken, setShowToken] = useState(false);
    const [isGuideOpen, setIsGuideOpen] = useState(false);
    const [copiedCmd, setCopiedCmd] = useState<string | null>(null);

    const loadData = useCallback(async () => {
        setIsLoading(true);
        try {
            const [cfg, stat] = await Promise.all([
                telegramService.getConfig().catch((err) => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'TelegramPage.loadData',
                        triggerAction: 'get_telegram_config',
                    });
                    return null;
                }),
                telegramService.getStatus().catch((err) => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'TelegramPage.loadData',
                        triggerAction: 'get_telegram_status',
                    });
                    return null;
                }),
            ]);

            if (cfg) {
                setConfig(cfg);
            }
            if (stat) {
                setStatus(stat);
            }
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        void loadData();
    }, [loadData]);

    const handleSave = async () => {
        setIsSaving(true);
        try {
            await telegramService.saveConfig(config);
            showToast('Telegram configuration saved successfully', 'success');
            const newStat = await telegramService.getStatus();
            setStatus(newStat);
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                { source: 'TelegramPage.handleSave', triggerAction: 'save_telegram_config' }
            );
        } finally {
            setIsSaving(false);
        }
    };

    const handleTestBot = async () => {
        if (!config.bot_token.trim()) {
            showToast('Please enter a Bot Token first', 'warning');
            return;
        }
        setIsTesting(true);
        try {
            const username = await telegramService.testBot(config.bot_token.trim());
            setBotUsername(username);
            showToast(`Bot verified: @${username}`, 'success');
        } catch (err) {
            showToast('Bot verification failed. Check token validity.', 'error');
            useErrorStore.getState().trackWarning(err, {
                source: 'TelegramPage.handleTestBot',
                triggerAction: 'test_telegram_bot',
            });
        } finally {
            setIsTesting(false);
        }
    };

    const handleDetectChat = async () => {
        if (!config.bot_token.trim()) {
            showToast('Please enter a Bot Token first', 'warning');
            return;
        }
        setIsDetecting(true);
        try {
            const detected = await telegramService.detectChatId(config.bot_token.trim());
            const updatedConfig: TelegramConfig = {
                ...config,
                allowed_chat_id: detected.chat_id,
                is_enabled: true,
            };
            setConfig(updatedConfig);
            if (detected.bot_username) {
                setBotUsername(detected.bot_username);
            }

            try {
                await telegramService.saveConfig(updatedConfig);
                const newStat = await telegramService.getStatus();
                setStatus(newStat);
                showToast('Chat ID detected & configuration saved! Telegram daemon activated.', 'success');
            } catch (saveErr) {
                useErrorStore.getState().trackWarning(saveErr, {
                    source: 'TelegramPage.handleDetectChat.autoSave',
                    triggerAction: 'save_telegram_config',
                });
                showToast(`Chat ID detected: ${detected.chat_id}, but auto-save failed. Please click Save manually.`, 'warning');
            }
        } catch (err) {
            showToast('No chat ID detected. Send a message to your bot on Telegram first, then try again.', 'warning');
            useErrorStore.getState().trackWarning(err, {
                source: 'TelegramPage.handleDetectChat',
                triggerAction: 'detect_telegram_chat_id',
            });
        } finally {
            setIsDetecting(false);
        }
    };

    const handleSendTestMessage = async () => {
        if (!config.bot_token.trim() || !config.allowed_chat_id) {
            showToast('Both Bot Token and Allowed Chat ID are required to send a test message', 'warning');
            return;
        }
        setIsSendingTest(true);
        try {
            await telegramService.sendTestMessage(config.bot_token.trim(), config.allowed_chat_id);
            showToast('Test message dispatched to Telegram chat!', 'success');
        } catch (err) {
            showToast('Failed to send test message. Check token and chat ID permissions.', 'error');
            useErrorStore.getState().trackWarning(err, {
                source: 'TelegramPage.handleSendTestMessage',
                triggerAction: 'send_telegram_test_message',
            });
        } finally {
            setIsSendingTest(false);
        }
    };

    const handleCopyCli = async (cmd: string) => {
        try {
            await copyToClipboard(cmd);
            setCopiedCmd(cmd);
            showToast('CLI command copied', 'success');
            setTimeout(() => setCopiedCmd(null), 2000);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'TelegramPage.handleCopyCli',
                triggerAction: 'copy_cli_command',
            });
        }
    };

    const isRunning = Boolean(status?.is_running);

    return (
        <div className="h-full overflow-y-auto bg-slate-50/50 dark:bg-[#090611] text-slate-900 dark:text-slate-100">
            <div className="max-w-[1400px] mx-auto px-4 sm:px-6 py-6 space-y-6">
                {/* Header Banner */}
                <div className="agm-hover-shade rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-xs">
                    <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                        <div className="flex items-center gap-3.5">
                            <div className="w-12 h-12 rounded-2xl bg-sky-500/10 dark:bg-sky-500/15 border border-sky-500/20 text-sky-500 flex items-center justify-center shadow-xs">
                                <MessageSquare className="w-6 h-6" />
                            </div>
                            <div>
                                <div className="flex items-center gap-2">
                                    <h1 className="text-xl font-bold text-slate-900 dark:text-white tracking-tight">
                                        {t('telegram.page_title', 'Telegram Remote Control Hub')}
                                    </h1>
                                    {botUsername && (
                                        <span className="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-sky-500/10 text-sky-600 dark:text-sky-300 border border-sky-500/25">
                                            @{botUsername}
                                        </span>
                                    )}
                                </div>
                                <p className="text-xs text-slate-500 dark:text-slate-400 mt-1 max-w-2xl leading-relaxed">
                                    {t(
                                        'telegram.page_desc',
                                        'Real-time bidirectional daemon for conversational remote control, cross-machine prompt delegation, automated quota reports, and instant cluster telemetry.'
                                    )}
                                </p>
                            </div>
                        </div>

                        <div className="flex items-center gap-2.5 self-stretch sm:self-auto">
                            {/* Live Watcher Status Capsule */}
                            <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-slate-100 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e] text-xs font-medium text-slate-700 dark:text-slate-300">
                                {isRunning ? (
                                    <>
                                        <span className="relative flex h-2 w-2">
                                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                                        </span>
                                        <span className="text-emerald-600 dark:text-emerald-400 font-semibold">Listener Active</span>
                                    </>
                                ) : (
                                    <>
                                        <span className="h-2 w-2 rounded-full bg-slate-400"></span>
                                        <span>Listener Stopped</span>
                                    </>
                                )}
                            </div>

                            <button
                                type="button"
                                onClick={() => setIsGuideOpen(true)}
                                className="px-3 py-1.5 rounded-lg text-xs font-semibold bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/60 border border-sky-200 dark:border-sky-800/60 shadow-2xs flex items-center gap-1.5 transition-colors cursor-pointer"
                            >
                                <HelpCircle className="w-3.5 h-3.5" />
                                <span>Setup Guide</span>
                            </button>

                            <button
                                type="button"
                                onClick={() => void loadData()}
                                disabled={isLoading}
                                className="p-1.5 rounded-lg text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#25173e] transition-colors cursor-pointer disabled:opacity-50"
                                title="Refresh status and config"
                            >
                                <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
                            </button>
                        </div>
                    </div>
                </div>

                {/* Grid: Configuration & Fast CLI Quick Actions */}
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* Left 2 Cols: Bot Configuration */}
                    <div className="agm-hover-shade lg:col-span-2 rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-xs space-y-5">
                        <div className="flex items-center justify-between pb-3 border-b border-slate-200 dark:border-[#40285e]">
                            <div className="flex items-center gap-2">
                                <Bot className="w-4 h-4 text-sky-500" />
                                <h2 className="text-sm font-bold text-slate-900 dark:text-white">
                                    Bot Credentials & Polling Configuration
                                </h2>
                            </div>
                            <label className="flex items-center gap-2 cursor-pointer select-none">
                                <span className="text-xs font-medium text-slate-600 dark:text-slate-400">
                                    Enable Listener
                                </span>
                                <input
                                    type="checkbox"
                                    checked={config.is_enabled}
                                    onChange={(e) => setConfig({ ...config, is_enabled: e.target.checked })}
                                    className="toggle toggle-sm toggle-primary"
                                />
                            </label>
                        </div>

                        {/* Bot Token */}
                        <div className="space-y-1.5">
                            <label className="text-xs font-semibold text-slate-700 dark:text-slate-300 flex items-center justify-between">
                                <span className="flex items-center gap-1.5">
                                    <Key className="w-3.5 h-3.5 text-slate-400" />
                                    <span>Bot Token (@BotFather)</span>
                                </span>
                                <button
                                    type="button"
                                    onClick={handleTestBot}
                                    disabled={isTesting || !config.bot_token.trim()}
                                    className="text-[11px] font-semibold text-sky-600 dark:text-sky-400 hover:underline flex items-center gap-1 disabled:opacity-50 cursor-pointer"
                                >
                                    {isTesting ? <RefreshCw className="w-3 h-3 animate-spin" /> : <CheckCircle2 className="w-3 h-3" />}
                                    <span>Verify Token</span>
                                </button>
                            </label>
                            <div className="relative">
                                <input
                                    type={showToken ? 'text' : 'password'}
                                    value={config.bot_token}
                                    onChange={(e) => setConfig({ ...config, bot_token: e.target.value })}
                                    placeholder="123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ"
                                    className="w-full rounded-xl bg-slate-50 dark:bg-[#090611] pl-3.5 pr-10 py-2 text-xs font-mono text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#40285e] focus:outline-none focus:ring-2 focus:ring-sky-500/30"
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowToken(!showToken)}
                                    className="absolute right-3 top-2.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 cursor-pointer"
                                    title={showToken ? 'Hide token' : 'Show token'}
                                >
                                    {showToken ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
                                </button>
                            </div>
                        </div>

                        {/* Allowed Chat ID & Polling Interval */}
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="space-y-1.5">
                                <label className="text-xs font-semibold text-slate-700 dark:text-slate-300 flex items-center justify-between">
                                    <span className="flex items-center gap-1.5">
                                        <Hash className="w-3.5 h-3.5 text-slate-400" />
                                        <span>Authorized Chat ID</span>
                                    </span>
                                    <button
                                        type="button"
                                        onClick={handleDetectChat}
                                        disabled={isDetecting || !config.bot_token.trim()}
                                        className="text-[11px] font-semibold text-emerald-600 dark:text-emerald-400 hover:underline flex items-center gap-1 disabled:opacity-50 cursor-pointer"
                                    >
                                        {isDetecting ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Sparkles className="w-3 h-3" />}
                                        <span>Auto-Detect</span>
                                    </button>
                                </label>
                                <input
                                    type="number"
                                    value={config.allowed_chat_id ?? ''}
                                    onChange={(e) => {
                                        const val = e.target.value ? parseInt(e.target.value, 10) : null;
                                        setConfig({ ...config, allowed_chat_id: val });
                                    }}
                                    placeholder="e.g. 123456789"
                                    className="w-full rounded-xl bg-slate-50 dark:bg-[#090611] px-3.5 py-2 text-xs font-mono text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#40285e] focus:outline-none focus:ring-2 focus:ring-sky-500/30"
                                />
                            </div>

                            <div className="space-y-1.5">
                                <label className="text-xs font-semibold text-slate-700 dark:text-slate-300 flex items-center gap-1.5">
                                    <Radio className="w-3.5 h-3.5 text-slate-400" />
                                    <span>Polling Interval (Seconds)</span>
                                </label>
                                <input
                                    type="number"
                                    min={1}
                                    max={60}
                                    value={config.poll_interval_secs}
                                    onChange={(e) => setConfig({ ...config, poll_interval_secs: Math.max(1, parseInt(e.target.value, 10) || 5) })}
                                    className="w-full rounded-xl bg-slate-50 dark:bg-[#090611] px-3.5 py-2 text-xs font-mono text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#40285e] focus:outline-none focus:ring-2 focus:ring-sky-500/30"
                                />
                            </div>
                        </div>

                        {/* Toggles and Actions */}
                        <div className="pt-2 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-t border-slate-200 dark:border-[#40285e]">
                            <label className="flex items-center gap-2 cursor-pointer select-none">
                                <input
                                    type="checkbox"
                                    checked={config.notify_on_system_update ?? true}
                                    onChange={(e) => setConfig({ ...config, notify_on_system_update: e.target.checked })}
                                    className="checkbox checkbox-sm checkbox-primary"
                                />
                                <span className="text-xs text-slate-600 dark:text-slate-400">
                                    Send notifications on system updates & version bumps
                                </span>
                            </label>

                            <div className="flex items-center gap-2.5 w-full sm:w-auto">
                                <button
                                    type="button"
                                    onClick={handleSendTestMessage}
                                    disabled={isSendingTest || !config.bot_token || !config.allowed_chat_id}
                                    className="flex-1 sm:flex-none px-3 py-2 rounded-xl text-xs font-semibold bg-slate-100 hover:bg-slate-200 dark:bg-[#201538] dark:hover:bg-[#2b1c4b] text-slate-700 dark:text-slate-300 transition-colors cursor-pointer flex items-center justify-center gap-1.5 disabled:opacity-50"
                                >
                                    {isSendingTest ? <RefreshCw className="w-3.5 h-3.5 animate-spin" /> : <Send className="w-3.5 h-3.5 text-sky-500" />}
                                    <span>Send Test Message</span>
                                </button>

                                <button
                                    type="button"
                                    onClick={handleSave}
                                    disabled={isSaving}
                                    className="flex-1 sm:flex-none px-4 py-2 rounded-xl text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white shadow-xs transition-colors cursor-pointer flex items-center justify-center gap-1.5 disabled:opacity-50"
                                >
                                    {isSaving ? <RefreshCw className="w-3.5 h-3.5 animate-spin" /> : <Save className="w-3.5 h-3.5" />}
                                    <span>Save Configuration</span>
                                </button>
                            </div>
                        </div>
                    </div>

                    {/* Right 1 Col: Daemon Diagnostics & Status */}
                    <div className="agm-hover-shade rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-xs space-y-4">
                        <div className="flex items-center gap-2 pb-3 border-b border-slate-200 dark:border-[#40285e]">
                            <Radio className="w-4 h-4 text-emerald-500" />
                            <h2 className="text-sm font-bold text-slate-900 dark:text-white">
                                Daemon Diagnostics
                            </h2>
                        </div>

                        <div className="space-y-3 text-xs">
                            <div className="p-3 rounded-xl bg-slate-50 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e] space-y-1">
                                <span className="text-[10px] uppercase font-bold text-slate-400">
                                    Listener State
                                </span>
                                <div className="flex items-center gap-2">
                                    <span className={`w-2 h-2 rounded-full ${isRunning ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'}`} />
                                    <span className="font-semibold text-slate-800 dark:text-slate-200">
                                        {isRunning ? 'Continuous Polling Daemon Running' : 'Idle / Disabled'}
                                    </span>
                                </div>
                            </div>

                            <div className="p-3 rounded-xl bg-slate-50 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e] space-y-1">
                                <span className="text-[10px] uppercase font-bold text-slate-400">
                                    Last Polled Timestamp
                                </span>
                                <p className="font-mono text-slate-700 dark:text-slate-300">
                                    {status?.last_poll_at ? new Date(status.last_poll_at * 1000).toLocaleString() : 'Never'}
                                </p>
                            </div>

                            {status?.last_message_received && (
                                <div className="p-3 rounded-xl bg-slate-50 dark:bg-[#090611] border border-slate-200 dark:border-[#40285e] space-y-1">
                                    <span className="text-[10px] uppercase font-bold text-slate-400">
                                        Last Inbound Command
                                    </span>
                                    <p className="font-mono text-slate-700 dark:text-slate-300 truncate">
                                        {status.last_message_received}
                                    </p>
                                </div>
                            )}

                            {status?.error_message && (
                                <div className="p-3 rounded-xl bg-rose-50 dark:bg-rose-950/30 border border-rose-200 dark:border-rose-900/50 space-y-1 text-rose-700 dark:text-rose-300">
                                    <span className="text-[10px] uppercase font-bold text-rose-500 flex items-center gap-1">
                                        <AlertCircle className="w-3 h-3" />
                                        <span>Last Polling Error</span>
                                    </span>
                                    <p className="font-mono text-[11px] leading-tight break-all">
                                        {status.error_message}
                                    </p>
                                </div>
                            )}
                        </div>

                        {/* Fast CLI Cheatsheet */}
                        <div className="pt-2 border-t border-slate-200 dark:border-[#40285e] space-y-2">
                            <span className="text-[10px] uppercase font-bold text-slate-400 block">
                                Quick Terminal Commands
                            </span>
                            <div className="space-y-1.5 font-mono text-[11px]">
                                {[
                                    'agm telegram status',
                                    'agm telegram listen',
                                    'agm telegram setup',
                                ].map((cmd) => (
                                    <button
                                        key={cmd}
                                        type="button"
                                        onClick={() => handleCopyCli(cmd)}
                                        className="w-full text-left px-2.5 py-1.5 rounded-lg bg-slate-100 dark:bg-[#201538] hover:bg-slate-200 dark:hover:bg-[#2b1c4b] text-slate-800 dark:text-slate-200 flex items-center justify-between cursor-pointer transition-colors"
                                    >
                                        <span className="truncate">{cmd}</span>
                                        {copiedCmd === cmd ? (
                                            <Check className="w-3 h-3 text-emerald-500 shrink-0" />
                                        ) : (
                                            <Copy className="w-3 h-3 text-slate-400 shrink-0" />
                                        )}
                                    </button>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

                {/* Bottom Section: Full Telegram Remote Command Reference */}
                    <div className="agm-hover-shade rounded-2xl border border-slate-200/80 dark:border-[#40285e] bg-white dark:bg-[#171025] p-6 shadow-xs">
                    <TelegramSettings />
                </div>
            </div>

            {/* Setup Guide Modal */}
            <TelegramGuideModal
                isOpen={isGuideOpen}
                onClose={() => setIsGuideOpen(false)}
                botUsername={botUsername}
                isBotRunning={isRunning}
            />
        </div>
    );
}
