import React, { useState } from 'react';
import {
  AlertCircle,
  X,
  Trash2,
  Search,
  ChevronRight,
} from 'lucide-react';
import { useErrorStore, type CapturedError } from '../../stores/error-store';

interface ErrorHistoryDrawerProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ErrorHistoryDrawer({ isOpen, onClose }: ErrorHistoryDrawerProps): React.ReactNode {
  const { recentErrors, openErrorModal, clearRecentErrors } = useErrorStore();
  const [searchQuery, setSearchQuery] = useState('');

  if (!isOpen) {
    return null;
  }

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
    openErrorModal(error);
  };

  return (
    <div className="fixed inset-0 z-[9998] flex">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/40 backdrop-blur-xs transition-opacity"
        onClick={onClose}
      />

      {/* Slide-in Drawer from Left */}
      <div className="relative z-10 w-full max-w-md bg-white dark:bg-base-100 shadow-2xl flex flex-col h-full border-r border-gray-200 dark:border-base-300 animate-in slide-in-from-left duration-200">
        {/* Header */}
        <div className="p-4 border-b border-gray-100 dark:border-base-300 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="p-1.5 rounded-lg bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400">
              <AlertCircle className="w-4 h-4" />
            </div>
            <div>
              <h3 className="font-bold text-sm text-gray-900 dark:text-gray-100">
                Error Manager History
              </h3>
              <p className="text-[11px] text-gray-400">
                {recentErrors.length} captured error(s)
              </p>
            </div>
          </div>

          <div className="flex items-center gap-1">
            {recentErrors.length > 0 && (
              <button
                type="button"
                onClick={clearRecentErrors}
                className="p-1.5 rounded-md text-gray-400 hover:text-red-600 hover:bg-gray-100 dark:hover:bg-base-200 transition-colors"
                title="Clear error history"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            )}
            <button
              type="button"
              onClick={onClose}
              className="p-1.5 rounded-md text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-base-200 transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Search */}
        <div className="p-3 border-b border-gray-100 dark:border-base-300">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search errors by message, code, endpoint..."
              className="w-full pl-9 pr-3 py-1.5 text-xs bg-gray-50 dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
          </div>
        </div>

        {/* Errors List */}
        <div className="flex-1 overflow-y-auto p-3 space-y-2">
          {filteredErrors.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-48 text-center text-gray-400">
              <AlertCircle className="w-8 h-8 opacity-30 mb-2" />
              <p className="text-xs font-medium">No errors found</p>
              <p className="text-[10px] text-gray-400 mt-0.5">
                Runtime errors and IPC exceptions will be logged here
              </p>
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
                  className="group p-3 rounded-xl border border-gray-200/80 dark:border-base-300 bg-white dark:bg-base-200/50 hover:bg-red-50/40 dark:hover:bg-red-950/20 hover:border-red-200 dark:hover:border-red-900/40 transition-all cursor-pointer flex flex-col gap-1.5"
                >
                  <div className="flex items-center justify-between gap-2">
                    <div className="flex items-center gap-1.5">
                      <span className="px-1.5 py-0.5 text-[10px] font-mono font-bold rounded bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300">
                        {err.code}
                      </span>
                      {err.endpoint && (
                        <span className="text-[10px] font-mono text-gray-500 dark:text-gray-400 truncate max-w-[160px]">
                          {err.endpoint}
                        </span>
                      )}
                    </div>
                    <span className="text-[10px] text-gray-400 font-mono">
                      {time}
                    </span>
                  </div>

                  <p className="text-xs font-medium text-gray-800 dark:text-gray-200 line-clamp-2 break-words">
                    {err.message}
                  </p>

                  <div className="flex items-center justify-between text-[10px] text-gray-400 pt-0.5">
                    <span>
                      {err.parsedFrames && err.parsedFrames.length > 0
                        ? `${err.parsedFrames.length} stack frame(s)`
                        : err.backendStackTrace
                        ? 'Backend stack'
                        : 'Inspect diagnostics'}
                    </span>
                    <span className="flex items-center gap-0.5 text-blue-600 dark:text-blue-400 opacity-0 group-hover:opacity-100 transition-opacity font-medium">
                      Details <ChevronRight className="w-3 h-3" />
                    </span>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="p-3 border-t border-gray-100 dark:border-base-300 bg-gray-50/50 dark:bg-base-200/30 flex items-center justify-between text-[11px] text-gray-500">
          <span>Click any card to inspect stack trace</span>
          <span className="font-mono text-[10px]">02-spec/03-error-manage</span>
        </div>
      </div>
    </div>
  );
}
