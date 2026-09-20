import React, { useState, useRef } from 'react';
import { AlertCircle, Bug } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';
import { ErrorHistoryDrawer } from './error-history-drawer';
import { cn } from '../../utils/cn';

export function ErrorQueueBadge(): React.ReactNode {
  const { recentErrors, openErrorModal } = useErrorStore();
  const [drawerOpen, setDrawerOpen] = useState(false);
  const clickTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const errorCount = recentErrors.length;
  const hasErrors = errorCount > 0;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (clickTimeoutRef.current) {
      clearTimeout(clickTimeoutRef.current);
      clickTimeoutRef.current = null;
    }
    clickTimeoutRef.current = setTimeout(() => {
      setDrawerOpen(true);
      clickTimeoutRef.current = null;
    }, 220);
  };

  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (clickTimeoutRef.current) {
      clearTimeout(clickTimeoutRef.current);
      clickTimeoutRef.current = null;
    }
    setDrawerOpen(false);
    if (hasErrors) {
      openErrorModal(recentErrors[0], 'stack');
    } else {
      const diag = useErrorStore.getState().captureError(
        {
          message: 'Error Manager Diagnostics — System Healthy',
          code: 'E1000',
          level: 'info',
          details: 'Application services and background monitors are running normally. No fatal exceptions reported.',
        },
        {
          triggerAction: 'user_double_click_badge',
          source: 'error_queue_badge',
        }
      );
      openErrorModal(diag, 'stack');
    }
  };

  return (
    <>
      <button
        type="button"
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
        className={cn(
          "relative flex items-center gap-1.5 px-2 py-1 rounded-lg transition-all duration-200 cursor-pointer text-xs",
          hasErrors
            ? "bg-rose-50 hover:bg-rose-100 dark:bg-rose-950/50 dark:hover:bg-rose-900/60 text-rose-600 dark:text-rose-400 border border-rose-200/80 dark:border-rose-800/80 shadow-xs"
            : "text-gray-500 hover:text-gray-700 dark:text-slate-400 dark:hover:text-slate-200 hover:bg-gray-100 dark:hover:bg-slate-800 border border-transparent"
        )}
        title={
          hasErrors
            ? `Error Manager: ${errorCount} error(s) captured. Click to open History Drawer | Double-click for Stack Trace.`
            : "Error Manager (System Healthy). Click to open History Drawer | Double-click for Diagnostics & Stack Trace."
        }
      >
        {hasErrors ? (
          <AlertCircle className="w-3.5 h-3.5 text-rose-600 dark:text-rose-400 animate-pulse" />
        ) : (
          <Bug className="w-3.5 h-3.5 opacity-70" />
        )}

        <span
          className={cn(
            "text-[10px] font-mono font-bold px-1 py-0.2 rounded leading-tight",
            hasErrors
              ? "bg-rose-600 text-white"
              : "bg-gray-200/70 dark:bg-white/10 text-gray-600 dark:text-gray-400"
          )}
        >
          {errorCount > 99 ? '99+' : errorCount}
        </span>
      </button>

      <ErrorHistoryDrawer
        isOpen={drawerOpen}
        onClose={() => setDrawerOpen(false)}
      />
    </>
  );
}
