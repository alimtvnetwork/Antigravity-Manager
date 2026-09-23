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
  Trash2,
  ClipboardList,
} from 'lucide-react';
import { useErrorStore, type CapturedError, type ErrorModalTab } from '../../stores/error-store';
import {
  generateCompactReport,
  generateComprehensiveAllDataReport,
  generateJsonReport,
  getSuggestedFixes,
} from '../../lib/error-report-generator';
import { showToast } from '../common/ToastContainer';
import { cn } from '../../utils/cn';

interface ItemCopyButtonProps {
  text: string;
  label?: string;
  successMessage?: string;
}

function ItemCopyButton({
  text,
  label = 'Copy',
  successMessage,
}: ItemCopyButtonProps): React.ReactNode {
  const [copied, setCopied] = useState(false);

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
    const msg = successMessage || `${label} copied to clipboard!`;
    showToast(msg, 'success');
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[11px] font-medium text-slate-700 dark:text-slate-300 hover:text-blue-600 dark:hover:text-blue-400 bg-white/80 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 hover:border-blue-300 dark:hover:border-blue-600 transition-colors cursor-pointer shadow-2xs"
      title={`Copy ${label}`}
    >
      {copied ? (
        <Check className="w-3 h-3 text-emerald-500 shrink-0" />
      ) : (
        <Copy className="w-3 h-3 shrink-0" />
      )}
      <span className={copied ? 'text-emerald-600 dark:text-emerald-400 font-semibold' : ''}>
        {copied ? 'Copied!' : label}
      </span>
    </button>
  );
}

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
    removeError,
  } = useErrorStore();

  const [copiedAi, setCopiedAi] = useState(false);
  const [copiedAll, setCopiedAll] = useState(false);

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
    showToast('Diagnostic AI report copied! Ready to share with AI.', 'success');
  };

  const handleCopyAllData = () => {
    const text = generateComprehensiveAllDataReport(selectedError);
    navigator.clipboard.writeText(text);
    setCopiedAll(true);
    setTimeout(() => setCopiedAll(false), 2000);
    showToast('All comprehensive error data copied to clipboard!', 'success');
  };

  const handleCopyJson = () => {
    const text = generateJsonReport(selectedError);
    navigator.clipboard.writeText(text);
    showToast('Raw error JSON copied to clipboard.', 'info');
  };

  const handleRemove = () => {
    removeError(selectedError.id);
    showToast(`Removed error [${selectedError.code}] from history`, 'info');
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden text-slate-900 dark:text-slate-100">
        <ModalHeader
          error={selectedError}
          queueLength={errorQueue.length}
          queueIndex={currentQueueIndex}
          onNavigate={navigateQueue}
          onClose={closeErrorModal}
          onCopyAllData={handleCopyAllData}
          onRemove={handleRemove}
          copiedAll={copiedAll}
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
          copiedAll={copiedAll}
          onCopyAi={handleCopyAi}
          onCopyAllData={handleCopyAllData}
          onCopyJson={handleCopyJson}
          onRemove={handleRemove}
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
  onCopyAllData: () => void;
  onRemove: () => void;
  copiedAll: boolean;
}

