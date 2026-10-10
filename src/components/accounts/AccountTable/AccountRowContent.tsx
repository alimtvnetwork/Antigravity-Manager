import { useTranslation } from 'react-i18next';
import {
    Ban,
    Clock,
    Lock,
    Tag,
    X,
    Check,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { maskEmail } from '../../../utils/maskEmail';
import { TierBadge } from '../../common/TierBadge';
import { PriorityBadge } from '../PriorityBadge';
import { QuotaProgressBar } from '../QuotaProgressBar';
import { ACTIVE_PILL_CLASSES } from '../../common/selectedState';
import { formatDateTimeShort } from './types';
import { formatDateTime } from '../../../utils/date';
import type { AccountRowContentProps } from './types';
import { useAccountRowState } from './useAccountRowState';
import { ActionsCell } from './cells/ActionsCell';

/**
 * 账号行内容组件
 * 渲染邮箱、配额、最后使用时间和操作按钮等列
 */
export function AccountRowContent(props: AccountRowContentProps) {
    const {
        account,
        selected = false,
        isCurrent,
        isFocused = false,
        isDisabled,
        onUpdatePriority,
        onViewError,
        showPriority = false,
        showAllEmails = false,
    } = props;

    const rowState = useAccountRowState(props);
    const { t } = useTranslation();
    const {
        validationBlockedLabel,
        isEditingLabel,
        labelInput, setLabelInput,
        showEmail,
        isHoverUnmasked, setIsHoverUnmasked,
        editingPriority, setEditingPriority,
        boundInstance,
        leaseInfo,
        handleSaveLabel, handleCancelLabel, handleKeyDown,
        openPriorityEditor,
        fourHourModel, weeklyCell,
    } = rowState;

    return (
        <>
            <td
                className="px-2 py-0.5 align-middle border-b border-slate-200/90 dark:border-slate-800/90"
                onMouseLeave={() => setIsHoverUnmasked(false)}
            >
                <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5">
                    <span
                        className={cn(
                        "font-medium text-xs break-all transition-colors cursor-pointer select-text",
                        isFocused || selected
                            ? "text-blue-950 dark:text-blue-200 font-bold"
                            : isCurrent
                            ? "text-blue-900 dark:text-amber-300 font-bold drop-shadow-xs"
                            : "text-gray-900 dark:text-gray-100 group-hover:text-blue-600 dark:group-hover:text-cyan-300"
                        )}
                        title={showAllEmails || showEmail || isHoverUnmasked ? account.email : maskEmail(account.email)}
                        onDoubleClick={(event) => {
                            event.stopPropagation();
                            setIsHoverUnmasked(true);
                            openPriorityEditor();
                        }}
                    >
                        {showAllEmails || showEmail || isHoverUnmasked ? account.email : maskEmail(account.email)}
                    </span>

                    <div className="flex items-center gap-1 shrink-0">
                        {isCurrent ? (
                            <span className={cn(ACTIVE_PILL_CLASSES, "text-[9px] px-1.5 py-0.2")}>
                                {t('accounts.current').toUpperCase()}
                            </span>
                        ) : null}
                        {isDisabled ? (
                            <span
                                className="px-1.5 py-0.5 rounded-[5px] bg-rose-500/10 text-rose-600 dark:text-rose-400 border border-rose-400/20 text-[9px] font-semibold flex items-center gap-0.5 shadow-xs"
                            >
                                <Ban className="w-2.5 h-2.5" />
                                <span>{t('accounts.disabled')}</span>
                            </span>
                        ) : null}

                        {account.proxy_disabled ? (
                            <span
                                className="px-1.5 py-0.5 rounded-[5px] bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-400/20 text-[9px] font-semibold flex items-center gap-0.5 shadow-xs"
                            >
                                <Ban className="w-2.5 h-2.5" />
                                <span>{t('accounts.proxy_disabled')}</span>
                            </span>
                        ) : null}

                        {account.quota?.is_forbidden ? (
                            <span className="px-1.5 py-0.5 rounded-[5px] bg-rose-500/10 text-rose-600 dark:text-rose-400 border border-rose-400/20 text-[9px] font-semibold flex items-center gap-0.5 shadow-xs">
                                <Lock className="w-2.5 h-2.5" />
                                <span>{t('accounts.forbidden')}</span>
                            </span>
                        ) : null}
                        {account.validation_blocked ? (
                            <span className="px-1.5 py-0.5 rounded-[5px] bg-amber-500/10 text-amber-600 dark:text-amber-400 border border-amber-400/20 text-[9px] font-semibold flex items-center gap-0.5 shadow-xs">
                                <Clock className="w-2.5 h-2.5" />
                                <span>{validationBlockedLabel}</span>
                            </span>
                        ) : null}

                        {/* 订阅类型徽章 */}
                        <TierBadge tier={account.quota?.subscription_tier} size="xs" />
                        {/* 绑定实例徽章 */}
                        {boundInstance && (
                            <span
                                className="flex items-center gap-1 px-1.5 py-0.2 rounded bg-indigo-50 dark:bg-indigo-900/40 text-indigo-700 dark:text-indigo-300 text-[9px] font-bold shadow-xs border border-indigo-200/50 dark:border-indigo-800/50 cursor-default"
                                title={`Bound to profile: ${boundInstance.config.name}`}
                            >
                                <span className={cn(
                                    "w-1.5 h-1.5 rounded-full shrink-0",
                                    boundInstance.is_running ? "bg-teal-500 animate-pulse" : "bg-indigo-400"
                                )} />
                                <span>{boundInstance.config.name}</span>
                            </span>
                        )}
                        {/* 优先级徽章 */}
                        {showPriority && (
                            <PriorityBadge
                                priority={account.priority}
                                accountId={account.id}
                                onUpdatePriority={onUpdatePriority}
                                isEditing={editingPriority}
                                onEditChange={setEditingPriority}
                                size="xs"
                            />
                        )}
                        {/* 远程节点租赁徽章 */}
                        {leaseInfo && (
                            <span
                                className="flex items-center gap-1 px-1.5 py-0.2 rounded bg-purple-50 dark:bg-purple-900/40 text-purple-700 dark:text-purple-300 text-[9px] font-bold shadow-xs border border-purple-200/50 dark:border-purple-800/50 cursor-default"
                                title={`Leased by Node: ${leaseInfo.node_alias} (${leaseInfo.profile_name})`}
                            >
                                <Lock className="w-2.5 h-2.5" />
                                <span>{leaseInfo.node_alias}</span>
                            </span>
                        )}
                        {/* 自定义标签 */}
                        {account.custom_label ? (
                            isEditingLabel ? null : (
                                <span className="flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 text-[9px] font-bold shadow-xs border border-orange-200/50 dark:border-orange-800/50">
                                    <Tag className="w-2.5 h-2.5" />
                                    {account.custom_label}
                                </span>
                            )
                        ) : null}
                        {/* 标签编辑输入框 */}
                        {isEditingLabel && (
                            <div className="flex items-center gap-1">
                                <input
                                    type="text"
                                    className="px-1.5 py-0.2 text-[10px] w-20 border border-orange-300 dark:border-orange-700 rounded focus:outline-none focus:ring-1 focus:ring-orange-500 bg-white dark:bg-base-200"
                                    placeholder={t('accounts.custom_label_placeholder', 'Label')}
                                    value={labelInput}
                                    onChange={(e) => setLabelInput(e.target.value)}
                                    onKeyDown={handleKeyDown}
                                    autoFocus
                                    maxLength={15}
                                    onClick={(e) => e.stopPropagation()}
                                />
                                <button
                                    className="p-0.5 text-cyan-600 dark:text-cyan-400 hover:bg-cyan-50 dark:hover:bg-cyan-900/30 rounded-[5px] transition-all"
                                    onClick={(e) => { e.stopPropagation(); handleSaveLabel(); }}
                                >
                                    <Check className="w-3 h-3" />
                                </button>
                                <button
                                    className="p-0.5 text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-[5px] transition-all"
                                    onClick={(e) => { e.stopPropagation(); handleCancelLabel(); }}
                                >
                                    <X className="w-3 h-3" />
                                </button>
                            </div>
                        )}
                    </div>

                </div>
            </td>

            <td className="px-2 py-px align-middle min-w-[210px] bg-slate-50/50 dark:bg-slate-900/40 border-l border-slate-300 dark:border-[#15334d] border-b border-slate-200/90 dark:border-slate-800/90">
                {isDisabled || account.quota?.is_forbidden || account.validation_blocked ? (
                    <div className={cn(
                        "flex items-center justify-center gap-1.5 py-0.5 px-2 rounded-md border group/error",
                        account.validation_blocked ? "bg-amber-50/50 dark:bg-amber-900/10 border-amber-100/50 dark:border-amber-900/20" : "bg-red-50/50 dark:bg-red-900/10 border-red-100/50 dark:border-red-900/20"
                    )}>
                        <div className={cn(
                            "flex items-center gap-1",
                            account.validation_blocked ? "text-amber-600 dark:text-amber-400" : "text-red-600 dark:text-red-400"
                        )}>
                            {account.validation_blocked ? <Clock className="w-2.5 h-2.5" /> : (account.quota?.is_forbidden ? <Lock className="w-2.5 h-2.5" /> : <Ban className="w-2.5 h-2.5" />)}
                            <span className={cn(
                                "text-[9px] font-bold truncate max-w-[80px]",
                                account.validation_blocked ? "text-amber-700/80 dark:text-amber-400" : "text-red-700/80 dark:text-red-400"
                            )}>
                                {account.validation_blocked ? validationBlockedLabel : (isDisabled ? t('accounts.status.disabled') : t('accounts.forbidden_msg'))}
                            </span>
                        </div>
                        <div className={cn(
                            "w-px h-2.5",
                            account.validation_blocked ? "bg-amber-200 dark:bg-amber-800/50" : "bg-red-200 dark:bg-red-800/50"
                        )} />
                        <button
                            onClick={(e) => { e.stopPropagation(); onViewError(); }}
                            className="text-[9px] font-medium text-blue-600 dark:text-blue-400 hover:underline flex items-center shrink-0"
                        >
                            {t('accounts.view_error')}
                        </button>
                    </div>
                ) : (
                    <QuotaProgressBar
                        compact
                        percentage={fourHourModel?.percentage ?? 0}
                        resetTime={fourHourModel?.resetTime}
                        isProtected={fourHourModel?.isProtected}
                        liveLimit={fourHourModel?.liveLimit}
                    />
                )}
            </td>

            <td className="px-2 py-px align-middle min-w-[210px] bg-slate-50/50 dark:bg-slate-900/40 border-r border-slate-300 dark:border-[#15334d] border-b border-slate-200/90 dark:border-slate-800/90">
                {isDisabled || account.quota?.is_forbidden || account.validation_blocked ? (
                    <span className="text-[10px] text-gray-400 italic">--</span>
                ) : (
                    <QuotaProgressBar
                        compact
                        isWeekly
                        percentage={weeklyCell.percentage}
                        resetTime={weeklyCell.resetTime}
                    />
                )}
            </td>

            <td className="px-2.5 py-0.5 align-middle whitespace-nowrap w-[95px] border-b border-slate-200/90 dark:border-slate-800/90">
                <span className="text-[10px] font-medium text-gray-600 dark:text-gray-400 font-mono" title={formatDateTime(account.last_used)}>
                    {formatDateTimeShort(account.last_used)}
                </span>
            </td>

            <ActionsCell {...props} rowState={rowState} />
        </>
    );
}
