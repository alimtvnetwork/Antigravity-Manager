/**
 * Account form component
 * Supports drag and drop sorting function, users can adjust the order of accounts by dragging rows
 */
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
    DndContext,
    closestCenter,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    DragOverlay,
} from '@dnd-kit/core';
import type { DragEndEvent, DragStartEvent } from '@dnd-kit/core';
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    verticalListSortingStrategy,
} from '@dnd-kit/sortable';
import { ArrowUpDown, ArrowUp, ArrowDown, Clock, GripVertical } from 'lucide-react';
import { cn } from '../../utils/cn';
import { Gemini } from '@lobehub/icons';
import type { Account } from '../../types/account';
import { computeAccountAlgorithmScore } from '../../utils/accountAlgorithmScore';
import { SortableAccountRow } from './AccountTable/SortableAccountRow';
import { AccountRowContent } from './AccountTable/AccountRowContent';
import { extractAccountResetTime, type AccountTableProps } from './AccountTable/types';

export type { AccountTableProps } from './AccountTable/types';

const EMAIL_COL_MIN = 220;
const EMAIL_COL_MAX = 550;
const EMAIL_COL_DEFAULT = 300;
const EMAIL_WIDTH_STORAGE_KEY = 'agm_account_table_email_width';

export function AccountTable({
    accounts,
    selectedIds,
    refreshingIds,
    onToggleSelect,
    onToggleAll,
    currentAccountId,
    currentAccountEmail,
    switchingAccountId,
    focusedAccountId,
    onSwitch,
    onRefresh,
    onViewDevice,
    onViewDetails,
    onExport,
    onDelete,
    onToggleProxy,
    onReorder,
    onWarmup,
    onUpdateLabel,
    onUpdatePriority,
    onViewError,
    showAllEmails = false,
}: AccountTableProps) {
    const { t } = useTranslation();

    const [emailColWidth, setEmailColWidth] = useState<number>(() => {
        try {
            const saved = localStorage.getItem(EMAIL_WIDTH_STORAGE_KEY);
            if (saved) {
                const parsed = parseInt(saved, 10);
                if (!isNaN(parsed) && parsed >= EMAIL_COL_MIN && parsed <= EMAIL_COL_MAX) {
                    return parsed;
                }
            }
        } catch {
            // localStorage not available
        }
        return EMAIL_COL_DEFAULT;
    });

    const handleResizeMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        const startX = e.clientX;
        const startWidth = emailColWidth;

        const onMouseMove = (moveEvent: MouseEvent) => {
            const delta = moveEvent.clientX - startX;
            const newWidth = Math.min(EMAIL_COL_MAX, Math.max(EMAIL_COL_MIN, startWidth + delta));
            setEmailColWidth(newWidth);
        };

        const onMouseUp = (upEvent: MouseEvent) => {
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
            const finalDelta = upEvent.clientX - startX;
            const finalWidth = Math.min(EMAIL_COL_MAX, Math.max(EMAIL_COL_MIN, startWidth + finalDelta));
            try {
                localStorage.setItem(EMAIL_WIDTH_STORAGE_KEY, String(finalWidth));
            } catch {
                // ignore
            }
        };

        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    };

    const handleResetEmailWidth = (e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setEmailColWidth(EMAIL_COL_DEFAULT);
        try {
            localStorage.setItem(EMAIL_WIDTH_STORAGE_KEY, String(EMAIL_COL_DEFAULT));
        } catch {
            // ignore
        }
    };

    const isAccountCurrent = (acc: Account) => {
        const isIdMatch = Boolean(currentAccountId && acc.id === currentAccountId);
        if (isIdMatch) {
            return true;
        }
        const isEmailMatch = Boolean(
            currentAccountEmail && acc.email && acc.email.toLowerCase() === currentAccountEmail.toLowerCase()
        );
        if (isEmailMatch) {
            return true;
        }
        return false;
    };

    const [modelFilter, setModelFilter] = useState<'gemini' | 'claude'>('gemini');
    const showPriority = useMemo(() => {
        const values = new Set(accounts.map((account) => account.priority ?? 50));
        return values.size > 1;
    }, [accounts]);
    const [activeId, setActiveId] = useState<string | null>(null);
    // Sorting status configuration: Supports sorting by quota reset time (reset_time), last used time (last_used) or algorithm score (score)
    const [sortConfig, setSortConfig] = useState<{
        key: 'reset_time' | 'last_used' | 'score' | null;
        direction: 'asc' | 'desc' | null;
    }>({
        key: null,
        direction: null,
    });

    const isSortingActive = sortConfig.key !== null && sortConfig.direction !== null;

    const handleSortToggle = (key: 'reset_time' | 'last_used' | 'score') => {
        setSortConfig(prev => {
            if (prev.key !== key) {
                return { key, direction: key === 'score' ? 'desc' : 'asc' };
            }
            if (key === 'score') {
                if (prev.direction === 'desc') {
                    return { key, direction: 'asc' };
                }
                return { key: null, direction: null };
            }
            if (prev.direction === 'asc') {
                return { key, direction: 'desc' };
            }
            return { key: null, direction: null };
        });
    };

    // Interception sorting of accounts based on sorting status
    const sortedAccounts = useMemo(() => {
        if (!isSortingActive) return accounts;

        return [...accounts].sort((a, b) => {
            if (sortConfig.key === 'reset_time') {
                const timeA = extractAccountResetTime(a);
                const timeB = extractAccountResetTime(b);

                // Those without reset_time are queued to the back.
                if (timeA === null && timeB === null) return 0;
                if (timeA === null) return 1;
                if (timeB === null) return -1;

                return sortConfig.direction === 'asc' ? timeA - timeB : timeB - timeA;
            }

            if (sortConfig.key === 'last_used') {
                const timeA = a.last_used || 0;
                const timeB = b.last_used || 0;

                return sortConfig.direction === 'asc' ? timeA - timeB : timeB - timeA;
            }

            if (sortConfig.key === 'score') {
                const scoreA = computeAccountAlgorithmScore(a).score;
                const scoreB = computeAccountAlgorithmScore(b).score;

                if (scoreA !== scoreB) {
                    return sortConfig.direction === 'desc' ? scoreB - scoreA : scoreA - scoreB;
                }
                return (a.email || '').localeCompare(b.email || '');
            }

            return 0;
        });
    }, [accounts, sortConfig, isSortingActive]);

    // Configure drag sensor
    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: { distance: 8 }, // Need to move 8px to trigger drag
        }),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        })
    );

    const accountIds = useMemo(() => sortedAccounts.map(a => a.id), [sortedAccounts]);
    const activeAccount = useMemo(() => sortedAccounts.find(a => a.id === activeId), [sortedAccounts, activeId]);

    const handleDragStart = (event: DragStartEvent) => {
        if (isSortingActive) return;
        setActiveId(event.active.id as string);
    };

    const handleDragEnd = (event: DragEndEvent) => {
        const { active, over } = event;
        setActiveId(null);

        if (isSortingActive) return;

        if (over && active.id !== over.id) {
            const oldIndex = accountIds.indexOf(active.id as string);
            const newIndex = accountIds.indexOf(over.id as string);

            if (oldIndex !== -1 && newIndex !== -1 && onReorder) {
                onReorder(arrayMove(accountIds, oldIndex, newIndex));
            }
        }
    };

    if (accounts.length === 0) {
        return (
            <div className="bg-white dark:bg-base-100 rounded-2xl p-12 shadow-sm border border-gray-100 dark:border-base-200 text-center">
                <p className="text-gray-400 mb-2">{t('accounts.empty.title')}</p>
                <p className="text-sm text-gray-400">{t('accounts.empty.desc')}</p>
            </div>
        );
    }

    return (
        <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragStart={handleDragStart}
            onDragEnd={handleDragEnd}
        >
            <div className="overflow-x-auto rounded-xl border border-slate-200/80 dark:border-slate-800/80 bg-white dark:bg-[#071724] shadow-xs">
                <table className="w-full min-w-[1060px]">
                    <thead>
                        <tr className="border-b border-slate-200/80 dark:border-slate-800/80 bg-slate-50/90 dark:bg-[#061220]">
                            <th className="pl-2 py-0.5 text-left w-7">
                                <span className="sr-only">{t('accounts.drag_to_reorder')}</span>
                            </th>
                            <th className="px-1.5 py-0.5 text-left w-8">
                                <input
                                    type="checkbox"
                                    className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                                    checked={accounts.length > 0 && selectedIds.size === accounts.length}
                                    onChange={onToggleAll}
                                />
                            </th>
                            <th
                                className="relative px-2 py-0.5 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider whitespace-nowrap select-none"
                                style={{ width: `${emailColWidth}px`, minWidth: `${emailColWidth}px`, maxWidth: `${emailColWidth}px` }}
                            >
                                <span>{t('accounts.table.email')}</span>
                                {/* Interactive drag handle on right border */}
                                <div
                                    role="separator"
                                    aria-orientation="vertical"
                                    aria-label="Resize Email column"
                                    onMouseDown={handleResizeMouseDown}
                                    onDoubleClick={handleResetEmailWidth}
                                    title={t('accounts.table.resize_column_hint', 'Drag to resize, double-click to reset')}
                                    className="absolute top-0 right-0 h-full w-2 cursor-col-resize flex items-center justify-end group/resize z-10"
                                >
                                    <div className="w-[2px] h-3.5 bg-slate-300 dark:bg-slate-700 group-hover/resize:bg-blue-500 group-hover/resize:w-[3px] transition-all rounded-full mr-0.5" />
                                </div>
                            </th>
                            {/* Column 1: 4H Quota with Gemini Icon and Model Toggle */}
                            <th className="px-2 py-0.5 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider min-w-[195px] whitespace-nowrap bg-slate-50/50 dark:bg-slate-900/40 border-l border-slate-300 dark:border-[#40285e]">
                                <div className="flex items-center justify-between gap-1 w-full">
                                    <button
                                        type="button"
                                        onClick={() => handleSortToggle('reset_time')}
                                        className={cn(
                                            "inline-flex items-center gap-1 hover:text-blue-600 dark:hover:text-blue-400 transition-colors uppercase font-medium",
                                            sortConfig.key === 'reset_time' && "text-blue-600 dark:text-blue-400 font-semibold"
                                        )}
                                        title={t('accounts.table.sort_by_reset_time', 'Sort by reset time')}
                                    >
                                        <Gemini.Color className="w-3.5 h-3.5 shrink-0" />
                                        <span>4H {t('accounts.table.quota', 'Quota')}</span>
                                        {sortConfig.key === 'reset_time' ? (
                                            sortConfig.direction === 'asc' ? <ArrowUp className="w-3 h-3 text-blue-600" /> : <ArrowDown className="w-3 h-3 text-blue-600" />
                                        ) : (
                                            <ArrowUpDown className="w-3 h-3 text-gray-400 opacity-60" />
                                        )}
                                    </button>

                                    {/* Gemini / Claude Pill Switch */}
                                    <div className="inline-flex items-center p-0.5 rounded-md bg-slate-200/80 dark:bg-slate-900 border border-slate-300/80 dark:border-slate-800 text-[9px] font-semibold">
                                        <button
                                            type="button"
                                            onClick={() => setModelFilter('gemini')}
                                            className={cn(
                                                "px-1.5 py-0.5 rounded-[4px] transition-all cursor-pointer font-medium",
                                                modelFilter === 'gemini'
                                                    ? "bg-white dark:bg-slate-800 text-cyan-600 dark:text-cyan-400 font-bold border border-cyan-500/30 dark:border-cyan-400/30 shadow-2xs"
                                                    : "text-gray-500 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-200"
                                            )}
                                            title="Only show Gemini"
                                        >
                                            Gemini
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => setModelFilter('claude')}
                                            className={cn(
                                                "px-1.5 py-0.5 rounded-[4px] transition-all cursor-pointer font-medium",
                                                modelFilter === 'claude'
                                                    ? "bg-white dark:bg-slate-800 text-cyan-600 dark:text-cyan-400 font-bold border border-cyan-500/30 dark:border-cyan-400/30 shadow-2xs"
                                                    : "text-gray-500 dark:text-slate-400 hover:text-gray-900 dark:hover:text-slate-200"
                                            )}
                                            title="Only show Claude"
                                        >
                                            Claude
                                        </button>
                                    </div>
                                </div>
                            </th>

                            {/* Column 2: Weekly Quota Column */}
                            <th className="px-2 py-0.5 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider min-w-[195px] whitespace-nowrap bg-slate-50/50 dark:bg-slate-900/40 border-r border-slate-300 dark:border-[#40285e]">
                                <div className="flex items-center gap-1.5">
                                    <Clock className="w-3.5 h-3.5 text-gray-400" />
                                    <span>{t('accounts.table.weekly_quota', 'Weekly Quota')}</span>
                                </div>
                            </th>

                            {/* Column: Algorithm Candidate Score */}
                            <th className="px-2 py-0.5 text-left rtl:text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider w-[110px] min-w-[100px] whitespace-nowrap">
                                <button
                                    type="button"
                                    onClick={() => handleSortToggle('score')}
                                    className={cn(
                                        "inline-flex items-center gap-1 hover:text-blue-600 dark:hover:text-blue-400 transition-colors uppercase font-medium",
                                        sortConfig.key === 'score' && "text-blue-600 dark:text-blue-400 font-semibold"
                                    )}
                                    title={t('accounts.table.sort_by_score', 'Sort by algorithm candidate score')}
                                >
                                    <span>{t('accounts.table.score', 'Score')}</span>
                                    {sortConfig.key === 'score' ? (
                                        sortConfig.direction === 'asc' ? (
                                            <ArrowUp className="w-3 h-3 text-blue-600 dark:text-blue-400" />
                                        ) : (
                                            <ArrowDown className="w-3 h-3 text-blue-600 dark:text-blue-400" />
                                        )
                                    ) : (
                                        <ArrowUpDown className="w-3 h-3 text-gray-400 opacity-60" />
                                    )}
                                </button>
                            </th>
                            <th className="px-2 py-0.5 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider whitespace-nowrap sticky right-0 w-[220px] xl:w-[280px] bg-slate-50 dark:bg-[#061220] z-20 shadow-[-12px_0_12px_-12px_rgba(0,0,0,0.1)] dark:shadow-[-12px_0_12px_-12px_rgba(255,255,255,0.05)] text-center">{t('accounts.table.actions')}</th>
                        </tr>
                    </thead>
                    <SortableContext items={accountIds} strategy={verticalListSortingStrategy}>
                        <tbody className="divide-y divide-slate-200/90 dark:divide-slate-800/90">
                            {sortedAccounts.map((account) => (
                                <SortableAccountRow
                                    key={account.id}
                                    account={account}
                                    selected={selectedIds.has(account.id)}
                                    isRefreshing={refreshingIds.has(account.id)}
                                    isCurrent={isAccountCurrent(account)}
                                    isFocused={Boolean(focusedAccountId && account.id === focusedAccountId)}
                                    isSwitching={account.id === switchingAccountId}
                                    isDragging={account.id === activeId}
                                    onSelect={() => onToggleSelect(account.id)}
                                    onSwitch={(targetIde?: string) => onSwitch(account.id, targetIde)}
                                    onRefresh={() => onRefresh(account.id)}
                                    onViewDevice={() => onViewDevice(account.id)}
                                    onViewDetails={() => onViewDetails(account.id)}
                                    onExport={() => onExport(account.id)}
                                    onDelete={() => onDelete(account.id)}
                                    onToggleProxy={() => onToggleProxy(account.id)}
                                    onWarmup={onWarmup ? () => onWarmup(account.id) : undefined}
                                    onUpdateLabel={onUpdateLabel ? (label: string) => onUpdateLabel(account.id, label) : undefined}
                                    onUpdatePriority={onUpdatePriority ? (priority: number) => onUpdatePriority(account.id, priority) : undefined}
                                    showPriority={showPriority}
                                    onViewError={() => onViewError(account.id)}
                                    isDragDisabled={isSortingActive}
                                    modelFilter={modelFilter}
                                    showAllEmails={showAllEmails}
                                    emailColWidth={emailColWidth}
                                />
                            ))}
                        </tbody>
                    </SortableContext>
                </table >
            </div >

            {/* Drag and drop the floating preview layer */}
            <DragOverlay>
                {
                    activeAccount ? (
                        <table className="w-full bg-white dark:bg-[#171025] shadow-2xl rounded-lg border border-blue-500/40 dark:border-blue-700/60">
                            <tbody>
                                <tr className="bg-blue-50/90 dark:bg-[#171025] border-b border-slate-200/90 dark:border-slate-800/90">
                                    <td className="pl-2 py-0.5 w-7">
                                        <div className="flex items-center justify-center w-5 h-5 text-blue-500">
                                            <GripVertical className="w-3.5 h-3.5" />
                                        </div>
                                    </td>
                                    <td className="px-1.5 py-0.5 w-8">
                                        <input
                                            type="checkbox"
                                            className="checkbox checkbox-xs rounded border-2"
                                            checked={selectedIds.has(activeAccount.id)}
                                            readOnly
                                        />
                                    </td>
                                    <AccountRowContent
                                        account={activeAccount}
                                        selected={selectedIds.has(activeAccount.id)}
                                        isCurrent={isAccountCurrent(activeAccount)}
                                        isRefreshing={refreshingIds.has(activeAccount.id)}
                                        isSwitching={activeAccount.id === switchingAccountId}
                                        onSwitch={() => { }}
                                        onRefresh={() => { }}
                                        onViewDevice={() => { }}
                                        onViewDetails={() => { }}
                                        onExport={() => { }}
                                        onDelete={() => { }}
                                        onToggleProxy={() => { }}
                                        isDisabled={Boolean(activeAccount.disabled)}
                                        onViewError={() => { }}
                                        emailColWidth={emailColWidth}
                                    />
                                </tr>
                            </tbody>
                        </table>
                    ) : null
                }
            </DragOverlay>
        </DndContext>
    );
}

export default AccountTable;
