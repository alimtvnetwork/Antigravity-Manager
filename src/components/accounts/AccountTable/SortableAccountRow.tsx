import { useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { GripVertical } from 'lucide-react';
import { cn } from '../../../utils/cn';
import { SELECTED_ROW_CLASSES } from '../../common/selectedState';
import type { SortableRowProps } from './types';
import { AccountRowContent } from './AccountRowContent';

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
}: SortableRowProps) {
    const { t } = useTranslation();
    const rowRef = useRef<HTMLTableRowElement | null>(null);

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
        <tr
            id={`account-row-${account.id}`}
            ref={setMergedRef}
            style={style as React.CSSProperties}
            className={cn(
                "group border-b border-slate-200/90 dark:border-slate-800/90 border-l-2 transition-[color,background-color,border-color] duration-[180ms] ease-in-out",
                isFocused
                    ? "bg-teal-50/90 dark:bg-[#0e2c44] text-slate-900 dark:text-cyan-300 font-bold border-l-cyan-500 dark:border-l-cyan-400 border-slate-200/90 dark:border-slate-800/90 shadow-md ring-1 ring-cyan-500/30"
                    : isCurrent
                    ? cn(SELECTED_ROW_CLASSES, "border-b border-slate-200/90 dark:border-slate-800/90 hover:bg-slate-200/60 dark:hover:bg-[#0c2438]")
                    : selected
                    ? "bg-blue-50/90 dark:bg-[#0f273d] text-blue-950 dark:text-blue-100 border-l-blue-500 font-semibold shadow-xs ring-1 ring-blue-500/30"
                    : isDragging
                    ? "bg-blue-100 dark:bg-blue-900/30 shadow-lg"
                    : "border-l-transparent text-gray-800 dark:text-gray-200 hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60 hover:border-l-blue-500/70"
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
            />
        </tr>
    );
}
