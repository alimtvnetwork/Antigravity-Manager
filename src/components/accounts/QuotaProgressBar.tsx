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
    /** Compact density mode for dense tables (thinner bar, smaller markers/labels). Defaults to false. */
    compact?: boolean;
}

/**
 * Local scale color at each checkpoint position. These match the stops of
 * --ui-quota-gradient (src/styles/ui-tokens.css) so a filled marker always
 * corresponds to its position on the scale.
 */
const CHECKPOINT_SCALE_COLORS: Record<number, string> = {
    90: '#1af18d',
    80: '#10b981',
    60: '#eab308',
    40: '#f97316',
    20: '#ef4444',
    100: '#1af18d',
    75: '#10b981',
    50: '#eab308',
    25: '#f97316',
    0: '#7f1d1d',
};

export function QuotaProgressBar({
    percentage,
    resetTime,
    label,
    isProtected = false,
    isWeekly = false,
    isWeeklyConstrained = false,
    className,
    compact = false,
    heightClassName = compact ? "h-2" : "h-2.5",
    checkpoints = [20, 40, 60, 80, 90],
    showCheckpoints = true,
    Icon,
}: QuotaProgressBarProps) {
    const clamped = Math.min(100, Math.max(0, Number.isFinite(percentage) ? percentage : 0));

    const getPercentColorClass = (pct: number) => {
        if (pct >= 50) return 'text-emerald-700 dark:text-[var(--ui-success)]';
        if (pct >= 25) return 'text-amber-700 dark:text-[var(--ui-warning)]';
        return 'text-rose-600 dark:text-[var(--ui-danger)]';
    };

    const getTimeColorClass = (time?: string) => {
        if (!time) return 'text-gray-400 dark:text-gray-500';
        const color = getTimeRemainingColor(time);
        switch (color) {
            case 'success': return 'text-emerald-700 dark:text-[var(--ui-success)]';
            case 'warning': return 'text-amber-700 dark:text-[var(--ui-warning)]';
            default: return 'text-blue-700 dark:text-blue-400';
        }
    };

    const internalCheckpoints = (checkpoints && checkpoints.length > 0 ? checkpoints : [20, 40, 60, 80, 90])
        .filter((cp) => cp > 0 && cp < 100)
        .slice(0, 5);
    const sortedCheckpoints = [...internalCheckpoints].sort((a, b) => a - b);

    return (
        <div
            id="agm-progress-container"
            className={cn(
                "w-full flex items-center opacity-85 hover:opacity-100 group-hover:opacity-100 transition-all duration-200 hover:drop-shadow-[0_0_8px_rgba(26,241,141,0.5)] group-hover:drop-shadow-[0_0_8px_rgba(26,241,141,0.5)]",
                compact ? "gap-1" : "gap-2",
                className
            )}
        >
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

            {/* Progress Bar Track — the spatial scale gradient spans the FULL track
                width; a mask reveals only the filled percentage. The gradient is
                never stretched into a partial fill. */}
            <div className="relative flex-1 min-w-[140px] flex items-center">
                <div
                    id="agm-progress-track"
                    className={cn(
                        "relative w-full rounded-full overflow-hidden bg-slate-200 dark:bg-[var(--ui-quota-track)] border border-slate-300 dark:border-[var(--ui-quota-track-border)] shadow-none",
                        heightClassName
                    )}
                    role="progressbar"
                    aria-valuemin={0}
                    aria-valuemax={100}
                    aria-valuenow={clamped}
                    aria-label={label || (isWeekly ? 'Weekly quota' : 'Quota')}
                >
                    {/* Full-width spatial scale */}
                    <div
                        id="agm-progress-fill"
                        className="absolute inset-0"
                        style={{ background: 'var(--ui-quota-gradient)' }}
                        aria-hidden="true"
                    />
                    {/* Mask covering the unfilled portion */}
                    <div
                        className="absolute inset-y-0 right-0 bg-slate-200 dark:bg-[var(--ui-quota-track)] transition-[width] duration-[240ms] ease-[cubic-bezier(0.22,1,0.36,1)]"
                        style={{ width: `${100 - clamped}%` }}
                        aria-hidden="true"
                    />
                </div>

                {/* Ambient neon glow element */}
                <div
                    id="agm-progress-glow"
                    className="absolute inset-0 rounded-full pointer-events-none opacity-0 group-hover:opacity-100 hover:opacity-100 transition-opacity duration-200 shadow-[0_0_12px_rgba(26,241,141,0.6)]"
                />

                {/* Milestone Checkpoint Nodes (20px protruding nodes, centered vertically) */}
                {showCheckpoints && sortedCheckpoints.map((cp, idx) => {
                    const isFilled = clamped >= cp;
                    const scaleColor = CHECKPOINT_SCALE_COLORS[cp] ?? (cp >= 75 ? '#1af18d' : cp >= 50 ? '#eab308' : '#ef4444');

                    return (
                        <div
                            key={cp}
                            id={`agm-progress-node-${idx + 1}`}
                            className={cn(
                                "agm-progress-node absolute top-1/2 -translate-y-1/2 -translate-x-1/2 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none border-[1.5px]",
                                compact ? "w-4 h-4" : "w-5 h-5",
                                isFilled
                                    ? "border-white shadow-[0_0_8px_rgba(0,0,0,0.4),inset_0_1px_2px_rgba(255,255,255,0.5)] group-hover:shadow-[0_0_10px_rgba(26,241,141,0.8),0_0_4px_#ffffff]"
                                    : "bg-slate-200/90 dark:bg-[#171025]/90 border-slate-300 dark:border-[#40285e] shadow-[inset_0_1px_2px_rgba(0,0,0,0.5)]"
                            )}
                            style={{
                                left: `${cp}%`,
                                ...(isFilled ? { backgroundColor: scaleColor, borderColor: '#ffffff' } : {}),
                            }}
                            title={`Milestone ${cp}%`}
                        >
                            {isFilled ? (
                                <svg
                                    className={cn("text-white pointer-events-none drop-shadow-[0_1px_1px_rgba(0,0,0,0.5)]", compact ? "w-2.5 h-2.5" : "w-3 h-3")}
                                    viewBox="0 0 16 16"
                                    fill="none"
                                    stroke="currentColor"
                                    strokeWidth="2.2"
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                >
                                    <path d="M3.5 8.5L6.5 11.5L12.5 4.5" />
                                </svg>
                            ) : (
                                <span className={cn("rounded-full bg-slate-400/40 dark:bg-white/20", compact ? "w-1 h-1" : "w-1.5 h-1.5")} />
                            )}
                        </div>
                    );
                })}
            </div>

            {/* Single Column for Remaining Time and Percentage */}
            <div className={cn("flex flex-col items-end shrink-0", compact ? "leading-none" : "leading-tight")}>
                {resetTime ? (
                    <span
                        className={cn(
                            "font-bold flex items-center gap-0.5 font-mono",
                            compact ? "text-[9px]" : "text-[10px]",
                            getTimeColorClass(resetTime)
                        )}
                        title={`Resets in ${formatTimeRemaining(resetTime)}`}
                    >
                        <Clock className={cn("shrink-0", compact ? "w-2 h-2" : "w-2.5 h-2.5")} />
                        {formatTimeRemaining(resetTime)}
                    </span>
                ) : null}

                <div className="flex items-center gap-1 justify-end">
                    {isProtected && (
                        <span title="Quota protected">
                            <Lock className={cn("text-amber-500", compact ? "w-2 h-2" : "w-2.5 h-2.5")} />
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
                            "font-black font-mono px-1 py-[0.5px] rounded border shadow-2xs",
                            compact ? "text-[10px]" : "text-[11px]",
                            clamped === 50
                                ? "bg-amber-100 dark:bg-amber-950/50 text-amber-800 dark:text-amber-300 border-amber-300 dark:border-amber-700/60"
                                : "bg-rose-100 dark:bg-rose-950/50 text-rose-800 dark:text-rose-300 border-rose-300 dark:border-rose-700/60"
                        )}>
                            {clamped}%
                        </span>
                    ) : (
                        <span className={cn("font-black font-mono", compact ? "text-[9px]" : "text-[10px]", getPercentColorClass(clamped))}>
                            {clamped}%
                        </span>
                    )}
                </div>
            </div>
        </div>
    );
}

export default QuotaProgressBar;
