import { EmailNotificationsApi } from './useEmailNotifications';
import { showToast } from '../common/ToastContainer';

export function TelegramSection(props: EmailNotificationsApi) {
    const { accounts, telegramConfig, setTelegramConfig, telegramStatus, isTestingTelegram, isDetectingChatId, isSavingTelegram, isSendingTelegramPing, telegramBotUsername, showBotToken, setShowBotToken, handleSaveTelegram, handleTestTelegram, handleDetectTelegramChatId, handleSendTelegramPing, parsed, target, text } = props;

    return (
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
                    <div>
                        <h3 className="text-sm sm:text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <MessageSquare className="w-4 h-4 text-sky-500" />
                            Telegram Bot & Alert Notifications
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400">
                            Receive real-time push alerts, account switch logs, and remote command executions directly on Telegram.
                        </p>
                    </div>

                    <div className="flex items-center gap-2">
                        {telegramStatus?.is_running ? (
                            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
                                <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                                Active & Polling
                            </span>
                        ) : (
                            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold bg-gray-100 dark:bg-slate-800 text-gray-600 dark:text-slate-400 border border-gray-200 dark:border-slate-700">
                                Inactive
                            </span>
                        )}

                        <button
                            type="button"
                            onClick={() => {
                                const cmd = `agm telegram connect "${telegramConfig?.bot_token || '<BOT_TOKEN>'}"${telegramConfig?.allowed_chat_id ? ` ${telegramConfig.allowed_chat_id}` : ''}`;
                                navigator.clipboard.writeText(cmd);
                                showToast('AGM CLI connect command copied', 'info');
                            }}
                            className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-sky-200 dark:border-sky-900/50 bg-sky-50/50 dark:bg-sky-950/20 text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/40 transition-all flex items-center gap-1.5 shadow-xs cursor-pointer"
                            title="Copy AGM CLI one-command connect string"
                        >
                            <Terminal className="w-3.5 h-3.5" />
                            <span>Copy CLI Connect</span>
                        </button>

                        <button
                            type="button"
                            onClick={() => {
                                const cmd = `.\\03-ai-scripts\\telegram-bot-helper.ps1 -BotToken "${telegramConfig?.bot_token || 'TOKEN'}" -ChatId "${telegramConfig?.allowed_chat_id || 'CHAT_ID'}"`;
                                navigator.clipboard.writeText(cmd);
                                showToast('PowerShell verification command copied', 'info');
                            }}
                            className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-sky-200 dark:border-sky-900/50 bg-sky-50/50 dark:bg-sky-950/20 text-sky-700 dark:text-sky-300 hover:bg-sky-100 dark:hover:bg-sky-900/40 transition-all flex items-center gap-1.5 shadow-xs cursor-pointer"
                            title="Copy PowerShell step-by-step helper command"
                        >
                            <Copy className="w-3.5 h-3.5" />
                            <span>Copy PS Helper Script</span>
                        </button>
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
                                    {isDetectingChatId ? <Loader2 className="w-3 h-3 animate-spin" /> : <Sparkles className="w-3 h-3" />}
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
                                    Enable Telegram Bot Notifications & Inbound Daemon
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
                                    setTelegramConfig((prev) =>
                                        prev ? { ...prev, poll_interval_secs: val } : null
                                    );
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

                    {/* Remote Bot & CLI Command Reference */}
                    <div className="p-3 rounded-lg bg-sky-50/40 dark:bg-sky-950/20 border border-sky-100 dark:border-sky-900/40 text-[11px] text-gray-600 dark:text-slate-300 space-y-1.5">
                        <div className="font-semibold text-sky-800 dark:text-sky-300 flex items-center gap-1.5">
                            <Terminal className="w-3.5 h-3.5" />
                            <span>Supported Telegram Chat Commands &amp; AGM / GitMap Terminal CLI:</span>
                        </div>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-1.5 font-mono text-[10px]">
                            <div><code>/ping</code> — Node IP, version &amp; git telemetry</div>
                            <div><code>/tree [all]</code> — Dual <code>[AGM:P001 | GM:#1]</code> &amp; <code>[AGM:C001 | GM:&lt;cid&gt;]</code> 200w tree</div>
                            <div><code>/prompt C001 &lt;msg&gt;</code> — Inject prompt by AGM or GitMap (<code>GM:#1</code>) seq ID</div>
                            <div><code>/prompt C001 --instance #2 --node vm-1 &lt;msg&gt;</code> — Scoped to instance &amp; SSH node</div>
                            <div><code>/agy [active|running-prompts|fpug|sug|rerun]</code> — GitMap AGY suite</div>
                            <div><code>/gitmap agy prompt -n is-done -t &quot;...&quot;</code> — GitMap named template prompt</div>
                            <div><code>/observe</code> or <code>/status</code> — Active vs Idle workspace report</div>
                            <div><code>/ssh [nodes|exec|prompt|update]</code> — Remote SSH fleet delegation</div>
                            <div><code>/update [agm|gitmap|ssh|all]</code> — Self-update AGM &amp; GitMap</div>
                            <div><code>/backup</code> &amp; <code>/restore</code> — Snapshot &amp; restore running storage prompts</div>
                            <div><code>agm tree [--all]</code> — Terminal bracketed project &amp; conversation tree</div>
                            <div><code>agm prompt C001 &quot;&lt;text&gt;&quot; [--instance #2] [--node vm-1]</code> — CLI prompt inject</div>
                            <div><code>agm instances assign #2 &lt;repo_path&gt;</code> — Bind project workspace to instance</div>
                            <div><code>agm agy [tree|active|backup|restore|fpug|sug]</code> — GitMap AGY parity</div>
                        </div>
                    </div>

                    {/* Action Buttons */}
                    <div className="flex flex-wrap items-center justify-between gap-2 pt-1 border-t border-gray-100 dark:border-slate-800">
                        <div className="text-[11px] text-gray-500 dark:text-slate-400">
                            Send <code className="font-semibold text-sky-600 dark:text-sky-400">/ping</code> to your bot in Telegram, then click <span className="font-semibold text-sky-600 dark:text-sky-400">Auto-Detect Chat ID</span>.
                        </div>

                        <div className="flex flex-wrap items-center gap-2">
                            <button
                                type="button"
                                onClick={handleTestTelegram}
                                disabled={isTestingTelegram}
                                className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isTestingTelegram ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <RefreshCw className="w-3.5 h-3.5" />}
                                Test Bot Token
                            </button>
                            <button
                                type="button"
                                onClick={handleDetectTelegramChatId}
                                disabled={isDetectingChatId}
                                className="px-3 py-1.5 text-xs font-medium border border-sky-200 dark:border-sky-800 bg-sky-50/60 dark:bg-sky-950/30 hover:bg-sky-100 dark:hover:bg-sky-900/50 text-sky-700 dark:text-sky-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isDetectingChatId ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Sparkles className="w-3.5 h-3.5" />}
                                Auto-Detect Chat ID
                            </button>
                            <button
                                type="button"
                                onClick={handleSendTelegramPing}
                                disabled={isSendingTelegramPing}
                                className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isSendingTelegramPing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Send className="w-3.5 h-3.5" />}
                                Send Test Alert
                            </button>
                            <button
                                type="button"
                                onClick={handleSaveTelegram}
                                disabled={isSavingTelegram}
                                className="px-4 py-1.5 bg-sky-600 hover:bg-sky-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isSavingTelegram ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <CheckCircle2 className="w-3.5 h-3.5" />}
                                Save Telegram Settings
                            </button>
                        </div>
                    </div>
                </div>
            </div>

    );
}
