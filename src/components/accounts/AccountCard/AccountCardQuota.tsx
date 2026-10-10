import { Ban, Clock, Lock } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { Account } from '../../../types/account';
import { cn } from '../../../utils/cn';
import { QuotaItem } from '../QuotaItem';
import { getModelQuotaDisplay } from '../../../utils/quotaDisplay';
import { getLiveLimitForModel } from '../../../utils/liveLimit';
import type { DisplayModel, WeeklyItem } from './types';

interface AccountCardQuotaProps {
    account: Account;
    isDisabled: boolean;
    validationBlockedLabel: string;
    displayModels: DisplayModel[];
    weeklyItems: WeeklyItem[];
    isModelProtected: (key?: string) => boolean;
    onViewError: () => void;
}

/**
 * Quota display section: blocked-state banner or model quota list.
 * Extracted from AccountCard to keep the component under the 500-line limit.
 */
export function AccountCardQuota({
    account,
    isDisabled,
    validationBlockedLabel,
    displayModels,
    weeklyItems,
    isModelProtected,
    onViewError,
}: AccountCardQuotaProps) {
    const { t } = useTranslation();

    return (
        <div className="flex-1 px-1.5 mb-1.5 overflow-y-auto scrollbar-thin scrollbar-thumb-slate-200 dark:scrollbar-thumb-[#15334d]/60 scrollbar-track-transparent">
            {isDisabled || account.quota?.is_forbidden || account.proxy_disabled || account.validation_blocked ? (
                <div className="flex flex-wrap items-center justify-center gap-x-3 gap-y-1 h-full py-4 text-center">
                    <div className={cn(
                        "flex items-center gap-1.5",
                        account.validation_blocked ? "text-amber-600 dark:text-amber-400" : "text-red-600 dark:text-red-400"
                    )}>
                        {account.validation_blocked ? <Clock className="w-4 h-4" /> : (isDisabled || account.proxy_disabled ? <Ban className="w-4 h-4" /> : <Lock className="w-4 h-4" />)}
                        <span className="text-[11px] font-bold">
                            {account.validation_blocked ? validationBlockedLabel : (isDisabled ? t('accounts.status.disabled') : account.proxy_disabled ? t('accounts.status.proxy_disabled') : t('accounts.forbidden_msg'))}
                        </span>
                    </div>
                    <div className={cn(
                        "w-px h-3 hidden sm:block",
                        account.validation_blocked ? "bg-amber-200 dark:bg-amber-800/50" : "bg-red-200 dark:bg-red-800/50"
                    )} />
                    <button
                        onClick={(e) => { e.stopPropagation(); onViewError(); }}
                        className="text-[10px] text-blue-600 dark:text-blue-400 hover:underline font-medium"
                    >
                        {t('accounts.view_error')}
                    </button>
                </div>
            ) : (
                <div className="grid grid-cols-1 gap-2 content-start">
                    {displayModels[0] ? (
                        <QuotaItem
                            key={displayModels[0].id}
                            label="4h"
                            percentage={displayModels[0].data.percentage}
                            resetTime={displayModels[0].data.reset_time}
                            Icon={displayModels[0].Icon}
                        />
                    ) : null}
                    {weeklyItems[0] ? (
                        <QuotaItem
                            key={weeklyItems[0].id}
                            label="Weekly"
                            percentage={weeklyItems[0].percentage}
                            resetTime={weeklyItems[0].resetTime}
                            Icon={weeklyItems[0].Icon}
                        />
                    ) : (
                        displayModels.slice(1).map((model) => (
                            <QuotaItem
                                key={model.id}
                                label={model.label}
                                {...getModelQuotaDisplay(model.id, model.data, account.quota?.quota_groups)}
                                isProtected={isModelProtected(model.protectedKey)}
                                liveLimit={getLiveLimitForModel(account, model.id, model.protectedKey)}
                                Icon={model.Icon}
                            />
                        ))
                    )}
                </div>
            )}
        </div>
    );
}
