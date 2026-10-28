import { cn } from '../../utils/cn';
import { PortalTooltip } from '../common/PortalTooltip';
import { formatTimeRemaining } from '../../utils/format';

export interface DualLayerQuotaProgressBarProps {
    fourHourPercentage: number;
    weeklyPercentage: number;
    fourHourReset?: string;
    weeklyReset?: string;
    className?: string;
    showLabels?: boolean;
}

export function clampPercentage(val: number): number {
    if (!Number.isFinite(val)) return 0;
    return Math.min(100, Math.max(0, val));
}

export function formatQuotaTooltipText(
    fourHourPct: number,
    weeklyPct: number,
    fourHourReset?: string,
    weeklyReset?: string
): string {
    const formatted4h = Math.round(fourHourPct);
    const formattedWeekly = Math.round(weeklyPct);
    const reset4hStr = fourHourReset ? ` (Resets in ${formatTimeRemaining(fourHourReset)})` : '';
    const resetWeeklyStr = weeklyReset ? ` (Resets in ${formatTimeRemaining(weeklyReset)})` : '';
    return `4h Quota: ${formatted4h}%${reset4hStr} | Weekly Quota: ${formattedWeekly}%${resetWeeklyStr}`;
}

function renderTopBar(percentage: number) {
    return (
        <div className="w-full bg-slate-200 dark:bg-slate-700/60 rounded-full h-1.5 overflow-hidden">
            <div
                className="bg-emerald-500 rounded-full h-1.5 transition-all duration-500 ease-out"
                style={{ width: `${percentage}%` }}
                role="progressbar"
                aria-valuenow={percentage}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-label="4-hour rolling quota progress"
            />
        </div>
    );
}

function renderBottomBar(percentage: number) {
    return (
        <div className="w-full bg-slate-200 dark:bg-slate-700/60 rounded-full h-1 overflow-hidden">
            <div
                className="bg-orange-500/80 rounded-full h-1 transition-all duration-500 ease-out"
                style={{ width: `${percentage}%` }}
                role="progressbar"
                aria-valuenow={percentage}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-label="Weekly reset quota progress"
            />
        </div>
    );
}

export function DualLayerQuotaProgressBar({
    fourHourPercentage,
    weeklyPercentage,
    fourHourReset,
    weeklyReset,
    className,
    showLabels = false,
}: DualLayerQuotaProgressBarProps) {
    const clamped4h = clampPercentage(fourHourPercentage);
    const clampedWeekly = clampPercentage(weeklyPercentage);
    const tooltipText = formatQuotaTooltipText(clamped4h, clampedWeekly, fourHourReset, weeklyReset);

    return (
        <PortalTooltip content={tooltipText} placement="top">
            <div
                className={cn('w-full flex flex-col gap-1 cursor-help py-0.5', className)}
                title={tooltipText}
            >
                {showLabels && (
                    <div className="flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400">
                        <span>4h: {Math.round(clamped4h)}%</span>
                        <span>Weekly: {Math.round(clampedWeekly)}%</span>
                    </div>
                )}
                {renderTopBar(clamped4h)}
                {renderBottomBar(clampedWeekly)}
            </div>
        </PortalTooltip>
    );
}

export default DualLayerQuotaProgressBar;
