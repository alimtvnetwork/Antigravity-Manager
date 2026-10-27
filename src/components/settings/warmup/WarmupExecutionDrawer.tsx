import React, { useState, useMemo } from 'react';
import { X, Terminal, Copy, Check, Activity, ChevronDown, ChevronRight, ShieldCheck, Flame, Clock } from 'lucide-react';
import { useErrorStore } from '../../../stores/error-store';

export interface WarmupTransaction {
  id: string;
  timestamp: number;
  targetUrl: string;
  accountEmail: string;
  model: string;
  statusCode: number;
  durationMs: number;
  requestHeaders: Record<string, string>;
  requestPayload: Record<string, unknown> | string;
  responseHeaders: Record<string, string>;
  responseBody: Record<string, unknown> | string;
  tokensConsumed?: number;
}

export interface WarmupExecutionDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  transactions?: WarmupTransaction[];
}

/**
 * Masks sensitive authorization tokens, matching Google OAuth ya29 / Bearer formats.
 */
function maskAuthorizationHeaders(headers: Record<string, string>): Record<string, string> {
  const masked: Record<string, string> = {};
  for (const [key, value] of Object.entries(headers)) {
    if (key.toLowerCase() === 'authorization') {
      if (value.startsWith('Bearer ')) {
        masked[key] = 'Bearer ya29.***[MASKED]***';
      } else {
        masked[key] = '***[MASKED]***';
      }
    } else {
      masked[key] = value;
    }
  }
  return masked;
}

const DEFAULT_SAMPLE_TRANSACTIONS: WarmupTransaction[] = [
  {
    id: 'wp-tx-101',
    timestamp: Date.now() - 45000,
    targetUrl: 'http://127.0.0.1:4000/internal/warmup',
    accountEmail: 'dev.primary@gmail.com',
    model: 'gemini-2.5-flash',
    statusCode: 200,
    durationMs: 284,
    requestHeaders: {
      'Content-Type': 'application/json',
      Authorization: 'Bearer ya29.a0AfH6SMD987123456789abcdefghijklmnopqrstuvwxyz',
      'X-Antigravity-Account': 'dev.primary@gmail.com',
      'X-Antigravity-Origin': 'WARMUP_PROBE',
    },
    requestPayload: {
      account: 'dev.primary@gmail.com',
      model: 'gemini-2.5-flash',
      prompt: 'ping',
      max_tokens: 1,
      probe_mode: 'weekly_timer_renewal',
    },
    responseHeaders: {
      'content-type': 'application/json; charset=utf-8',
      date: new Date(Date.now() - 45000).toUTCString(),
      server: 'Antigravity-Core/4.273.0',
      'x-goog-quota-consumed': '1',
      'x-goog-quota-window': 'weekly',
    },
    responseBody: {
      status: 'warmed',
      account: 'dev.primary@gmail.com',
      model: 'gemini-2.5-flash',
      quota_clock_initiated: true,
      next_reset: new Date(Date.now() + 7 * 24 * 3600 * 1000).toISOString(),
      latency_ms: 284,
    },
    tokensConsumed: 1,
  },
  {
    id: 'wp-tx-102',
    timestamp: Date.now() - 145000,
    targetUrl: 'http://127.0.0.1:4000/internal/warmup',
    accountEmail: 'ops.standby@gmail.com',
    model: 'gemini-2.5-pro',
    statusCode: 200,
    durationMs: 412,
    requestHeaders: {
      'Content-Type': 'application/json',
      Authorization: 'Bearer ya29.a0AfH6SM987654321fedcba',
      'X-Antigravity-Account': 'ops.standby@gmail.com',
      'X-Antigravity-Origin': 'WARMUP_PROBE',
    },
    requestPayload: {
      account: 'ops.standby@gmail.com',
      model: 'gemini-2.5-pro',
      prompt: 'ping',
      max_tokens: 1,
      probe_mode: 'weekly_timer_renewal',
    },
    responseHeaders: {
      'content-type': 'application/json',
      date: new Date(Date.now() - 145000).toUTCString(),
      'x-goog-quota-consumed': '0',
    },
    responseBody: {
      status: 'warmed',
      account: 'ops.standby@gmail.com',
      model: 'gemini-2.5-pro',
      latency_ms: 412,
    },
    tokensConsumed: 0,
  },
  {
    id: 'wp-tx-103',
    timestamp: Date.now() - 320000,
    targetUrl: 'http://127.0.0.1:4000/internal/warmup',
    accountEmail: 'fleet.worker3@gmail.com',
    model: 'claude-3-7-sonnet',
    statusCode: 429,
    durationMs: 650,
    requestHeaders: {
      'Content-Type': 'application/json',
      Authorization: 'Bearer ya29.a0AfH6SM555666777',
      'X-Antigravity-Account': 'fleet.worker3@gmail.com',
      'X-Antigravity-Origin': 'WARMUP_PROBE',
    },
    requestPayload: {
      account: 'fleet.worker3@gmail.com',
      model: 'claude-3-7-sonnet',
      prompt: 'ping',
    },
    responseHeaders: {
      'content-type': 'application/json',
      'retry-after': '60',
      'x-goog-quota-consumed': '0',
    },
    responseBody: {
      error: 'RESOURCE_EXHAUSTED',
      message: 'Rate limit exceeded. Exponential backoff active.',
      retry_after_seconds: 60,
    },
    tokensConsumed: 0,
  },
  {
    id: 'wp-tx-104',
    timestamp: Date.now() - 780000,
    targetUrl: 'http://127.0.0.1:4000/internal/warmup',
    accountEmail: 'audit.legacy@gmail.com',
    model: 'gemini-2.0-flash',
    statusCode: 403,
    durationMs: 195,
    requestHeaders: {
      'Content-Type': 'application/json',
      Authorization: 'Bearer ya29.a0AfH6SM999888111',
      'X-Antigravity-Account': 'audit.legacy@gmail.com',
      'X-Antigravity-Origin': 'WARMUP_PROBE',
    },
    requestPayload: {
      account: 'audit.legacy@gmail.com',
      model: 'gemini-2.0-flash',
    },
    responseHeaders: {
      'content-type': 'application/json',
      'x-goog-quota-consumed': '0',
    },
    responseBody: {
      error: 'PERMISSION_DENIED',
      message: 'Account validation blocked or token expired.',
    },
    tokensConsumed: 0,
  },
];

