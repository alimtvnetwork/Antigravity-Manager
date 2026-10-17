import { useState } from 'react';
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
    FastForward,
} from 'lucide-react';
import { cn } from '../../../../utils/cn';
import { useAccountStore } from '../../../../stores/useAccountStore';
import { useInstanceStore } from '../../../../stores/useInstanceStore';
import { useErrorStore } from '../../../../stores/error-store';
import { showToast } from '../../../common/ToastContainer';
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

    const [isFastForwarding, setIsFastForwarding] = useState(false);

    const handleFastForward = async (e: React.MouseEvent) => {
        e.stopPropagation();
        if (isFastForwarding || isDisabled) return;
        setIsFastForwarding(true);
        try {
            const targetInst = activeInstanceId || 'default';
            const msg = await useInstanceStore.getState().fastForwardInstance(targetInst);
            showToast(msg || t('instances.rotated_success', 'Rotated to next best profile!'), 'success');
        } catch (err: any) {
            useErrorStore.getState().trackWarning(err, { source: 'ActionsCell.handleFastForward', triggerAction: 'fast_forward' });
            showToast(err?.message || String(err) || t('common.error'), 'error');
        } finally {
            setIsFastForwarding(false);
        }
    };

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
        : "bg-white dark:bg-[#081826]"
)}>
    <div className="flex items-center justify-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
        {/* 1. Fast-Forward */}
        <button
            type="button"
            role="button"
            aria-label={t('instances.fast_forward', 'Fast-Forward')}
            className={cn(
                "p-1 rounded-[5px] transition-all",
                (isFastForwarding || isDisabled)
                    ? "cursor-not-allowed text-gray-400 dark:text-gray-600 opacity-40"
                    : "text-gray-600 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-200 hover:text-blue-600 dark:hover:text-cyan-400 hover:bg-slate-200/80 dark:hover:bg-slate-700/80"
            )}
            onClick={handleFastForward}
            title={t('instances.fast_forward_tooltip', 'Fast-Forward: Rotate instance to next best profile')}
            disabled={isFastForwarding || isDisabled}
        >
            <FastForward className={cn("w-3.5 h-3.5", isFastForwarding && "animate-spin")} />
        </button>

        {/* 2. Refresh */}
        <button
            type="button"
            role="button"
            aria-label={t('common.refresh', 'Refresh')}
            className={cn(
                "p-1 rounded-[5px] transition-all",
                (isRefreshing || isDisabled)
                    ? "cursor-not-allowed text-gray-400 dark:text-gray-600 opacity-40"
                    : "text-gray-600 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-200 hover:text-blue-600 dark:hover:text-cyan-400 hover:bg-slate-200/80 dark:hover:bg-slate-700/80"
            )}
            onClick={(event) => { event.stopPropagation(); onRefresh(); }}
            title={t('common.refresh')}
            disabled={isRefreshing || isDisabled}
        >
            <RefreshCw className={cn("w-3.5 h-3.5", isRefreshing && "animate-spin")} />
        </button>

        {/* 3. Switch (Riseup Yellow 3D gradient, flattened when row idle) */}
        <div className="relative inline-flex items-center" ref={menuRef}>
            <button
                type="button"
                role="button"
                aria-label={t('accounts.switch_account', 'Switch Account')}
                className={cn(
                    "p-1 rounded-[5px] transition-all font-medium inline-flex items-center justify-center",
                    (isSwitching || isDisabled)
                        ? "bg-yellow-200/40 text-slate-400 border border-yellow-300/40 cursor-not-allowed opacity-40"
                        : cn(
                            "bg-transparent opacity-40 group-hover:opacity-100 border border-yellow-500/40 text-yellow-700 dark:text-yellow-300",
                            "group-hover:bg-gradient-to-b group-hover:from-[#FDE047] group-hover:via-[#FACC15] group-hover:to-[#EAB308] group-hover:text-slate-950 group-hover:font-bold group-hover:border-[#CA8A04] group-hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]",
                            "hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:text-slate-950 hover:font-bold hover:border-[#CA8A04] hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]",
                            "active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]"
                        )
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

        {/* 4. Delete (flattened when idle, red highlight on hover) */}
        <button
            type="button"
            role="button"
            aria-label={t('common.delete', 'Delete Account')}
            className={cn(
                "p-1 rounded-[5px] transition-all font-medium border inline-flex items-center justify-center",
                isDeleting
                    ? "bg-red-300/40 text-white/50 border-red-400/40 cursor-not-allowed opacity-40"
                    : cn(
                        "bg-transparent opacity-40 group-hover:opacity-100 border-transparent text-gray-400 dark:text-gray-500",
                        "group-hover:bg-red-500/10 group-hover:text-red-500 group-hover:border-red-500/30",
                        "hover:!bg-red-600 hover:!text-white hover:!border-red-700 hover:shadow-sm active:scale-95"
                    )
            )}
            onClick={(e) => { e.stopPropagation(); if (!isDeleting) onDelete(); }}
            title={t('common.delete')}
            disabled={isDeleting}
        >
            <Trash2 className="w-3.5 h-3.5" />
        </button>

        {/* 5. More actions dropdown */}
        <button
            ref={moreBtnRef}
            type="button"
            role="button"
            aria-label={t('common.more', 'More actions')}
            className="p-1 text-gray-600 dark:text-gray-400 group-hover:text-slate-800 dark:group-hover:text-slate-200 hover:text-slate-950 dark:hover:text-white hover:bg-slate-200/80 dark:hover:bg-slate-700/80 rounded-[5px] transition-all"
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
                {onUpdateLabel && (
                    <button
                        type="button"
                        role="button"
                        aria-label={t('accounts.edit_label', 'Edit Label')}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                        onClick={() => {
                            setShowMoreMenu(false);
                            setIsEditingLabel(true);
                        }}
                    >
                        <Tag className="w-3.5 h-3.5 text-orange-500" />
                        {t('accounts.edit_label', 'Edit Label')}
                    </button>
                )}
                {onWarmup && (
                    <button
                        type="button"
                        role="button"
                        aria-label={t('accounts.warmup_this', 'Warm up Account')}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                        disabled={isRefreshing || isDisabled}
                        onClick={() => {
                            setShowMoreMenu(false);
                            onWarmup();
                        }}
                    >
                        <Sparkles className="w-3.5 h-3.5 text-amber-500" />
                        {t('accounts.warmup_this', 'Warm up Account')}
                    </button>
                )}
                {onToggleProxy && (
                    <button
                        type="button"
                        role="button"
                        aria-label={account.proxy_disabled ? t('accounts.enable_proxy', 'Enable Proxy') : t('accounts.disable_proxy', 'Disable Proxy')}
                        className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs hover:bg-slate-100"
                        onClick={() => {
                            setShowMoreMenu(false);
                            onToggleProxy();
                        }}
                    >
                        {account.proxy_disabled ? (
                            <>
                                <ToggleRight className="w-3.5 h-3.5 text-cyan-500" />
                                {t('accounts.enable_proxy', 'Enable Proxy')}
                            </>
                        ) : (
                            <>
                                <ToggleLeft className="w-3.5 h-3.5 text-orange-500" />
                                {t('accounts.disable_proxy', 'Disable Proxy')}
                            </>
                        )}
                    </button>
                )}
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
    </div>
</td>
    );
}
