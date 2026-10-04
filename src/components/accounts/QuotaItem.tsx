import { AlertTriangle, Clock, Lock } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { formatTimeRemaining, getTimeRemainingColor } from '../../utils/format';
import type { LiveLimitStatus } from '../../types/account';
import { formatCompactDuration, getLiveLimitState } from '../../utils/liveLimit';
import { WaterDrainProgressBar } from '../common/WaterDrainProgressBar';

interface QuotaItemProps {
    label?: string;
    percentage: number;
    resetTime?: string;
    isProtected?: boolean;
    liveLimit?: LiveLimitStatus;
    isWeeklyConstrained?: boolean;
    weeklyResetTime?: string;
    weeklyTokens?: number | null;
    className?: string;
    Icon?: React.ComponentType<{ size?: number; className?: string }>;
}

export function QuotaItem({
    label = '',
    percentage,
    resetTime,
    isProtected,
    liveLimit,
    isWeeklyConstrained,
    weeklyResetTime,
    className,
    Icon,
}: QuotaItemProps) {
    const { t } = useTranslation();
    const liveState = getLiveLimitState(liveLimit);
    const showLiveIssue = liveState.shouldShow || isWeeklyConstrained;
    const isUnavailable = liveState.isActive || isWeeklyConstrained;
    const liveStatus = isWeeklyConstrained ? t('accounts.quota_window_weekly_short', 'Weekly') : liveLimit?.status || 'ERR';
    const liveLimitTitle = isWeeklyConstrained
        ? `${label || t('accounts.table.weekly_quota', 'Weekly')}: ${t('accounts.weekly_exhausted_tooltip', 'Weekly quota exhausted (0%); waiting for weekly reset')} (${weeklyResetTime ? formatTimeRemaining(weeklyResetTime) || weeklyResetTime : ''})`
        : liveLimit
        ? [
            liveState.isActive
                ? `Live image endpoint is temporarily unavailable for ${formatCompactDuration(liveState.secondsRemaining)}.`
                : `Image endpoint returned ${liveStatus} ${formatCompactDuration(liveState.secondsAgo)} ago.`,
            `Reason: ${liveLimit.reason}.`,
            `Quota snapshot can still show ${percentage}%.`,
            liveLimit.message ? `Message: ${liveLimit.message}` : null,
        ].filter(Boolean).join(' ')
        : label;

    // Upgraded high-contrast text colors for Light Mode (>= 4.5:1 WCAG)
    const getTextColorClass = (p: number) => {
        if (p >= 50) return 'text-teal-700 dark:text-cyan-400';
        if (p >= 20) return 'text-amber-700 dark:text-amber-400';
        return 'text-rose-700 dark:text-rose-400';
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

    return (
        <div className="min-w-0">
            <div
                className={cn(
                    "relative h-[22px] flex items-center px-1.5 rounded-md border border-slate-200/80 dark:border-slate-800/80 bg-slate-100/90 dark:bg-[#071a27]/90 group/quota gap-1.5",
                    showLiveIssue && "border-amber-400/70 dark:border-amber-500/70 bg-amber-50/80 dark:bg-amber-950/30 ring-1 ring-amber-400/30",
                    isUnavailable && "border-rose-400/70 dark:border-rose-500/70 bg-rose-50/80 dark:bg-rose-950/30 ring-rose-400/30",
                    className
                )}
                title={showLiveIssue ? liveLimitTitle : label}
            >
                {/* Content */}
                <div className="relative z-10 w-full flex items-center text-[10px] font-mono leading-none gap-1.5">
                    {/* Model Icon & Label (suppressed when label is empty and no icon/issue) */}
                    {(label || Icon || showLiveIssue) && (
                        <span
                            className={cn(
                                "min-w-0 text-gray-800 dark:text-slate-100 font-bold truncate text-left flex items-center gap-1 shrink-0",
                                label ? "max-w-[70px]" : "",
                                showLiveIssue && "text-amber-700 dark:text-amber-300",
                                isUnavailable && "text-rose-700 dark:text-rose-300"
                            )}
                            title={showLiveIssue ? liveLimitTitle : label}
                        >
                            {showLiveIssue && (
                                <AlertTriangle
                                    size={12}
                                    className={cn(
                                        "shrink-0",
                                        isUnavailable ? "text-rose-500" : "text-amber-500"
                                    )}
                                />
                            )}
                            {Icon && <Icon size={12} className="shrink-0" />}
                            {label ? <span className="truncate">{label}</span> : null}
                        </span>
                    )}

                    {/* Inline WaterDrainProgressBar (expands to fill available width) */}
                    <div className="flex-1 min-w-[48px] flex items-center px-1">
                        <WaterDrainProgressBar percentage={percentage} />
                    </div>

                    {/* Right-aligned Stats Container: Countdown ETA + Percentage */}
                    <div className="flex items-center gap-1.5 shrink-0 justify-end ml-auto">
                        {resetTime ? (
                            <span className={cn("flex items-center gap-0.5 font-medium transition-colors text-[9px] font-mono", getTimeColorClass(resetTime))}>
                                <Clock className="w-2.5 h-2.5 shrink-0" />
                                {formatTimeRemaining(resetTime)}
                            </span>
                        ) : null}

                        <span
                            className={cn(
                                "text-right font-bold transition-colors flex items-center justify-end gap-0.5 min-w-[28px] text-[10px]",
                                showLiveIssue
                                    ? (isUnavailable ? "text-rose-700 dark:text-rose-300" : "text-amber-700 dark:text-amber-300")
                                    : getTextColorClass(percentage)
                            )}
                        >
                            {isProtected && (
                                <span title={t('accounts.quota_protected')}>
                                    <Lock className="w-2.5 h-2.5 text-amber-500" />
                                </span>
                            )}
                            {showLiveIssue && (
                                <span
                                    className={cn(
                                        "rounded px-1 py-[1px] text-[9px] leading-none",
                                        isUnavailable
                                            ? "bg-rose-500/15 text-rose-700 dark:text-rose-300"
                                            : "bg-amber-500/15 text-amber-700 dark:text-amber-300"
                                    )}
                                >
                                    {liveStatus}
                                </span>
                            )}
                            {percentage}%
                        </span>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default QuotaItem;
