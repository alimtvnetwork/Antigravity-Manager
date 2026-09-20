import React, { useState } from 'react';
import {
  AlertCircle,
  X,
  Trash2,
  Search,
  ChevronRight,
  Copy,
  Check,
  Download,
} from 'lucide-react';
import { useErrorStore, type CapturedError } from '../../stores/error-store';
import {
  generateAllErrorsMarkdownReport,
  generateCompactReport,
} from '../../lib/error-report-generator';
import { showToast } from '../common/ToastContainer';

interface ErrorHistoryDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ErrorHistoryDrawer({ isOpen, onClose }: ErrorHistoryDrawerProps): React.ReactNode {
  const { recentErrors, openErrorModal, clearRecentErrors, removeError } = useErrorStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [copiedAll, setCopiedAll] = useState(false);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [downloading, setDownloading] = useState(false);

  if (!isOpen) {
    return null;
  }

  const handleRemoveSingle = (error: CapturedError, e: React.MouseEvent) => {
    e.stopPropagation();
    removeError(error.id);
    showToast(`Removed error [${error.code}] from history`, 'info');
  };

  const query = searchQuery.trim().toLowerCase();
  const filteredErrors = recentErrors.filter((err) => {
    if (!query) return true;
    return (
      err.message.toLowerCase().includes(query) ||
      err.code.toLowerCase().includes(query) ||
      (err.endpoint && err.endpoint.toLowerCase().includes(query)) ||
      (err.details && err.details.toLowerCase().includes(query))
    );
  });

  const handleSelectError = (error: CapturedError) => {
    openErrorModal(error, 'stack');
  };

