import { AutoSwitcherApi } from './useAutoSwitcher';
import { cn } from '../../../utils/cn';

export function PollingSettings(props: AutoSwitcherApi) {
    const { t, currentConfig, daemonStatus, remainingSeconds, handleIntervalChange, handleThresholdChange, onChange } = props;

    return (
        <>
                        <div className="flex flex-wrap items-center justify-between gap-3 p-3 rounded-[5px] bg-slate-50 dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] shadow-2xs">
                            <div className="flex items-center gap-2.5 min-w-0">
                                <span className="relative flex h-2.5 w-2.5 shrink-0">
                                    <span className={cn(
                                        "animate-ping absolute inline-flex h-full w-full rounded-full opacity-75",
                                        daemonStatus?.is_daemon_running ? "bg-emerald-400" : "bg-slate-400"
                                    )} />
                                    <span className={cn(
                                        "relative inline-flex rounded-full h-2.5 w-2.5",
                                        daemonStatus?.is_daemon_running ? "bg-emerald-500" : "bg-slate-400"
                                    )} />
                                </span>
                                <div className="flex items-center gap-2 flex-wrap">
                                    <span className="text-xs font-semibold text-slate-800 dark:text-slate-200">
                                        Next evaluation in {remainingSeconds > 0 ? remainingSeconds : (daemonStatus?.next_check_in_seconds ?? currentConfig.check_interval_seconds)}s
                                    </span>
                                    {(() => {
                                        const stage = daemonStatus?.current_stage || 'Normal';
                                        const isCritical = stage.toLowerCase() === 'critical';
                                        const isCaution = stage.toLowerCase() === 'caution';
                                        return (
                                            <span className={cn(
                                                "px-2 py-0.5 rounded-[5px] text-[10px] font-bold uppercase tracking-wider border",
                                                isCritical
                                                    ? "bg-rose-500/15 text-rose-700 dark:text-rose-400 border-rose-500/30"
                                                    : isCaution
                                                    ? "bg-amber-500/15 text-amber-700 dark:text-amber-400 border-amber-500/30"
                                                    : "bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border-emerald-500/30"
                                            )}>
                                                Stage: {stage}
                                            </span>
                                        );
                                    })()}
                                </div>
                            </div>
                            <div className="flex items-center gap-3 text-xs font-mono text-slate-500 dark:text-slate-400">
                                {daemonStatus?.current_quota_percent !== undefined && (
                                    <span>Quota: <strong className="text-blue-600 dark:text-cyan-400">{daemonStatus.current_quota_percent.toFixed(0)}%</strong></span>
                                )}
                                {daemonStatus?.monitored_instance_count !== undefined && (
                                    <span>Monitored: <strong className="text-slate-700 dark:text-slate-200">{daemonStatus.monitored_instance_count}</strong></span>
                                )}
                            </div>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            {/* Polling Interval */}
                            <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 space-y-2">
                                <div className="flex justify-between items-center text-xs font-semibold text-slate-800 dark:text-slate-200">
                                    <span>{t('settings.auto_switcher.interval_label', 'Check Interval (seconds)')}</span>
                                    <span className="text-blue-600 dark:text-blue-400 font-mono font-bold">
                                        {currentConfig.check_interval_seconds}s
                                    </span>
                                </div>
                                <input
                                    type="range"
                                    min="15"
                                    max="600"
                                    step="15"
                                    value={currentConfig.check_interval_seconds}
                                    onChange={(e) => handleIntervalChange(Number(e.target.value))}
                                    className="w-full accent-blue-600 cursor-pointer"
                                />
                                <div className="flex justify-between text-[10px] text-slate-500 dark:text-slate-400 font-mono">
                                    <span>15s</span>
                                    <span>60s</span>
                                    <span>300s (5m Default)</span>
                                    <span>600s</span>
                                </div>
                            </div>

                            {/* Low Quota Threshold */}
                            <div className="p-3.5 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 space-y-2">
                                <div className="flex justify-between items-center text-xs font-semibold text-slate-800 dark:text-slate-200">
                                    <span>{t('settings.auto_switcher.threshold_label', 'Low Quota Threshold (%)')}</span>
                                    <span className="text-rose-600 dark:text-rose-400 font-mono font-bold">
                                        &lt; {currentConfig.low_quota_threshold_percent.toFixed(0)}%
                                    </span>
                                </div>
                                <input
                                    type="range"
                                    min="1"
                                    max="99"
                                    step="1"
                                    value={currentConfig.low_quota_threshold_percent}
                                    onChange={(e) => handleThresholdChange(Number(e.target.value))}
                                    className="w-full accent-rose-600 cursor-pointer"
                                />
                                <div className="flex justify-between text-[10px] text-slate-500 dark:text-slate-400 font-mono">
                                    <span>1%</span>
                                    <span>15% (Default)</span>
                                    <span>50%</span>
                                    <span>98% (Test)</span>
                                </div>
                                <div className="flex gap-2 pt-1">
                                    <button
                                        type="button"
                                        onClick={() => handleThresholdChange(15)}
                                        className={`px-2.5 py-1 text-[11px] font-medium rounded-lg border transition-all cursor-pointer ${
                                            Math.round(currentConfig.low_quota_threshold_percent) === 15
                                                ? 'bg-rose-50 dark:bg-rose-950/40 text-rose-600 dark:text-rose-400 border-rose-300 dark:border-rose-800 shadow-xs'
                                                : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-slate-300'
                                        }`}
                                    >
                                        🛡️ 15% (Production Standard)
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => handleThresholdChange(98)}
                                        className={`px-2.5 py-1 text-[11px] font-medium rounded-lg border transition-all cursor-pointer ${
                                            Math.round(currentConfig.low_quota_threshold_percent) === 98
                                                ? 'bg-amber-50 dark:bg-amber-950/40 text-amber-600 dark:text-amber-400 border-amber-300 dark:border-amber-800 shadow-xs'
                                                : 'bg-white dark:bg-slate-900 text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-700 hover:border-slate-300'
                                        }`}
                                    >
                                        🧪 98% (Simulation / Testing)
                                    </button>
                                </div>
                                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed pt-0.5">
                                    {Math.round(currentConfig.low_quota_threshold_percent) >= 90
                                        ? '🧪 Simulation mode active: Failover triggers immediately as soon as an account uses any quota, allowing rapid verification of backup, rotation, and prompt resumption.'
                                        : '🛡️ Production standard (15%): Seamlessly rotates accounts before exhaustion without premature switching.'}
                                </p>
                            </div>

                            {/* Caution Polling Interval (< 15% Credits) */}
                            <div className="p-3.5 rounded-xl bg-amber-50/60 dark:bg-amber-950/30 border border-amber-200/60 dark:border-amber-900/40 space-y-2">
                                <div className="flex justify-between items-center text-xs font-semibold text-amber-800 dark:text-amber-300">
                                    <span>Caution Polling Interval (&lt; 15% Credits)</span>
                                    <span className="font-mono font-bold">
                                        {Math.round((currentConfig.caution_interval_seconds || 60) / 60)} min ({currentConfig.caution_interval_seconds || 60}s)
                                    </span>
                                </div>
                                <input
                                    type="range"
                                    min="30"
                                    max="180"
                                    step="15"
                                    value={currentConfig.caution_interval_seconds || 60}
                                    onChange={(e) => onChange({ ...currentConfig, caution_interval_seconds: Number(e.target.value) })}
                                    className="w-full accent-amber-600 cursor-pointer"
                                />
                                <div className="flex justify-between text-[10px] text-amber-600/70 dark:text-amber-400/60 font-mono">
                                    <span>30s</span>
                                    <span>60s (1m Default)</span>
                                    <span>180s</span>
                                </div>
                            </div>

                            {/* Critical Polling Interval (<= 12% Credits) */}
                            <div className="p-3.5 rounded-xl bg-rose-50/60 dark:bg-rose-950/30 border border-rose-200/60 dark:border-rose-900/40 space-y-2">
                                <div className="flex justify-between items-center text-xs font-semibold text-rose-800 dark:text-rose-300">
                                    <span>Critical Polling Interval (&le; 12% Credits)</span>
                                    <span className="font-mono font-bold">
                                        {currentConfig.critical_interval_seconds || 40}s
                                    </span>
                                </div>
                                <input
                                    type="range"
                                    min="10"
                                    max="120"
                                    step="10"
                                    value={currentConfig.critical_interval_seconds || 40}
                                    onChange={(e) => onChange({ ...currentConfig, critical_interval_seconds: Number(e.target.value) })}
                                    className="w-full accent-rose-600 cursor-pointer"
                                />
                                <div className="flex justify-between text-[10px] text-rose-600/70 dark:text-rose-400/60 font-mono">
                                    <span>10s</span>
                                    <span>40s (Default)</span>
                                    <span>120s</span>
                                </div>
                            </div>
                        </div>
        </>
    );
}
