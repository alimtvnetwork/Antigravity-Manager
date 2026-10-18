import { useState, useEffect } from 'react';
import { Clock, Calendar } from 'lucide-react';
import { useConfigStore } from '../../stores/useConfigStore';
import { formatDateTime, type DateFormatMode } from '../../utils/date';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';

interface DateTimeFormatSectionProps {
    value?: DateFormatMode;
    onChange?: (mode: DateFormatMode) => void;
}

export function DateTimeFormatSection({ value, onChange }: DateTimeFormatSectionProps) {
    const { config, saveConfig } = useConfigStore();
    const currentMode: DateFormatMode = value || config?.date_format_mode || '24h';
    const [now, setNow] = useState(() => new Date());

    useEffect(() => {
        const timer = setInterval(() => {
            setNow(new Date());
        }, 1000);
        return () => clearInterval(timer);
    }, []);

    const handleModeChange = async (newMode: DateFormatMode) => {
        if (onChange) {
            onChange(newMode);
        }
        try {
            await saveConfig({
                ...config,
                date_format_mode: newMode,
                time_format_preference: newMode,
            });
            showToast(`Date & Time format set to ${newMode}`, 'success');
        } catch (err) {
            useErrorStore.getState().captureError(err, {
                source: 'GeneralSettingsTab.handleModeChange',
                triggerAction: 'saveConfig.date_format_mode',
            });
        }
    };

    const previewFormatted = formatDateTime(now, currentMode);

    return (
        <div className="p-3.5 bg-gray-50 dark:bg-base-200 rounded-lg border border-gray-100 dark:border-base-300 space-y-2.5">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                    <Clock className="w-4 h-4 text-blue-500" />
                    <div>
                        <div className="font-medium text-sm text-gray-900 dark:text-base-content">
                            Date &amp; Time Format
                        </div>
                        <p className="text-xs text-gray-600 dark:text-gray-400">
                            Choose the display format for timestamps across prompt trees, instances, and telemetry.
                        </p>
                    </div>
                </div>

                {/* Live Preview Capsule */}
                <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-blue-500/10 dark:bg-blue-500/20 text-blue-600 dark:text-blue-300 border border-blue-500/30 text-xs font-mono self-start sm:self-auto">
                    <Calendar className="w-3 h-3 text-blue-400" />
                    <span>Preview: {previewFormatted}</span>
                </div>
            </div>

            {/* Selector Options */}
            <div className="flex flex-wrap items-center gap-2 pt-1">
                {(
                    [
                        { id: '24h', label: '24-Hour Clock (e.g. 18-Oct-2026 21:40)', badge: 'Default' },
                        { id: '12h', label: '12-Hour Clock (e.g. 18-Oct-2026 09:40 PM)' },
                        { id: 'iso', label: 'ISO 8601 (e.g. 2026-10-18 21:40:00)' },
                    ] as const
                ).map((opt) => {
                    const isSelected = currentMode === opt.id;
                    return (
                        <button
                            key={opt.id}
                            type="button"
                            onClick={() => handleModeChange(opt.id)}
                            className={`px-3 py-1.5 rounded-full text-xs font-medium transition-all cursor-pointer flex items-center gap-1.5 border ${
                                isSelected
                                    ? 'bg-blue-500 text-white border-blue-500 shadow-sm'
                                    : 'bg-white dark:bg-base-300 text-gray-700 dark:text-gray-300 border-gray-200 dark:border-base-100 hover:border-blue-400'
                            }`}
                        >
                            <span>{opt.label}</span>
                            {opt.badge && !isSelected && (
                                <span className="px-1.5 py-0.2 rounded-full text-[9px] bg-blue-500/15 text-blue-500 font-bold">
                                    {opt.badge}
                                </span>
                            )}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}

export default DateTimeFormatSection;
