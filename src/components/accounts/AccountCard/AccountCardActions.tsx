import { ArrowRightLeft, RefreshCw, Trash2, Download, Info, ToggleLeft, ToggleRight, Fingerprint, Sparkles, Tag, X, Check, Repeat2, Terminal } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { Account } from '../../../types/account';
import { cn } from '../../../utils/cn';

interface AccountCardActionsProps {
    account: Account;
    isRefreshing: boolean;
    isSwitching: boolean;
    isDisabled: boolean;
    isEditingLabel: boolean;
    labelInput: string;
    onSwitch: (targetIde?: string) => void;
    onRefresh: () => void;
    onViewDetails: () => void;
    onViewDevice: () => void;
    onExport: () => void;
    onDelete: () => void;
    onToggleProxy: () => void;
    onWarmup?: () => void;
    onUpdateLabel?: (label: string) => void;
    setIsEditingLabel: (v: boolean) => void;
    setLabelInput: (v: string) => void;
    handleSaveLabel: () => void;
    handleCancelLabel: () => void;
    handleKeyDown: (e: React.KeyboardEvent) => void;
}

/**
 * Footer action buttons: refresh, switch variants, details, label edit, export, proxy, delete.
 * Extracted from AccountCard to keep the component under the 500-line limit.
 */
