import { Clock } from 'lucide-react';
import { cn } from '../../utils/cn';
import { formatNormalizedResetTime, type NormalizedResetTimeResult } from '../../utils/timeFormat';
import { PortalTooltip } from '../common/PortalTooltip';

export interface TimeZoneNormalizedClockProps {
    resetTime?: string | number | Date | null;
    showIcon?: boolean;
    className?: string;
    iconClassName?: string;
}

function renderClockTooltipContent(timeData: NormalizedResetTimeResult) {
    return (
        <div className="flex flex-col gap-1 text-[11px] font-mono leading-tight py-0.5 min-w-[200px]">
            <div className="flex items-center justify-between gap-3 text-slate-300">
                <span className="text-slate-400 font-sans">Local:</span>
                <span className="text-white font-medium">{timeData.localFormatted}</span>
            </div>
            <div className="flex items-center justify-between gap-3 text-slate-300">
                <span className="text-slate-400 font-sans">UTC:</span>
                <span className="text-sky-300 font-medium">{timeData.utcFormatted}</span>
            </div>
            <div className="flex items-center justify-between gap-3 text-slate-300">
                <span className="text-slate-400 font-sans">Epoch:</span>
                <span className="text-amber-300 font-medium">{timeData.epochSecs}s</span>
            </div>
        </div>
    );
}

export function TimeZoneNormalizedClock({
    resetTime,
    showIcon = true,
    className,
    iconClassName,
}: TimeZoneNormalizedClockProps) {
    const timeData = formatNormalizedResetTime(resetTime);
    const tooltipContent = renderClockTooltipContent(timeData);

    return (
        <PortalTooltip content={tooltipContent} placement="top">
            <span
                className={cn(
                    'inline-flex items-center gap-1.5 underline decoration-dashed underline-offset-4 cursor-help',
                    'text-xs text-slate-700 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white transition-colors',
                    className
                )}
            >
                {showIcon && (
                    <Clock
                        className={cn('w-3.5 h-3.5 text-slate-400 dark:text-slate-500 shrink-0', iconClassName)}
                    />
                )}
                <span>{timeData.localFormatted}</span>
            </span>
        </PortalTooltip>
    );
}

export default TimeZoneNormalizedClock;
