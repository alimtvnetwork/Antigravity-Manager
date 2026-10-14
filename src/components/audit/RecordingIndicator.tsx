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
            className="inline-flex items-center rounded-full border border-slate-700/60 bg-slate-800/60 dark:bg-slate-900/80 divide-x divide-slate-700/60 shadow-xs text-[11px] font-mono"
            role="group"
            aria-label={t('audit.indicator_label', 'Audit recording controls')}
        >
            <button
                type="button"
                onClick={() => void toggleRecording()}
                title={isRecording
                    ? t('audit.stop_recording', 'Stop audit recording')
                    : t('audit.start_recording', 'Start audit recording')}
                className="px-2.5 py-1 flex items-center gap-1.5 text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-l-full transition-colors cursor-pointer"
            >
                <span
                    className={`inline-block w-2 h-2 rounded-full shrink-0 ${
                        isRecording ? 'bg-red-500 animate-pulse shadow-[0_0_8px_rgba(239,68,68,0.6)]' : 'bg-slate-500'
                    }`}
                />
                <span className={isRecording ? 'text-red-300' : 'text-slate-400'}>
                    {isRecording ? `REC ${elapsed}` : 'IDLE'}
                </span>
            </button>
            <button
                type="button"
                onClick={() => setPanelOpen(true)}
                title={t('audit.open_panel', 'Open audit log')}
                className="px-2.5 py-1 flex items-center gap-1.5 text-slate-300 hover:text-white hover:bg-slate-700/50 rounded-r-full transition-colors cursor-pointer"
            >
                <ScrollText className="w-3.5 h-3.5 text-slate-400" />
                <span>{t('audit.log', 'Log')}</span>
            </button>
        </div>
    );
}
