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
    100: '#20d99a',
    75: '#70ddb0',
    50: '#e7d36a',
    25: '#f49a45',
    0: '#f2556d',
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
    heightClassName = compact ? "h-2" : "h-3",
    checkpoints = [100, 75, 50, 25, 0],
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

    const activeCheckpoints = (checkpoints && checkpoints.length > 0 ? checkpoints : [100, 75, 50, 25, 0]).slice(0, 5);
    const sortedCheckpoints = [...activeCheckpoints].sort((a, b) => b - a);

    return (
        <div className={cn("w-[82%] max-w-[82%] flex items-center", compact ? "gap-1.5" : "gap-2", className)}>
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

            {/* Progress Bar Track — the 8-stop scale gradient spans the FULL track
                width; a mask reveals only the filled percentage. The gradient is
                never stretched into a partial fill. */}
            <div className="relative flex-1 min-w-[60px] flex items-center">
                <div
                    className={cn(
                        "relative w-full rounded-full overflow-hidden bg-slate-200 dark:bg-[var(--ui-quota-track)] border border-slate-300 dark:border-[var(--ui-quota-track-border)]",
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

                {/* Milestone Checkpoint Nodes */}
                {showCheckpoints && sortedCheckpoints.map((cp) => {
                    const isFilled = clamped >= cp;
                    const leftPos = cp >= 100 ? '100%' : cp <= 0 ? '0%' : `${cp}%`;
                    const transform = cp >= 100 ? 'translate(-100%, -50%)' : cp <= 0 ? 'translate(0, -50%)' : 'translate(-50%, -50%)';
                    const isCritical = clamped < 25;
                    const scaleColor = CHECKPOINT_SCALE_COLORS[cp] ?? '#20d99a';

                    return (
                        <div
                            key={cp}
                            className={cn(
                                "absolute top-1/2 rounded-full flex items-center justify-center transition-all duration-300 z-10 pointer-events-none border-[1.5px]",
                                compact ? "w-2.5 h-2.5" : "w-3.5 h-3.5",
                                isFilled
                                    ? (cp >= 50
                                        // Restrained glow only at the current endpoint in the healthy range
                                        ? 'shadow-[0_0_8px_rgb(32_217_154/0.35)]'
                                        : 'shadow-none')
                                    : 'bg-slate-200/50 dark:bg-[var(--ui-surface-2)] border-slate-300 dark:border-[var(--ui-border-subtle)] shadow-none'
                            )}
                            style={{
                                left: leftPos,
                                transform,
                                ...(isFilled ? { backgroundColor: scaleColor, borderColor: scaleColor } : {}),
                            }}
                            title={`Checkpoint ${cp}%`}
                        >
                            {isFilled ? (
                                isCritical && cp <= 25 ? (
                                    <span className={cn("font-black font-mono text-white leading-none tracking-tighter select-none", compact ? "text-[6px]" : "text-[7.5px]")}>
                                        {Math.round(clamped)}%
                                    </span>
                                ) : (
                                    <svg
                                        className={cn("fill-none stroke-current text-white stroke-[2.5]", compact ? "w-1.5 h-1.5" : "w-2 h-2")}
                                        viewBox="0 0 12 12"
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                    >
                                        <path d="M2.5 6.5L4.8 8.8L9.5 3.5" />
                                    </svg>
                                )
                            ) : (
                                <span className="w-1 h-1 rounded-full bg-slate-400/40 dark:bg-white/20" />
                            )}
                        </div>
                    );
                })}
            </div>

            {/* Single Column for Remaining Time and Percentage */}
            <div className={cn("flex flex-col items-end shrink-0 w-[18%] max-w-[18%]", compact ? "leading-none" : "leading-tight")}>
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
