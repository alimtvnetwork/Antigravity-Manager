import { copyToClipboard } from '../../utils/clipboard';
import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertCircle, RefreshCw, Home, Copy, Check } from 'lucide-react';
import { useErrorStore } from '../../stores/error-store';

interface Props {
  children: ReactNode;
  variant?: 'full' | 'card' | 'inline';
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  copied: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  public override state: State = {
    hasError: false,
    error: null,
    errorInfo: null,
    copied: false,
  };

  public static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
      errorInfo: null,
      copied: false,
    };
  }

  public override componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error('[ErrorBoundary] Uncaught UI exception caught:', error, errorInfo);
    // Tracked in the error module (no modal: the boundary already renders its own fallback UI).
    useErrorStore.getState().trackWarning(error, {
      source: 'ErrorBoundary.componentDidCatch',
      triggerAction: 'uncaught_ui_exception',
      context: { componentStack: errorInfo.componentStack?.slice(0, 500) ?? null },
    });
    this.setState({ errorInfo });
  }

  private handleReload = (): void => {
    window.location.reload();
  };

  private handleRetry = (): void => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  private handleGoHome = (): void => {
    window.location.href = '/';
  };

  private handleCopyError = (): void => {
    const { error, errorInfo } = this.state;
    const errorText = `[UI Error]\nMessage: ${error?.message || 'Unknown'}\nStack: ${error?.stack || ''}\nComponent Stack: ${errorInfo?.componentStack || ''}`;
    copyToClipboard(errorText);
    this.setState({ copied: true });
    setTimeout(() => this.setState({ copied: false }), 2000);
  };

  public override render(): ReactNode {
    if (this.state.hasError) {
      const { variant = 'full' } = this.props;

      if (variant === 'inline') {
        return (
          <div className="flex items-center justify-between gap-2 p-2.5 bg-red-500/10 border border-red-500/30 rounded-lg text-xs text-red-300">
            <div className="flex items-center gap-1.5 min-w-0">
              <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />
              <span className="truncate">{this.state.error?.message || 'UI component error'}</span>
            </div>
            <button
              type="button"
              onClick={this.handleRetry}
              className="px-2 py-0.5 bg-red-500/20 hover:bg-red-500/30 text-red-200 rounded text-[11px] font-medium transition-colors cursor-pointer shrink-0"
            >
              Retry
            </button>
          </div>
        );
      }

      if (variant === 'card') {
        return (
          <div className="w-full bg-slate-900/90 border border-red-500/30 rounded-xl p-4 shadow-xl backdrop-blur-md text-slate-100">
            <div className="flex items-center gap-2 mb-2">
              <AlertCircle className="w-5 h-5 text-red-400 shrink-0" />
              <h2 className="text-sm font-semibold text-white">Section Error</h2>
            </div>
            {this.state.error && (
              <div className="mb-3 p-2 bg-slate-950/80 rounded border border-slate-700/50 text-xs font-mono text-red-300 overflow-x-auto max-h-24">
                {this.state.error.message}
              </div>
            )}
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={this.handleRetry}
                className="inline-flex items-center gap-1 px-2.5 py-1 bg-blue-600 hover:bg-blue-500 text-white rounded text-xs font-medium transition-colors cursor-pointer"
              >
                <RefreshCw className="w-3 h-3" />
                Retry Component
              </button>
              <button
                type="button"
                onClick={this.handleCopyError}
                className="inline-flex items-center gap-1 px-2.5 py-1 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded text-xs font-medium transition-colors cursor-pointer ml-auto"
              >
                {this.state.copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                {this.state.copied ? 'Copied' : 'Copy'}
              </button>
            </div>
          </div>
        );
      }

      return (
        <div className="min-h-screen w-full flex items-center justify-center bg-slate-900 text-slate-100 p-6">
          <div className="max-w-lg w-full bg-slate-800/90 border border-red-500/30 rounded-2xl p-6 shadow-2xl backdrop-blur-md">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center text-red-400 shrink-0">
                <AlertCircle className="w-6 h-6" />
              </div>
              <div>
                <h1 className="text-lg font-semibold text-white">Something went wrong</h1>
                <p className="text-xs text-slate-400">The application encountered an unexpected UI error.</p>
              </div>
            </div>

            {this.state.error && (
              <div className="mb-4 p-3 bg-slate-950/80 rounded-lg border border-slate-700/50 text-xs font-mono text-red-300 overflow-x-auto max-h-40">
                {this.state.error.message}
              </div>
            )}

            <div className="flex flex-wrap items-center gap-2 pt-2 border-t border-slate-700/50">
              <button
                type="button"
                onClick={this.handleReload}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-medium transition-colors cursor-pointer"
              >
                <RefreshCw className="w-3.5 h-3.5" />
                Reload App
              </button>
              <button
                type="button"
                onClick={this.handleGoHome}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded-lg text-xs font-medium transition-colors cursor-pointer"
              >
                <Home className="w-3.5 h-3.5" />
                Dashboard
              </button>
              <button
                type="button"
                onClick={this.handleCopyError}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded-lg text-xs font-medium transition-colors cursor-pointer ml-auto"
              >
                {this.state.copied ? (
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

    return this.props.children;
  }
}
