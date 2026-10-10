import { EmailNotificationsApi } from './useEmailNotifications';
import { showToast } from '../../common/ToastContainer';
import { AlertCircle, CheckCircle2, Copy, Loader2, Play, Sparkles, Terminal } from 'lucide-react';

export function RemoteCommandSection(props: EmailNotificationsApi) {
    const { setIsSampleTemplatesOpen, testCliCommand, setTestCliCommand, isExecutingCli, cliExecResult, setCliExecResult, handleTestExecuteCli } = props;

    return (
            <div className="bg-white dark:bg-slate-900 rounded-xl p-4 sm:p-5 shadow-sm border border-gray-100 dark:border-slate-800 space-y-4">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                    <div>
                        <h3 className="text-base font-semibold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                            <Terminal className="w-4 h-4 text-purple-500" />
                            Interactive Remote Command &amp; Prompt Testing
                        </h3>
                        <p className="text-xs text-gray-500 dark:text-slate-400 mt-0.5">
                            Simulate and execute CLI instructions, GitMap scans, or AI prompts locally on this machine node before sending via email.
                        </p>
                    </div>
                    <button
                        type="button"
                        onClick={() => setIsSampleTemplatesOpen(true)}
                        className="px-3 py-1.5 text-xs font-medium border border-purple-200 dark:border-purple-900/60 bg-purple-50/50 dark:bg-purple-950/30 hover:bg-purple-100/50 dark:hover:bg-purple-900/50 text-purple-600 dark:text-purple-400 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer self-start sm:self-auto"
                    >
                        <Sparkles className="w-3.5 h-3.5 text-purple-500" />
                        Command Cheat Sheets
                    </button>
                </div>

                {/* Quick Templates */}
                <div className="flex flex-wrap items-center gap-1.5">
                    <span className="text-[11px] font-semibold text-gray-400 dark:text-slate-500 mr-1">Templates:</span>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('Get-Process | Select-Object -First 5')}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                        title="PowerShell process list (auto-detected on Windows)"
                    >
                        PS Processes
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand("Get-Service -Name '*wsl*', '*docker*' -ErrorAction SilentlyContinue")}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                        title="PowerShell service inspection"
                    >
                        PS Services
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('gitmap --version')}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                        title="Query gitmap version"
                    >
                        gitmap Version
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('gitmap status')}
                        className="px-2 py-1 text-[11px] rounded-md border border-gray-200 dark:border-slate-700 bg-gray-50 dark:bg-slate-800 hover:bg-gray-100 dark:hover:bg-slate-750 text-gray-700 dark:text-slate-300 font-mono transition-colors cursor-pointer"
                        title="Query gitmap status"
                    >
                        gitmap Status
                    </button>
                    <button
                        type="button"
                        onClick={() => setTestCliCommand('Project: Antigravity-Manager\nScan repository health and report open issues')}
                        className="px-2 py-1 text-[11px] rounded-md border border-purple-200 dark:border-purple-900/60 bg-purple-50/50 dark:bg-purple-950/30 hover:bg-purple-100 dark:hover:bg-purple-900/50 text-purple-700 dark:text-purple-300 font-mono transition-colors cursor-pointer"
                        title="AI instruction simulation"
                    >
                        AI Prompt Instruction
                    </button>
                </div>

                {/* Command Input Area */}
                <div className="space-y-2">
                    <div className="relative">
                        <textarea
                            rows={3}
                            value={testCliCommand}
                            onChange={(e) => setTestCliCommand(e.target.value)}
                            placeholder="Enter CLI command (e.g. ps: Get-Process or Get-Process) or AI prompt instruction (e.g. Project: Antigravity-Manager)..."
                            className="w-full px-3 py-2.5 font-mono text-xs border border-gray-200 dark:border-slate-700 rounded-lg bg-gray-900 text-gray-100 placeholder:text-gray-500 focus:outline-none focus:ring-2 focus:ring-purple-500 leading-relaxed resize-y"
                        />
                    </div>
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <div className="text-[11px] text-gray-500 dark:text-slate-400">
                            Optional prefix: <code className="text-purple-600 dark:text-purple-400 font-bold">ps:</code>, <code className="text-purple-600 dark:text-purple-400 font-bold">bash:</code>, or execute directly (defaults to PowerShell on Windows). AI prompts starting with <code className="text-purple-600 dark:text-purple-400 font-bold">Project:</code> or <code className="text-purple-600 dark:text-purple-400 font-bold">Prompt:</code> simulate execution receipts.
                        </div>
                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={() => {
                                    navigator.clipboard.writeText(testCliCommand);
                                    showToast('Command copied to clipboard', 'info');
                                }}
                                className="px-3 py-1.5 text-xs font-medium border border-gray-200 dark:border-slate-700 hover:bg-gray-50 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 rounded-lg transition-colors flex items-center gap-1.5 cursor-pointer"
                            >
                                <Copy className="w-3.5 h-3.5" />
                                Copy
                            </button>
                            <button
                                type="button"
                                onClick={handleTestExecuteCli}
                                disabled={isExecutingCli}
                                className="px-4 py-1.5 bg-purple-600 hover:bg-purple-700 text-white text-xs font-semibold rounded-lg transition-colors shadow-sm flex items-center gap-1.5 cursor-pointer disabled:opacity-60"
                            >
                                {isExecutingCli ? (
                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                ) : (
                                    <Play className="w-3.5 h-3.5 fill-current" />
                                )}
                                {isExecutingCli ? 'Executing...' : 'Run Local Test'}
                            </button>
                        </div>
                    </div>
                </div>

                {/* Execution Output Console */}
                {cliExecResult ? (
                    <div className="mt-3 p-3.5 rounded-xl bg-gray-950 border border-gray-800 text-gray-200 space-y-2">
                        <div className="flex flex-wrap items-center justify-between gap-2 text-xs border-b border-gray-800 pb-2">
                            <div className="flex items-center gap-2">
                                <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider ${
                                    cliExecResult.success ? 'bg-emerald-900/60 text-emerald-300 border border-emerald-700/60' : 'bg-rose-900/60 text-rose-300 border border-rose-700/60'
                                }`}>
                                    {cliExecResult.success ? <CheckCircle2 className="w-3 h-3" /> : <AlertCircle className="w-3 h-3" />}
                                    Exit {cliExecResult.exit_code}
                                </span>
                                <span className="font-mono text-[11px] text-gray-400">
                                    Node: <span className="text-gray-200 font-semibold">{cliExecResult.machine_name}</span> ({cliExecResult.machine_ip})
                                </span>
                            </div>
                            <button
                                type="button"
                                onClick={() => setCliExecResult(null)}
                                className="text-[11px] text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
                            >
                                Clear Console
                            </button>
                        </div>

                        {cliExecResult.stdout.length > 0 ? (
                            <div>
                                <div className="text-[10px] font-bold text-gray-400 uppercase tracking-wider mb-1">Standard Output:</div>
                                <pre className="p-2.5 bg-black/50 rounded-lg font-mono text-[11px] text-emerald-400 overflow-x-auto max-h-48 whitespace-pre-wrap leading-relaxed">
                                    {cliExecResult.stdout}
                                </pre>
                            </div>
                        ) : null}

                        {cliExecResult.stderr.length > 0 ? (
                            <div>
                                <div className="text-[10px] font-bold text-rose-400 uppercase tracking-wider mb-1">Standard Error:</div>
                                <pre className="p-2.5 bg-black/50 rounded-lg font-mono text-[11px] text-rose-400 overflow-x-auto max-h-36 whitespace-pre-wrap leading-relaxed">
                                    {cliExecResult.stderr}
                                </pre>
                            </div>
                        ) : null}

                        {cliExecResult.stdout === '' && cliExecResult.stderr === '' ? (
                            <div className="text-[11px] text-gray-400 italic py-1">
                                Command executed with empty output stream.
                            </div>
                        ) : null}
                    </div>
                ) : null}
            </div>

    );
}
