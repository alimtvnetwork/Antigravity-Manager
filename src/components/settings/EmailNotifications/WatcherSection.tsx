import { EmailNotificationsApi } from './useEmailNotifications';
import { Loader2, Send, Sparkles, Zap } from 'lucide-react';

export function WatcherSection(props: EmailNotificationsApi) {
    const { settings, setSettings, isSaving, isPinging, handleSaveSettings, handleTriggerManualCheck, handleDispatchPingTest } = props;

    return (
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800 space-y-4">
                <div className="flex items-center justify-between">
                    <div>
                        <h3 className="text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Sparkles className="w-4 h-4 text-purple-500" />
                            Background Watcher Daemon & Multi-Trigger Sensors
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400">
                            Configure automatic quota monitoring, idle workspace detection, and remote mailbox commands.
                        </p>
                    </div>

                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            checked={settings.is_enabled}
                            onChange={(e) => setSettings({ ...settings, is_enabled: e.target.checked })}
                            className="sr-only peer"
                        />
                        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-blue-600"></div>
                    </label>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-slate-200 block">
                            Baseline Polling Interval (Idle Cadence)
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={5}
                                max={10}
                                value={settings.baseline_polling_interval_minutes || 5}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        baseline_polling_interval_minutes: parseInt(e.target.value) || 5,
                                    })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.baseline_polling_interval_minutes || 5} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            Standard background frequency for monitoring incoming email instructions.
                        </p>
                    </div>

                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-slate-200 block">
                            Active Awaiting Interval (Fast Adaptive)
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={5}
                                max={10}
                                value={settings.active_awaiting_interval_seconds || 10}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        active_awaiting_interval_seconds: parseInt(e.target.value) || 10,
                                    })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.active_awaiting_interval_seconds || 10} sec
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            High-speed 5–10s polling interval when awaiting reply after outbound dispatch.
                        </p>
                    </div>

                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-slate-200 block">
                            Telemetry & Sensor Loop Interval
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={1}
                                max={5}
                                value={settings.polling_interval_minutes}
                                onChange={(e) =>
                                    setSettings({ ...settings, polling_interval_minutes: parseInt(e.target.value) || 3 })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.polling_interval_minutes} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            Interval for telemetry collection and quota/idle workspace sensor sampling.
                        </p>
                    </div>

                    <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-3">
                        <label className="text-xs font-semibold text-gray-700 dark:text-slate-200 block">
                            Inbound Mailbox Check Interval
                        </label>
                        <div className="flex items-center gap-3">
                            <input
                                type="range"
                                min={1}
                                max={5}
                                value={settings.inbox_check_interval_minutes}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        inbox_check_interval_minutes: parseInt(e.target.value) || 1,
                                    })
                                }
                                className="flex-1 accent-blue-600"
                            />
                            <span className="text-xs font-bold text-blue-600 dark:text-blue-400 w-16 text-right">
                                {settings.inbox_check_interval_minutes} min
                            </span>
                        </div>
                        <p className="text-[11px] text-gray-400 dark:text-slate-400">
                            Frequency of reading the last 5 unread instructions via IMAP.
                        </p>
                    </div>
                </div>

                {/* Sensor Toggles */}
                <div className="space-y-3 border-t border-gray-100 dark:border-slate-800 pt-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-gray-400 dark:text-slate-400">Sensor Notification Triggers</h4>

                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_quota_drop"
                                checked={settings.notify_on_quota_drop}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_quota_drop: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_quota_drop" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Quota Drop Alert</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Trigger when credit &lt; 15%</div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_ws_switch"
                                checked={settings.notify_on_workspace_switch}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_workspace_switch: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_ws_switch" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Workspace Switch Notice</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Email notice before auto-rotation</div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_idle_ws"
                                checked={settings.notify_on_idle_workspace}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_idle_workspace: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_idle_ws" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Idle Workspace Alert</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Ask for prompt when queue is empty</div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_system_update"
                                checked={settings.notify_on_system_update ?? true}
                                onChange={(e) =>
                                    setSettings({ ...settings, notify_on_system_update: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_system_update" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">System Update Notice</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Receipt when application updates</div>
                            </label>
                        </div>
                    </div>
                </div>

                {/* Remote Execution Toggles */}
                <div className="space-y-3 border-t border-gray-100 dark:border-slate-800 pt-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-gray-400 dark:text-slate-400">Inbound Mailbox Command Capabilities</h4>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_allow_prompt"
                                checked={settings.allow_remote_prompt_execution}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_prompt_execution: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_prompt" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Prompt Injection</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Match <code>Project: &lt;name&gt;</code></div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_allow_cli"
                                checked={settings.allow_remote_cli_execution}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_cli_execution: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_cli" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">CLI Execution</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Match <code>exec: &lt;ip&gt;</code></div>
                            </label>
                        </div>

                        <div className="p-3 rounded-lg border border-gray-200 dark:border-slate-700 flex items-start gap-2.5 bg-white/50 dark:bg-slate-800/40">
                            <input
                                type="checkbox"
                                id="chk_allow_instance"
                                checked={settings.allow_remote_instance_rotation}
                                onChange={(e) =>
                                    setSettings({ ...settings, allow_remote_instance_rotation: e.target.checked })
                                }
                                className="mt-0.5 rounded text-blue-600"
                            />
                            <label htmlFor="chk_allow_instance" className="text-xs text-gray-700 dark:text-slate-200 cursor-pointer">
                                <div className="font-semibold">Instance Launch / Rotate</div>
                                <div className="text-[11px] text-gray-400 dark:text-slate-400">Match <code>instance: new</code></div>
                            </label>
                        </div>
                    </div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-3 pt-4 border-t border-gray-100 dark:border-slate-800">
                    <div className="flex flex-wrap items-center gap-2">
                        <button
                            type="button"
                            onClick={handleTriggerManualCheck}
                            className="px-4 py-2 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-200 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer"
                        >
                            <Send className="w-3.5 h-3.5 text-blue-500" />
                            Dispatch Test Cheat Sheet
                        </button>
                        <button
                            type="button"
                            onClick={handleDispatchPingTest}
                            disabled={isPinging}
                            className="px-4 py-2 text-xs font-medium border border-blue-200 dark:border-blue-900/60 bg-blue-50/50 dark:bg-blue-950/30 hover:bg-blue-100/50 dark:hover:bg-blue-900/50 text-blue-600 dark:text-blue-400 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                        >
                            {isPinging ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Zap className="w-3.5 h-3.5 text-amber-500" />
                            )}
                            Dispatch Ping Command Test
                        </button>
                    </div>

                    <button
                        type="button"
                        onClick={handleSaveSettings}
                        disabled={isSaving}
                        className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm cursor-pointer disabled:opacity-60"
                    >
                        {isSaving ? 'Saving Settings...' : 'Save Watcher Settings'}
                    </button>
                </div>
            </div>

    );
}