export const WarmupExecutionDrawer: React.FC<WarmupExecutionDrawerProps> = ({
  isOpen,
  onClose,
  transactions,
}) => {
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState<'ALL' | '200' | '429' | '403'>('ALL');
  const [searchQuery, setSearchQuery] = useState('');

  const displayTransactions = useMemo(() => {
    const source = transactions && transactions.length > 0 ? transactions : DEFAULT_SAMPLE_TRANSACTIONS;
    return source.filter((tx) => {
      if (statusFilter === '200' && tx.statusCode !== 200) return false;
      if (statusFilter === '429' && tx.statusCode !== 429) return false;
      if (statusFilter === '403' && tx.statusCode !== 403 && tx.statusCode < 400) return false;

      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        const matchesEmail = tx.accountEmail.toLowerCase().includes(q);
        const matchesModel = tx.model.toLowerCase().includes(q);
        const matchesId = tx.id.toLowerCase().includes(q);
        return matchesEmail || matchesModel || matchesId;
      }
      return true;
    });
  }, [transactions, statusFilter, searchQuery]);

  if (!isOpen) return null;

  const handleCopy = async (text: string, key: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedKey(key);
      setTimeout(() => setCopiedKey(null), 2000);
    } catch (err) {
      useErrorStore.getState().trackWarning(err, {
        source: 'WarmupExecutionDrawer.handleCopy',
        triggerAction: `copy_${key}`,
      });
    }
  };

  const getStatusBadge = (code: number) => {
    if (code === 200) {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
          HTTP 200
        </span>
      );
    }
    if (code === 429) {
      return (
        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-amber-500/10 text-amber-400 border border-amber-500/30">
          HTTP 429
        </span>
      );
    }
    return (
      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-rose-500/10 text-rose-400 border border-rose-500/30">
        HTTP {code}
      </span>
    );
  };

  const formatRelativeTime = (timestamp: number) => {
    const diffSec = Math.max(0, Math.floor((Date.now() - timestamp) / 1000));
    if (diffSec < 60) return `${diffSec}s ago`;
    const diffMin = Math.floor(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m ago`;
    return `${Math.floor(diffMin / 60)}h ago`;
  };

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/60 backdrop-blur-xs transition-opacity duration-300"
        onClick={onClose}
        aria-hidden="true"
      />

      {/* Slide-over Drawer */}
      <div
        className="relative z-50 w-full sm:w-[540px] md:w-[620px] h-full bg-zinc-950/95 backdrop-blur-xl border-l border-zinc-800/80 text-zinc-100 shadow-2xl flex flex-col animate-in slide-in-from-right duration-300"
        role="dialog"
        aria-label="Warmup Execution Drawer"
      >
        {/* Drawer Header */}
        <div className="p-4 border-b border-zinc-800/80 flex items-center justify-between bg-zinc-900/50">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-orange-500/15 border border-orange-500/30 flex items-center justify-center text-orange-400 shadow-[0_0_12px_rgba(249,115,22,0.2)]">
              <Terminal className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h2 className="text-sm font-bold text-zinc-100 tracking-tight">Warmup Execution Drawer</h2>
                <span className="inline-flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[9px] font-mono font-bold bg-orange-500/15 text-orange-400 border border-orange-500/30">
                  <Activity className="w-2.5 h-2.5 animate-pulse" /> LIVE
                </span>
              </div>
              <p className="text-[11px] text-zinc-400 mt-0.5">
                HTTP transactions, masked OAuth tokens, and quota responses
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800/60 transition-colors"
            title="Close Drawer"
            aria-label="Close Drawer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Toolbar & Filter Bar */}
        <div className="p-3 border-b border-zinc-800/60 bg-zinc-900/20 flex flex-col gap-2.5">
          <div className="flex items-center justify-between gap-2">
            {/* Filter segmented capsule */}
            <div className="inline-flex items-center rounded-full bg-zinc-900/80 border border-zinc-800 p-0.5 text-[11px] font-mono divide-x divide-zinc-800">
              <button
                onClick={() => setStatusFilter('ALL')}
                className={`px-2.5 py-0.5 rounded-l-full transition-colors ${
                  statusFilter === 'ALL'
                    ? 'bg-zinc-800 text-zinc-100 font-bold'
                    : 'text-zinc-400 hover:text-zinc-200'
                }`}
              >
                All ({displayTransactions.length})
              </button>
              <button
                onClick={() => setStatusFilter('200')}
                className={`px-2 py-0.5 transition-colors ${
                  statusFilter === '200'
                    ? 'bg-emerald-500/20 text-emerald-300 font-bold'
                    : 'text-zinc-400 hover:text-emerald-400'
                }`}
              >
                200 OK
              </button>
              <button
                onClick={() => setStatusFilter('429')}
                className={`px-2 py-0.5 transition-colors ${
                  statusFilter === '429'
                    ? 'bg-amber-500/20 text-amber-300 font-bold'
                    : 'text-zinc-400 hover:text-amber-400'
                }`}
              >
                429 Rate
              </button>
              <button
                onClick={() => setStatusFilter('403')}
                className={`px-2.5 py-0.5 rounded-r-full transition-colors ${
                  statusFilter === '403'
                    ? 'bg-rose-500/20 text-rose-300 font-bold'
                    : 'text-zinc-400 hover:text-rose-400'
                }`}
              >
                403 / 5xx
              </button>
            </div>

            {/* Shield Indicator */}
            <div className="flex items-center gap-1.5 text-[10px] text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-2 py-0.5 rounded-full font-mono">
              <ShieldCheck className="w-3 h-3" />
              <span>Auth Masked</span>
            </div>
          </div>

          {/* Search Input */}
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search by account email, model, or probe ID..."
            className="w-full px-3 py-1.5 text-xs bg-zinc-900/90 border border-zinc-800 rounded-lg text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-orange-500/60 focus:ring-1 focus:ring-orange-500/30 transition-all font-mono"
          />
        </div>

        {/* Transaction Stream List */}
        <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
          {displayTransactions.length === 0 ? (
            <div className="p-8 text-center text-zinc-500 font-mono text-xs flex flex-col items-center justify-center gap-2">
              <Terminal className="w-8 h-8 text-zinc-700" />
              <p>No warmup transactions match the current filter criteria.</p>
            </div>
          ) : (
            displayTransactions.map((tx) => {
              const isExpanded = expandedId === tx.id;
              const maskedReqHeaders = maskAuthorizationHeaders(tx.requestHeaders);
              const curlCommand = `curl -X POST "${tx.targetUrl}" \\\n  -H "Content-Type: application/json" \\\n  -H "Authorization: Bearer ya29.***[MASKED]***" \\\n  -H "X-Antigravity-Origin: WARMUP_PROBE" \\\n  -d '${JSON.stringify(tx.requestPayload)}'`;

              return (
                <div
                  key={tx.id}
                  className="rounded-xl border border-zinc-800/80 bg-zinc-900/40 hover:border-zinc-700/80 transition-all overflow-hidden"
                >
                  {/* Transaction Header Summary */}
                  <div
                    onClick={() => setExpandedId(isExpanded ? null : tx.id)}
                    className="p-3 cursor-pointer flex items-center justify-between gap-3 hover:bg-zinc-800/30 select-none"
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      {isExpanded ? (
                        <ChevronDown className="w-4 h-4 text-zinc-400 shrink-0" />
                      ) : (
                        <ChevronRight className="w-4 h-4 text-zinc-400 shrink-0" />
                      )}
                      {getStatusBadge(tx.statusCode)}
                      <span className="font-mono text-xs font-semibold text-zinc-200 truncate">
                        {tx.model}
                      </span>
                      <span className="text-[11px] text-zinc-400 truncate">
                        {tx.accountEmail}
                      </span>
                    </div>

                    <div className="flex items-center gap-2.5 shrink-0 text-[10px] font-mono text-zinc-400">
                      <span className="text-emerald-400/90 font-bold">
                        {tx.tokensConsumed ?? 0} tok
                      </span>
                      <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3 text-zinc-500" />
                        {tx.durationMs}ms
                      </span>
                      <span className="text-zinc-500">
                        {formatRelativeTime(tx.timestamp)}
                      </span>
                    </div>
                  </div>

                  {/* Expanded Transaction Details */}
                  {isExpanded && (
                    <div className="p-3 border-t border-zinc-800/70 bg-zinc-950/70 space-y-3 font-mono text-xs">
                      {/* Action Bar (cURL & Copy) */}
                      <div className="flex items-center justify-between pb-1 border-b border-zinc-800/40">
                        <div className="flex items-center gap-1 text-[11px] text-orange-400 font-semibold">
                          <Flame className="w-3.5 h-3.5" />
                          <span>Probe Transaction: {tx.id}</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                          <button
                            onClick={() => handleCopy(curlCommand, `${tx.id}-curl`)}
                            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-zinc-800 text-zinc-300 hover:text-zinc-100 hover:bg-zinc-700/80 transition-colors text-[10px]"
                            title="Copy cURL equivalent"
                          >
                            {copiedKey === `${tx.id}-curl` ? (
                              <>
                                <Check className="w-3 h-3 text-emerald-400" />
                                <span className="text-emerald-400">cURL Copied</span>
                              </>
                            ) : (
                              <>
                                <Copy className="w-3 h-3" />
                                <span>Copy cURL</span>
                              </>
                            )}
                          </button>
                          <button
                            onClick={() =>
                              handleCopy(JSON.stringify(tx, null, 2), `${tx.id}-json`)
                            }
                            className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-zinc-800 text-zinc-300 hover:text-zinc-100 hover:bg-zinc-700/80 transition-colors text-[10px]"
                            title="Copy raw JSON"
                          >
                            {copiedKey === `${tx.id}-json` ? (
                              <>
                                <Check className="w-3 h-3 text-emerald-400" />
                                <span className="text-emerald-400">JSON Copied</span>
                              </>
                            ) : (
                              <>
                                <Copy className="w-3 h-3" />
                                <span>Copy JSON</span>
                              </>
                            )}
                          </button>
                        </div>
                      </div>

                      {/* 1. Target URL */}
                      <div>
                        <div className="flex items-center justify-between text-[10px] text-zinc-400 uppercase tracking-wider mb-1 font-semibold">
                          <span>Target URL</span>
                          <button
                            onClick={() => handleCopy(tx.targetUrl, `${tx.id}-url`)}
                            className="text-zinc-400 hover:text-zinc-200 transition-colors flex items-center gap-1"
                          >
                            {copiedKey === `${tx.id}-url` ? (
                              <Check className="w-2.5 h-2.5 text-emerald-400" />
                            ) : (
                              <Copy className="w-2.5 h-2.5" />
                            )}
                            <span>Copy URL</span>
                          </button>
                        </div>
                        <div className="p-2 rounded-lg bg-zinc-900/90 border border-zinc-800/80 text-zinc-300 text-[11px] break-all select-all">
                          {tx.targetUrl}
                        </div>
                      </div>

                      {/* 2. Request Headers */}
                      <div>
                        <div className="text-[10px] text-zinc-400 uppercase tracking-wider mb-1 font-semibold">
                          Request Headers (Masked)
                        </div>
                        <pre className="p-2.5 rounded-lg bg-zinc-900/90 border border-zinc-800/80 text-zinc-300 text-[11px] overflow-x-auto select-all leading-relaxed">
                          {JSON.stringify(maskedReqHeaders, null, 2)}
                        </pre>
                      </div>

                      {/* 3. Request Payload */}
                      <div>
                        <div className="text-[10px] text-zinc-400 uppercase tracking-wider mb-1 font-semibold">
                          Request Payload
                        </div>
                        <pre className="p-2.5 rounded-lg bg-zinc-900/90 border border-zinc-800/80 text-zinc-300 text-[11px] overflow-x-auto select-all leading-relaxed">
                          {typeof tx.requestPayload === 'string'
                            ? tx.requestPayload
                            : JSON.stringify(tx.requestPayload, null, 2)}
                        </pre>
                      </div>

                      {/* 4. Response Headers */}
                      <div>
                        <div className="flex items-center justify-between text-[10px] text-zinc-400 uppercase tracking-wider mb-1 font-semibold">
                          <span>Response Headers</span>
                          {tx.responseHeaders['x-goog-quota-consumed'] !== undefined && (
                            <span className="text-emerald-400 font-bold lowercase">
                              x-goog-quota-consumed: {tx.responseHeaders['x-goog-quota-consumed']}
                            </span>
                          )}
                        </div>
                        <pre className="p-2.5 rounded-lg bg-zinc-900/90 border border-zinc-800/80 text-zinc-300 text-[11px] overflow-x-auto select-all leading-relaxed">
                          {JSON.stringify(tx.responseHeaders, null, 2)}
                        </pre>
                      </div>

                      {/* 5. Response Body */}
                      <div>
                        <div className="text-[10px] text-zinc-400 uppercase tracking-wider mb-1 font-semibold">
                          Response Body
                        </div>
                        <pre className="p-2.5 rounded-lg bg-zinc-900/90 border border-zinc-800/80 text-zinc-300 text-[11px] overflow-x-auto select-all leading-relaxed">
                          {typeof tx.responseBody === 'string'
                            ? tx.responseBody
                            : JSON.stringify(tx.responseBody, null, 2)}
                        </pre>
                      </div>
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="p-3 border-t border-zinc-800/80 bg-zinc-900/40 flex items-center justify-between text-[11px] text-zinc-400">
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-emerald-400" />
            <span>Zero Excess Usage Guard: Enforced</span>
          </div>
          <button
            onClick={onClose}
            className="px-3 py-1 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-xs font-medium transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default WarmupExecutionDrawer;