  const handleCopySingle = async (error: CapturedError, e: React.MouseEvent) => {
    e.stopPropagation();
    const text = generateCompactReport(error);
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(error.id);
      setTimeout(() => {
        setCopiedId((curr) => (curr === error.id ? null : curr));
      }, 2000);
      showToast(`Copied [${error.code}] diagnostic report!`, 'success');
    } catch (err) {
      console.error('Failed to copy single error report:', err);
      showToast('Failed to copy error report', 'error');
    }
  };

  const handleCopyAll = async () => {
    if (recentErrors.length === 0) {
      showToast('No errors to copy', 'info');
      return;
    }
    const markdown = generateAllErrorsMarkdownReport(recentErrors);
    try {
      await navigator.clipboard.writeText(markdown);
      setCopiedAll(true);
      setTimeout(() => setCopiedAll(false), 2000);
      showToast(`Copied ${recentErrors.length} error logs to clipboard (Markdown)!`, 'success');
    } catch (err) {
      console.error('Failed to copy error logs:', err);
      showToast('Failed to copy to clipboard', 'error');
    }
  };

  const handleDownloadMd = () => {
    if (recentErrors.length === 0) {
      showToast('No errors to download', 'info');
      return;
    }
    const markdown = generateAllErrorsMarkdownReport(recentErrors);
    try {
      setDownloading(true);
      const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
      link.href = url;
      link.download = `error-manager-history-${timestamp}.md`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
      setTimeout(() => setDownloading(false), 1200);
      showToast('Downloaded error history report (.md)!', 'success');
    } catch (err) {
      console.error('Failed to download markdown report:', err);
      setDownloading(false);
      showToast('Failed to download report', 'error');
    }
  };

  return (
    <div className="fixed inset-0 z-[9998] flex">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 backdrop-blur-xs transition-opacity"
        onClick={onClose}
      />

      {/* Slide-in Drawer from Left */}
      <div className="relative z-10 w-full max-w-md bg-slate-50 dark:bg-slate-900 shadow-2xl flex flex-col h-full border-r border-slate-200 dark:border-slate-800 animate-in slide-in-from-left duration-200">
        {/* Header */}
        <div className="p-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-white dark:bg-slate-900/90">
          <div className="flex items-center gap-2.5">
            <div className="p-1.5 rounded-lg bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20 dark:border-red-500/30">
              <AlertCircle className="w-4 h-4" />
            </div>
            <div>
              <h3 className="font-bold text-sm text-slate-900 dark:text-slate-100">
                Error Manager History
              </h3>
              <p className="text-[11px] text-slate-500 dark:text-slate-400">
                {recentErrors.length} captured error(s)
              </p>
            </div>
          </div>

          <div className="flex items-center gap-1">
            {recentErrors.length > 0 && (
              <>
                <button
                  type="button"
                  onClick={handleCopyAll}
                  className="px-2 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors flex items-center gap-1 text-xs cursor-pointer border border-transparent hover:border-slate-200 dark:hover:border-slate-700"
                  title="Copy all error logs as Markdown"
                >
                  {copiedAll ? (
                    <Check className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                  ) : (
                    <Copy className="w-3.5 h-3.5" />
                  )}
                  <span className="text-[11px] font-medium hidden sm:inline">
                    {copiedAll ? 'Copied' : 'Copy All'}
                  </span>
                </button>

                <button
                  type="button"
                  onClick={handleDownloadMd}
                  disabled={downloading}
                  className="px-2 py-1.5 rounded-lg text-slate-700 dark:text-slate-200 hover:text-emerald-600 dark:hover:text-emerald-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors flex items-center gap-1 text-xs cursor-pointer border border-transparent hover:border-slate-200 dark:hover:border-slate-700 disabled:opacity-50"
                  title="Download all error logs as a single .md file"
                >
                  <Download className="w-3.5 h-3.5" />
                  <span className="text-[11px] font-medium hidden sm:inline">Export .md</span>
                </button>

                <button
                  type="button"
                  onClick={clearRecentErrors}
                  className="p-1.5 rounded-lg text-slate-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
                  title="Clear error history"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </>
            )}
            <button
              type="button"
              onClick={onClose}
              className="p-1.5 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
              title="Close history drawer"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Search */}
        <div className="p-3 border-b border-slate-200 dark:border-slate-800 bg-slate-100/50 dark:bg-slate-900/50">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-400 dark:text-slate-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search errors by message, code, endpoint..."
              className="w-full pl-9 pr-3 py-1.5 text-xs bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-100 placeholder:text-slate-400 dark:placeholder:text-slate-500 border border-slate-200 dark:border-slate-700 rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
        </div>

        {/* Errors List */}
        <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
          {filteredErrors.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-48 text-center text-slate-400 dark:text-slate-400 px-4">
              <AlertCircle className="w-8 h-8 opacity-30 mb-2" />
              <p className="text-xs font-medium text-slate-600 dark:text-slate-300">No errors recorded</p>
              <p className="text-[10px] text-slate-400 dark:text-slate-400 mt-0.5 mb-3">
                Runtime errors and IPC exceptions will be logged here
              </p>
              <button
                type="button"
                onClick={() => {
                  const diag = useErrorStore.getState().captureError(
                    {
                      message: 'System Status & Error Diagnostics',
                      code: 'E1000',
                      level: 'info',
                      details: 'All core modules and background tasks are healthy.',
                    },
                    { triggerAction: 'drawer_diagnostics_inspect', source: 'error_history_drawer' }
                  );
                  openErrorModal(diag, 'stack');
                }}
                className="px-3 py-1.5 text-xs font-medium text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950/40 rounded-lg border border-blue-200 dark:border-blue-800 transition-colors cursor-pointer"
              >
                Inspect Diagnostics Modal
              </button>
            </div>
          ) : (
            filteredErrors.map((err) => {
              const time = new Date(err.createdAt).toLocaleTimeString([], {
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
              });

              return (
                <div
                  key={err.id}
                  onClick={() => handleSelectError(err)}
                  onDoubleClick={() => handleSelectError(err)}
                  className="error-history-card group p-3.5 rounded-xl transition-all cursor-pointer flex flex-col gap-2 shadow-xs"
                >
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-2 min-w-0">
                      <span className="px-1.5 py-0.5 text-[10px] font-mono font-bold rounded bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20 dark:border-red-500/30 shrink-0">
                        {err.code}
                      </span>
                      {err.endpoint && (
                        <span className="text-[11px] font-mono text-slate-600 dark:text-slate-300 font-medium truncate">
                          {err.endpoint}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-1.5 shrink-0">
                      <span className="text-[11px] text-slate-500 dark:text-slate-400 font-mono">
                        {time}
                      </span>
                    </div>
                  </div>

                  <p className="text-xs font-semibold text-slate-900 dark:text-slate-100 line-clamp-2 break-words leading-relaxed">
                    {err.message}
                  </p>

                  <div className="flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400 pt-1.5 border-t border-slate-200/50 dark:border-slate-700/50 mt-0.5">
                    <span className="truncate max-w-[150px] sm:max-w-[180px]">
                      {err.parsedFrames && err.parsedFrames.length > 0
                        ? `${err.parsedFrames.length} stack frame(s)`
                        : err.backendStackTrace
                        ? 'Backend stack'
                        : 'Inspect diagnostics'}
                    </span>

                    <div className="flex items-center gap-1.5 shrink-0">
                      <button
                        type="button"
                        onClick={(e) => handleCopySingle(err, e)}
                        className="px-2 py-1 rounded-md text-slate-600 dark:text-slate-300 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-slate-200/70 dark:hover:bg-slate-700/80 transition-colors flex items-center gap-1 text-[11px] font-medium border border-slate-200 dark:border-slate-700 bg-white/60 dark:bg-slate-800/60 cursor-pointer shadow-2xs"
                        title="Copy this error's diagnostic report as Markdown"
                      >
                        {copiedId === err.id ? (
                          <>
                            <Check className="w-3 h-3 text-emerald-500 shrink-0" />
                            <span className="text-emerald-600 dark:text-emerald-400 font-semibold text-[10px]">
                              Copied!
                            </span>
                          </>
                        ) : (
                          <>
                            <Copy className="w-3 h-3 text-slate-500 dark:text-slate-400 shrink-0" />
                            <span className="text-[10px]">Copy</span>
                          </>
                        )}
                      </button>

                      <button
                        type="button"
                        onClick={(e) => handleRemoveSingle(err, e)}
                        className="p-1 rounded-md text-slate-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors cursor-pointer border border-transparent hover:border-red-200 dark:hover:border-red-900/40"
                        title="Remove this error from history"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>

                      <button
                        type="button"
                        onClick={() => handleSelectError(err)}
                        className="px-2 py-1 rounded-md text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950/40 transition-colors flex items-center gap-0.5 font-semibold text-[11px] cursor-pointer"
                        title="Inspect stack trace and full diagnostics"
                      >
                        <span>Details</span>
                        <ChevronRight className="w-3.5 h-3.5 group-hover:translate-x-0.5 transition-transform" />
                      </button>
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="p-3 border-t border-slate-200 dark:border-slate-800 bg-slate-100/70 dark:bg-slate-900/80 flex items-center justify-between text-[11px] text-slate-500 dark:text-slate-400">
          <span>Click any card to inspect stack trace</span>
          <span className="font-mono text-[10px]">02-spec/03-error-manage</span>
        </div>
      </div>
    </div>
  );
}
