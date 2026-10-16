/**
 * RecordingIndicator — pill badge showing the audit recording state.
 *
 * Segmented pill capsule (house UI convention): left segment toggles
 * recording on click, right segment opens the audit panel.
 */
import { useEffect, useState } from 'react';
import { ScrollText } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAuditStore } from './audit-store';

function formatElapsed(ms: number): string {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

export function RecordingIndicator() {
    const { t } = useTranslation();
    const isRecording = useAuditStore((s) => s.isRecording);
    const recordingSince = useAuditStore((s) => s.recordingSince);
    const toggleRecording = useAuditStore((s) => s.toggleRecording);
    const setPanelOpen = useAuditStore((s) => s.setPanelOpen);
    const [now, setNow] = useState(() => Date.now());

    useEffect(() => {
        if (!isRecording) return;
        const timer = window.setInterval(() => setNow(Date.now()), 1000);
        return () => window.clearInterval(timer);
    }, [isRecording]);

    const elapsed = recordingSince ? formatElapsed(Math.max(0, now - recordingSince)) : '00:00';

    return (
        <div
            className="inline-flex items-center h-7 rounded-full bg-gray-100 dark:bg-[#171025]/90 border border-gray-200/60 dark:border-[#40285e] p-0.5 divide-x divide-gray-200/50 dark:divide-slate-700/60 shadow-xs text-[10.5px] font-mono"
            role="group"
            aria-label={t('audit.indicator_label', 'Audit recording controls')}
        >
            <button
                type="button"
                onClick={() => void toggleRecording()}
                title={isRecording
                    ? t('audit.stop_recording', 'Stop audit recording')
                    : t('audit.start_recording', 'Start audit recording')}
                className="h-full px-2 flex items-center gap-1.5 text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-200/80 dark:hover:bg-white/10 active:bg-gray-300/80 dark:active:bg-white/15 rounded-l-full transition-colors cursor-pointer"
            >
                <span
                    className={`inline-block w-2 h-2 rounded-full shrink-0 ${
                        isRecording ? 'bg-red-500 animate-pulse shadow-[0_0_8px_rgba(239,68,68,0.6)]' : 'bg-gray-400 dark:bg-slate-500'
                    }`}
                />
                <span className={isRecording ? 'text-red-600 dark:text-red-300 font-semibold' : 'text-gray-600 dark:text-slate-400 font-medium'}>
                    {isRecording ? `REC ${elapsed}` : 'IDLE'}
                </span>
            </button>
            <button
                type="button"
                onClick={() => setPanelOpen(true)}
                title={t('audit.open_panel', 'Open audit log')}
                className="h-full px-2 flex items-center gap-1.5 text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-200/80 dark:hover:bg-white/10 active:bg-gray-300/80 dark:active:bg-white/15 rounded-r-full transition-colors cursor-pointer"
            >
                <ScrollText className="w-3.5 h-3.5 text-gray-500 dark:text-gray-400" />
                <span className="font-medium">{t('audit.log', 'Log')}</span>
            </button>
        </div>
    );
}
