
import { AlertTriangle, Clock, Lock } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { getQuotaColor, formatTimeRemaining, getTimeRemainingColor } from '../../utils/format';
import type { LiveLimitStatus } from '../../types/account';
import { formatCompactDuration, getLiveLimitState } from '../../utils/liveLimit';

interface QuotaItemProps {
    label: string;
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

export function QuotaItem({ label, percentage, resetTime, isProtected, liveLimit, isWeeklyConstrained, weeklyResetTime, className, Icon }: QuotaItemProps) {
    const { t } = useTranslation();
    const liveState = getLiveLimitState(liveLimit);
    const showLiveIssue = liveState.shouldShow || isWeeklyConstrained;
    const isUnavailable = liveState.isActive || isWeeklyConstrained;
    const liveStatus = isWeeklyConstrained ? t('accounts.quota_window_weekly_short', 'Weekly') : liveLimit?.status || 'ERR';
    const liveLimitTitle = isWeeklyConstrained
        ? `${label}: ${t('accounts.weekly_exhausted_tooltip', 'Weekly quota exhausted (0%); waiting for weekly reset')} (${weeklyResetTime ? formatTimeRemaining(weeklyResetTime) || weeklyResetTime : ''})`
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
    const getBgGradientClass = (p: number) => {
        const color = getQuotaColor(p);
        switch (color) {
            case 'success':
                return 'bg-gradient-to-r from-emerald-500 via-teal-400 to-cyan-400';
            case 'warning':
                return 'bg-gradient-to-r from-amber-500 via-orange-400 to-yellow-400';
            case 'error':
                return 'bg-gradient-to-r from-rose-500 via-red-400 to-pink-500';
            default:
                return 'bg-gradient-to-r from-slate-500 to-gray-500';
        }
    };

    const getTextColorClass = (p: number) => {
        const color = getQuotaColor(p);
        switch (color) {
            case 'success': return 'text-emerald-600 dark:text-emerald-400';
            case 'warning': return 'text-amber-600 dark:text-amber-400';
            case 'error': return 'text-rose-600 dark:text-rose-400';
            default: return 'text-gray-500';
        }
    };

    const getTimeColorClass = (time?: string) => {
        if (!time) return 'text-gray-300 dark:text-gray-600';
        const color = getTimeRemainingColor(time);
        switch (color) {
            case 'success': return 'text-emerald-600 dark:text-emerald-400';
            case 'warning': return 'text-amber-600 dark:text-amber-400';
            default: return 'text-blue-600 dark:text-blue-400';
        }
    };

    return (
        <div className="min-w-0">
        <div className={cn(
            "relative h-[22px] flex items-center px-1.5 rounded-md overflow-hidden border border-gray-200/80 dark:border-[#15334d] bg-gray-50/70 dark:bg-[#071a27]/90 group/quota",
            showLiveIssue && "border-amber-400/70 dark:border-amber-500/70 bg-amber-50/80 dark:bg-amber-950/30 ring-1 ring-amber-400/30",
            isUnavailable && "border-rose-400/70 dark:border-rose-500/70 bg-rose-50/80 dark:bg-rose-950/30 ring-rose-400/30",
            className
        )}
            title={showLiveIssue ? liveLimitTitle : label}
        >
            {/* Background Progress Bar with modern multi-stop vibrant glowing gradient */}
            <div
                className={cn(
                    "absolute inset-y-0 left-0 transition-all duration-700 ease-out opacity-25 dark:opacity-35 shadow-xs",
                    showLiveIssue
                        ? (isUnavailable ? "bg-gradient-to-r from-rose-600 to-pink-500" : "bg-gradient-to-r from-amber-500 to-yellow-400")
                        : getBgGradientClass(percentage)
                )}
                style={{ width: `${percentage}%` }}
            />

            {/* Content */}
            <div className="relative z-10 w-full flex items-center text-[10px] font-mono leading-none gap-1">
                {/* Model Name */}
                <span className={cn(
                    "flex-1 min-w-0 text-gray-800 dark:text-slate-100 font-bold truncate text-left flex items-center gap-1",
                    showLiveIssue && "text-amber-700 dark:text-amber-300",
                    isUnavailable && "text-rose-700 dark:text-rose-300"
                )} title={showLiveIssue ? liveLimitTitle : label}>
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
                    {label}
                </span>

                {/* Reset Time */}
                <div className="w-[62px] flex justify-start shrink-0">
                    {resetTime ? (
                        <span className={cn("flex items-center gap-0.5 font-medium transition-colors truncate", getTimeColorClass(resetTime))}>
                            <Clock className="w-2.5 h-2.5 shrink-0" />
                            {formatTimeRemaining(resetTime)}
                        </span>
                    ) : (
                        <span className="text-gray-400 dark:text-slate-400 italic text-[9px]">N/A</span>
                    )}
                </div>

                {/* Percentage */}
                <span className={cn(
                    "text-right font-bold transition-colors flex items-center justify-end gap-0.5 shrink-0",
                    showLiveIssue ? "min-w-[58px]" : "w-[28px]",
                    showLiveIssue ? (isUnavailable ? "text-rose-700 dark:text-rose-300" : "text-amber-700 dark:text-amber-300") : getTextColorClass(percentage)
                )}>
                    {isProtected && (
                        <span title={t('accounts.quota_protected')}><Lock className="w-2.5 h-2.5 text-amber-500" /></span>
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
    );
}
