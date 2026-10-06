import { Gem, Diamond, Circle } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';

export interface TierBadgeProps {
    tier?: string | null | undefined;
    size?: 'sm' | 'md' | 'xs';
    className?: string;
}

export function TierBadge({ tier, size = 'sm', className }: TierBadgeProps) {
    const { t } = useTranslation();

    const normalizedTier = tier?.trim().toLowerCase();
    const isUltra = Boolean(normalizedTier?.includes('ultra'));
    const isPro = Boolean(normalizedTier?.includes('pro'));
    const isFree = Boolean(normalizedTier?.includes('free'));

    const sizeClasses = {
        xs: 'text-[9px] px-1.5 py-0.5 gap-0.5 rounded shadow-xs',
        sm: 'text-[9px] px-1.5 py-0.5 gap-1 rounded-md shadow-xs',
        md: 'text-[10px] px-2 py-0.5 gap-1 rounded-md shadow-sm',
    }[size];

    if (isUltra) {
        return (
            <span
                className={cn(
                    'inline-flex items-center font-bold bg-gradient-to-r from-purple-600 to-pink-600 text-white cursor-default select-none',
                    sizeClasses,
                    className
                )}
            >
                <Gem className="w-2.5 h-2.5 fill-current shrink-0" />
                <span>{t('accounts.ultra', 'ULTRA')}</span>
            </span>
        );
    }

    if (isPro) {
        return (
            <span
                className={cn(
                    'inline-flex items-center font-bold bg-gradient-to-r from-blue-600 to-indigo-600 text-white cursor-default select-none',
                    sizeClasses,
                    className
                )}
            >
                <Diamond className="w-2.5 h-2.5 fill-current shrink-0" />
                <span>{t('accounts.pro', 'PRO')}</span>
            </span>
        );
    }

    if (isFree) {
        return (
            <span
                className={cn(
                    'inline-flex items-center font-bold bg-gray-100 dark:bg-[#15334d] text-gray-600 dark:text-gray-400 border border-gray-200 dark:border-[#15334d] hover:bg-gray-200 dark:hover:bg-[#1a3d5c] transition-colors cursor-default select-none',
                    sizeClasses,
                    className
                )}
            >
                <Circle className="w-2.5 h-2.5 shrink-0" />
                <span>{t('accounts.free', 'FREE')}</span>
            </span>
        );
    }

    const unknownSizeClasses = {
        xs: 'text-[9px] px-1.5 py-0.5 rounded shadow-xs',
        sm: 'text-[9px] px-1.5 py-0.5 rounded-md shadow-xs',
        md: 'text-[10px] px-2 py-0.5 rounded-md shadow-sm',
    }[size];

    return (
        <span
            className={cn(
                'inline-flex items-center justify-center font-bold bg-gray-100 dark:bg-[#15334d] text-gray-500 dark:text-gray-400 border border-gray-200 dark:border-[#15334d] cursor-help select-none',
                unknownSizeClasses,
                className
            )}
            title={t('accounts.tier_not_fetched', 'Tier not fetched yet')}
        >
            ?
        </span>
    );
}

export default TierBadge;
