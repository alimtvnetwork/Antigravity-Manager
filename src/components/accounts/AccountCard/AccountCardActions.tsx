import { useState, useRef, useEffect } from 'react';
import {
    ArrowRightLeft,
    RefreshCw,
    Trash2,
    Download,
    Info,
    ToggleLeft,
    ToggleRight,
    Fingerprint,
    Sparkles,
    Tag,
    X,
    Check,
    Repeat2,
    Terminal,
    MoreHorizontal,
    FastForward,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { Account } from '../../../types/account';
import { cn } from '../../../utils/cn';
import { useInstanceStore } from '../../../stores/useInstanceStore';
import { useErrorStore } from '../../../stores/error-store';
import { showToast } from '../../common/ToastContainer';

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
 * Footer action buttons: refresh, switch, delete, fast-forward, more menu.
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
    const isDeleting = Boolean((account as any)?.isDeleting);
    const [isFastForwarding, setIsFastForwarding] = useState(false);
    const [showMoreMenu, setShowMoreMenu] = useState(false);
    const moreMenuRef = useRef<HTMLDivElement>(null);
    const moreBtnRef = useRef<HTMLButtonElement>(null);

    useEffect(() => {
        if (!showMoreMenu) return;
        const handleClickOutside = (e: MouseEvent) => {
            if (
                moreMenuRef.current &&
                !moreMenuRef.current.contains(e.target as Node) &&
                moreBtnRef.current &&
                !moreBtnRef.current.contains(e.target as Node)
            ) {
                setShowMoreMenu(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [showMoreMenu]);

    const handleFastForward = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (isFastForwarding || isDisabled) return;
        setIsFastForwarding(true);
        try {
            const targetInst = useInstanceStore.getState().activeInstanceId || 'default';
            const msg = await useInstanceStore.getState().fastForwardInstance(targetInst);
            showToast(msg || t('instances.rotated_success', 'Rotated to next best profile!'), 'success');
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, {
                source: 'AccountCardActions.handleFastForward',
                triggerAction: 'fast_forward',
            });
            showToast(err?.message || String(err) || t('common.error'), 'error');
        } finally {
            setIsFastForwarding(false);
        }
    };

    return (
        <div className="flex-none flex items-center justify-center pt-2 pb-1 border-t border-slate-200/80 dark:border-slate-800/80 relative">
            {/* Label edit popup */}
            {isEditingLabel && (
                <div className="absolute inset-0 bg-white/95 dark:bg-base-100/95 rounded-xl z-20 flex items-center justify-center p-4">
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
                            type="button"
                            role="button"
                            aria-label={t('common.save', 'Save')}
                            className="p-1.5 text-cyan-600 dark:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-900/30 rounded-[5px] transition-all"
                            onClick={handleSaveLabel}
                            title={t('common.save', 'Save')}
                        >
                            <Check className="w-4 h-4" />
                        </button>
                        <button
                            type="button"
                            role="button"
                            aria-label={t('common.cancel', 'Cancel')}
                            className="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-[5px] transition-all"
                            onClick={handleCancelLabel}
                            title={t('common.cancel', 'Cancel')}
                        >
                            <X className="w-4 h-4" />
                        </button>
                    </div>
                </div>
            )}

            {/* 5-Button Segmented Capsule Toolbar */}
            <div className="flex items-center rounded-[5px] bg-slate-100 dark:bg-[#090611] border border-slate-200/80 dark:border-[#40285e] p-0.5 divide-x divide-slate-200 dark:divide-[#40285e] shadow-2xs w-full max-w-xs relative">
                {/* 1. Refresh */}
                <button
                    type="button"
                    role="button"
                    aria-label={t('common.refresh', 'Refresh')}
                    className={cn(
                        "flex-1 flex items-center justify-center py-1 rounded-l-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                        isRefreshing
                            ? "text-cyan-600 bg-cyan-50 dark:bg-cyan-900/20"
                            : "text-slate-600 dark:text-slate-400 hover:text-cyan-600 dark:hover:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-900/20"
                    )}
                    onClick={(e) => {
                        e.stopPropagation();
                        onRefresh();
                    }}
                    disabled={isRefreshing || isDisabled}
                    title={isDisabled ? t('accounts.disabled_tooltip') : t('common.refresh')}
                >
                    <RefreshCw className={cn("w-3.5 h-3.5", isRefreshing && "animate-spin")} />
                </button>

                {/* 2. Switch (Flat idle, Riseup yellow 3D gradient on hover) */}
                <button
                    type="button"
                    role="button"
                    aria-label={t('accounts.switch_to_classic', 'Switch to Antigravity (Classic)')}
                    className={cn(
                        "flex-1 flex items-center justify-center py-1 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                        "bg-transparent text-slate-600 dark:text-slate-400",
                        "hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:text-slate-950 hover:font-bold hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]",
                        "active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]"
                    )}
                    onClick={(e) => {
                        e.stopPropagation();
                        onSwitch();
                    }}
                    title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('accounts.switch_to_classic', 'Switch to Antigravity (Classic)'))}
                    disabled={isSwitching || isDisabled}
                >
                    <ArrowRightLeft className={cn("w-3.5 h-3.5", isSwitching && "animate-spin")} />
                </button>

                {/* 3. Delete (Flat idle, Red on hover) */}
                <button
                    type="button"
                    role="button"
                    aria-label={t('common.delete', 'Delete Account')}
                    className={cn(
                        "flex-1 flex items-center justify-center py-1 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                        "bg-transparent text-slate-600 dark:text-slate-400",
                        "hover:bg-red-600 hover:text-white dark:hover:bg-red-600 dark:hover:text-white active:scale-95"
                    )}
                    onClick={(e) => {
                        e.stopPropagation();
                        if (!isDeleting) onDelete();
                    }}
                    title={t('common.delete')}
                    disabled={isDeleting}
                >
                    <Trash2 className="w-3.5 h-3.5" />
                </button>

                {/* 4. Fast-Forward (Flat idle, Riseup yellow 3D gradient on hover) */}
                <button
                    type="button"
                    role="button"
                    aria-label={t('instances.fast_forward', 'Fast-Forward')}
                    className={cn(
                        "flex-1 flex items-center justify-center py-1 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                        "bg-transparent text-slate-600 dark:text-slate-400",
                        "hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:text-slate-950 hover:font-bold hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]",
                        "active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]"
                    )}
                    onClick={handleFastForward}
                    title={t('instances.fast_forward_tooltip', 'Fast-Forward: Rotate instance to next best profile')}
                    disabled={isFastForwarding || isDisabled}
                >
                    <FastForward className={cn("w-3.5 h-3.5", isFastForwarding && "animate-pulse")} />
                </button>

                {/* 5. More Actions Dropdown */}
                <div className="flex-1 relative">
                    <button
                        ref={moreBtnRef}
                        type="button"
                        role="button"
                        aria-label={t('common.more', 'More actions')}
                        className={cn(
                            "w-full flex items-center justify-center py-1 rounded-r-[4px] transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                            showMoreMenu
                                ? "bg-slate-200 dark:bg-[#40285e] text-slate-900 dark:text-white"
                                : "text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-[#40285e]"
                        )}
                        onClick={(e) => {
                            e.stopPropagation();
                            setShowMoreMenu((prev) => !prev);
                        }}
                        title={t('common.more', 'More')}
                    >
                        <MoreHorizontal className="w-3.5 h-3.5" />
                    </button>

                    {/* Popover Dropdown Menu */}
                    {showMoreMenu && (
                        <div
                            ref={moreMenuRef}
                            className="absolute right-0 bottom-full mb-1 z-30 min-w-[180px] rounded-[5px] border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] py-1 text-slate-800 dark:text-slate-200 shadow-xl text-xs"
                            onClick={(e) => e.stopPropagation()}
                        >
                            {/* Edit Label */}
                            {onUpdateLabel && (
                                <button
                                    type="button"
                                    role="button"
                                    aria-label={t('accounts.edit_label', 'Edit Label')}
                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        setShowMoreMenu(false);
                                        setIsEditingLabel(true);
                                    }}
                                >
                                    <Tag className="w-3.5 h-3.5 text-orange-500" />
                                    <span>{t('accounts.edit_label', 'Edit Label')}</span>
                                </button>
                            )}

                            {/* Switch to IDE */}
                            <button
                                type="button"
                                role="button"
                                aria-label={t('accounts.switch_to_ide', 'Switch to Antigravity IDE')}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                disabled={isSwitching || isDisabled}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setShowMoreMenu(false);
                                    onSwitch('ide');
                                }}
                            >
                                <Repeat2 className={cn("w-3.5 h-3.5 text-sky-500", isSwitching && "animate-spin")} />
                                <span>{t('accounts.switch_to_ide', 'Switch to Antigravity IDE')}</span>
                            </button>

                            {/* Switch to CLI */}
                            <button
                                type="button"
                                role="button"
                                aria-label={t('accounts.switch_to_agy', 'Switch to Antigravity CLI (agy)')}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                disabled={isSwitching || isDisabled}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setShowMoreMenu(false);
                                    onSwitch('agy');
                                }}
                            >
                                <Terminal className={cn("w-3.5 h-3.5 text-teal-500", isSwitching && "animate-spin")} />
                                <span>{t('accounts.switch_to_agy', 'Switch to Antigravity CLI (agy)')}</span>
                            </button>

                            {/* Details */}
                            <button
                                type="button"
                                role="button"
                                aria-label={t('common.details')}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setShowMoreMenu(false);
                                    onViewDetails();
                                }}
                            >
                                <Info className="w-3.5 h-3.5 text-blue-500" />
                                <span>{t('common.details')}</span>
                            </button>

                            {/* Device Fingerprint */}
                            <button
                                type="button"
                                role="button"
                                aria-label={t('accounts.device_fingerprint')}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setShowMoreMenu(false);
                                    onViewDevice();
                                }}
                            >
                                <Fingerprint className="w-3.5 h-3.5 text-indigo-500" />
                                <span>{t('accounts.device_fingerprint')}</span>
                            </button>

                            {/* Warmup */}
                            {onWarmup && (
                                <button
                                    type="button"
                                    role="button"
                                    aria-label={t('accounts.warmup_this', 'Warm up this account')}
                                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled={isRefreshing || isDisabled}
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        setShowMoreMenu(false);
                                        onWarmup();
                                    }}
                                >
                                    <Sparkles className={cn("w-3.5 h-3.5 text-amber-500", isRefreshing && "animate-pulse")} />
                                    <span>{t('accounts.warmup_this', 'Warm up Account')}</span>
                                </button>
                            )}

                            {/* Export */}
                            <button
                                type="button"
                                role="button"
                                aria-label={t('common.export')}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setShowMoreMenu(false);
                                    onExport();
                                }}
                            >
                                <Download className="w-3.5 h-3.5 text-slate-500" />
                                <span>{t('common.export')}</span>
                            </button>

                            {/* Proxy Toggle */}
                            <button
                                type="button"
                                role="button"
                                aria-label={account.proxy_disabled ? t('accounts.enable_proxy') : t('accounts.disable_proxy')}
                                className="flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-[#40285e] text-slate-700 dark:text-slate-200 cursor-pointer"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setShowMoreMenu(false);
                                    onToggleProxy();
                                }}
                            >
                                {account.proxy_disabled ? (
                                    <>
                                        <ToggleRight className="w-3.5 h-3.5 text-cyan-500" />
                                        <span>{t('accounts.enable_proxy')}</span>
                                    </>
                                ) : (
                                    <>
                                        <ToggleLeft className="w-3.5 h-3.5 text-orange-500" />
                                        <span>{t('accounts.disable_proxy')}</span>
                                    </>
                                )}
                            </button>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
