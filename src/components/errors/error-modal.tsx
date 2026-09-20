import React, { useState } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Info,
  Copy,
  Check,
  ChevronLeft,
  ChevronRight,
  X,
  Terminal,
  Layers,
  Wrench,
  Bot,
} from 'lucide-react';
import { useErrorStore, type CapturedError, type ErrorModalTab } from '../../stores/error-store';
import {
  generateCompactReport,
  generateJsonReport,
  getSuggestedFixes,
} from '../../lib/error-report-generator';
import { showToast } from '../common/ToastContainer';
import { cn } from '../../utils/cn';

export function ErrorModal(): React.ReactNode {
  const {
    selectedError,
    isModalOpen,
    closeErrorModal,
    errorQueue,
    currentQueueIndex,
    navigateQueue,
    activeTab,
    setActiveTab,
  } = useErrorStore();

  const [copiedAi, setCopiedAi] = useState(false);

  if (!isModalOpen) {
    return null;
  }
  if (!selectedError) {
    return null;
  }

  const handleCopyAi = () => {
    const text = generateCompactReport(selectedError);
    navigator.clipboard.writeText(text);
    setCopiedAi(true);
    setTimeout(() => setCopiedAi(false), 2000);
    showToast('Diagnostic report copied! Ready to share with AI.', 'success');
  };

  const handleCopyJson = () => {
    const text = generateJsonReport(selectedError);
    navigator.clipboard.writeText(text);
    showToast('Raw error JSON copied to clipboard.', 'info');
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden text-gray-900 dark:text-slate-100">
        <ModalHeader
          error={selectedError}
          queueLength={errorQueue.length}
          queueIndex={currentQueueIndex}
          onNavigate={navigateQueue}
          onClose={closeErrorModal}
        />

        <TabNav activeTab={activeTab} onSelect={setActiveTab} />

        <div className="flex-1 overflow-y-auto p-6 space-y-4">
          {activeTab === 'overview' && (
            <OverviewTab error={selectedError} onSelectTab={setActiveTab} />
          )}
          {activeTab === 'backend' && <BackendTab error={selectedError} />}
          {activeTab === 'stack' && <StackTab error={selectedError} />}
          {activeTab === 'context' && <ContextTab error={selectedError} />}
        </div>

        <ModalFooter
          copiedAi={copiedAi}
          onCopyAi={handleCopyAi}
          onCopyJson={handleCopyJson}
          onClose={closeErrorModal}
        />
      </div>
    </div>
  );
}

interface ModalHeaderProps {
  error: CapturedError;
  queueLength: number;
  queueIndex: number;
  onNavigate: (direction: 'prev' | 'next') => void;
  onClose: () => void;
}

function ModalHeader({
  error,
  queueLength,
  queueIndex,
  onNavigate,
  onClose,
}: ModalHeaderProps): React.ReactNode {
  const isErr = error.level === 'error';
  const isWarn = error.level === 'warn';

  return (
    <div className="px-6 py-4 border-b border-gray-200 dark:border-slate-800 flex items-center justify-between bg-gray-50/50 dark:bg-slate-900/50">
      <div className="flex items-center gap-3">
        {isErr && <AlertCircle className="w-6 h-6 text-red-500 shrink-0" />}
        {isWarn && <AlertTriangle className="w-6 h-6 text-amber-500 shrink-0" />}
        {!isErr && !isWarn && <Info className="w-6 h-6 text-blue-500 shrink-0" />}

        <div>
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs px-2 py-0.5 rounded-md font-semibold bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20">
              {error.code}
            </span>
            <span className="text-xs text-gray-500 dark:text-slate-400 font-mono">
              {new Date(error.createdAt).toLocaleTimeString()}
            </span>
          </div>
          <h3 className="text-base font-semibold leading-tight mt-1 line-clamp-1 text-gray-900 dark:text-slate-100">
            {error.message}
          </h3>
        </div>
      </div>

      <div className="flex items-center gap-2">
        {queueLength > 1 && (
          <div className="flex items-center gap-1 mr-2 px-2 py-1 rounded-lg bg-gray-100 dark:bg-slate-800 text-xs font-medium">
            <button
              onClick={() => onNavigate('prev')}
              className="p-0.5 hover:bg-gray-200 dark:hover:bg-slate-700 rounded text-gray-600 dark:text-slate-300"
              title="Previous error"
            >
              <ChevronLeft className="w-3.5 h-3.5" />
            </button>
            <span className="px-1 font-mono text-gray-700 dark:text-slate-300">
              {queueIndex + 1} / {queueLength}
            </span>
            <button
              onClick={() => onNavigate('next')}
              className="p-0.5 hover:bg-gray-200 dark:hover:bg-slate-700 rounded text-gray-600 dark:text-slate-300"
              title="Next error"
            >
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>
        )}
        <button
          onClick={onClose}
          className="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-slate-200 rounded-lg hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors"
        >
          <X className="w-5 h-5" />
        </button>
      </div>
    </div>
  );
}