export function AccountCardActions({
    account,
    isRefreshing,
    isSwitching,
    isDisabled,
    isEditingLabel,
    labelInput,
    onSwitch,
    onRefresh,
    onViewDetails,
    onViewDevice,
    onExport,
    onDelete,
    onToggleProxy,
    onWarmup,
    onUpdateLabel,
    setIsEditingLabel,
    setLabelInput,
    handleSaveLabel,
    handleCancelLabel,
    handleKeyDown,
}: AccountCardActionsProps) {
    const { t } = useTranslation();

    return (
<div className="flex-none flex items-center justify-center pt-2 pb-1 border-t border-slate-200/80 dark:border-slate-800/80">
    {/* Label edit popup */}
    {isEditingLabel && (
        <div className="absolute inset-0 bg-white/95 dark:bg-base-100/95 rounded-xl z-10 flex items-center justify-center p-4">
            <div className="flex items-center gap-2 w-full max-w-xs">
                <input
                    type="text"
                    className="flex-1 px-2 py-1 text-sm border border-orange-300 dark:border-orange-700 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500 bg-white dark:bg-base-200"
                    placeholder={t('accounts.custom_label_placeholder', 'Enter custom label')}
                    value={labelInput}
                    onChange={(e) => setLabelInput(e.target.value)}
                    onKeyDown={handleKeyDown}
                    autoFocus
                    maxLength={15}
                />
                <button
                    className="p-1.5 text-cyan-600 dark:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-900/30 rounded-[5px] transition-all"
                    onClick={handleSaveLabel}
                    title={t('common.save', 'Save')}
                >
                    <Check className="w-4 h-4" />
                </button>
                <button
                    className="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-[5px] transition-all"
                    onClick={handleCancelLabel}
                    title={t('common.cancel', 'Cancel')}
                >
                    <X className="w-4 h-4" />
                </button>
            </div>
        </div>
    )}
    <div className="flex flex-wrap items-center justify-center gap-1 w-full">
        {/* 1. Refresh button (top priority) */}
        <button
            className={`p-1.5 rounded-[5px] transition-all ${isRefreshing
                ? 'text-cyan-600 bg-cyan-50 dark:bg-cyan-900/20'
                : 'text-gray-400 hover:text-cyan-600 hover:bg-cyan-50 dark:hover:bg-cyan-900/20'}`}
            onClick={(e) => { e.stopPropagation(); onRefresh(); }}
            disabled={isRefreshing || isDisabled}
            title={isDisabled ? t('accounts.disabled_tooltip') : t('common.refresh')}
        >
            <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
        </button>

        {/* 2. Switch operations group */}
        <button
            className={`p-1.5 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'text-blue-600 bg-blue-50 dark:text-blue-400 dark:bg-blue-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30'}`}
            onClick={(e) => { e.stopPropagation(); onSwitch(); }}
            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_classic', 'Switch to Antigravity (Classic)'))}
            disabled={isSwitching || isDisabled}
        >
            <ArrowRightLeft className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
        </button>
        <button
            className={`p-1.5 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'text-blue-600 bg-blue-50 dark:text-blue-400 dark:bg-blue-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-sky-600 dark:hover:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-900/30'}`}
            onClick={(e) => { e.stopPropagation(); onSwitch('ide'); }}
            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_ide', 'Switch to Antigravity IDE'))}
            disabled={isSwitching || isDisabled}
        >
            <Repeat2 className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
        </button>
        <button
            className={`p-1.5 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'text-blue-600 bg-blue-50 dark:text-blue-400 dark:bg-blue-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-cyan-600 dark:hover:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-900/30'}`}
            onClick={(e) => { e.stopPropagation(); onSwitch('agy'); }}
            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_agy', 'Switch to Antigravity CLI (agy)'))}
            disabled={isSwitching || isDisabled}
        >
            <Terminal className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
        </button>

        {/* 3. Details and auxiliary operations */}
        <button
            className="p-1.5 text-gray-400 hover:text-sky-600 dark:hover:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-900/30 rounded-[5px] transition-all"
            onClick={(e) => { e.stopPropagation(); onViewDetails(); }}
            title={t('common.details')}
        >
            <Info className="w-3.5 h-3.5" />
        </button>
        <button
            className="p-1.5 text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 rounded-[5px] transition-all"
            onClick={(e) => { e.stopPropagation(); onViewDevice(); }}
            title={t('accounts.device_fingerprint')}
        >
            <Fingerprint className="w-3.5 h-3.5" />
        </button>
        {/* Custom label button */}
        {onUpdateLabel && (
            <button
                className={cn(
                    "p-1.5 rounded-[5px] transition-all",
                    account.custom_label
                        ? "text-orange-500 hover:text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-900/30"
                        : "text-gray-400 hover:text-orange-500 hover:bg-orange-50 dark:hover:bg-orange-900/30"
                )}
                onClick={(e) => { e.stopPropagation(); setIsEditingLabel(true); }}
                title={t('accounts.edit_label', 'Edit Label')}
            >
                <Tag className="w-3.5 h-3.5" />
            </button>
        )}
        <button
            className={`p-1.5 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'text-blue-600 bg-blue-50 dark:text-blue-400 dark:bg-blue-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30'}`}
            onClick={(e) => { e.stopPropagation(); onSwitch('classic'); }}
            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_classic', 'Switch to Antigravity (Classic)'))}
            disabled={isSwitching || isDisabled}
        >
            <ArrowRightLeft className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
        </button>
        <button
            className={`p-1.5 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'text-blue-600 bg-blue-50 dark:text-blue-400 dark:bg-blue-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-sky-600 dark:hover:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-900/30'}`}
            onClick={(e) => { e.stopPropagation(); onSwitch('ide'); }}
            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_ide', 'Switch to Antigravity IDE'))}
            disabled={isSwitching || isDisabled}
        >
            <Repeat2 className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
        </button>
        <button
            className={`p-1.5 rounded-[5px] transition-all ${(isSwitching || isDisabled) ? 'text-blue-600 bg-blue-50 dark:text-blue-400 dark:bg-blue-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-cyan-600 dark:hover:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-900/30'}`}
            onClick={(e) => { e.stopPropagation(); onSwitch('agy'); }}
            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_agy', 'Switch to Antigravity CLI (agy)'))}
            disabled={isSwitching || isDisabled}
        >
            <Terminal className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
        </button>
        {onWarmup && (
            <button
                className={`p-1.5 rounded-[5px] transition-all ${(isRefreshing || isDisabled) ? 'text-orange-600 bg-orange-50 dark:bg-orange-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-orange-500 hover:bg-orange-50 dark:hover:bg-orange-900/30'}`}
                onClick={(e) => { e.stopPropagation(); onWarmup(); }}
                title={isDisabled ? t('accounts.disabled_tooltip') : (isRefreshing ? t('common.loading') : t('accounts.warmup_this', 'Warm up this account'))}
                disabled={isRefreshing || isDisabled}
            >
                <Sparkles className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-pulse' : ''}`} />
            </button>
        )}
        <button
            className="p-1.5 text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 rounded-[5px] transition-all"
            onClick={(e) => { e.stopPropagation(); onExport(); }}
            title={t('common.export')}
        >
            <Download className="w-3.5 h-3.5" />
        </button>
        <button
            className={cn(
                "p-1.5 rounded-[5px] transition-all",
                account.proxy_disabled
                    ? "text-gray-400 hover:text-cyan-600 hover:bg-cyan-50"
                    : "text-gray-400 hover:text-orange-600 hover:bg-orange-50"
            )}
            onClick={(e) => { e.stopPropagation(); onToggleProxy(); }}
            title={account.proxy_disabled ? t('accounts.enable_proxy') : t('accounts.disable_proxy')}
        >
            {account.proxy_disabled ? (
                <ToggleRight className="w-3.5 h-3.5" />
            ) : (
                <ToggleLeft className="w-3.5 h-3.5" />
            )}
        </button>
        <button
            className="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-[5px] transition-all"
            onClick={(e) => { e.stopPropagation(); onDelete(); }}
            title={t('common.delete')}
        >
            <Trash2 className="w-3.5 h-3.5" />
        </button>
    </div>
</div>
    );
}
