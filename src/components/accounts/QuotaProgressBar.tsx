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
    liveLimit?: any;
}

export function QuotaProgressBar({
    percentage,
    resetTime,
    label,
    isProtected = false,
    isWeekly = false,
    isWeeklyConstrained = false,
    className,
    heightClassName = "h-3.5",
    checkpoints = [100, 75, 50, 25],
    showCheckpoints = true,
    Icon,
}: QuotaProgressBarProps) {
    const clamped = Math.min(100, Math.max(0, Number.isFinite(percentage) ? percentage : 0));
    const isCritical = clamped < 25;
    const hasCheckpoints = showCheckpoints;

    // Piecewise gradient matching modern specification:
    // < 25%: deep dark red transition
    // >= 75%: neon emerald
    // 50-74%: emerald to teal
    // 25-49%: amber to orange
    const getTrackGradient = (pct: number) => {
        if (pct < 25) {
            return 'bg-gradient-to-r from-[#7f1d1d] via-[#991b1b] to-[#dc2626]';
        }
        if (pct >= 75) {
            return 'bg-gradient-to-r from-emerald-400 to-[#1af18d]';
        }
        if (pct >= 50) {
            return 'bg-gradient-to-r from-emerald-500 via-teal-400 to-[#1af18d]';
        }
        return 'bg-gradient-to-r from-amber-400 via-amber-500 to-orange-500';
    };

    const getTrackGlow = (pct: number) => {
        if (pct >= 75) return 'shadow-[0_0_10px_rgba(26,241,141,0.75)]';
        if (pct < 25) return 'shadow-[0_0_8px_rgba(220,38,38,0.75)]';
        return '';
    };

    const getPercentColorClass = (pct: number) => {
        if (pct >= 50) return 'text-emerald-700 dark:text-[#1af18d]';
        if (pct >= 25) return 'text-amber-700 dark:text-amber-400';
        return 'text-rose-600 dark:text-rose-400';
    };

    const getTimeColorClass = (time?: string) => {
        if (!time) return 'text-gray-400 dark:text-gray-500';
        const color = getTimeRemainingColor(time);
        switch (color) {
            case 'success': return 'text-emerald-700 dark:text-[#1af18d]';
            case 'warning': return 'text-amber-700 dark:text-amber-400';
            default: return 'text-blue-700 dark:text-blue-400';
        }
    };

    // Milestone bubble styling blending seamlessly with track gradient at that position
    const getNodeStyle = (checkpoint: number, isFilled: boolean, isCriticalState: boolean) => {
        if (isCriticalState && checkpoint <= 25) {
            return 'bg-[#991b1b] border-[1.5px] border-[#dc2626] shadow-[0_0_8px_rgba(220,38,38,0.75)]';
        }
        if (isFilled) {
            if (checkpoint >= 100) {
                return 'bg-[#1af18d] border-[1.5px] border-[#1af18d] shadow-[0_0_8px_rgba(26,241,141,0.85)]';
            }
            if (checkpoint >= 75) {
                return 'bg-[#34d399] border-[1.5px] border-[#34d399] shadow-[0_0_6px_rgba(52,211,153,0.6)]';
            }
            if (checkpoint >= 50) {
                return 'bg-[#f59e0b] border-[1.5px] border-[#f59e0b] shadow-none';
            }
            if (checkpoint >= 25) {
                return 'bg-[#f97316] border-[1.5px] border-[#f97316] shadow-none';
            }
            return 'bg-rose-500 border-[1.5px] border-rose-400 shadow-none';
        }
        return 'bg-slate-200/50 dark:bg-[#0c2438] border border-slate-300 dark:border-[#15334d]/60 shadow-none';
    };

    // Cap milestone nodes strictly at max 5
    const sortedCheckpoints = [...checkpoints]
        .filter((cp) => cp > 0)
        .sort((a, b) => b - a)
        .slice(0, 5);

    return (
        <div className={cn("w-[82%] max-w-[82%] flex items-center gap-2", className)}>
            <style>{`
                @keyframes agm-water-shimmer {
                    0% { transform: translateX(-100%); }
                    100% { transform: translateX(200%); }
                }
            `}</style>

            {/* Optional Icon / Label on Left */}
            {(Icon || label) && (
                <div className="flex items-center gap-1 shrink-0 max-w-[18%] min-w-0 text-slate-700 dark:text-slate-300">
                    {Icon && <Icon size={13} className="shrink-0" />}
                    {label && (
                        <span className="text-[11px] font-semibold truncate" title={label}>
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
                            getTrackGradient(clamped),
                            getTrackGlow(clamped)
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
                {hasCheckpoints && sortedCheckpoints.map((cp) => {
                    const isFilled = clamped >= cp || (isCritical && cp <= 25);
                    const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                    const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';

                    return (
                        <div
                            key={cp}
                            className={cn(
                                "absolute top-1/2 w-3.5 h-3.5 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none",
                                isCritical && cp <= 25 && "w-[15px] h-[15px]",
                                getNodeStyle(cp, isFilled, isCritical)
                            )}
                            style={{ left: leftPos, transform }}
                            title={`Checkpoint ${cp}%`}
                        >
                            {isCritical && cp <= 25 ? (
                                <span className="text-[7.5px] font-mono font-black text-white leading-none tracking-tight">
                                    {clamped}%
                                </span>
                            ) : isCritical ? null : (
                                <svg
                                    className={cn(
                                        "w-2 h-2 fill-none stroke-current transition-colors",
                                        isFilled ? "text-white stroke-[2.5]" : "text-gray-400/50 dark:text-white/30 stroke-[2.2]"
                                    )}
                                    viewBox="0 0 12 12"
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                >
                                    <path d="M2.5 6.5L4.8 8.8L9.5 3.5" />
                                </svg>
                            )}
                        </div>
                    );
                })}
            </div>

            {/* Single Column for Remaining Time and Percentage */}
            <div className="flex flex-col items-end shrink-0 leading-tight w-[18%] max-w-[18%]">
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
                    {isWeekly && (clamped === 50 || clamped === 30 || isCritical) ? (
                        <span className={cn(
                            "text-[11px] font-black font-mono px-1 py-[0.5px] rounded border shadow-2xs",
                            isCritical
                                ? "bg-rose-950/80 text-white border-rose-500/60 shadow-xs"
                                : clamped === 50
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
