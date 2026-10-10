import React, { useState, useMemo } from 'react';
import { formatCompactNumber } from '../../../utils/format';
import { useErrorStore } from '../../../stores/error-store';
import {
    DEFAULT_COL_WIDTHS,
    MIN_COL_WIDTH_PX,
    PROXY_LOG_COL_WIDTHS_STORAGE_KEY,
} from './types';
import type { ColumnWidths, LogTableProps } from './types';

export const LogTable: React.FC<LogTableProps> = ({
    logs,
    loading,
    onLogClick,
    t
}) => {
    const [colWidths, setColWidths] = useState<ColumnWidths>(() => {
        try {
            const saved = localStorage.getItem(PROXY_LOG_COL_WIDTHS_STORAGE_KEY);
            if (saved) {
                return { ...DEFAULT_COL_WIDTHS, ...JSON.parse(saved) };
            }
        } catch (e) {
            // Best-effort column-width restore; corrupt or unavailable storage falls back to defaults. Tracked.
            useErrorStore.getState().trackWarning(e, {
              source: 'ProxyMonitor.colWidths',
              triggerAction: 'restore_column_widths',
            });
        }
        return DEFAULT_COL_WIDTHS;
    });

    // 类似 Excel 的鼠标拖拽调整表头列宽机制
    const handleResizeStart = (colKey: keyof ColumnWidths, e: React.MouseEvent) => {
        e.preventDefault();
        e.stopPropagation();
        const startX = e.clientX;
        const startWidth = colWidths[colKey];

        const originalCursor = document.body.style.cursor;
        const originalUserSelect = document.body.style.userSelect;
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';

        const onMouseMove = (moveEvent: MouseEvent) => {
            const diff = moveEvent.clientX - startX;
            const newWidth = Math.max(MIN_COL_WIDTH_PX, startWidth + diff);
            setColWidths((prev) => ({
                ...prev,
                [colKey]: newWidth,
            }));
        };

        const onMouseUp = (upEvent: MouseEvent) => {
            document.body.style.cursor = originalCursor;
            document.body.style.userSelect = originalUserSelect;
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);

            const finalDiff = upEvent.clientX - startX;
            const finalWidth = Math.max(MIN_COL_WIDTH_PX, startWidth + finalDiff);
            setColWidths((prev) => {
                const next = { ...prev, [colKey]: finalWidth };
                try {
                    localStorage.setItem(PROXY_LOG_COL_WIDTHS_STORAGE_KEY, JSON.stringify(next));
                } catch (e) {
                    // Best-effort column-width persist; storage may throw in restricted contexts. Tracked, in-memory widths still applied.
                    useErrorStore.getState().trackWarning(e, {
                      source: 'ProxyMonitor.colWidths',
                      triggerAction: 'persist_column_widths',
                    });
                }
                return next;
            });
        };

        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('mouseup', onMouseUp);
    };

    const totalTableWidth = useMemo(() => {
        return Object.values(colWidths).reduce((a, b) => a + b, 0);
    }, [colWidths]);

    return (
        <div
            className="flex-1 overflow-y-auto overflow-x-auto bg-white dark:bg-base-100 relative scrollbar-thin"
        >
            <table
                className="table table-sm border-separate border-spacing-0"
                style={{ minWidth: `${totalTableWidth}px`, width: `${totalTableWidth}px`, tableLayout: 'fixed' }}
            >
                <thead className="bg-gray-100/90 dark:bg-base-200 text-gray-700 dark:text-gray-200 text-xs font-semibold sticky top-0 z-10 backdrop-blur-sm border-b border-gray-200 dark:border-base-300">
                    <tr>
                        <th style={{ width: `${colWidths.status}px`, minWidth: `${colWidths.status}px`, maxWidth: `${colWidths.status}px` }} className="py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.status')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('status', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.method}px`, minWidth: `${colWidths.method}px`, maxWidth: `${colWidths.method}px` }} className="py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.method')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('method', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.model}px`, minWidth: `${colWidths.model}px`, maxWidth: `${colWidths.model}px` }} className="py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.model')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('model', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.protocol}px`, minWidth: `${colWidths.protocol}px`, maxWidth: `${colWidths.protocol}px` }} className="py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.protocol')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('protocol', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.account}px`, minWidth: `${colWidths.account}px`, maxWidth: `${colWidths.account}px` }} className="py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.account')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('account', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.path}px`, minWidth: `${colWidths.path}px`, maxWidth: `${colWidths.path}px` }} className="py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.path')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('path', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.usage}px`, minWidth: `${colWidths.usage}px`, maxWidth: `${colWidths.usage}px` }} className="text-right py-2.5 px-3 relative group select-none whitespace-nowrap">
                            <div className="truncate">{t('monitor.table.usage')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('usage', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.duration}px`, minWidth: `${colWidths.duration}px`, maxWidth: `${colWidths.duration}px` }} className="text-right py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.duration')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('duration', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                        <th style={{ width: `${colWidths.time}px`, minWidth: `${colWidths.time}px`, maxWidth: `${colWidths.time}px` }} className="text-right py-2.5 px-3 relative group select-none">
                            <div className="truncate">{t('monitor.table.time')}</div>
                            <div
                                onMouseDown={(e) => handleResizeStart('time', e)}
                                className="absolute right-0 top-0 bottom-0 w-1.5 cursor-col-resize hover:bg-blue-500 active:bg-blue-600 transition-colors z-20 group-hover:bg-gray-300 dark:group-hover:bg-gray-600"
                                title="拖动调整列宽"
                            />
                        </th>
                    </tr>
                </thead>
                <tbody className="font-mono text-gray-800 dark:text-gray-100 text-xs divide-y divide-gray-100 dark:divide-base-200">
                    {logs.map((log) => (
                        <tr
                            key={log.id}
                            className="hover:bg-blue-50/80 dark:hover:bg-base-200/80 cursor-pointer transition-colors"
                            onClick={() => onLogClick(log)}
                        >
                            <td style={{ width: `${colWidths.status}px`, maxWidth: `${colWidths.status}px` }} className="py-2 px-3 truncate">
                                <span className={`badge badge-sm font-bold text-white border-none shadow-xs ${
                                    log.status >= 200 && log.status < 400
                                        ? 'bg-emerald-600 dark:bg-emerald-600'
                                        : 'bg-rose-600 dark:bg-rose-600'
                                }`}>
                                    {log.status}
                                </span>
                            </td>
                            <td className="font-bold text-gray-900 dark:text-white py-2 px-3 truncate" style={{ width: `${colWidths.method}px`, maxWidth: `${colWidths.method}px` }}>{log.method}</td>
                            <td
                                className="text-sky-600 dark:text-sky-400 font-semibold truncate py-2 px-3"
                                style={{ width: `${colWidths.model}px`, maxWidth: `${colWidths.model}px` }}
                                title={log.mapped_model && log.model !== log.mapped_model ? `${log.model} => ${log.mapped_model}` : (log.model || '')}
                            >
                                {log.mapped_model && log.model !== log.mapped_model
                                    ? `${log.model} => ${log.mapped_model}`
                                    : (log.model || '-')}
                            </td>
                            <td style={{ width: `${colWidths.protocol}px`, maxWidth: `${colWidths.protocol}px` }} className="py-2 px-3 truncate">
                                {log.protocol && (
                                    <span className={`badge badge-xs px-2 py-0.5 font-bold text-white border-none shadow-xs ${
                                        log.protocol === 'openai' ? 'bg-emerald-600 dark:bg-emerald-600' :
                                            log.protocol === 'anthropic' ? 'bg-amber-600 dark:bg-amber-600' :
                                                log.protocol === 'gemini' ? 'bg-blue-600 dark:bg-blue-600' :
                                                    'bg-gray-600 dark:bg-gray-600'
                                    }`}>
                                        {log.protocol === 'openai' ? 'OpenAI' :
                                            log.protocol === 'anthropic' ? 'Claude' :
                                                log.protocol === 'gemini' ? 'Gemini' : log.protocol}
                                    </span>
                                )}
                            </td>
                            <td className="text-gray-600 dark:text-gray-300 font-sans truncate text-xs py-2 px-3" style={{ width: `${colWidths.account}px`, maxWidth: `${colWidths.account}px` }} title={log.account_email || ''}>
                                {log.account_email ? log.account_email.replace(/(.{3}).*(@.*)/, '$1***$2') : '-'}
                            </td>
                            <td className="text-gray-700 dark:text-gray-300 truncate text-xs py-2 px-3" style={{ width: `${colWidths.path}px`, maxWidth: `${colWidths.path}px` }} title={log.url || ''}>{log.url}</td>
                            <td className="text-right text-xs py-2 px-3 whitespace-nowrap truncate" style={{ width: `${colWidths.usage}px`, maxWidth: `${colWidths.usage}px` }}>
                                {log.input_tokens != null && (() => {
                                    const totalIn = (log.cached_tokens && log.cached_tokens > log.input_tokens)
                                        ? log.input_tokens + log.cached_tokens
                                        : log.input_tokens;
                                    const hitRate = (log.cached_tokens && totalIn > 0)
                                        ? Math.min(100, Math.max(0, (log.cached_tokens / totalIn) * 100))
                                        : 0;
                                    const hitRateText = totalIn > 0 && log.cached_tokens
                                        ? (hitRate >= 100 ? '100%' : (hitRate % 1 === 0 ? `${hitRate.toFixed(0)}%` : `${hitRate.toFixed(1)}%`))
                                        : '';
                                    return (
                                        <div>
                                            <div className="text-gray-700 dark:text-gray-200">{t('monitor.input')}: <span className="font-semibold">{formatCompactNumber(totalIn)}</span></div>
                                            {log.cached_tokens ? (
                                                <div
                                                    className="text-emerald-600 dark:text-emerald-400 font-semibold text-[11px] leading-tight"
                                                    title={`${t('token_stats.cached', 'Cache')}: ${log.cached_tokens.toLocaleString()}${hitRateText ? ` (${hitRateText})` : ''}`}
                                                >
                                                    ({t('monitor.cached', 'Cache')}: {formatCompactNumber(log.cached_tokens)}{hitRateText ? ` ${hitRateText}` : ''})
                                                </div>
                                            ) : null}
                                        </div>
                                    );
                                })()}
                                {log.output_tokens != null && <div className="text-gray-700 dark:text-gray-200">{t('monitor.output')}: <span className="font-semibold">{formatCompactNumber(log.output_tokens)}</span></div>}
                            </td>
                            <td className="text-right text-gray-700 dark:text-gray-300 text-xs font-medium py-2 px-3 truncate" style={{ width: `${colWidths.duration}px`, maxWidth: `${colWidths.duration}px` }}>{log.duration}ms</td>
                            <td className="text-right text-gray-500 dark:text-gray-400 text-xs py-2 px-3 truncate" style={{ width: `${colWidths.time}px`, maxWidth: `${colWidths.time}px` }}>
                                {new Date(log.timestamp).toLocaleTimeString()}
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>

            {/* Loading indicator */}
            {loading && (
                <div className="flex items-center justify-center p-4 bg-white dark:bg-base-100">
                    <div className="loading loading-spinner loading-md text-blue-600"></div>
                    <span className="ml-3 text-sm text-gray-500 dark:text-gray-400">{t('common.loading')}</span>
                </div>
            )}

            {/* Empty state */}
            {!loading && logs.length === 0 && (
                <div className="flex items-center justify-center p-8 text-gray-400 dark:text-gray-500 text-sm">
                    {t('monitor.table.empty') || '暂无请求记录'}
                </div>
            )}
        </div>
    );
};
