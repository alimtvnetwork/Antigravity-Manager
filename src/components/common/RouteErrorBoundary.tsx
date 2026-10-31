import { useState, useEffect } from 'react';
import { useRouteError, isRouteErrorResponse } from 'react-router-dom';
import { AlertCircle, RefreshCw, Home, Copy, Check } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';
import { copyToClipboard } from '../../utils/clipboard';

export function RouteErrorBoundary() {
  const error = useRouteError();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (error) {
      console.error('[RouteErrorBoundary] Route error caught:', error);
      const errObj = error instanceof Error ? error : new Error(String(error));
      useErrorStore.getState().trackWarning(errObj, {
        source: 'RouteErrorBoundary',
        triggerAction: 'route_error',
        context: { isRouteResponse: isRouteErrorResponse(error) },
      });
    }
  }, [error]);

  const errorMessage = error instanceof Error
    ? error.message
    : (isRouteErrorResponse(error)
      ? `${error.status} ${error.statusText}: ${typeof error.data === 'string' ? error.data : JSON.stringify(error.data)}`
      : String(error || 'Unknown route error'));

  const errorStack = error instanceof Error ? error.stack : '';

  const handleReload = () => {
    window.location.reload();
  };

  const handleGoHome = () => {
    window.location.href = '/';
  };

  const handleCopyError = () => {
    const errorText = `[Route Error]\nMessage: ${errorMessage}\nStack: ${errorStack || 'N/A'}`;
    copyToClipboard(errorText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="min-h-screen w-full flex items-center justify-center bg-slate-900 text-slate-100 p-6">
      <div className="max-w-lg w-full bg-slate-800/90 border border-red-500/30 rounded-2xl p-6 shadow-2xl backdrop-blur-md">
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center text-red-400 shrink-0">
            <AlertCircle className="w-6 h-6" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-white">Route Error Encountered</h1>
            <p className="text-xs text-slate-400">The application encountered an unexpected route or page error.</p>
          </div>
        </div>

        <div className="mb-4 p-3 bg-slate-950/80 rounded-lg border border-slate-700/50 text-xs font-mono text-red-300 overflow-x-auto max-h-40">
          {errorMessage}
        </div>

        <div className="flex flex-wrap items-center gap-2 pt-2 border-t border-slate-700/50">
          <button
            type="button"
            onClick={handleReload}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-medium transition-colors cursor-pointer"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            Reload App
          </button>
          <button
            type="button"
            onClick={handleGoHome}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded-lg text-xs font-medium transition-colors cursor-pointer"
          >
            <Home className="w-3.5 h-3.5" />
            Dashboard
          </button>
          <button
            type="button"
            onClick={handleCopyError}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded-lg text-xs font-medium transition-colors cursor-pointer ml-auto"
          >
            {copied ? (
              <>
                <Check className="w-3.5 h-3.5 text-emerald-400" />
                Copied
              </>
            ) : (
              <>
                <Copy className="w-3.5 h-3.5" />
                Copy Details
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
