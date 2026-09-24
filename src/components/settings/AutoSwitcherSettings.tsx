import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
    RotateCw,
    Play,
    CheckCircle2,
    AlertCircle,
    Sparkles,
    Clock,
    Gauge,
    Award,
    History,
    ShieldCheck,
    Download,
    Upload,
    SlidersHorizontal,
    ChevronDown,
    RotateCcw,
    Cpu,
    ArrowRightLeft,
} from 'lucide-react';
import { showToast } from '../common/ToastContainer';
import { AutoProfileSwitcherConfig } from '../../types/config';
import { useInstanceStore } from '../../stores/useInstanceStore';

interface AutoSwitcherSettingsProps {
    config?: AutoProfileSwitcherConfig;
    onChange: (config: AutoProfileSwitcherConfig) => void;
}

const DEFAULT_CONFIG: AutoProfileSwitcherConfig = {
    is_enabled: true,
    check_interval_seconds: 60,
    low_quota_threshold_percent: 10.0,
    target_model: 'gemini-pro',
    has_auto_resume: true,
    cooldown_seconds: 180,
    auto_resume_recent_prompts: true,
    auto_focus_window: true,
    watchdog_interval_seconds: 120,
    prompt_recency_threshold_seconds: 3600,
};

export const AutoSwitcherSettings: React.FC<AutoSwitcherSettingsProps> = ({ config, onChange }) => {
    const { t } = useTranslation();
    const currentConfig = config || DEFAULT_CONFIG;
    const { smartRotateProfileAccount, activeInstanceId } = useInstanceStore();
    const [rotationFeedback, setRotationFeedback] = useState<string | null>(null);
    const [isRotating, setIsRotating] = useState(false);
    const [isActionsOpen, setIsActionsOpen] = useState(false);
    const actionsRef = React.useRef<HTMLDivElement>(null);
    const fileInputRef = React.useRef<HTMLInputElement>(null);

    React.useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (actionsRef.current && !actionsRef.current.contains(e.target as Node)) {
                setIsActionsOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const handleExport = () => {
        setIsActionsOpen(false);
        const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(currentConfig, null, 2));
        const downloadAnchor = document.createElement('a');
        downloadAnchor.setAttribute("href", dataStr);
        downloadAnchor.setAttribute("download", `auto_switcher_config_${Date.now()}.json`);
        document.body.appendChild(downloadAnchor);
        downloadAnchor.click();
        downloadAnchor.remove();
        showToast('Auto-Switcher configuration exported', 'success');
    };

    const handleImportFile = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = (event) => {
            try {
                const parsed = JSON.parse(event.target?.result as string);
                if (typeof parsed === 'object' && parsed !== null) {
                    onChange({ ...DEFAULT_CONFIG, ...parsed });
                    showToast('Auto-Switcher configuration imported successfully', 'success');
                } else {
                    showToast('Invalid configuration file format', 'error');
                }
            } catch (err: any) {
                showToast(`Failed to parse JSON: ${err?.message || err}`, 'error');
            }
        };
        reader.readAsText(file);
        e.target.value = '';
    };

    const handleResetDefaults = () => {
        setIsActionsOpen(false);
        onChange({ ...DEFAULT_CONFIG });
        showToast('Auto-Switcher configuration reset to defaults', 'info');
    };

    const handleToggleEnabled = (is_enabled: boolean) => {
        onChange({ ...currentConfig, is_enabled });
    };

    const handleIntervalChange = (val: number) => {
        const check_interval_seconds = Math.max(15, Math.min(600, val));
        onChange({ ...currentConfig, check_interval_seconds });
    };

    const handleThresholdChange = (val: number) => {
        const low_quota_threshold_percent = Math.max(1.0, Math.min(50.0, val));
        onChange({ ...currentConfig, low_quota_threshold_percent });
    };

    const handleManualRotate = async () => {
        setIsRotating(true);
        setRotationFeedback(null);
        try {
            const targetId = activeInstanceId || 'default';
            const result = await smartRotateProfileAccount(targetId);
            const resumeNote = (result.resumedProjectsCount ?? 0) > 0
                ? ` · Auto-resumed ${result.resumedProjectsCount} project(s) (<1h)`
                : '';
            setRotationFeedback(`Successfully rotated to profile '${result.instanceName}' with account '${result.accountEmail}'${resumeNote}`);
        } catch (e: any) {
            setRotationFeedback(`Error: ${e?.message || e?.toString() || 'Rotation failed'}`);
        } finally {
            setIsRotating(false);
        }
    };

    return (
        <div className="space-y-4">
            {/* Header with toggle and actions */}
            <div className="flex items-center justify-between gap-3 flex-wrap">
                <div className="flex items-center gap-4">
                    <div
                        className={`w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-300 ${currentConfig.is_enabled
                            ? 'bg-blue-600 text-white shadow-md shadow-blue-500/20'
                            : 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400'
                            }`}
                    >
                        <RotateCw size={20} className={currentConfig.is_enabled ? 'animate-spin-slow' : ''} />
                    </div>
                    <div>
                        <div className="font-bold text-gray-900 dark:text-gray-100">
                            {t('settings.auto_switcher.title', 'Auto Profile Switcher & Task Resumption')}
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                            {t(
                                'settings.auto_switcher.desc',
                                'Monitors the active IDE profile on a timer and automatically switches to the next best profile when quota drops below threshold, resuming pending tasks.'
                            )}
                        </p>
                    </div>
                </div>

                <div className="flex items-center gap-3">
                    <input
                        type="file"
                        ref={fileInputRef}
                        accept=".json,application/json"
                        style={{ display: 'none' }}
                        onChange={handleImportFile}
                    />

                    {/* Import / Export Actions Dropdown */}
                    <div className="relative" ref={actionsRef}>
                        <button
                            type="button"
                            onClick={() => setIsActionsOpen(!isActionsOpen)}
                            className="px-2.5 py-1.5 text-xs font-medium rounded-lg border border-gray-300 dark:border-slate-700 bg-white dark:bg-slate-800 text-gray-700 dark:text-slate-300 hover:bg-gray-50 dark:hover:bg-slate-700 transition flex items-center gap-1.5 cursor-pointer shadow-2xs"
                            title="Import, Export, or Reset Switcher Config"
                        >
                            <SlidersHorizontal className="w-3.5 h-3.5" />
                            <span>Actions</span>
                            <ChevronDown className={`w-3.5 h-3.5 transition-transform duration-150 ${isActionsOpen ? 'rotate-180' : ''}`} />
                        </button>

                        {isActionsOpen && (
                            <div className="absolute right-0 mt-1.5 w-44 bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-lg z-50 py-1 text-xs animate-in fade-in zoom-in-95">
                                <button
                                    type="button"
                                    onClick={handleExport}
                                    className="w-full px-3 py-1.5 text-left text-gray-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                >
                                    <Download className="w-3.5 h-3.5 text-blue-500" />
                                    <span>Export Config</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => {
                                        setIsActionsOpen(false);
                                        fileInputRef.current?.click();
                                    }}
                                    className="w-full px-3 py-1.5 text-left text-gray-700 dark:text-slate-300 hover:bg-gray-100 dark:hover:bg-slate-800 flex items-center gap-2 transition-colors cursor-pointer"
                                >
                                    <Upload className="w-3.5 h-3.5 text-emerald-500" />
                                    <span>Import Config</span>
                                </button>
                                <div className="border-t border-gray-100 dark:border-slate-800 my-1" />
                                <button
                                    type="button"
                                    onClick={handleResetDefaults}
                                    className="w-full px-3 py-1.5 text-left text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 flex items-center gap-2 transition-colors cursor-pointer"
                                >
                                    <RotateCcw className="w-3.5 h-3.5" />
                                    <span>Reset to Defaults</span>
                                </button>
                            </div>
                        )}
                    </div>

                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            className="sr-only peer"
                            checked={currentConfig.is_enabled}
                            onChange={(e) => handleToggleEnabled(e.target.checked)}
                        />
                        <div className="w-11 h-6 bg-gray-200 dark:bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600 shadow-inner"></div>
                    </label>
                </div>
            </div>

            {/* Config details when enabled */}
            {currentConfig.is_enabled && (
                <div className="mt-4 pt-4 border-t border-slate-200 dark:border-slate-800 space-y-4 animate-in slide-in-from-top-2 duration-200">
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
                                max="300"
                                step="15"
                                value={currentConfig.check_interval_seconds}
                                onChange={(e) => handleIntervalChange(Number(e.target.value))}
                                className="w-full accent-blue-600 cursor-pointer"
                            />
                            <div className="flex justify-between text-[10px] text-slate-500 dark:text-slate-400 font-mono">
                                <span>15s</span>
                                <span>60s (Default)</span>
                                <span>120s</span>
                                <span>300s</span>
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
                                max="50"
                                step="1"
                                value={currentConfig.low_quota_threshold_percent}
                                onChange={(e) => handleThresholdChange(Number(e.target.value))}
                                className="w-full accent-rose-600 cursor-pointer"
                            />
                            <div className="flex justify-between text-[10px] text-slate-500 dark:text-slate-400 font-mono">
                                <span>1%</span>
                                <span>15% (Default)</span>
                                <span>30%</span>
                                <span>50%</span>
                            </div>
                        </div>

                        {/* Caution Polling Interval (< 20% Credits) */}
                        <div className="p-3.5 rounded-xl bg-amber-50/60 dark:bg-amber-950/30 border border-amber-200/60 dark:border-amber-900/40 space-y-2">
                            <div className="flex justify-between items-center text-xs font-semibold text-amber-800 dark:text-amber-300">
                                <span>Caution Polling Interval (&lt; 20% Credits)</span>
                                <span className="font-mono font-bold">
                                    {Math.round((currentConfig.caution_interval_seconds || 180) / 60)} min ({currentConfig.caution_interval_seconds || 180}s)
                                </span>
                            </div>
                            <input
                                type="range"
                                min="60"
                                max="300"
                                step="30"
                                value={currentConfig.caution_interval_seconds || 180}
                                onChange={(e) => onChange({ ...currentConfig, caution_interval_seconds: Number(e.target.value) })}
                                className="w-full accent-amber-600 cursor-pointer"
                            />
                            <div className="flex justify-between text-[10px] text-amber-600/70 dark:text-amber-400/60 font-mono">
                                <span>1 min</span>
                                <span>3 min (Default)</span>
                                <span>5 min</span>
                            </div>
                        </div>

                        {/* Critical Polling Interval (<= 12% Credits) */}
                        <div className="p-3.5 rounded-xl bg-rose-50/60 dark:bg-rose-950/30 border border-rose-200/60 dark:border-rose-900/40 space-y-2">
                            <div className="flex justify-between items-center text-xs font-semibold text-rose-800 dark:text-rose-300">
                                <span>Critical Polling Interval (&le; 12% Credits)</span>
                                <span className="font-mono font-bold">
                                    {currentConfig.critical_interval_seconds || 60}s
                                </span>
                            </div>
                            <input
                                type="range"
                                min="15"
                                max="120"
                                step="15"
                                value={currentConfig.critical_interval_seconds || 60}
                                onChange={(e) => onChange({ ...currentConfig, critical_interval_seconds: Number(e.target.value) })}
                                className="w-full accent-rose-600 cursor-pointer"
                            />
                            <div className="flex justify-between text-[10px] text-rose-600/70 dark:text-rose-400/60 font-mono">
                                <span>15s</span>
                                <span>60s (Default)</span>
                                <span>120s</span>
                            </div>
                        </div>
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 items-stretch">
                        {/* Target Model & Failover Benchmark Card */}
                        <div className="p-4 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 flex flex-col justify-between space-y-4 shadow-2xs">
                            <div>
                                <div className="flex items-center justify-between mb-2">
                                    <div className="flex items-center gap-2">
                                        <div className="p-1.5 rounded-lg bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400 border border-blue-200/60 dark:border-blue-500/30">
                                            <Cpu className="w-4 h-4" />
                                        </div>
                                        <div>
                                            <h4 className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wider">
                                                {t('settings.auto_switcher.target_model_label', 'Primary Evaluated Model')}
                                            </h4>
                                            <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                                Quota benchmark evaluated to trigger automated account failover
                                            </p>
                                        </div>
                                    </div>
                                    <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-blue-50 dark:bg-blue-500/10 text-blue-600 dark:text-blue-400 border border-blue-200/50 dark:border-blue-500/20 font-semibold">
                                        Trigger Metric
                                    </span>
                                </div>

                                <div className="relative mt-3">
                                    <select
                                        value={currentConfig.target_model}
                                        onChange={(e) => onChange({ ...currentConfig, target_model: e.target.value })}
                                        className="w-full appearance-none px-3 py-2 pr-9 bg-white dark:bg-slate-900 border border-slate-300 dark:border-slate-700 rounded-lg text-xs font-medium text-slate-800 dark:text-slate-100 shadow-2xs focus:outline-none focus:ring-2 focus:ring-blue-500/30 transition-all cursor-pointer"
                                    >
                                        <option value="gemini-3.8-flash">Gemini 3.8 Flash (Primary, Recommended)</option>
                                        <option value="claude-sonnet-4.6">Claude Sonnet 4.6 (Failover / Fallback)</option>
                                        <option value="gemini-pro">Gemini Pro</option>
                                    </select>
                                    <ChevronDown className="w-4 h-4 text-slate-400 pointer-events-none absolute right-3 top-2.5" />
                                </div>

                                {/* Contextual Model Traits / Hints */}
                                <div className="mt-3 p-2.5 rounded-lg bg-blue-50/60 dark:bg-slate-900/60 border border-blue-100/80 dark:border-slate-700/60 text-[11px] space-y-1">
                                    {currentConfig.target_model === 'gemini-3.8-flash' && (
                                        <div className="flex items-start gap-1.5 text-blue-700 dark:text-blue-300">
                                            <Sparkles className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                                            <span><strong>High Throughput &amp; Large Budget:</strong> Optimized for real-time completions with rolling 5-hour quota reset tracking.</span>
                                        </div>
                                    )}
                                    {currentConfig.target_model === 'claude-sonnet-4.6' && (
                                        <div className="flex items-start gap-1.5 text-purple-700 dark:text-purple-300">
                                            <Sparkles className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                                            <span><strong>Deep Reasoning Budget:</strong> Failover activates when Claude Sonnet extended thinking budget hits low threshold.</span>
                                        </div>
                                    )}
                                    {currentConfig.target_model === 'gemini-pro' && (
                                        <div className="flex items-start gap-1.5 text-slate-700 dark:text-slate-300">
                                            <Sparkles className="w-3.5 h-3.5 shrink-0 mt-0.5" />
                                            <span><strong>Standard Chat Baseline:</strong> Evaluates standard Gemini Pro quota headroom across all profile slots.</span>
                                        </div>
                                    )}
                                </div>
                            </div>

                            {/* Rotation Cooldown Period Setting */}
                            <div className="pt-3 border-t border-slate-200 dark:border-slate-700/60">
                                <div className="flex justify-between items-center text-xs font-semibold text-slate-800 dark:text-slate-200 mb-1.5">
                                    <span className="flex items-center gap-1.5">
                                        <Clock className="w-3.5 h-3.5 text-slate-500" />
                                        <span>Rotation Cooldown Guard</span>
                                    </span>
                                    <span className="text-blue-600 dark:text-blue-400 font-mono font-bold">
                                        {Math.round((currentConfig.cooldown_seconds || 180) / 60)} min ({currentConfig.cooldown_seconds || 180}s)
                                    </span>
                                </div>
                                <input
                                    type="range"
                                    min="60"
                                    max="600"
                                    step="30"
                                    value={currentConfig.cooldown_seconds || 180}
                                    onChange={(e) => onChange({ ...currentConfig, cooldown_seconds: Number(e.target.value) })}
                                    className="w-full accent-blue-600 cursor-pointer"
                                />
                                <div className="flex justify-between text-[10px] text-slate-500 dark:text-slate-400 font-mono mt-1">
                                    <span>1 min</span>
                                    <span>3 min (Default)</span>
                                    <span>5 min</span>
                                    <span>10 min</span>
                                </div>
                            </div>
                        </div>

                        {/* Task Continuity & Watchdog Card */}
                        <div className="p-4 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700/80 flex flex-col justify-between space-y-3 shadow-2xs">
                            <div>
                                <div className="flex items-center justify-between mb-2">
                                    <div className="flex items-center gap-2">
                                        <div className="p-1.5 rounded-lg bg-emerald-100 dark:bg-emerald-500/20 text-emerald-600 dark:text-emerald-400 border border-emerald-200/60 dark:border-emerald-500/30">
                                            <ShieldCheck className="w-4 h-4" />
                                        </div>
                                        <div>
                                            <h4 className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wider">
                                                Task Continuity &amp; Watchdog
                                            </h4>
                                            <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                                Zero-loss task resumption, prompt recovery, and process healing
                                            </p>
                                        </div>
                                    </div>
                                    <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-emerald-50 dark:bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-200/50 dark:border-emerald-500/20 font-semibold">
                                        Active Guard
                                    </span>
                                </div>

                                <div className="space-y-2 mt-3">
                                    {/* Toggle 1: Auto-Resume Pending Tasks */}
                                    <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                        <input
                                            type="checkbox"
                                            id="auto_resume_cb"
                                            checked={currentConfig.has_auto_resume}
                                            onChange={(e) => onChange({ ...currentConfig, has_auto_resume: e.target.checked })}
                                            className="checkbox checkbox-xs checkbox-primary rounded mt-0.5"
                                        />
                                        <div className="text-xs">
                                            <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                {t('settings.auto_switcher.auto_resume_label', 'Snapshot and auto-resume pending tasks on restart')}
                                            </span>
                                            <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                Preserves in-flight task queue state and restores session context upon account swap.
                                            </p>
                                        </div>
                                    </label>

                                    {/* Toggle 2: Fast Forward on Critical */}
                                    <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                        <input
                                            type="checkbox"
                                            id="auto_ff_cb"
                                            checked={currentConfig.auto_fast_forward_on_critical ?? true}
                                            onChange={(e) => onChange({ ...currentConfig, auto_fast_forward_on_critical: e.target.checked })}
                                            className="checkbox checkbox-xs checkbox-secondary rounded mt-0.5"
                                        />
                                        <div className="text-xs">
                                            <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                Auto-trigger Fast-Forward when credits drop &le; 12%
                                            </span>
                                            <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                Instantly delegates to the highest-credit candidate without waiting for full exhaustion.
                                            </p>
                                        </div>
                                    </label>

                                    {/* Toggle 3: Auto-Resume Recent Active Prompts */}
                                    <div className="space-y-1.5">
                                        <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                            <input
                                                type="checkbox"
                                                id="auto_resume_recent_cb"
                                                checked={currentConfig.auto_resume_recent_prompts ?? true}
                                                onChange={(e) => onChange({ ...currentConfig, auto_resume_recent_prompts: e.target.checked })}
                                                className="checkbox checkbox-xs checkbox-accent rounded mt-0.5"
                                            />
                                            <div className="text-xs">
                                                <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                    Auto-Resume Recent Active Prompts on Fast-Forward
                                                </span>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                    Transfers recent prompt history and attached images into freshly booted IDE instance.
                                                </p>
                                            </div>
                                        </label>

                                        {/* Recency Cutoff Window Sub-Panel */}
                                        {(currentConfig.auto_resume_recent_prompts ?? true) && (
                                            <div className="ml-7 p-3 bg-slate-100/90 dark:bg-slate-900/90 rounded-xl border border-slate-200 dark:border-slate-700/80 space-y-2 animate-in fade-in slide-in-from-top-1 duration-150 shadow-2xs">
                                                <div className="flex items-center justify-between gap-3">
                                                    <span className="flex items-center gap-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
                                                        <Clock className="w-3.5 h-3.5 text-cyan-500" />
                                                        <span>Recency Cutoff Window:</span>
                                                    </span>
                                                    <div className="relative">
                                                        <select
                                                            value={currentConfig.prompt_recency_threshold_seconds || 3600}
                                                            onChange={(e) => onChange({
                                                                ...currentConfig,
                                                                prompt_recency_threshold_seconds: Number(e.target.value),
                                                            })}
                                                            className="appearance-none px-2.5 py-1 pr-7 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-700 rounded-lg text-xs font-medium text-slate-800 dark:text-slate-200 shadow-2xs focus:outline-none focus:ring-2 focus:ring-cyan-500/30 cursor-pointer"
                                                        >
                                                            <option value={1800}>30 Minutes</option>
                                                            <option value={3600}>1 Hour (Recommended)</option>
                                                            <option value={7200}>2 Hours</option>
                                                            <option value={18000}>5 Hours</option>
                                                            <option value={86400}>1 Day (24 Hours)</option>
                                                        </select>
                                                        <ChevronDown className="w-3.5 h-3.5 text-slate-400 pointer-events-none absolute right-2 top-1.5" />
                                                    </div>
                                                </div>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-relaxed">
                                                    Projects executed within this timeframe are seamlessly restored with full prompt context. Older idle workspaces are safely bypassed for a clean startup.
                                                </p>
                                            </div>
                                        )}
                                    </div>

                                    {/* Toggle 4: 2-Minute IDE Crash & Focus Watchdog */}
                                    <label className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-slate-100/80 dark:hover:bg-slate-700/40 transition-colors cursor-pointer">
                                        <input
                                            type="checkbox"
                                            id="auto_focus_cb"
                                            checked={currentConfig.auto_focus_window ?? true}
                                            onChange={(e) => onChange({ ...currentConfig, auto_focus_window: e.target.checked })}
                                            className="checkbox checkbox-xs checkbox-info rounded mt-0.5"
                                        />
                                        <div className="text-xs">
                                            <span className="font-semibold text-slate-800 dark:text-slate-200">
                                                2-Minute IDE Crash &amp; Focus Watchdog
                                            </span>
                                            <p className="text-[11px] text-slate-500 dark:text-slate-400 leading-snug">
                                                Auto-focuses foreground PID and automatically relaunches non-responsive IDE windows.
                                            </p>
                                        </div>
                                    </label>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Manual Rotation Action */}
                    <div className="pt-2 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 bg-blue-50/50 dark:bg-blue-950/20 p-3.5 rounded-xl border border-blue-200/60 dark:border-blue-900/40 shadow-2xs">
                        <div className="flex items-center gap-2.5 text-xs text-slate-600 dark:text-slate-400">
                            <div className="p-1 rounded-md bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400">
                                <ArrowRightLeft className="w-3.5 h-3.5" />
                            </div>
                            <div>
                                <span className="font-semibold text-slate-900 dark:text-slate-100">Test Failover:</span>{' '}
                                Manually rotate the IDE to the next best profile candidate right now.
                            </div>
                        </div>
                        <button
                            type="button"
                            onClick={handleManualRotate}
                            disabled={isRotating}
                            className="btn btn-xs btn-primary gap-1.5 shadow-xs shrink-0 cursor-pointer"
                        >
                            <Play size={12} className={isRotating ? 'animate-spin' : ''} />
                            <span>{isRotating ? 'Rotating...' : 'Rotate Now'}</span>
                        </button>
                    </div>

                    {rotationFeedback ? (
                        <div className="p-2.5 rounded-lg bg-slate-100 dark:bg-slate-900 border border-slate-200 dark:border-slate-700/80 text-xs flex items-center gap-2 text-slate-800 dark:text-slate-200">
                            {rotationFeedback.startsWith('Error') ? (
                                <AlertCircle size={14} className="text-rose-500 shrink-0" />
                            ) : (
                                <CheckCircle2 size={14} className="text-emerald-500 shrink-0" />
                            )}
                            <span className="truncate">{rotationFeedback}</span>
                        </div>
                    ) : null}
                </div>
            )}

            {/* Prominent "How Auto Rotation Works" Guide */}
            <div className="mt-4 pt-4 border-t border-slate-200 dark:border-slate-800">
                <div className="p-4 rounded-xl bg-gradient-to-br from-blue-50/70 via-indigo-50/40 to-sky-50/60 dark:from-slate-900/80 dark:via-slate-900/60 dark:to-slate-950/80 backdrop-blur-md border border-blue-100/80 dark:border-white/10 shadow-xs dark:shadow-[0_8px_32px_rgba(0,0,0,0.4)] space-y-3.5 transition-all duration-200">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <div className="p-1.5 rounded-lg bg-blue-600 text-white shadow-xs">
                                <Sparkles className="w-4 h-4" />
                            </div>
                            <div>
                                <h4 className="text-xs font-bold text-gray-900 dark:text-slate-100 uppercase tracking-wider">
                                    {t('settings.auto_switcher.guide_title', 'How Auto Rotation Works')}
                                </h4>
                                <p className="text-[11px] text-gray-500 dark:text-slate-400">
                                    {t('settings.auto_switcher.guide_subtitle', 'Autonomous quota failover and zero-loss task protection mechanism')}
                                </p>
                            </div>
                        </div>
                        <span className="text-[10px] font-semibold px-2 py-0.5 rounded-md bg-blue-100 dark:bg-blue-500/20 text-blue-700 dark:text-blue-300 border border-blue-200/50 dark:border-blue-500/30">
                            Agm Tool By Alim
                        </span>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-1">
                        {/* Step 1: Background Polling */}
                        <div className="flex items-start gap-2.5 p-3 rounded-xl bg-white/70 dark:bg-slate-900/50 backdrop-blur-md border border-blue-100/60 dark:border-white/10 dark:hover:border-blue-500/40 shadow-xs dark:shadow-[0_4px_20px_-4px_rgba(0,0,0,0.5)] transition-all duration-200 group">
                            <div className="p-1.5 rounded-lg bg-blue-50 dark:bg-blue-500/15 text-blue-600 dark:text-blue-400 shrink-0 mt-0.5 border border-blue-100/50 dark:border-blue-500/20">
                                <Clock className="w-3.5 h-3.5" />
                            </div>
                            <div className="text-xs">
                                <div className="font-semibold text-gray-900 dark:text-slate-100">
                                    1. {t('settings.auto_switcher.step1_title', 'Background Quota Polling')}
                                </div>
                                <div className="text-gray-500 dark:text-slate-400 text-[11px] mt-0.5 leading-normal">
                                    {t('settings.auto_switcher.step1_desc', 'Monitors the currently active Antigravity profile in the background at your configured polling interval (15s–600s), checking live token consumption.')}
                                </div>
                            </div>
                        </div>

                        {/* Step 2: Threshold Trigger */}
                        <div className="flex items-start gap-2.5 p-3 rounded-xl bg-white/70 dark:bg-slate-900/50 backdrop-blur-md border border-blue-100/60 dark:border-white/10 dark:hover:border-blue-500/40 shadow-xs dark:shadow-[0_4px_20px_-4px_rgba(0,0,0,0.5)] transition-all duration-200 group">
                            <div className="p-1.5 rounded-lg bg-rose-50 dark:bg-rose-500/15 text-rose-600 dark:text-rose-400 shrink-0 mt-0.5 border border-rose-100/50 dark:border-rose-500/20">
                                <Gauge className="w-3.5 h-3.5" />
                            </div>
                            <div className="text-xs">
                                <div className="font-semibold text-gray-900 dark:text-slate-100">
                                    2. {t('settings.auto_switcher.step2_title', 'Threshold Trigger (<15%)')}
                                </div>
                                <div className="text-gray-500 dark:text-slate-400 text-[11px] mt-0.5 leading-normal">
                                    {t('settings.auto_switcher.step2_desc', 'When the primary evaluated model falls below your threshold (default <15%), AGM automatically flags the account for seamless rotation.')}
                                </div>
                            </div>
                        </div>

                        {/* Step 3: Best Profile Selection */}
                        <div className="flex items-start gap-2.5 p-3 rounded-xl bg-white/70 dark:bg-slate-900/50 backdrop-blur-md border border-blue-100/60 dark:border-white/10 dark:hover:border-blue-500/40 shadow-xs dark:shadow-[0_4px_20px_-4px_rgba(0,0,0,0.5)] transition-all duration-200 group">
                            <div className="p-1.5 rounded-lg bg-emerald-50 dark:bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 shrink-0 mt-0.5 border border-emerald-100/50 dark:border-emerald-500/20">
                                <Award className="w-3.5 h-3.5" />
                            </div>
                            <div className="text-xs">
                                <div className="font-semibold text-gray-900 dark:text-slate-100">
                                    3. {t('settings.auto_switcher.step3_title', 'Dynamic Best Profile Selection')}
                                </div>
                                <div className="text-gray-500 dark:text-slate-400 text-[11px] mt-0.5 leading-normal">
                                    {t('settings.auto_switcher.step3_desc', 'Ranks all available profiles by quota remaining, cooldown timer, and tier, picking the highest-capacity account automatically.')}
                                </div>
                            </div>
                        </div>

                        {/* Step 4: Split Repo DB Direct Dispatch */}
                        <div className="flex items-start gap-2.5 p-3 rounded-xl bg-white/70 dark:bg-slate-900/50 backdrop-blur-md border border-blue-100/60 dark:border-white/10 dark:hover:border-blue-500/40 shadow-xs dark:shadow-[0_4px_20px_-4px_rgba(0,0,0,0.5)] transition-all duration-200 group">
                            <div className="p-1.5 rounded-lg bg-purple-50 dark:bg-purple-500/15 text-purple-600 dark:text-purple-400 shrink-0 mt-0.5 border border-purple-100/50 dark:border-purple-500/20">
                                <History className="w-3.5 h-3.5" />
                            </div>
                            <div className="text-xs">
                                <div className="font-semibold text-gray-900 dark:text-slate-100">
                                    4. {t('settings.auto_switcher.step4_title', 'Split Repo DB Direct Dispatch')}
                                </div>
                                <div className="text-gray-500 dark:text-slate-400 text-[11px] mt-0.5 leading-normal">
                                    {t('settings.auto_switcher.step4_desc', 'Snapshots running prompts into dedicated repo_prompts.db before switching and dispatches them directly to active projects without queuing.')}
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Step 5: How to enable & leave in background */}
                    <div className="p-3 rounded-xl bg-blue-500/10 dark:bg-slate-900/60 backdrop-blur-md border border-blue-200/60 dark:border-blue-500/30 flex items-start gap-2.5 shadow-xs transition-all duration-200">
                        <ShieldCheck className="w-4 h-4 text-blue-600 dark:text-blue-400 shrink-0 mt-0.5" />
                        <div className="text-xs leading-normal">
                            <span className="font-semibold text-blue-950 dark:text-blue-300">
                                {t('settings.auto_switcher.step5_title', 'How to use: ')}
                            </span>
                            <span className="text-blue-900/80 dark:text-slate-300">
                                {t('settings.auto_switcher.step5_desc', 'Toggle on the Auto Profile Switcher switch above. Keep Agm Tool By Alim running in the background or minimized to the system tray. Your editor will automatically cycle between fresh quota pools 24/7 without losing task context.')}
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default AutoSwitcherSettings;
