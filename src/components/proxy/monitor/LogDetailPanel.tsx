import React, { useMemo } from 'react';
import type { TFunction } from 'react-i18next';
import { X, Sparkles, FileCode2, Eye, EyeOff } from 'lucide-react';
import { formatCompactNumber } from '../../../utils/format';
import { copyToClipboard } from '../../../utils/clipboard';
import { VirtualizedPayloadViewer } from '../VirtualizedPayloadViewer';
import { extractConcisePayload } from './payloadExtractors';
import { parseTimingFromHeadersAndBody } from './timingUtils';
import { TimingDiagnosticsCard } from './TimingDiagnosticsCard';
import { COPY_FEEDBACK_TIMEOUT_MS } from './types';
import type { PayloadViewMode, ProxyRequestLog } from './types';

export interface LogDetailPanelProps {
    t: TFunction;
    log: ProxyRequestLog;
    loadingDetail: boolean;
    onClose: () => void;
    payloadViewMode: PayloadViewMode;
    onPayloadViewModeChange: (mode: PayloadViewMode) => void;
    showMetadata: boolean;
    onToggleMetadata: () => void;
    copiedCard: string | null;
    onCopiedCardChange: (id: string | null) => void;
}

/** Full-screen detail drawer for a single proxy request log (metadata + payload viewers). */
export const LogDetailPanel: React.FC<LogDetailPanelProps> = ({
    t,
    log,
    loadingDetail,
    onClose,
    payloadViewMode,
    onPayloadViewModeChange,
    showMetadata,
    onToggleMetadata,
    copiedCard,
    onCopiedCardChange,
}) => {
    const conciseRequestBody = useMemo(() => {
        return log?.request_body
            ? extractConcisePayload(log.request_body, 'request', log)
            : '';
    }, [log?.request_body, log?.id]);

    const conciseUpstreamBody = useMemo(() => {
        return log?.upstream_request_body
            ? extractConcisePayload(log.upstream_request_body, 'upstream', log)
            : '';
    }, [log?.upstream_request_body, log?.id]);

    const conciseResponseBody = useMemo(() => {
        return log?.response_body
            ? extractConcisePayload(log.response_body, 'response', log)
            : '';
    }, [log?.response_body, log?.id, log?.input_tokens, log?.output_tokens, log?.cached_tokens]);

    const timingInfo = useMemo(() => {
        return parseTimingFromHeadersAndBody(
            log?.response_headers,
            log?.response_body,
            log?.duration
        );
    }, [log?.response_headers, log?.response_body, log?.duration]);

    const timingNode = timingInfo ? (
        <div className="p-2.5">
            <TimingDiagnosticsCard
                key={log?.id}
                timing={timingInfo}
                onCopyText={async (text) => {
                    const success = await copyToClipboard(text);
                    if (success) {
                        onCopiedCardChange('timing');
                        setTimeout(() => onCopiedCardChange(null), COPY_FEEDBACK_TIMEOUT_MS);
                    }
                }}
            />
        </div>
    ) : undefined;

    return (
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-2 sm:p-3 md:p-4" onClick={onClose}>
                <div className="bg-white dark:bg-base-100 rounded-2xl shadow-2xl w-full max-w-[98vw] xl:max-w-[1720px] h-[94vh] max-h-[94vh] flex flex-col overflow-hidden border border-gray-200 dark:border-base-200" onClick={e => e.stopPropagation()}>
                    {/* Modal Header */}
                    <div className="px-4 py-2.5 border-b border-gray-200 dark:border-base-300 flex items-center justify-between bg-gray-50 dark:bg-base-200 shrink-0">
                        <div className="flex items-center gap-3 min-w-0">
                            {loadingDetail && <div className="loading loading-spinner loading-sm shrink-0"></div>}
                            <span className={`badge badge-sm font-bold text-white border-none shrink-0 shadow-xs ${
                                log.status >= 200 && log.status < 400
                                    ? 'bg-emerald-600'
                                    : 'bg-rose-600'
                            }`}>
                                {log.status}
                            </span>
                            <span className="font-mono font-bold text-gray-900 dark:text-white text-sm shrink-0">{log.method}</span>
                            <span className="text-xs text-gray-500 dark:text-gray-400 font-mono truncate max-w-lg hidden sm:inline" title={log.url}>{log.url}</span>
                        </div>
                        <button onClick={onClose} className="btn btn-ghost btn-sm btn-circle text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-base-200" aria-label="关闭"><X size={18} /></button>
                    </div>

                    {/* Modal Content */}
                    <div className="flex-1 min-h-0 flex flex-col p-3 sm:p-4 space-y-2.5 bg-gray-100/50 dark:bg-base-100 overflow-hidden">
                        {/* Metadata Section (Collapsible) */}
                        {showMetadata && (
                            <div className="bg-white dark:bg-base-200 p-3 sm:p-3.5 rounded-xl border border-gray-200 dark:border-base-300 shadow-sm shrink-0 text-xs transition-all duration-200">
                                <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-3">
                                    <div>
                                        <span className="block text-gray-500 dark:text-gray-400 uppercase font-bold text-[10px] tracking-wider">{t('monitor.details.time')}</span>
                                        <span className="font-mono font-semibold text-gray-900 dark:text-white text-xs truncate block" title={new Date(log.timestamp).toLocaleString()}>{new Date(log.timestamp).toLocaleString()}</span>
                                    </div>
                                    <div>
                                        <span className="block text-gray-500 dark:text-gray-400 uppercase font-bold text-[10px] tracking-wider">{t('monitor.details.duration')}</span>
                                        <span className="font-mono font-semibold text-gray-900 dark:text-white text-xs">{log.duration}ms</span>
                                    </div>
                                    <div>
                                        <span className="block text-gray-500 dark:text-gray-400 uppercase font-bold text-[10px] tracking-wider">{t('monitor.details.tokens')}</span>
                                        <div className="font-mono text-[11px] flex items-center gap-1.5 mt-0.5">
                                            {(() => {
                                                const totalIn = (log.cached_tokens && log.cached_tokens > (log.input_tokens ?? 0))
                                                    ? (log.input_tokens ?? 0) + log.cached_tokens
                                                    : (log.input_tokens ?? 0);
                                                return (
                                                    <span className="text-blue-700 dark:text-blue-300 bg-blue-100 dark:bg-blue-900/40 px-1.5 py-0.5 rounded font-bold" title={`Total Input Tokens: ${totalIn}`}>
                                                        In: {formatCompactNumber(totalIn)}
                                                    </span>
                                                );
                                            })()}
                                            <span className="text-emerald-700 dark:text-emerald-300 bg-emerald-100 dark:bg-emerald-900/40 px-1.5 py-0.5 rounded font-bold">Out: {formatCompactNumber(log.output_tokens ?? 0)}</span>
                                            {log.cached_tokens != null && log.cached_tokens > 0 && (() => {
                                                const totalIn = (log.cached_tokens && log.cached_tokens > (log.input_tokens ?? 0))
                                                    ? (log.input_tokens ?? 0) + log.cached_tokens
                                                    : (log.input_tokens ?? 0);
                                                const hitRate = totalIn > 0 ? Math.min(100, Math.max(0, (log.cached_tokens / totalIn) * 100)) : 0;
                                                const hitRateText = totalIn > 0 ? (hitRate >= 100 ? '100%' : (hitRate % 1 === 0 ? `${hitRate.toFixed(0)}%` : `${hitRate.toFixed(1)}%`)) : '';
                                                return (
                                                    <span
                                                        className="text-purple-700 dark:text-purple-300 bg-purple-100 dark:bg-purple-900/40 px-1.5 py-0.5 rounded font-bold"
                                                        title={`Cache: ${log.cached_tokens.toLocaleString()}${hitRateText ? ` (${hitRateText})` : ''}`}
                                                    >
                                                        Cache: {formatCompactNumber(log.cached_tokens)}{hitRateText ? ` (${hitRateText})` : ''}
                                                    </span>
                                                );
                                            })()}
                                        </div>
                                    </div>
                                    <div>
                                        <span className="block text-gray-500 dark:text-gray-400 uppercase font-bold text-[10px] tracking-wider">{t('monitor.details.protocol')}</span>
                                        <span className={`inline-block px-2 py-0.5 rounded-full font-mono font-bold text-[11px] uppercase mt-0.5 text-white shadow-xs ${
                                            log.protocol === 'openai' ? 'bg-emerald-600' :
                                            log.protocol === 'anthropic' ? 'bg-amber-600' :
                                            log.protocol === 'gemini' ? 'bg-blue-600' :
                                            'bg-gray-600'
                                        }`}>
                                            {log.protocol || '-'}
                                        </span>
                                    </div>
                                    <div>
                                        <span className="block text-gray-500 dark:text-gray-400 uppercase font-bold text-[10px] tracking-wider">{t('monitor.details.model')}</span>
                                        <span className="font-mono font-bold text-blue-600 dark:text-blue-400 truncate block text-xs" title={log.model}>{log.model || '-'}</span>
                                        {log.mapped_model && log.model !== log.mapped_model && (
                                            <span className="font-mono text-emerald-600 dark:text-emerald-400 truncate block text-[11px]" title={log.mapped_model}>➔ {log.mapped_model}</span>
                                        )}
                                    </div>
                                    <div>
                                        <span className="block text-gray-500 dark:text-gray-400 uppercase font-bold text-[10px] tracking-wider">{t('monitor.details.account_used')}</span>
                                        <span className="font-mono font-medium text-gray-900 dark:text-white truncate block text-xs" title={log.account_email || '-'}>{log.account_email || '-'}</span>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Mode & Toolbar Bar */}
                        <div className="flex flex-wrap items-center justify-between gap-2 px-1 shrink-0">
                            <div className="flex items-center gap-2">
                                <div className="inline-flex items-center p-1 bg-gray-200/70 dark:bg-base-200 rounded-xl border border-gray-300/70 dark:border-base-300 gap-1 shadow-inner">
                                    <button
                                        type="button"
                                        onClick={() => onPayloadViewModeChange('concise')}
                                        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 cursor-pointer select-none ${
                                            payloadViewMode === 'concise'
                                                ? 'bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-sm border border-gray-200 dark:border-base-300'
                                                : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                                        }`}
                                    >
                                        <Sparkles size={13} className={payloadViewMode === 'concise' ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'} />
                                        <span>{t('monitor.details.concise_mode', '简要模式')}</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => onPayloadViewModeChange('full')}
                                        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-150 cursor-pointer select-none ${
                                            payloadViewMode === 'full'
                                                ? 'bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-sm border border-gray-200 dark:border-base-300'
                                                : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'
                                        }`}
                                    >
                                        <FileCode2 size={13} className={payloadViewMode === 'full' ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'} />
                                        <span>{t('monitor.details.full_mode', '完整模式')}</span>
                                    </button>
                                </div>
                                <span className="hidden sm:inline-block text-[11px] text-gray-500 dark:text-gray-400">
                                    {payloadViewMode === 'concise'
                                        ? t('monitor.details.concise_desc', '已为您精简工具参数与冗余字段，突出思考块、用量与对话主体')
                                        : '显示原始完整未修剪报文'}
                                </span>
                            </div>

                            <div className="flex items-center gap-2">
                                <button
                                    type="button"
                                    onClick={onToggleMetadata}
                                    className="btn btn-xs btn-ghost text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-base-200 gap-1 text-[11px]"
                                    title={showMetadata ? '折叠元数据以增大报文视野' : '展开元数据信息'}
                                >
                                    {showMetadata ? <EyeOff size={13} /> : <Eye size={13} />}
                                    <span>{showMetadata ? '收起元数据' : '展开元数据'}</span>
                                </button>
                            </div>
                        </div>

                        {/* Horizontal 3-Column Grid */}
                        <div className="grid grid-cols-1 lg:grid-cols-3 gap-3 flex-1 min-h-0 overflow-hidden">
                            <VirtualizedPayloadViewer
                                cardId="req"
                                title={t('monitor.details.request_payload', '请求报文 (Request)')}
                                badge="REQUEST"
                                badgeStyle="bg-blue-50 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300 border-blue-200 dark:border-blue-800/60"
                                rawPayload={log.request_body}
                                concisePayload={conciseRequestBody}
                                headersJson={log.request_headers}
                                viewMode={payloadViewMode}
                                emptyPlaceholder={t('monitor.details.payload_empty', '无请求报文')}
                                onCopy={async (text) => {
                                    const success = await copyToClipboard(text);
                                    if (success) {
                                        onCopiedCardChange('req');
                                        setTimeout(() => onCopiedCardChange(null), COPY_FEEDBACK_TIMEOUT_MS);
                                    }
                                }}
                                isCopied={copiedCard === 'req'}
                            />
                            <VirtualizedPayloadViewer
                                cardId="upstream"
                                title={t('monitor.details.upstream_request_payload', '中转报文 (Forwarded)')}
                                badge="FORWARDED"
                                badgeStyle="bg-amber-50 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300 border-amber-200 dark:border-amber-800/60"
                                rawPayload={log.upstream_request_body}
                                concisePayload={conciseUpstreamBody}
                                headersJson={log.upstream_request_headers}
                                viewMode={payloadViewMode}
                                emptyPlaceholder={t('monitor.details.no_upstream_payload', '无中转报文 (直接转发或未记录)')}
                                onCopy={async (text) => {
                                    const success = await copyToClipboard(text);
                                    if (success) {
                                        onCopiedCardChange('upstream');
                                        setTimeout(() => onCopiedCardChange(null), COPY_FEEDBACK_TIMEOUT_MS);
                                    }
                                }}
                                isCopied={copiedCard === 'upstream'}
                            />
                            <VirtualizedPayloadViewer
                                cardId="resp"
                                title={t('monitor.details.response_payload', '响应报文 (Response)')}
                                badge="RESPONSE"
                                badgeStyle="bg-emerald-50 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800/60"
                                rawPayload={log.response_body}
                                concisePayload={conciseResponseBody}
                                headersJson={log.response_headers}
                                viewMode={payloadViewMode}
                                emptyPlaceholder={t('monitor.details.payload_empty', '无响应报文')}
                                duration={log.duration}
                                timingNode={timingNode}
                                onCopy={async (text) => {
                                    const success = await copyToClipboard(text);
                                    if (success) {
                                        onCopiedCardChange('resp');
                                        setTimeout(() => onCopiedCardChange(null), COPY_FEEDBACK_TIMEOUT_MS);
                                    }
                                }}
                                isCopied={copiedCard === 'resp'}
                            />
                        </div>
                    </div>
                </div>
            </div>
    );
};
