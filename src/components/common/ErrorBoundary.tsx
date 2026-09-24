import { Component, type ErrorInfo, type ReactNode } from 'react';
import { AlertCircle, RefreshCw, Home, Copy, Check } from 'lucide-react';

interface Props {
  children: ReactNode;
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
    this.setState({ errorInfo });
  }

  private handleReload = (): void => {
    window.location.reload();
  };

  private handleGoHome = (): void => {
    window.location.href = '/';
  };

  private handleCopyError = (): void => {
    const { error, errorInfo } = this.state;
    const errorText = `[UI Error]\nMessage: ${error?.message || 'Unknown'}\nStack: ${error?.stack || ''}\nComponent Stack: ${errorInfo?.componentStack || ''}`;
    navigator.clipboard.writeText(errorText);
    this.setState({ copied: true });
    setTimeout(() => this.setState({ copied: false }), 2000);
  };

  public override render(): ReactNode {
    if (this.state.hasError) {
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
