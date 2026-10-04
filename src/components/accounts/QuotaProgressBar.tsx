import React from 'react';
import { Clock, Lock } from 'lucide-react';
import { cn } from '../../utils/cn';
import { formatTimeRemaining, getTimeRemainingColor } from '../../utils/format';

export interface QuotaProgressBarProps {
    percentage: number;
    resetTime?: string;
    label?: string;
    isProtected?: boolean;
    isWeekly?: boolean;
    isWeeklyConstrained?: boolean;
    className?: string;
    heightClassName?: string;
    checkpoints?: number[];
    showCheckpoints?: boolean;
    Icon?: React.ComponentType<{ size?: number; className?: string }>;
}

export function QuotaProgressBar({
    percentage,
    resetTime,
    label,
    isProtected = false,
    isWeekly = false,
    isWeeklyConstrained = false,
    className,
    heightClassName = "h-2.5",
    checkpoints = [100, 75, 50, 25],
    showCheckpoints = true,
    Icon,
}: QuotaProgressBarProps) {
    const clamped = Math.min(100, Math.max(0, Number.isFinite(percentage) ? percentage : 0));

    // Orange-to-red color progression:
    // >= 75%: Vibrant emerald green
    // >= 50%: Transitioning from green to amber
    // >= 25%: Warm amber to orange
    // < 25%: High-alert Orange to vivid Red warning
    const getTrackGradient = (pct: number) => {
        if (pct >= 75) return 'bg-gradient-to-r from-[#1af18d] via-[#10b981] to-[#059669]';
        if (pct >= 50) return 'bg-gradient-to-r from-[#059669] via-[#84cc16] to-[#eab308]';
        if (pct >= 25) return 'bg-gradient-to-r from-[#eab308] via-[#f59e0b] to-[#f97316]';
        return 'bg-gradient-to-r from-amber-500 via-[#ea580c] to-rose-600';
    };

    const getPercentColorClass = (pct: number) => {
        if (pct >= 50) return 'text-teal-700 dark:text-cyan-400';
        if (pct >= 25) return 'text-amber-700 dark:text-amber-400';
        return 'text-rose-600 dark:text-rose-400';
    };

    const getTimeColorClass = (time?: string) => {
        if (!time) return 'text-gray-400 dark:text-gray-500';
        const color = getTimeRemainingColor(time);
        switch (color) {
            case 'success': return 'text-teal-700 dark:text-cyan-400';
            case 'warning': return 'text-amber-700 dark:text-amber-400';
            default: return 'text-blue-700 dark:text-blue-400';
        }
    };

    // Milestone bubble styling
    const getNodeStyle = (idx: number, isFilled: boolean) => {
        if (!isFilled) {
            return "bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none";
        }
        switch (idx) {
            case 0:
                return "bg-[#1af18d] border-[1.5px] border-[#12b27d] shadow-[0_0_8px_rgba(26,241,141,0.6)]";
            case 1:
                return "bg-[#059669] border-[1.5px] border-[#047857] shadow-none";
            case 2:
                return "bg-[#eab308] border-[1.5px] border-[#ca8a04] shadow-none";
            case 3:
                return "bg-[#f97316] border-[1.5px] border-[#ea580c] shadow-none";
            default:
                return "bg-rose-500 border-[1.5px] border-rose-600 shadow-none";
        }
    };

    const sortedCheckpoints = [...checkpoints].sort((a, b) => b - a);

    return (
        <div className={cn("w-full flex items-center gap-2", className)}>
            <style>{`
                @keyframes agm-water-shimmer {
                    0% { transform: translateX(-100%); }
                    100% { transform: translateX(200%); }
                }
            `}</style>

            {/* Optional Icon / Label on Left */}
            {(Icon || label) && (
                <div className="flex items-center gap-1 shrink-0 text-slate-700 dark:text-slate-300">
                    {Icon && <Icon size={13} className="shrink-0" />}
                    {label && (
                        <span className="text-[11px] font-semibold truncate max-w-[80px]" title={label}>
                            {label}
                        </span>
                    )}
                </div>
            )}

            {/* Resilient "Fatty" Progress Bar Track */}
            <div className="relative flex-1 min-w-[60px] flex items-center">
                <div className={cn(
                    "relative w-full rounded-full overflow-hidden bg-slate-200 dark:bg-[#071a27] border border-slate-300 dark:border-[#15334d]",
                    heightClassName
                )}>
                    {/* Fill */}
                    <div
                        className={cn(
                            "h-full rounded-full relative overflow-hidden transition-all duration-500 ease-out",
                            getTrackGradient(clamped)
                        )}
                        style={{ width: `${clamped}%` }}
                    >
                        <div
                            className="absolute inset-0 bg-gradient-to-r from-transparent via-white/35 to-transparent pointer-events-none"
                            style={{ animation: 'agm-water-shimmer 2s infinite linear' }}
                        />
                    </div>
                </div>

                {/* Milestone Checkpoint Nodes */}
                {showCheckpoints && sortedCheckpoints.map((cp, idx) => {
                    const isFilled = clamped >= cp;
                    const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                    const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';

                    return (
                        <div
                            key={cp}
                            className={cn(
                                "absolute top-1/2 w-3 h-3 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none",
                                getNodeStyle(idx, isFilled)
                            )}
                            style={{ left: leftPos, transform }}
                            title={`Checkpoint ${cp}%`}
                        >
                            <svg
                                className={cn(
                                    "w-1.5 h-1.5 fill-none stroke-current transition-colors",
                                    isFilled ? "text-white stroke-[2.5]" : "text-gray-400/50 dark:text-white/30 stroke-[2.2]"
                                )}
                                viewBox="0 0 12 12"
                                strokeLinecap="round"
                                strokeLinejoin="round"
                            >
                                <path d="M2.5 6.5L4.8 8.8L9.5 3.5" />
                            </svg>
                        </div>
                    );
                })}
            </div>

            {/* Single Column for Remaining Time and Percentage */}
            <div className="flex flex-col items-end shrink-0 leading-tight min-w-[36px]">
                {resetTime ? (
                    <span
                        className={cn(
                            "text-[10px] font-bold flex items-center gap-0.5 font-mono",
                            getTimeColorClass(resetTime)
                        )}
                        title={`Resets in ${formatTimeRemaining(resetTime)}`}
                    >
                        <Clock className="w-2.5 h-2.5 shrink-0" />
                        {formatTimeRemaining(resetTime)}
                    </span>
                ) : null}

                <div className="flex items-center gap-1 justify-end">
                    {isProtected && (
                        <span title="Quota protected">
                            <Lock className="w-2.5 h-2.5 text-amber-500" />
                        </span>
                    )}

                    {isWeeklyConstrained && (
                        <span className="px-1 py-[0.5px] rounded bg-rose-500/15 text-rose-700 dark:text-rose-300 text-[9px] font-bold">
                            Weekly
                        </span>
                    )}

                    {/* Enlarge 30% and 50% weekly quota typography and badges */}
                    {isWeekly && (clamped === 50 || clamped === 30) ? (
                        <span className={cn(
                            "text-[11px] font-black font-mono px-1 py-[0.5px] rounded border shadow-2xs",
                            clamped === 50
                                ? "bg-amber-100 dark:bg-amber-950/50 text-amber-800 dark:text-amber-300 border-amber-300 dark:border-amber-700/60"
                                : "bg-rose-100 dark:bg-rose-950/50 text-rose-800 dark:text-rose-300 border-rose-300 dark:border-rose-700/60"
                        )}>
                            {clamped}%
                        </span>
                    ) : (
                        <span className={cn("text-[10px] font-black font-mono", getPercentColorClass(clamped))}>
                            {clamped}%
                        </span>
                    )}
                </div>
            </div>
        </div>
    );
}

export default QuotaProgressBar;
