import { createPortal } from 'react-dom';
import {
    ArrowRightLeft,
    RefreshCw,
    Trash2,
    Download,
    Fingerprint,
    Info,
    ToggleLeft,
    ToggleRight,
    Sparkles,
    Tag,
    Repeat2,
    Terminal,
    MoreHorizontal,
    Ban,
} from 'lucide-react';
import { cn } from '../../../../utils/cn';
import { useAccountStore } from '../../../../stores/useAccountStore';
import { useErrorStore } from '../../../../stores/error-store';
import type { AccountRowContentProps } from '../types';
import type { AccountRowState } from '../useAccountRowState';

interface ActionsCellProps extends Pick<AccountRowContentProps,
    'account' | 'selected' | 'isCurrent' | 'isFocused' | 'isRefreshing' | 'isSwitching' | 'isDisabled' |
    'onSwitch' | 'onRefresh' | 'onViewDevice' | 'onViewDetails' | 'onExport' | 'onDelete' | 'onToggleProxy' |
    'onWarmup' | 'onUpdateLabel' | 'onUpdatePriority' | 'onViewError'
> {
    rowState: AccountRowState;
}

/**
 * Actions column: refresh, switch, details, menus, export, delete, proxy toggle.
 * Extracted from AccountRowContent to keep it under the 500-line limit.
 */