interface TabNavProps {
  activeTab: ErrorModalTab;
  onSelect: (tab: ErrorModalTab) => void;
}

function TabNav({ activeTab, onSelect }: TabNavProps): React.ReactNode {
  const tabs: Array<{ id: ErrorModalTab; label: string; icon: React.ReactNode }> = [
    { id: 'stack', label: 'Stack Trace', icon: <Layers className="w-3.5 h-3.5" /> },
    { id: 'overview', label: 'Overview', icon: <Info className="w-3.5 h-3.5" /> },
    { id: 'backend', label: 'Backend Logs', icon: <Terminal className="w-3.5 h-3.5" /> },
    { id: 'context', label: 'Context', icon: <Bot className="w-3.5 h-3.5" /> },
  ];

  return (
    <div className="flex border-b border-gray-200 dark:border-slate-800 px-6 gap-2 bg-gray-50/20 dark:bg-slate-900/40">
      {tabs.map((tab) => {
        const selected = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            onClick={() => onSelect(tab.id)}
            className={cn(
              'flex items-center gap-1.5 py-3 px-3 text-xs font-medium border-b-2 transition-colors cursor-pointer',
              selected
                ? 'border-blue-500 text-blue-600 dark:text-blue-400 font-bold'
                : 'border-transparent text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'
            )}
          >
            {tab.icon}
            {tab.label}
          </button>
        );
      })}
    </div>
  );
}