function ModalHeader({
  error,
  queueLength,
  queueIndex,
  onNavigate,
  onClose,
  onCopyAllData,
  onRemove,
  copiedAll,
}: ModalHeaderProps): React.ReactNode {
  const renderIcon = (level: string) => {
    if (level === 'error') {
      return <AlertCircle className="w-6 h-6 text-red-500 shrink-0" />;
    }
    if (level === 'warn') {
      return <AlertTriangle className="w-6 h-6 text-amber-500 shrink-0" />;
    }
    return <Info className="w-6 h-6 text-blue-500 shrink-0" />;
  };

  return (
    <div className="px-6 py-4 border-b border-slate-200 dark:border-slate-800 flex items-center justify-between bg-slate-50 dark:bg-slate-900/90">
      <div className="flex items-center gap-3 min-w-0 mr-2">
        {renderIcon(error.level)}

        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs px-2 py-0.5 rounded-md font-bold bg-red-500/10 text-red-600 dark:text-red-400 border border-red-500/20">
              {error.code}
            </span>
            <span className="text-xs text-slate-500 dark:text-slate-400 font-mono">
              {new Date(error.createdAt).toLocaleTimeString()}
            </span>
            {error.endpoint && (
              <span className="text-[11px] font-mono text-slate-600 dark:text-slate-300 truncate max-w-[200px]">
                {error.endpoint}
              </span>
            )}
          </div>
          <h3 className="text-base font-semibold leading-tight mt-1 line-clamp-1 text-slate-900 dark:text-slate-100">
            {error.message}
          </h3>
        </div>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        <button
          type="button"
          onClick={onCopyAllData}
          className="hidden sm:inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs font-semibold text-slate-700 dark:text-slate-200 hover:text-indigo-600 dark:hover:text-indigo-400 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-700 hover:border-indigo-400 dark:hover:border-indigo-500 transition-colors cursor-pointer shadow-2xs"
          title="Copy all error data to clipboard"
        >
          {copiedAll ? (
            <Check className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
          ) : (
            <ClipboardList className="w-3.5 h-3.5 text-indigo-500 shrink-0" />
          )}
          <span>{copiedAll ? 'Copied All!' : 'Copy All Data'}</span>
        </button>

        <button
          type="button"
          onClick={onRemove}
          className="p-1.5 text-slate-400 hover:text-red-600 dark:hover:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors cursor-pointer"
          title="Remove this error from history"
        >
          <Trash2 className="w-4 h-4" />
        </button>

        {queueLength > 1 && (
          <div className="flex items-center gap-1 px-2 py-1 rounded-lg bg-slate-100 dark:bg-slate-800 text-xs font-medium border border-slate-200 dark:border-slate-700">
            <button
              onClick={() => onNavigate('prev')}
              className="p-0.5 hover:bg-slate-200 dark:hover:bg-slate-700 rounded text-slate-600 dark:text-slate-300 cursor-pointer"
              title="Previous error"
            >
              <ChevronLeft className="w-3.5 h-3.5" />
            </button>
            <span className="px-1 font-mono text-slate-700 dark:text-slate-300">
              {queueIndex + 1} / {queueLength}
            </span>
            <button
              onClick={() => onNavigate('next')}
              className="p-0.5 hover:bg-slate-200 dark:hover:bg-slate-700 rounded text-slate-600 dark:text-slate-300 cursor-pointer"
              title="Next error"
            >
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>
        )}

        <button
          onClick={onClose}
          className="p-1.5 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors cursor-pointer"
          title="Close modal"
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
    <div className="flex border-b border-slate-200 dark:border-slate-800 px-6 gap-2 bg-slate-50/60 dark:bg-slate-900/60 overflow-x-auto scrollbar-none">
      {tabs.map((tab) => {
        const selected = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            onClick={() => onSelect(tab.id)}
            className={cn(
              'flex items-center gap-1.5 py-3 px-3 text-xs font-medium border-b-2 transition-colors cursor-pointer shrink-0 whitespace-nowrap',
              selected
                ? 'border-blue-500 text-blue-600 dark:text-blue-400 font-bold'
                : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
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
  const fullMessageText = `${error.message}${error.details ? `\n\nDetails:\n${error.details}` : ''}`;

  return (
    <div className="space-y-4">
      {/* Error Message & Details Card */}
      <div className="p-4 rounded-xl bg-red-50/80 dark:bg-red-950/30 border border-red-200 dark:border-red-900/50 shadow-2xs">
        <div className="flex items-center justify-between gap-2 mb-2">
          <div className="flex items-center gap-2">
            <h4 className="text-xs font-bold uppercase tracking-wider text-red-700 dark:text-red-400">
              Error Message & Details
            </h4>
            <ItemCopyButton text={fullMessageText} label="Copy Message" />
          </div>
          <button
            type="button"
            onClick={() => onSelectTab('stack')}
            className="text-xs text-blue-600 dark:text-blue-400 hover:underline font-semibold cursor-pointer"
          >
            Inspect Stack Trace →
          </button>
        </div>
        <p className="text-sm font-mono text-red-950 dark:text-red-100 break-words leading-relaxed">
          {error.message}
        </p>
        {error.details && (
          <div className="mt-3 pt-2.5 border-t border-red-200/70 dark:border-red-900/40">
            <span className="text-[11px] font-bold uppercase text-red-700 dark:text-red-400 tracking-wider">
              Diagnostic Details:
            </span>
            <p className="text-xs mt-1 font-mono text-red-900 dark:text-red-200 whitespace-pre-wrap leading-relaxed">
              {error.details}
            </p>
          </div>
        )}
      </div>

      {/* User Interaction Flow Card */}
      {error.uiClickPathArrow && (
        <div className="p-4 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 shadow-2xs">
          <div className="flex items-center justify-between gap-2 mb-2">
            <h4 className="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
              User Interaction Flow
            </h4>
            <ItemCopyButton text={error.uiClickPathArrow} label="Copy Flow" />
          </div>
          <p className="text-xs font-mono text-slate-800 dark:text-slate-200 leading-relaxed break-words">
            {error.uiClickPathArrow}
          </p>
        </div>
      )}

      {/* Stack Trace Preview Card */}
      {(error.stackTrace || error.backendStackTrace) && (
        <div className="p-4 rounded-xl bg-slate-50 dark:bg-slate-800/80 border border-slate-200 dark:border-slate-700 shadow-2xs">
          <div className="flex items-center justify-between gap-2 mb-2">
            <h4 className="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
              Stack Trace Preview
            </h4>
            <div className="flex items-center gap-2">
              <ItemCopyButton text={(error.stackTrace || error.backendStackTrace)!} label="Copy Stack" />
              <button
                type="button"
                onClick={() => onSelectTab('stack')}
                className="text-xs text-blue-600 dark:text-blue-400 hover:underline font-semibold cursor-pointer"
              >
                Full Stack →
              </button>
            </div>
          </div>
          <pre className="p-2.5 rounded-lg bg-slate-950 text-slate-100 font-mono text-[11px] overflow-x-auto max-h-36 whitespace-pre-wrap leading-relaxed border border-slate-800">
            {error.stackTrace || error.backendStackTrace}
          </pre>
        </div>
      )}

      {/* Suggested Fixes Card */}
      {fixes.length > 0 && (
        <div className="p-4 rounded-xl bg-blue-50/80 dark:bg-blue-950/30 border border-blue-200 dark:border-blue-900/50 shadow-2xs">
          <div className="flex items-center justify-between gap-2 mb-2.5">
            <div className="flex items-center gap-2">
              <Wrench className="w-4 h-4 text-blue-600 dark:text-blue-400" />
              <h4 className="text-xs font-bold uppercase tracking-wider text-blue-700 dark:text-blue-400">
                Suggested Troubleshooting Fixes ({error.code})
              </h4>
            </div>
            <ItemCopyButton text={fixes.map((f) => `- ${f}`).join('\n')} label="Copy Fixes" />
          </div>
          <ul className="space-y-1.5 pl-4 list-disc text-xs text-blue-950 dark:text-blue-200 leading-relaxed">
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

  const fullBackendText = [
    error.endpoint ? `Endpoint: ${error.method || 'INVOKE'} ${error.endpoint}${error.responseStatus ? ` (Status: ${error.responseStatus})` : ''}` : '',
    backendMsg ? `Backend Message:\n${backendMsg}` : '',
    backendStack.length > 0 ? `Backend Stack:\n${backendStack.join('\n')}` : '',
  ].filter(Boolean).join('\n\n');

  return (
    <div className="space-y-4">
      {error.endpoint && (
        <div className="flex items-center gap-2 text-xs font-mono p-2.5 rounded-lg bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700">
          <span className="font-semibold text-blue-600 dark:text-blue-400">
            {error.method || 'INVOKE'}
          </span>
          <span className="text-slate-800 dark:text-slate-200">{error.endpoint}</span>
          {error.responseStatus && (
            <span className="ml-auto px-2 py-0.5 rounded bg-red-100 dark:bg-red-900/40 text-red-600 dark:text-red-400 font-bold">
              {error.responseStatus}
            </span>
          )}
        </div>
      )}

      <div className="p-4 rounded-xl bg-slate-950 text-slate-100 font-mono text-xs overflow-x-auto border border-slate-800 shadow-2xs">
        <div className="flex items-center justify-between gap-2 mb-3 pb-2 border-b border-slate-800">
          <span className="text-slate-300 font-bold uppercase text-[11px] tracking-wider">
            Rust Backend Diagnostics
          </span>
          <ItemCopyButton text={fullBackendText} label="Copy Diagnostics" />
        </div>
        {backendMsg && <div className="text-red-400 mb-2 whitespace-pre-wrap font-medium">{backendMsg}</div>}
        {backendStack.length > 0 ? (
          <div className="space-y-1">
            {backendStack.map((line, idx) => (
              <div key={idx} className="text-slate-300">
                {line}
              </div>
            ))}
          </div>
        ) : (
          <div className="text-slate-500 italic">No structured backend stack captured.</div>
        )}
      </div>
    </div>
  );
}

function StackTab({ error }: { error: CapturedError }): React.ReactNode {
  const frames = error.parsedFrames || [];
  const hasFrames = frames.length > 0;
  const [showRaw, setShowRaw] = useState(frames.length === 0);
  const rawStack = error.stackTrace || error.backendStackTrace || error.details || error.message;
  const shouldShowRaw = Boolean(showRaw || frames.length === 0);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-slate-700 dark:text-slate-300">
          {hasFrames ? `${frames.length} Frame(s) Captured` : 'Raw Stack Trace'}
        </span>
        <div className="flex items-center gap-2">
          {rawStack && <ItemCopyButton text={rawStack} label="Copy Stack" />}
          {hasFrames && (
            <button
              type="button"
              onClick={() => setShowRaw(!showRaw)}
              className="text-xs text-blue-600 dark:text-blue-400 hover:underline font-semibold cursor-pointer ml-2"
            >
              {showRaw ? 'Show Formatted Table' : 'Show Raw Stack'}
            </button>
          )}
        </div>
      </div>

      {shouldShowRaw ? (
        <pre className="p-4 rounded-xl bg-slate-950 text-slate-100 font-mono text-xs overflow-x-auto max-h-80 whitespace-pre-wrap leading-relaxed border border-slate-800 shadow-2xs">
          {rawStack}
        </pre>
      ) : (
        <div className="border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-2xs">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-b border-slate-200 dark:border-slate-700 font-semibold">
              <tr>
                <th className="p-2.5">Function</th>
                <th className="p-2.5">File</th>
                <th className="p-2.5">Line</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
              {frames.map((f, i) => (
                <tr key={i} className={f.isInternal ? 'opacity-40' : 'font-semibold'}>
                  <td className="p-2.5 text-blue-600 dark:text-blue-400">{f.function}</td>
                  <td className="p-2.5 text-slate-700 dark:text-slate-300 truncate max-w-xs">
                    {f.file}
                  </td>
                  <td className="p-2.5 text-slate-600 dark:text-slate-400">{f.line}</td>
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
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-slate-700 dark:text-slate-300">
          Context Payload & Trigger Parameters
        </span>
        <ItemCopyButton text={json} label="Copy JSON" />
      </div>
      <pre className="p-4 rounded-xl bg-slate-950 text-slate-100 font-mono text-xs overflow-x-auto max-h-80 border border-slate-800 shadow-2xs">
        {json}
      </pre>
    </div>
  );
}

interface ModalFooterProps {
  copiedAi: boolean;
  copiedAll: boolean;
  onCopyAi: () => void;
  onCopyAllData: () => void;
  onCopyJson: () => void;
  onRemove: () => void;
  onClose: () => void;
}

function ModalFooter({
  copiedAi,
  copiedAll,
  onCopyAi,
  onCopyAllData,
  onCopyJson,
  onRemove,
  onClose,
}: ModalFooterProps): React.ReactNode {
  return (
    <div className="px-6 py-4 border-t border-slate-200 dark:border-slate-800 flex flex-wrap items-center justify-between gap-3 bg-slate-50 dark:bg-slate-900/90">
      <div className="flex flex-wrap items-center gap-2">
        <button
          onClick={onCopyAllData}
          className="flex items-center gap-1.5 px-4 py-2 rounded-xl text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 shadow-md shadow-indigo-500/20 transition-all cursor-pointer"
          title="Copy full diagnostic report including all data fields"
        >
          {copiedAll ? <Check className="w-4 h-4" /> : <ClipboardList className="w-4 h-4" />}
          <span>{copiedAll ? 'Copied All Error Data!' : 'Copy All Error Data'}</span>
        </button>

        <button
          onClick={onCopyAi}
          className="flex items-center gap-1.5 px-3.5 py-2 rounded-xl text-xs font-semibold text-white bg-blue-600 hover:bg-blue-700 shadow-md shadow-blue-500/20 transition-all cursor-pointer"
          title="Copy markdown summary optimized for AI models"
        >
          {copiedAi ? <Check className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
          <span>{copiedAi ? 'Copied AI Report!' : 'Copy for AI'}</span>
        </button>

        <button
          onClick={onCopyJson}
          className="flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-medium text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-slate-800 border border-slate-300 dark:border-slate-700 transition-colors cursor-pointer"
          title="Copy raw error object as JSON"
        >
          <Copy className="w-3.5 h-3.5" />
          <span>Copy JSON</span>
        </button>
      </div>

      <div className="flex items-center gap-2">
        <button
          onClick={onRemove}
          className="flex items-center gap-1 px-3 py-2 rounded-xl text-xs font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40 border border-red-200 dark:border-red-900/40 transition-colors cursor-pointer"
          title="Delete this error from history"
        >
          <Trash2 className="w-3.5 h-3.5" />
          <span>Delete Error</span>
        </button>

        <button
          onClick={onClose}
          className="px-4 py-2 rounded-xl text-xs font-medium text-slate-700 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-slate-800 transition-colors cursor-pointer border border-slate-300 dark:border-slate-700"
        >
          Dismiss
        </button>
      </div>
    </div>
  );
}
