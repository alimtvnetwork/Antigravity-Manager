import React, { useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Clock, CheckCircle, Copy, ChevronDown } from 'lucide-react';
import { formatSeconds } from './timingUtils';
import type { StageTimingInfo } from './timingUtils';

export interface TimingDiagnosticsCardProps {
    timing: StageTimingInfo;
    onCopyText: (text: string) => void;
}

export const TimingDiagnosticsCard: React.FC<TimingDiagnosticsCardProps> = ({ timing, onCopyText }) => {
    const { t } = useTranslation();
    const [isExpanded, setIsExpanded] = useState(false);
    const [isCopied, setIsCopied] = useState(false);

    const totalSec = timing.totalSec || 0;

    const stages = useMemo(() => [
        {
            key: 'clean',
            label: t('monitor.timing.clean', 'Session Cleaning (Clean)'),
            desc: t('monitor.timing.clean_desc', 'Clean cache control/merge same role/history purification'),
            sec: timing.cleanSec,
            color: 'bg-indigo-500',
            textColor: 'text-indigo-600 dark:text-indigo-400',
        },
        {
            key: 'norm',
            label: t('monitor.timing.norm', 'Normalize'),
            desc: t('monitor.timing.norm_desc', 'Model mapping/account scheduling/cross-protocol conversion'),
            sec: timing.normSec,
            color: 'bg-purple-500',
            textColor: 'text-purple-600 dark:text-purple-400',
        },
        {
            key: 'thinking',
            label: t('monitor.timing.thinking', 'ThinkingStore'),
            desc: t('monitor.timing.thinking_desc', 'Persistent thought chain and complete business agent historical signature'),
            sec: timing.thinkingSec,
            color: 'bg-amber-500',
            textColor: 'text-amber-600 dark:text-amber-400',
        },
        {
            key: 'ttft',
            label: t('monitor.timing.ttft', 'Waiting for first packet (TTFT)'),
            desc: t('monitor.timing.ttft_desc', 'The gateway is sent to receive the first data packet (including the first Token/thinking block)'),
            sec: timing.ttftSec,
            color: 'bg-emerald-500',
            textColor: 'text-emerald-600 dark:text-emerald-400',
        },
        {
            key: 'stream',
            label: t('monitor.timing.stream', 'Streaming'),
            desc: t('monitor.timing.stream_desc', 'The first data block arrives until the entire streaming response is completed'),
            sec: timing.streamSec,
            color: 'bg-sky-500',
            textColor: 'text-sky-600 dark:text-sky-400',
        },
    ], [timing, t]);

    const handleCopy = (e: React.MouseEvent) => {
        e.stopPropagation();
        const lines: string[] = [];
        if (timing.cleanSec !== undefined) lines.push(`Session Clean: ${formatSeconds(timing.cleanSec)}`);
        if (timing.normSec !== undefined) lines.push(`Relay Normalize: ${formatSeconds(timing.normSec)}`);
        if (timing.thinkingSec !== undefined) lines.push(`Thinking Store Backfill: ${formatSeconds(timing.thinkingSec)}`);
        if (timing.ttftSec !== undefined) lines.push(`Wait for First Token (TTFT): ${formatSeconds(timing.ttftSec)}`);
        if (timing.streamSec !== undefined) lines.push(`Stream Transmission: ${formatSeconds(timing.streamSec)}`);
        lines.push(`Total Duration: ${formatSeconds(timing.totalSec)}`);

        onCopyText(lines.join('\n'));
        setIsCopied(true);
        setTimeout(() => setIsCopied(false), 2000);
    };

    if (timing.isOldRecordWithoutStages) {
        return (
            <div className="mb-3 rounded-xl overflow-hidden border border-gray-200 dark:border-base-300 bg-gray-100/50 dark:bg-base-200">
                <div className="px-3 py-2 bg-gray-200/60 dark:bg-base-200 border-b border-gray-200 dark:border-base-300 flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Clock size={13} className="text-gray-500 dark:text-gray-400 shrink-0" />
                        <span className="text-xs font-bold tracking-wider text-gray-700 dark:text-gray-200 shrink-0 whitespace-nowrap">
                            {t('monitor.timing.title', 'Time-consuming diagnosis')}
                        </span>
                        <span className="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-emerald-50 text-emerald-700 dark:bg-emerald-950/70 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800/60">
                            {t('monitor.timing.total', 'Total time spent')}: {formatSeconds(timing.totalSec)}
                        </span>
                    </div>
                </div>
            </div>
        );
    }

    return (
        <div className="mb-3 rounded-xl overflow-hidden border border-emerald-500/30 dark:border-emerald-500/25 bg-emerald-50/25 dark:bg-base-100 shadow-sm">
            {/* Card Header */}
            <div
                className={`px-3 py-2 bg-emerald-500/10 dark:bg-emerald-950/30 flex items-center justify-between gap-2 select-none cursor-pointer hover:bg-emerald-500/15 transition-colors ${
                    isExpanded ? 'border-b border-emerald-500/20' : ''
                }`}
                onClick={() => setIsExpanded((prev) => !prev)}
            >
                <div className="flex items-center gap-2 min-w-0">
                    <Clock size={14} className="text-emerald-600 dark:text-emerald-400 shrink-0" />
                    <span className="text-xs font-bold tracking-wider text-emerald-950 dark:text-emerald-100 shrink-0 whitespace-nowrap">
                        {t('monitor.timing.title', 'Time-consuming diagnosis')}
                    </span>
                    {!isExpanded && totalSec > 0 && (
                        <span className="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-emerald-50 text-emerald-700 dark:bg-emerald-950/70 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800/60">
                            {t('monitor.timing.total', 'Total time spent')}: {formatSeconds(timing.totalSec)}
                        </span>
                    )}
                </div>

                <div className="flex items-center gap-1 shrink-0" onClick={(e) => e.stopPropagation()}>
                    <button
                        type="button"
                        onClick={handleCopy}
                        className="btn btn-ghost btn-xs h-6 px-2 text-emerald-800 dark:text-emerald-200 hover:bg-emerald-500/15 text-[11px] font-semibold gap-1"
                        title={isCopied ? t('common.copied', 'Copied') : t('common.copy', 'copy')}
                    >
                        {isCopied ? <CheckCircle size={12} className="text-emerald-500" /> : <Copy size={12} />}
                        <span>{isCopied ? t('common.copied', 'Copied') : t('common.copy', 'copy')}</span>
                    </button>
                    <button
                        type="button"
                        onClick={() => setIsExpanded((prev) => !prev)}
                        className="btn btn-ghost btn-xs p-1 h-6 min-h-0 text-emerald-800 dark:text-emerald-300 hover:bg-emerald-500/15"
                        title={isExpanded ? 'Close time-consuming diagnosis' : 'Expand time-consuming diagnosis'}
                    >
                        <ChevronDown size={14} className={`transition-transform duration-200 ${isExpanded ? '' : '-rotate-90'}`} />
                    </button>
                </div>
            </div>

            {/* Expandable Body */}
            {isExpanded && (
                <div className="p-3 space-y-2.5 font-mono text-xs">
                    {/* Multi-stage Stacked Progress Bar */}
                    {totalSec > 0 && (
                        <div className="space-y-1">
                            <div className="h-2 w-full bg-gray-200/80 dark:bg-base-300 rounded-full flex overflow-hidden shadow-inner">
                                {stages.map((st) => {
                                    if (st.sec === undefined || st.sec <= 0) return null;
                                    const pct = Math.min(100, Math.max(0.5, (st.sec / totalSec) * 100));
                                    return (
                                        <div
                                            key={st.key}
                                            style={{ width: `${pct}%` }}
                                            className={`${st.color} h-full transition-all duration-300 relative group`}
                                            title={`${st.label}: ${formatSeconds(st.sec)} (${((st.sec / totalSec) * 100).toFixed(1)}%)`}
                                        />
                                    );
                                })}
                            </div>
                        </div>
                    )}

                    {/* Stage Metrics Grid */}
                    <div className="grid grid-cols-1 gap-1.5 pt-0.5">
                        {stages.map((st) => {
                            const hasVal = st.sec !== undefined;
                            const pct = hasVal && totalSec > 0 ? ((st.sec! / totalSec) * 100).toFixed(1) : undefined;
                            return (
                                <div
                                    key={st.key}
                                    className="flex items-center justify-between gap-2 px-2.5 py-1.5 rounded-lg bg-white dark:bg-base-200 border border-gray-200/80 dark:border-base-300/90 hover:border-emerald-500/40 transition-colors shadow-2xs"
                                >
                                    <div className="flex items-center gap-2.5 min-w-0">
                                        <span className={`w-2.5 h-2.5 rounded-full ${st.color} shrink-0`} />
                                        <div className="min-w-0">
                                            <span className="font-bold text-gray-900 dark:text-white truncate block text-xs">
                                                {st.label}
                                            </span>
                                            <span className="text-[10px] text-gray-500 dark:text-gray-400 truncate block">
                                                {st.desc}
                                            </span>
                                        </div>
                                    </div>

                                    <div className="flex items-baseline gap-2 shrink-0 text-right font-mono">
                                        <span className={`text-xs font-black ${hasVal ? st.textColor : 'text-gray-400'}`}>
                                            {formatSeconds(st.sec)}
                                        </span>
                                        {pct !== undefined && (
                                            <span className="text-[11px] font-bold text-gray-500 dark:text-gray-400 w-11 text-right">
                                                {pct}%
                                            </span>
                                        )}
                                    </div>
                                </div>
                            );
                        })}

                        {/* Total Duration Row */}
                        <div className="flex items-center justify-between gap-2 px-2.5 py-1.5 rounded-lg bg-emerald-500/15 dark:bg-emerald-950/50 border border-emerald-500/40 font-bold">
                            <div className="flex items-center gap-2 min-w-0">
                                <span className="w-2.5 h-2.5 rounded-full bg-emerald-500 shrink-0" />
                                <span className="text-emerald-950 dark:text-emerald-100 text-xs font-bold">
                                    {t('monitor.timing.total', 'Total time spent')}
                                </span>
                            </div>
                            <div className="flex items-baseline gap-2 shrink-0 text-right font-mono">
                                <span className="text-sm font-black text-emerald-800 dark:text-emerald-200">
                                    {formatSeconds(timing.totalSec)}
                                </span>
                                <span className="text-[11px] text-emerald-700/80 dark:text-emerald-300/80 w-11 text-right font-bold">
                                    100%
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
