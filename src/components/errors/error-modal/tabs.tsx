import React from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Info,
  Copy,
  Check,
  Terminal,
  Layers,
  Wrench,
  Bot,
} from 'lucide-react';
import type { CapturedError } from '../../../stores/error-store';
import { getSuggestedFixes } from '../../../lib/error-report-generator';
import { cn } from '../../../utils/cn';

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
