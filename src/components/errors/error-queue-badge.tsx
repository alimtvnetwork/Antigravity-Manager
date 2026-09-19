import React, { useState } from 'react';
import { AlertCircle, Bug } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';
import { ErrorHistoryDrawer } from './error-history-drawer';
import { cn } from '../../utils/cn';

export function ErrorQueueBadge(): React.ReactNode {
  const { recentErrors, openErrorModal } = useErrorStore();
  const [drawerOpen, setDrawerOpen] = useState(false);

  const errorCount = recentErrors.length;
  const hasErrors = errorCount > 0;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    setDrawerOpen(true);
  };

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (hasErrors) {
      openErrorModal(recentErrors[0]);
    } else {
      setDrawerOpen(true);
    }
  };

  return (
    <>
      <button
        type="button"
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
        className={cn(
          "relative flex items-center justify-center p-1.5 rounded-lg transition-all duration-200 cursor-pointer",
          hasErrors
            ? "bg-rose-50 hover:bg-rose-100 dark:bg-rose-950/40 dark:hover:bg-rose-900/50 text-rose-600 dark:text-rose-400 border border-rose-200/60 dark:border-rose-800/60 shadow-xs animate-in fade-in"
            : "text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-base-200"
        )}
        title={
          hasErrors
            ? `Error Manager: ${errorCount} error(s) captured. Click to browse, double-click for latest stack trace.`
            : "Error Manager (No active errors). Click to open history."
        }
      >
        {hasErrors ? (
          <AlertCircle className="w-4 h-4 text-rose-600 dark:text-rose-400" />
        ) : (
          <Bug className="w-4 h-4 opacity-50" />
        )}

        {hasErrors && (
          <span className="absolute -top-1 -right-1 flex items-center justify-center min-w-4 h-4 px-1 text-[9px] font-bold font-mono text-white bg-rose-600 rounded-full shadow-xs">
            {errorCount > 99 ? '99+' : errorCount}
          </span>
        )}
      </button>

      <ErrorHistoryDrawer
        isOpen={drawerOpen}
        onClose={() => setDrawerOpen(false)}
      />
    </>
  );
}
