import React, { useState } from 'react';
import {
  AlertCircle,
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
import { useErrorStore, type CapturedError, type ErrorModalTab } from '../../../stores/error-store';
import { showToast } from '../../common/ToastContainer';
import { cn } from '../../../utils/cn';

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
