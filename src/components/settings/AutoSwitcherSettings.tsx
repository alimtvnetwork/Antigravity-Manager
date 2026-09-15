import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { RotateCw, Play, CheckCircle2, AlertCircle } from 'lucide-react';
import { AutoProfileSwitcherConfig } from '../../types/config';
import { useInstanceStore } from '../../stores/useInstanceStore';

interface AutoSwitcherSettingsProps {
    config?: AutoProfileSwitcherConfig;
    onChange: (config: AutoProfileSwitcherConfig) => void;
}

const DEFAULT_CONFIG: AutoProfileSwitcherConfig = {
    is_enabled: false,
    check_interval_seconds: 60,
    low_quota_threshold_percent: 10.0,
    target_model: 'gemini-pro',
    has_auto_resume: true,
    cooldown_seconds: 180,
};

export const AutoSwitcherSettings: React.FC<AutoSwitcherSettingsProps> = ({ config, onChange }) => {
    const { t } = useTranslation();
    const currentConfig = config || DEFAULT_CONFIG;
    const { triggerManualRotation } = useInstanceStore();
    const [rotationFeedback, setRotationFeedback] = useState<string | null>(null);
    const [isRotating, setIsRotating] = useState(false);

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
            const msg = await triggerManualRotation();
            setRotationFeedback(msg);
        } catch (e: any) {
            setRotationFeedback(`Error: ${e?.toString() || 'Rotation failed'}`);
        } finally {
            setIsRotating(false);
        }
    };

    return (
        <div className="space-y-4">
            {/* Header with toggle */}
            <div className="flex items-center justify-between">
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
                <label className="relative inline-flex items-center cursor-pointer">
                    <input
                        type="checkbox"
                        className="sr-only peer"
                        checked={currentConfig.is_enabled}
                        onChange={(e) => handleToggleEnabled(e.target.checked)}
                    />
                    <div className="w-11 h-6 bg-gray-200 dark:bg-base-300 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600 shadow-inner"></div>
                </label>
            </div>

            {/* Config details when enabled */}
            {currentConfig.is_enabled && (
                <div className="mt-4 pt-4 border-t border-gray-100 dark:border-base-300 space-y-4 animate-in slide-in-from-top-2 duration-200">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {/* Polling Interval */}
                        <div className="p-3.5 rounded-xl bg-gray-50/70 dark:bg-base-100/50 border border-gray-100 dark:border-base-300 space-y-2">
                            <div className="flex justify-between items-center text-xs font-semibold text-gray-700 dark:text-gray-300">
                                <span>{t('settings.auto_switcher.interval_label', 'Check Interval (seconds)')}</span>
                                <span className="text-blue-600 dark:text-blue-400 font-mono">
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
                                className="w-full accent-blue-600"
                            />
                            <div className="flex justify-between text-[10px] text-gray-400 font-mono">
                                <span>15s</span>
                                <span>60s (Default)</span>
                                <span>120s</span>
                                <span>300s</span>
                            </div>
                        </div>

                        {/* Low Quota Threshold */}
                        <div className="p-3.5 rounded-xl bg-gray-50/70 dark:bg-base-100/50 border border-gray-100 dark:border-base-300 space-y-2">
                            <div className="flex justify-between items-center text-xs font-semibold text-gray-700 dark:text-gray-300">
                                <span>{t('settings.auto_switcher.threshold_label', 'Low Quota Threshold (%)')}</span>
                                <span className="text-rose-600 dark:text-rose-400 font-mono">
                                    &lt; {currentConfig.low_quota_threshold_percent.toFixed(0)}%
                                </span>
                            </div>
                            <input
                                type="range"
                                min="1"
                                max="30"
                                step="1"
                                value={currentConfig.low_quota_threshold_percent}
                                onChange={(e) => handleThresholdChange(Number(e.target.value))}
                                className="w-full accent-rose-600"
                            />
                            <div className="flex justify-between text-[10px] text-gray-400 font-mono">
                                <span>1%</span>
                                <span>10% (Default)</span>
                                <span>20%</span>
                                <span>30%</span>
                            </div>
                        </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 items-center">
                        {/* Target Model to Evaluate */}
                        <div>
                            <label className="text-xs font-medium text-gray-600 dark:text-gray-400 block mb-1">
                                {t('settings.auto_switcher.target_model_label', 'Primary Evaluated Model')}
                            </label>
                            <select
                                value={currentConfig.target_model}
                                onChange={(e) => onChange({ ...currentConfig, target_model: e.target.value })}
                                className="select select-sm w-full bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 rounded-lg text-xs"
                            >
                                <option value="gemini-pro">Gemini Pro (Code & General)</option>
                                <option value="gemini-flash">Gemini Flash</option>
                                <option value="claude">Claude Sonnet</option>
                            </select>
                        </div>

                        {/* Task Resume Checkbox */}
                        <div className="flex items-center gap-3 pt-4">
                            <input
                                type="checkbox"
                                id="auto_resume_cb"
                                checked={currentConfig.has_auto_resume}
                                onChange={(e) => onChange({ ...currentConfig, has_auto_resume: e.target.checked })}
                                className="checkbox checkbox-sm checkbox-primary rounded"
                            />
                            <label htmlFor="auto_resume_cb" className="text-xs font-medium text-gray-700 dark:text-gray-300 cursor-pointer">
                                {t('settings.auto_switcher.auto_resume_label', 'Snapshot and auto-resume pending tasks on restart')}
                            </label>
                        </div>
                    </div>

                    {/* Manual Rotation Action */}
                    <div className="pt-2 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 bg-blue-50/40 dark:bg-blue-900/10 p-3 rounded-xl border border-blue-100/80 dark:border-blue-800/40">
                        <div className="text-xs text-gray-600 dark:text-gray-400">
                            <span className="font-semibold text-gray-900 dark:text-gray-200">Test Failover:</span>{' '}
                            Manually rotate the IDE to the next best profile right now.
                        </div>
                        <button
                            type="button"
                            onClick={handleManualRotate}
                            disabled={isRotating}
                            className="btn btn-xs btn-primary gap-1.5 shadow-xs shrink-0"
                        >
                            <Play size={12} className={isRotating ? 'animate-spin' : ''} />
                            <span>{isRotating ? 'Rotating...' : 'Rotate Now'}</span>
                        </button>
                    </div>

                    {rotationFeedback && (
                        <div className="p-2.5 rounded-lg bg-gray-100 dark:bg-base-100 text-xs flex items-center gap-2 text-gray-800 dark:text-gray-200">
                            {rotationFeedback.startsWith('Error') ? (
                                <AlertCircle size={14} className="text-rose-500 shrink-0" />
                            ) : (
                                <CheckCircle2 size={14} className="text-emerald-500 shrink-0" />
                            )}
                            <span className="truncate">{rotationFeedback}</span>
                        </div>
                    )}
                </div>
            )}
        </div>
    );
};

export default AutoSwitcherSettings;
