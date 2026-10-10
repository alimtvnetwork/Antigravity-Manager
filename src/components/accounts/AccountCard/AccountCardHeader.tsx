import { Ban, Clock, Lock, Tag } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { Account } from '../../../types/account';
import { cn } from '../../../utils/cn';
import { formatDateTime } from '../../../utils/date';
import { TierBadge } from '../../common/TierBadge';
import { PriorityBadge } from '../PriorityBadge';
import { ACTIVE_PILL_CLASSES } from '../../common/selectedState';

interface BoundInstance {
    config: { name: string };
    is_running: boolean;
}

interface AccountCardHeaderProps {
    account: Account;
    selected: boolean;
    isCurrent: boolean;
    isFocused: boolean;
    isDisabled: boolean;
    validationBlockedLabel: string;
    boundInstance: BoundInstance | undefined;
    onSelect: () => void;
    onUpdatePriority?: (priority: number) => Promise<void> | void;
}

/**
 * Card header: checkbox, email, status badges, bound instance, timestamp.
 * Extracted from AccountCard to keep the component under the 500-line limit.
 */
export function AccountCardHeader({
    account,
    selected,
    isCurrent,
    isFocused,
    isDisabled,
    validationBlockedLabel,
    boundInstance,
    onSelect,
    onUpdatePriority,
}: AccountCardHeaderProps) {
    const { t } = useTranslation();

    return (
        <div className="flex-none flex items-start gap-3 mb-1.5">
            <input
                type="checkbox"
                className="mt-1 checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                checked={selected}
                onChange={() => onSelect()}
                onClick={(e) => e.stopPropagation()}
            />
            <div className="flex-1 min-w-0 flex flex-col gap-1.5">
                <h3 className={cn(
                    "font-semibold text-sm truncate w-full",
                    isFocused || selected
                        ? "text-blue-950 dark:text-blue-200 font-bold"
                        : isCurrent
                        ? "text-blue-900 dark:text-amber-300 font-bold"
                        : "text-gray-900 dark:text-gray-100"
                )} title={account.email}>
                    {account.email}
                </h3>
                <div className="flex items-center justify-between w-full gap-2">
                    <div className="flex items-center gap-1.5 flex-wrap">
                        {isCurrent && (
                            <span className={cn(ACTIVE_PILL_CLASSES, "text-[9px] px-1.5 py-0.2")}>
                                {t('accounts.current').toUpperCase()}
                            </span>
                        )}
                        {isDisabled && (
                            <span
                                className="px-1.5 py-0.5 rounded-md bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300 text-[9px] font-bold flex items-center gap-1 shadow-sm border border-rose-200/50"
                            >
                                <Ban className="w-2.5 h-2.5" />
                                {t('accounts.disabled').toUpperCase()}
                            </span>
                        )}
                        {account.proxy_disabled && (
                            <span
                                className="px-1.5 py-0.5 rounded-md bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 text-[9px] font-bold flex items-center gap-1 shadow-sm border border-orange-200/50"
                            >
                                <Ban className="w-2.5 h-2.5" />
                                {t('accounts.proxy_disabled').toUpperCase()}
                            </span>
                        )}
                        {account.quota?.is_forbidden && (
                            <span className="px-1.5 py-0.5 rounded-md bg-red-100 dark:bg-red-900/40 text-red-600 dark:text-red-400 text-[9px] font-bold flex items-center gap-1 shadow-sm border border-red-200/50">
                                <Lock className="w-2.5 h-2.5" />
                                {t('accounts.forbidden').toUpperCase()}
                            </span>
                        )}
                        {account.validation_blocked && (
                            <span className="px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-400 text-[9px] font-bold flex items-center gap-1 shadow-sm border border-amber-200/50">
                                <Clock className="w-2.5 h-2.5" />
                                {validationBlockedLabel.toUpperCase()}
                            </span>
                        )}
                        {/* Subscription tier badge */}
                        <TierBadge tier={account.quota?.subscription_tier} size="sm" />
                        {/* Priority */}
                        <PriorityBadge
                            priority={account.priority}
                            accountId={account.id}
                            onUpdatePriority={onUpdatePriority}
                            size="sm"
                        />
                        {/* Custom label */}
                        {account.custom_label && (
                            <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 text-[9px] font-bold shadow-sm border border-orange-200/50 dark:border-orange-800/50">
                                <Tag className="w-2.5 h-2.5" />
                                {account.custom_label}
                            </span>
                        )}
                        {/* Bound instance badge */}
                        {boundInstance && (
                            <span
                                className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-indigo-50 dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300 text-[9px] font-bold shadow-xs border border-indigo-200/50 dark:border-indigo-800/50 cursor-default"
                                title={`Bound to profile: ${boundInstance.config.name}`}
                            >
                                <span className={cn(
                                    "w-1.5 h-1.5 rounded-full shrink-0",
                                    boundInstance.is_running ? "bg-teal-500 animate-pulse" : "bg-indigo-400"
                                )} />
                                <span>{boundInstance.config.name}</span>
                            </span>
                        )}
                    </div>
                    <span className="text-[10px] text-gray-400 dark:text-gray-500 font-mono shrink-0 whitespace-nowrap">
                        {formatDateTime(account.last_used)}
                    </span>
                </div>
            </div>
        </div>
    );
}
