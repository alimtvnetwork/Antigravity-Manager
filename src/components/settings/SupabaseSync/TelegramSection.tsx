import { SupabaseSyncApi } from './useSupabaseSync';
import { useErrorStore } from '../../stores/error-store';

export function TelegramSection(props: SupabaseSyncApi) {
    const { telegramConfig, setTelegramConfig, telegramStatus, isTestingTelegram, telegramBotUsername, isSavingTelegram, isSendingPing, setIsTgGuideOpen, handleSaveTelegram, handleTestTelegram, updated, handleDetectTelegramChatId, handleSendTelegramPing, text } = props;

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
                            Query cluster snapshots ("How many machines are running?"), fast-forward workspaces, and execute terminal commands from your phone via Telegram.
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
                                    telegramService.saveConfig(updated).catch((e) => {
                                        // Tracked in the error module; toggle already applied locally, persisted on next save.
                                        useErrorStore.getState().trackWarning(e, {
                                          source: 'SupabaseSyncSettings.telegramToggle',
                                          triggerAction: 'save_telegram_config',
                                        });
                                        console.error(e);
                                    });
                                }}
                                className="sr-only peer"
                            />
                            <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-sky-600"></div>
                        </label>
                    )}
                </div>

                {telegramConfig && (
                    <div className="space-y-3 pt-2 border-t border-slate-800/80">
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
                                        telegramService.saveConfig(updated).catch((e) => {
                                            // Tracked in the error module; toggle already applied locally, persisted on next save.
                                            useErrorStore.getState().trackWarning(e, {
                                              source: 'SupabaseSyncSettings.telegramToggle',
                                              triggerAction: 'save_telegram_config',
                                            });
                                            console.error(e);
                                        });
                                    }}
                                    className="w-3.5 h-3.5 rounded text-sky-600 focus:ring-sky-500 border-slate-700 bg-slate-950"
                                />
                                <span className="text-xs text-gray-300">
                                    Notify via Telegram on System Update (default enabled)
                                </span>
                            </label>
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
                                    className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs flex items-center gap-1.5 transition"
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
                                    className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs flex items-center gap-1.5 transition disabled:opacity-50"
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
                                    className="px-3 py-1.5 rounded-lg bg-sky-600 hover:bg-sky-500 text-white text-xs font-medium transition"
                                >
                                    Save Telegram
                                </button>
                            </div>
                        </div>

                        <div className="p-3 rounded-lg bg-slate-950/60 border border-slate-800/80 text-xs text-gray-400 space-y-1">
                            <span className="font-semibold text-gray-300">Supported Telegram Commands &amp; CLI:</span>
                            <div className="font-mono text-[11px] text-gray-400 space-y-0.5">
                                <p>• <code>/ping</code>, <code>/status</code>, <code>/observe</code> — Node connectivity, live workspaces &amp; prompt queues</p>
                                <p>• <code>/gitmap pe</code>, <code>/agm status</code>, <code>/api</code> — Run GitMap, AGM CLI, or API proxy status</p>
                                <p>• <code>/backup</code> (<code>/backpack</code>), <code>/restore</code> — Snapshot or restore running prompts in split SQLite DB</p>
                                <p>• <code>/email status</code>, <code>/email ping</code>, <code>/email help</code> — Check or dispatch email telemetry</p>
                                <p>• <code>/ff</code>, <code>/snapshot</code>, <code>CMD:&lt;node&gt;:&lt;cmd&gt;</code> — Fast-forward switch, cluster snapshot &amp; remote shell</p>
                                <p>• Terminal CLI: <code>agm telegram connect &lt;token&gt;</code> | <code>agm telegram observe</code> | <code>agm telegram poll</code></p>
                            </div>
                        </div>
                    </div>
                )}
            </div>

    );
}