export function ActionsCell(props: ActionsCellProps) {
    const { account, selected, isCurrent, isFocused, isRefreshing, isSwitching, isDisabled } = props;
    const isDeleting = Boolean((account as any)?.isDeleting || (props as any)?.isDeleting);
    const { onSwitch, onRefresh, onViewDevice, onViewDetails, onExport, onDelete, onToggleProxy, onWarmup, onUpdateLabel } = props;
    const {
        t,
        setIsEditingLabel,
        showInstanceMenu, setShowInstanceMenu,
        showMoreMenu, setShowMoreMenu,
        showEmail, setShowEmail,
        moreMenuPos,
        instances, activeInstanceId,
        menuRef, moreBtnRef, moreMenuRef,
        openMoreMenu,
        weeklyCell,
    } = props.rowState;

    return (
<td className={cn(
    "px-2 py-0.5 sticky right-0 z-10 w-[220px] xl:w-[280px] shadow-[-12px_0_12px_-12px_rgba(0,0,0,0.1)] dark:shadow-[-12px_0_12px_-12px_rgba(255,255,255,0.05)] text-center align-middle transition-colors border-b border-slate-200/90 dark:border-slate-800/90",
    // 动态高对比高亮处理 (100% 不透明，防止横向滚动文字透底重叠)
    isFocused
        ? "bg-teal-50 dark:bg-[#171025]"
        : selected
        ? "bg-blue-50 dark:bg-[#171025]"
        : isCurrent
        ? "bg-blue-50 dark:bg-[#091b2c]"
        : "bg-white dark:bg-[#081826]",
    "group-hover:!bg-white dark:group-hover:!bg-white"
)}>
    <div className="flex items-center justify-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
        <button
            type="button"
            role="button"
            aria-label={t('common.refresh', 'Refresh')}
            className={cn(
                "p-1 rounded-[5px] transition-all",
                (isRefreshing || isDisabled)
                    ? "cursor-not-allowed text-gray-400 dark:text-gray-600"
                    : "text-gray-600 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-800 hover:text-blue-600 dark:hover:text-cyan-400 hover:bg-slate-200/80 dark:hover:bg-slate-200/80"
            )}
            onClick={(event) => { event.stopPropagation(); onRefresh(); }}
            title={t('common.refresh')}
            disabled={isRefreshing || isDisabled}
        >
            <RefreshCw className={cn("w-3.5 h-3.5", isRefreshing && "animate-spin")} />
        </button>
        <div className="relative inline-flex items-center" ref={menuRef}>
            <button
                type="button"
                role="button"
                aria-label={t('accounts.switch_account', 'Switch Account')}
                className={cn(
                    "p-1 rounded-[5px] transition-all font-medium shadow-sm active:scale-95 border inline-flex items-center justify-center",
                    (isSwitching || isDisabled)
                        ? "bg-yellow-200/60 text-slate-500 border-yellow-300/60 cursor-not-allowed opacity-60"
                        : "bg-yellow-400 text-slate-900 border-yellow-500 hover:bg-yellow-300 hover:text-black hover:border-yellow-600"
                )}
                onClick={(e) => {
                    e.stopPropagation();
                    const targetInst = activeInstanceId || 'default';
                    onSwitch(`instance:${targetInst}`);
                }}
                onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    setShowInstanceMenu(!showInstanceMenu);
                }}
                title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : (() => {
                    const activeInstName = instances.find((i) => i.config.id === activeInstanceId)?.config.name || activeInstanceId || 'Default';
                    return `Switch account for instance "${activeInstName}" (Right-click to choose instance)`;
                })())}
                disabled={isSwitching || isDisabled}
            >
                <ArrowRightLeft className={cn("w-3.5 h-3.5", isSwitching && "animate-spin")} />
            </button>

            {showInstanceMenu && (
                <div className="absolute top-full left-0 mt-1 w-52 rounded-xl shadow-xl bg-white dark:bg-base-200 border border-gray-200 dark:border-base-100 py-1.5 z-50 animate-in fade-in zoom-in-95">
                    <div className="px-3 py-1 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
                        目标实例 / Target Instance
                    </div>
                    {instances.map((inst) => (
                        <button
                            type="button"
                            role="button"
                            aria-label={`Switch to instance ${inst.config.name}`}
                            key={inst.config.id}
                            onClick={(e) => {
                                e.stopPropagation();
                                setShowInstanceMenu(false);
                                onSwitch(`instance:${inst.config.id}`);
                            }}
                            className="w-full flex items-center justify-between px-3 py-1.5 text-xs text-left hover:bg-gray-50 dark:hover:bg-base-100 text-gray-700 dark:text-gray-300"
                        >
                            <div className="flex items-center gap-1.5 truncate">
                                <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${inst.is_running ? 'bg-teal-500' : 'bg-gray-400'}`} />
                                <span className="truncate">{inst.config.name}</span>
                            </div>
                            {inst.config.id === activeInstanceId && (
                                <span className="text-[10px] text-blue-600 font-medium">Active</span>
                            )}
                        </button>
                    ))}
                </div>
            )}
        </div>

        <button
            ref={moreBtnRef}
            type="button"
            role="button"
            aria-label={t('common.more', 'More actions')}
            className="p-1 text-gray-600 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-800 hover:text-slate-950 hover:bg-slate-200/80 dark:hover:bg-slate-200/80 rounded-[5px] transition-all"
            onClick={(event) => {
                event.stopPropagation();
                if (showMoreMenu) {
                    setShowMoreMenu(false);
                    return;
                }
                openMoreMenu();
            }}
            title={t('common.more', 'More')}
        >
            <MoreHorizontal className="w-3.5 h-3.5" />
        </button>
        {showMoreMenu && moreMenuPos && createPortal(
            <div
                ref={moreMenuRef}
                className="fixed z-[10000] min-w-[168px] rounded-[5px] border border-gray-200 bg-white py-1 text-slate-950 shadow-xl"
                style={{ top: moreMenuPos.top, left: moreMenuPos.left, transform: 'translateX(-100%)' }}
                onClick={(event) => event.stopPropagation()}
            >
                <button
                    type="button"
                    role="button"
                    aria-label={account.disabled ? t('accounts.enable_account', 'Enable Account') : t('accounts.disable_account', 'Disable Account')}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    onClick={async () => {
                        setShowMoreMenu(false);
                        try {
                            const store = useAccountStore.getState() as any;
                            if (typeof store.toggleAccountDisabled === 'function') {
                                await store.toggleAccountDisabled(account.id, !account.disabled);
                            } else {
                                const { invoke } = await import('@tauri-apps/api/core');
                                await invoke('toggle_account_disabled', {
                                    accountId: account.id,
                                    disabled: !account.disabled,
                                    reason: !account.disabled ? 'Disabled by user' : undefined,
                                });
                                await store.fetchAccounts();
                            }
                        } catch (err) {
                            useErrorStore.getState().trackWarning(err, { source: 'ActionsCell.toggleAccountDisabled', triggerAction: 'toggle_account_disabled' });
                        }
                    }}
                >
                    <Ban className="w-3.5 h-3.5" />
                    {account.disabled ? t('accounts.enable_account', 'Enable Account') : t('accounts.disable_account', 'Disable Account')}
                </button>
                <button
                    type="button"
                    role="button"
                    aria-label={t('accounts.switch_to_ide', 'IDE switch')}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    disabled={isSwitching || isDisabled}
                    onClick={() => { setShowMoreMenu(false); onSwitch('ide'); }}
                >
                    <Repeat2 className="w-3.5 h-3.5" />
                    {t('accounts.switch_to_ide', 'IDE switch')}
                </button>
                <button
                    type="button"
                    role="button"
                    aria-label={t('accounts.switch_to_agy', 'CLI switch')}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    disabled={isSwitching || isDisabled}
                    onClick={() => { setShowMoreMenu(false); onSwitch('agy'); }}
                >
                    <Terminal className="w-3.5 h-3.5" />
                    {t('accounts.switch_to_agy', 'CLI switch')}
                </button>
                <button
                    type="button"
                    role="button"
                    aria-label={t('common.details')}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    onClick={() => { setShowMoreMenu(false); onViewDetails(); }}
                >
                    <Info className="w-3.5 h-3.5" />
                    {t('common.details')}
                </button>
                <button
                    type="button"
                    role="button"
                    aria-label={t('accounts.device_fingerprint')}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    onClick={() => { setShowMoreMenu(false); onViewDevice(); }}
                >
                    <Fingerprint className="w-3.5 h-3.5" />
                    {t('accounts.device_fingerprint')}
                </button>
                <button
                    type="button"
                    role="button"
                    aria-label={t('common.export')}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    onClick={() => { setShowMoreMenu(false); onExport(); }}
                >
                    <Download className="w-3.5 h-3.5" />
                    {t('common.export')}
                </button>
                <button
                    type="button"
                    role="button"
                    aria-label={showEmail ? t('accounts.hide_email') : t('accounts.show_email')}
                    className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                    onClick={() => setShowEmail((current) => !current)}
                >
                    {showEmail ? t('accounts.hide_email') : t('accounts.show_email')}
                </button>
                <div className="px-3 py-1.5 text-xs text-slate-600">
                    Cycle tokens: {weeklyCell.weeklyTokens ?? 0}
                </div>
            </div>,
            document.body,
        )}
        {onUpdateLabel && (
            <button
                type="button"
                role="button"
                aria-label={t('accounts.edit_label', 'Edit Label')}
                className={cn(
                    "hidden lg:inline-flex p-1 rounded-[5px] transition-all",
                    account.custom_label
                        ? "text-orange-500 hover:text-orange-600 hover:bg-orange-100/80"
                        : "text-gray-500 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-800 hover:text-orange-500 hover:bg-slate-200/80 dark:hover:bg-slate-200/80"
                )}
                onClick={(e) => { e.stopPropagation(); setIsEditingLabel(true); }}
                title={t('accounts.edit_label', 'Edit Label')}
            >
                <Tag className="w-3.5 h-3.5" />
            </button>
        )}

        {onWarmup && (
            <button
                type="button"
                role="button"
                aria-label={t('accounts.warmup_this', 'Warm up Account')}
                className={cn(
                    "hidden xl:inline-flex p-1 rounded-[5px] transition-all",
                    (isRefreshing || isDisabled)
                        ? "bg-orange-50 dark:bg-orange-900/10 text-orange-600 cursor-not-allowed"
                        : "text-gray-500 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-800 hover:text-orange-500 hover:bg-slate-200/80 dark:hover:bg-slate-200/80"
                )}
                onClick={(e) => { e.stopPropagation(); onWarmup(); }}
                title={isDisabled ? t('accounts.disabled_tooltip') : (isRefreshing ? t('common.loading') : t('accounts.warmup_this', 'Warmup Account'))}
                disabled={isRefreshing || isDisabled}
            >
                <Sparkles className={cn("w-3.5 h-3.5", isRefreshing && "animate-pulse")} />
            </button>
        )}
        <button
            type="button"
            role="button"
            aria-label={account.proxy_disabled ? t('accounts.enable_proxy', 'Enable Proxy') : t('accounts.disable_proxy', 'Disable Proxy')}
            className={cn(
                "p-1 rounded-[5px] transition-all",
                account.proxy_disabled
                    ? "text-gray-500 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-800 hover:text-cyan-600 hover:bg-slate-200/80 dark:hover:bg-slate-200/80"
                    : "text-gray-500 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-800 hover:text-orange-600 hover:bg-slate-200/80 dark:hover:bg-slate-200/80"
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
            type="button"
            role="button"
            aria-label={t('common.delete', 'Delete Account')}
            className={cn(
                "p-1 rounded-[5px] transition-all font-medium shadow-sm active:scale-95 border inline-flex items-center justify-center",
                isDeleting
                    ? "bg-red-300/60 text-white/70 border-red-400/60 cursor-not-allowed opacity-60"
                    : "bg-red-600 text-white border-red-700 hover:bg-red-700 hover:text-white hover:border-red-800"
            )}
            onClick={(e) => { e.stopPropagation(); if (!isDeleting) onDelete(); }}
            title={t('common.delete')}
            disabled={isDeleting}
        >
            <Trash2 className="w-3.5 h-3.5" />
        </button>
    </div>
</td>
    );
}
