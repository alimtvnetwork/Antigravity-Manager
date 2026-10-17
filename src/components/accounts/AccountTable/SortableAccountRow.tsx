import { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { GripVertical } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { SELECTED_ROW_CLASSES } from '../../common/selectedState';
import { useAccountStore } from '../../../stores/useAccountStore';
import { useErrorStore } from '../../../stores/error-store';
import type { SortableRowProps } from './types';
import { AccountRowContent } from './AccountRowContent';
import { AccountRowContextMenu } from './AccountRowContextMenu';

/**
 * 可拖拽的表格行组件
 * 使用 @dnd-kit/sortable 实现拖拽功能
 */
export function SortableAccountRow({
    account,
    selected,
    isRefreshing,
    isCurrent,
    isFocused = false,
    isSwitching,
    isDragging,
    onSelect,
    onSwitch,
    onRefresh,
    onViewDevice,
    onViewDetails,
    onExport,
    onDelete,
    onToggleProxy,
    onWarmup,
    onUpdateLabel,
    onUpdatePriority,
    showPriority = false,
    onViewError,
    isDragDisabled = false,
    modelFilter = 'gemini',
    showAllEmails = false,
    emailColWidth,
}: SortableRowProps) {
    const { t } = useTranslation();
    const rowRef = useRef<HTMLTableRowElement | null>(null);
    const [contextMenuPos, setContextMenuPos] = useState<{ x: number; y: number } | null>(null);

    const handleToggleDisable = async () => {
        try {
            const nextDisabled = !account.disabled;
            await useAccountStore.getState().toggleAccountDisabled(
                account.id,
                nextDisabled,
                nextDisabled ? t('accounts.disabled_reason_user', 'Disabled by user') : undefined
            );
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'SortableAccountRow.handleToggleDisable',
                triggerAction: 'toggleAccountDisabled',
            });
        }
    };

    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging: isSortableDragging,
    } = useSortable({ id: account.id, disabled: isDragDisabled });

    const setMergedRef = (node: HTMLTableRowElement | null) => {
        setNodeRef(node);
        rowRef.current = node;
    };

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
        opacity: isSortableDragging ? 0.5 : 1,
        zIndex: isSortableDragging ? 1000 : 'auto',
    };

    return (
        <>
            <tr
                id={`account-row-${account.id}`}
                ref={setMergedRef}
                style={style as React.CSSProperties}
                onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    setContextMenuPos({ x: e.clientX, y: e.clientY });
                }}
                className={cn(
                    "group ui-interactive-row border-b border-slate-200/90 dark:border-slate-800/90 border-l-2 font-medium transition-all duration-200 ease-in-out hover:bg-slate-100/70 dark:hover:bg-slate-800/50",
                    isFocused
                        ? "bg-teal-50/90 dark:bg-[#171025] text-slate-900 dark:text-cyan-300 font-medium border-l-cyan-500 dark:border-l-cyan-400 border-slate-200/90 dark:border-slate-800/90 shadow-md ring-1 ring-cyan-500/30"
                        : isCurrent
                        ? cn(SELECTED_ROW_CLASSES, "font-medium border-b border-slate-200/90 dark:border-slate-800/90 hover:shadow-xs")
                        : selected
                        ? "bg-blue-50/90 dark:bg-[#171025] text-blue-950 dark:text-blue-100 border-l-blue-500 font-medium shadow-xs ring-1 ring-blue-500/30 hover:shadow-xs"
                        : isDragging
                        ? "bg-blue-100 dark:bg-blue-900/30 shadow-lg"
                        : "border-l-transparent text-gray-800 dark:text-gray-200 font-medium hover:border-l-blue-500/80 dark:hover:border-l-[var(--ui-interactive)] hover:shadow-xs"
                )}
            >
                {/* 拖拽手柄 */}
                <td className="pl-2 py-0.5 w-7 align-middle border-b border-slate-200/90 dark:border-slate-800/90">
                    <div
                        {...(!isDragDisabled ? attributes : {})}
                        {...(!isDragDisabled ? listeners : {})}
                        className={cn(
                            "flex items-center justify-center w-5 h-5 rounded transition-colors",
                            isDragDisabled
                                ? "text-gray-200 dark:text-gray-700 cursor-not-allowed opacity-40"
                                : "cursor-grab active:cursor-grabbing text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
                        )}
                        title={isDragDisabled ? t('accounts.drag_disabled_during_sort', '已激活列排序，拖拽排序已暂停') : t('accounts.drag_to_reorder')}
                    >
                        <GripVertical className="w-3.5 h-3.5" />
                    </div>
                </td>
                {/* 复选框 */}
                <td className="px-1.5 py-0.5 w-8 align-middle border-b border-slate-200/90 dark:border-slate-800/90">
                    <input
                        type="checkbox"
                        className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                        checked={selected}
                        onChange={onSelect}
                        disabled={isRefreshing}
                    />
                </td>
                <AccountRowContent
                    account={account}
                    selected={selected}
                    isCurrent={isCurrent}
                    isFocused={isFocused}
                    isRefreshing={isRefreshing}
                    isSwitching={isSwitching}
                    isDisabled={Boolean(account.disabled)}
                    onSwitch={onSwitch}
                    onRefresh={onRefresh}
                    onViewDevice={onViewDevice}
                    onViewDetails={onViewDetails}
                    onExport={onExport}
                    onDelete={onDelete}
                    onToggleProxy={onToggleProxy}
                    onWarmup={onWarmup}
                    onUpdateLabel={onUpdateLabel}
                    onUpdatePriority={onUpdatePriority}
                    showPriority={showPriority}
                    onViewError={onViewError}
                    modelFilter={modelFilter}
                    showAllEmails={showAllEmails}
                    emailColWidth={emailColWidth}
                />
            </tr>
            {Boolean(contextMenuPos) && (
                <AccountRowContextMenu
                    x={contextMenuPos!.x}
                    y={contextMenuPos!.y}
                    account={account}
                    onClose={() => setContextMenuPos(null)}
                    onSwitch={() => onSwitch()}
                    onRefresh={() => onRefresh()}
                    onDelete={() => onDelete()}
                    onToggleDisable={handleToggleDisable}
                />
            )}
        </>
    );
}
