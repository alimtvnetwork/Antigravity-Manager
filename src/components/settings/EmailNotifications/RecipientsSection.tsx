import { EmailNotificationsApi } from './useEmailNotifications';

export function RecipientsSection(props: EmailNotificationsApi) {
    const { recipients, settings, setSettings, developerTaskType, developerTargetRecipient, setDeveloperTargetRecipient, developerTaskPayload, setDeveloperTaskPayload, developerCustomSubject, setDeveloperCustomSubject, isDispatchingTask, newRecipientEmail, setNewRecipientEmail, newRecipientGroup, setNewRecipientGroup, domain, handleAddRecipient, handleDeleteRecipient, target, text, handleTaskTypeChange, handleDispatchDeveloperTask, handleSaveRecipientsIntervals } = props;

    return (
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800 space-y-6">
                <div>
                    <div className="flex items-center justify-between mb-3">
                        <h3 className="text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Send className="w-4 h-4 text-emerald-500" />
                            Notification Recipients & Worker Contacts
                        </h3>
                        <span className="text-xs text-gray-400 dark:text-slate-500 font-mono">
                            {recipients.length} registered
                        </span>
                    </div>

                    <form
                        onSubmit={(e) => {
                            e.preventDefault();
                            handleAddRecipient();
                        }}
                        className="flex flex-wrap items-center gap-2 mb-3"
                    >
                        <input
                            type="email"
                            placeholder="Recipient Email (e.g. user@domain.com)"
                            value={newRecipientEmail}
                            onChange={(e) => setNewRecipientEmail(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    handleAddRecipient();
                                }
                            }}
                            className="w-64 max-w-xs px-3 py-1.5 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                        <input
                            type="text"
                            placeholder="Group (e.g. default)"
                            value={newRecipientGroup}
                            onChange={(e) => setNewRecipientGroup(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    handleAddRecipient();
                                }
                            }}
                            className="w-28 px-2.5 py-1.5 text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                        <button
                            type="submit"
                            className="px-3.5 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-medium transition-colors shadow-sm cursor-pointer shrink-0"
                        >
                            Add Recipient
                        </button>
                    </form>

                    <div className="flex flex-wrap gap-2">
                        {recipients.map((rec) => (
                            <div
                                key={rec.id}
                                className="bg-gray-50 dark:bg-slate-800/80 border border-gray-200 dark:border-slate-700 rounded-lg px-3 py-1.5 flex items-center gap-2 text-xs"
                            >
                                <span className="font-medium text-gray-700 dark:text-slate-200">{rec.email}</span>
                                <span className="text-[10px] px-1.5 py-0.5 bg-gray-200 dark:bg-slate-700 text-gray-600 dark:text-slate-300 rounded font-mono">
                                    {rec.group_name}
                                </span>
                                <button
                                    onClick={() => handleDeleteRecipient(rec.id)}
                                    className="text-gray-400 hover:text-red-500 cursor-pointer"
                                >
                                    <Trash2 className="w-3 h-3" />
                                </button>
                            </div>
                        ))}
                        {recipients.length === 0 && (
                            <span className="text-xs text-gray-400 dark:text-slate-500 italic">No recipients registered. Alerts will be skipped.</span>
                        )}
                    </div>
                </div>

                {/* Developer Task Quick Dispatch */}
                <div className="pt-4 border-t border-gray-100 dark:border-slate-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <Terminal className="w-5 h-5 text-blue-500" />
                            <h4 className="text-sm sm:text-base font-bold uppercase tracking-wider text-gray-800 dark:text-slate-200">
                                Developer Task Quick Dispatch
                            </h4>
                        </div>
                        <span className="text-xs px-2.5 py-1 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-full font-semibold border border-blue-200 dark:border-blue-800/50">
                            Auto Receipt Active
                        </span>
                    </div>
                    <p className="text-sm text-gray-600 dark:text-slate-300 mb-4">
                        Dispatch a structured remote task to worker email addresses with instantaneous receipt acknowledgements.
                    </p>

                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-3">
                        <div>
                            <label className="text-sm font-semibold text-gray-700 dark:text-slate-300 mb-1.5 block">
                                Task Type
                            </label>
                            <select
                                value={developerTaskType}
                                onChange={(e) => handleTaskTypeChange(e.target.value)}
                                className="w-full px-3.5 py-2.5 text-sm border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                            >
                                <option value="help">help: - System Help &amp; Command Manual</option>
                                <option value="prompt">prompt: - AI Prompt Injection</option>
                                <option value="powershell">powershell: - Windows PowerShell Execution</option>
                                <option value="cmd">cmd: - Windows Command Prompt Execution</option>
                                <option value="gitmap">gitmap: - GitMap Autonomous CLI</option>
                                <option value="status">status: - Telemetry &amp; Quota Status Query</option>
                            </select>
                        </div>
                        <div>
                            <label className="text-sm font-semibold text-gray-700 dark:text-slate-300 mb-1.5 block">
                                Target Recipient
                            </label>
                            <select
                                value={developerTargetRecipient}
                                onChange={(e) => setDeveloperTargetRecipient(e.target.value)}
                                className="w-full px-3.5 py-2.5 text-sm border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            >
                                <option value="">All Registered Recipients (Broadcast)</option>
                                {recipients.map((rec) => (
                                    <option key={rec.id} value={rec.email}>
                                        {rec.email} ({rec.group_name})
                                    </option>
                                ))}
                            </select>
                        </div>
                    </div>

                    <div className="mb-3">
                        <label className="text-sm font-semibold text-gray-700 dark:text-slate-300 mb-1.5 block">
                            Custom Subject (Optional)
                        </label>
                        <input
                            type="text"
                            placeholder="e.g. * | help  or  [Node-1][192.168.1.100] prompt: Review PR"
                            value={developerCustomSubject}
                            onChange={(e) => setDeveloperCustomSubject(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault();
                                    handleDispatchDeveloperTask();
                                }
                            }}
                            className="w-full px-3.5 py-2.5 text-sm border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 placeholder:text-gray-400 dark:placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
                        />
                    </div>

                    <div className="mb-3">
                        <label className="text-sm font-semibold text-gray-700 dark:text-slate-300 mb-1.5 block">
                            Command / Prompt Payload (Press Ctrl+Enter to dispatch)
                        </label>
                        <textarea
                            rows={8}
                            value={developerTaskPayload}
                            onChange={(e) => setDeveloperTaskPayload(e.target.value)}
                            onKeyDown={(e) => {
                                if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                                    e.preventDefault();
                                    handleDispatchDeveloperTask();
                                }
                            }}
                            className="w-full px-3.5 py-2.5 text-sm border border-gray-200 dark:border-slate-700 rounded-lg bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono leading-relaxed"
                            placeholder="Enter command or AI prompt payload... (Ctrl+Enter to dispatch)"
                        />
                    </div>

                    <div className="flex justify-end">
                        <button
                            onClick={handleDispatchDeveloperTask}
                            disabled={isDispatchingTask}
                            className="px-5 py-2.5 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-bold transition-colors shadow-sm cursor-pointer flex items-center gap-2 disabled:opacity-50"
                        >
                            {isDispatchingTask ? (
                                <>
                                    <Loader2 className="w-4 h-4 animate-spin" />
                                    <span>Dispatching...</span>
                                </>
                            ) : (
                                <>
                                    <Play className="w-4 h-4" />
                                    <span>Dispatch Task</span>
                                </>
                            )}
                        </button>
                    </div>
                </div>

                {/* Adaptive Monitoring & Retry Cadence */}
                <div className="pt-4 border-t border-gray-100 dark:border-slate-800">
                    <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                            <SlidersHorizontal className="w-4 h-4 text-purple-500" />
                            <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-700 dark:text-slate-300">
                                Adaptive Monitoring & Retry Intervals
                            </h4>
                        </div>
                        <button
                            onClick={handleSaveRecipientsIntervals}
                            className="px-3 py-1 bg-purple-600 hover:bg-purple-700 text-white rounded-md text-[11px] font-medium transition-colors cursor-pointer shadow-sm"
                        >
                            Save Monitoring Intervals
                        </button>
                    </div>
                    <p className="text-xs text-gray-500 dark:text-slate-400 mb-3">
                        Configure initial mailbox wait cadence and fast-polling responsiveness for interactive remote replies.
                    </p>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="p-3.5 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-2">
                            <div className="flex justify-between items-center">
                                <label className="text-xs font-semibold text-gray-700 dark:text-slate-200">
                                    Initial Wait Cadence
                                </label>
                                <span className="text-xs font-bold text-purple-600 dark:text-purple-400">
                                    {settings.baseline_polling_interval_minutes || 5} min
                                </span>
                            </div>
                            <input
                                type="range"
                                min={5}
                                max={10}
                                step={1}
                                value={settings.baseline_polling_interval_minutes || 5}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        baseline_polling_interval_minutes: parseInt(e.target.value) || 5,
                                    })
                                }
                                className="w-full accent-purple-600"
                            />
                            <p className="text-[11px] text-gray-400 dark:text-slate-400">
                                Initial delay (5–10 min) before checking mailboxes for external instructions.
                            </p>
                        </div>

                        <div className="p-3.5 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-100 dark:border-slate-700 space-y-2">
                            <div className="flex justify-between items-center">
                                <label className="text-xs font-semibold text-gray-700 dark:text-slate-200">
                                    Fast Reply Quick-Poll
                                </label>
                                <span className="text-xs font-bold text-purple-600 dark:text-purple-400">
                                    {settings.active_awaiting_interval_seconds || 10} sec
                                </span>
                            </div>
                            <input
                                type="range"
                                min={5}
                                max={10}
                                step={1}
                                value={settings.active_awaiting_interval_seconds || 10}
                                onChange={(e) =>
                                    setSettings({
                                        ...settings,
                                        active_awaiting_interval_seconds: parseInt(e.target.value) || 10,
                                    })
                                }
                                className="w-full accent-purple-600"
                            />
                            <p className="text-[11px] text-gray-400 dark:text-slate-400">
                                High-speed polling (5–10 sec) during active command dispatch and receipt awaiting.
                            </p>
                        </div>
                    </div>
                </div>
            </div>

    );
}