function OverviewTab({
  error,
  onSelectTab,
}: {
  error: CapturedError;
  onSelectTab: (tab: ErrorModalTab) => void;
}): React.ReactNode {
  const fixes = getSuggestedFixes(error.code);

  return (
    <div className="space-y-4">
      <div className="p-4 rounded-xl bg-red-50 dark:bg-red-950/20 border border-red-200 dark:border-red-900/30">
        <div className="flex items-center justify-between mb-1">
          <h4 className="text-xs font-bold uppercase tracking-wider text-red-700 dark:text-red-400">
            Error Message
          </h4>
          <button
            type="button"
            onClick={() => onSelectTab('stack')}
            className="text-xs text-blue-600 dark:text-blue-400 hover:underline font-medium cursor-pointer"
          >
            Inspect Stack Trace →
          </button>
        </div>
        <p className="text-sm font-mono text-red-900 dark:text-red-200 break-words">
          {error.message}
        </p>
        {error.details && (
          <p className="text-xs mt-2 text-red-700 dark:text-red-300">
            {error.details}
          </p>
        )}
      </div>

      {error.uiClickPathArrow && (
        <div className="p-4 rounded-xl bg-gray-50 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-800">
          <h4 className="text-xs font-bold uppercase tracking-wider text-gray-500 mb-2">
            User Interaction Flow
          </h4>
          <p className="text-xs font-mono text-gray-700 dark:text-gray-300">
            {error.uiClickPathArrow}
          </p>
        </div>
      )}

      {fixes.length > 0 && (
        <div className="p-4 rounded-xl bg-blue-50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-900/30">
          <div className="flex items-center gap-2 mb-2">
            <Wrench className="w-4 h-4 text-blue-600 dark:text-blue-400" />
            <h4 className="text-xs font-bold uppercase tracking-wider text-blue-700 dark:text-blue-400">
              Suggested Fixes ({error.code})
            </h4>
          </div>
          <ul className="space-y-1.5 pl-4 list-disc text-xs text-blue-900 dark:text-blue-200">
            {fixes.map((fix, idx) => (
              <li key={idx}>{fix}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function BackendTab({ error }: { error: CapturedError }): React.ReactNode {
  const backendMsg = error.envelopeErrors?.BackendMessage || error.backendStackTrace || error.details || error.message;
  let backendStack: string[] = [];
  if (error.envelopeErrors?.Backend && error.envelopeErrors.Backend.length > 0) {
    backendStack = error.envelopeErrors.Backend;
  } else if (error.backendStackTrace) {
    backendStack = error.backendStackTrace.split('\n').filter(Boolean);
  } else if (error.stackTrace) {
    backendStack = error.stackTrace.split('\n').filter(Boolean);
  }

  return (
    <div className="space-y-4">
      {error.endpoint && (
        <div className="flex items-center gap-2 text-xs font-mono p-2.5 rounded-lg bg-gray-100 dark:bg-slate-800 border border-gray-200 dark:border-slate-700">
          <span className="font-semibold text-blue-600 dark:text-blue-400">
            {error.method || 'INVOKE'}
          </span>
          <span className="text-gray-600 dark:text-gray-300">{error.endpoint}</span>
          {error.responseStatus && (
            <span className="ml-auto px-2 py-0.5 rounded bg-red-100 dark:bg-red-900/40 text-red-600 dark:text-red-400 font-bold">
              {error.responseStatus}
            </span>
          )}
        </div>
      )}

      <div className="p-4 rounded-xl bg-gray-900 text-gray-100 font-mono text-xs overflow-x-auto border border-gray-800">
        <div className="text-gray-400 mb-2 font-bold uppercase text-[10px] tracking-wider">
          Rust Backend Diagnostics
        </div>
        {backendMsg && <div className="text-red-400 mb-2 whitespace-pre-wrap">{backendMsg}</div>}
        {backendStack.length > 0 ? (
          <div className="space-y-1">
            {backendStack.map((line, idx) => (
              <div key={idx} className="text-gray-300">
                {line}
              </div>
            ))}
          </div>
        ) : (
          <div className="text-gray-500 italic">No structured backend stack captured.</div>
        )}
      </div>
    </div>
  );
}

function StackTab({ error }: { error: CapturedError }): React.ReactNode {
  const frames = error.parsedFrames || [];
  const hasFrames = frames.length > 0;
  const [showRaw, setShowRaw] = useState(!hasFrames);
  const rawStack = error.stackTrace || error.backendStackTrace || error.details || error.message;

  const handleCopyStack = () => {
    if (rawStack) {
      navigator.clipboard.writeText(rawStack);
      showToast('Stack trace copied to clipboard', 'success');
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-gray-500 dark:text-slate-400">
          {hasFrames ? `${frames.length} Frame(s) Captured` : 'Raw Stack Trace'}
        </span>
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={handleCopyStack}
            className="text-xs text-blue-500 hover:underline font-medium cursor-pointer"
          >
            Copy Stack
          </button>
          {hasFrames && (
            <button
              type="button"
              onClick={() => setShowRaw(!showRaw)}
              className="text-xs text-blue-500 hover:underline font-medium cursor-pointer"
            >
              {showRaw ? 'Show Formatted Table' : 'Show Raw Stack'}
            </button>
          )}
        </div>
      </div>

      {showRaw || !hasFrames ? (
        <pre className="p-4 rounded-xl bg-gray-900 text-gray-100 font-mono text-xs overflow-x-auto max-h-80 whitespace-pre-wrap leading-relaxed border border-gray-800">
          {rawStack}
        </pre>
      ) : (
        <div className="border border-gray-200 dark:border-slate-800 rounded-xl overflow-hidden">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-gray-50 dark:bg-slate-800 text-gray-500 dark:text-slate-400 border-b border-gray-200 dark:border-slate-700">
              <tr>
                <th className="p-2.5">Function</th>
                <th className="p-2.5">File</th>
                <th className="p-2.5">Line</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100 dark:divide-slate-800">
              {frames.map((f, i) => (
                <tr key={i} className={f.isInternal ? 'opacity-40' : 'font-semibold'}>
                  <td className="p-2.5 text-blue-600 dark:text-blue-400">{f.function}</td>
                  <td className="p-2.5 text-gray-600 dark:text-gray-300 truncate max-w-xs">
                    {f.file}
                  </td>
                  <td className="p-2.5 text-gray-500 dark:text-slate-400">{f.line}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function ContextTab({ error }: { error: CapturedError }): React.ReactNode {
  const json = JSON.stringify(
    {
      context: error.context,
      route: error.route,
      triggerComponent: error.triggerComponent,
      triggerAction: error.triggerAction,
      responseStatus: error.responseStatus,
      endpoint: error.endpoint,
    },
    null,
    2
  );

  return (
    <pre className="p-4 rounded-xl bg-gray-900 text-gray-100 font-mono text-xs overflow-x-auto max-h-80 border border-gray-800">
      {json}
    </pre>
  );
}

interface ModalFooterProps {
  copiedAi: boolean;
  onCopyAi: () => void;
  onCopyJson: () => void;
  onClose: () => void;
}

function ModalFooter({
  copiedAi,
  onCopyAi,
  onCopyJson,
  onClose,
}: ModalFooterProps): React.ReactNode {
  return (
    <div className="px-6 py-4 border-t border-gray-200 dark:border-slate-800 flex items-center justify-between bg-gray-50/50 dark:bg-slate-900/50">
      <div className="flex items-center gap-2">
        <button
          onClick={onCopyAi}
          className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold text-white bg-blue-600 hover:bg-blue-700 shadow-md shadow-blue-500/20 transition-all cursor-pointer"
        >
          {copiedAi ? <Check className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
          <span>{copiedAi ? 'Copied AI Report!' : 'Copy Error for AI'}</span>
        </button>

        <button
          onClick={onCopyJson}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-medium text-gray-700 dark:text-slate-200 hover:bg-gray-100 dark:hover:bg-slate-800 border border-gray-300 dark:border-slate-700 transition-colors cursor-pointer"
        >
          <Copy className="w-3.5 h-3.5" />
          <span>Copy JSON</span>
        </button>
      </div>

      <button
        onClick={onClose}
        className="px-4 py-2 rounded-xl text-xs font-medium text-gray-600 dark:text-slate-400 hover:bg-gray-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
      >
        Dismiss
      </button>
    </div>
  );
}
