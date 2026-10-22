import React, { useState } from 'react';
import { Copy, Check, ChevronDown, ChevronRight, Terminal } from 'lucide-react';

interface ExpandableCodeBlockBannerProps {
  code: string;
  language?: string;
  title?: string;
  defaultExpanded?: boolean;
  maxCollapsedHeight?: string;
  className?: string;
}

export const ExpandableCodeBlockBanner: React.FC<ExpandableCodeBlockBannerProps> = ({
  code,
  language = 'json',
  title,
  defaultExpanded = false,
  maxCollapsedHeight = '140px',
  className = '',
}) => {
  const [expanded, setExpanded] = useState(defaultExpanded);
  const [copied, setCopied] = useState(false);

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className={`rounded-xl border border-slate-700/60 bg-slate-950/80 backdrop-blur-md overflow-hidden text-xs ${className}`}>
      <div
        role="button"
        tabIndex={0}
        aria-expanded={expanded}
        onClick={() => setExpanded(!expanded)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            setExpanded(!expanded);
          }
        }}
        className="flex items-center justify-between px-3 py-2 bg-slate-900/60 border-b border-slate-800/60 cursor-pointer select-none hover:bg-slate-900/80 focus:outline-none focus:ring-1 focus:ring-sky-500/50 transition-colors"
      >
        <div className="flex items-center gap-2 text-slate-300 font-mono font-medium">
          {expanded ? <ChevronDown className="w-3.5 h-3.5 text-slate-400" /> : <ChevronRight className="w-3.5 h-3.5 text-slate-400" />}
          <Terminal className="w-3.5 h-3.5 text-sky-400" />
          <span>{title || language.toUpperCase()}</span>
        </div>

        <button
          type="button"
          onClick={handleCopy}
          aria-label="Copy code to clipboard"
          className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[11px] font-mono text-slate-400 hover:text-white hover:bg-slate-800 focus:outline-none focus:ring-1 focus:ring-sky-400 transition-colors"
          title="Copy code"
        >
          {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
          <span>{copied ? 'Copied' : 'Copy'}</span>
        </button>
      </div>

      <div
        style={{ maxHeight: expanded ? 'none' : maxCollapsedHeight }}
        className="p-3 font-mono text-[11px] text-slate-300 overflow-x-auto overflow-y-hidden relative whitespace-pre"
      >
        <code>{code}</code>
        {!expanded && (
          <div
            role="button"
            tabIndex={0}
            onClick={() => setExpanded(true)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                setExpanded(true);
              }
            }}
            className="absolute inset-x-0 bottom-0 h-10 bg-gradient-to-t from-slate-950 to-transparent flex items-end justify-center pb-1 cursor-pointer text-[10px] text-slate-400 font-sans hover:text-sky-300 focus:outline-none focus:text-sky-300 select-none"
          >
            Click to expand
          </div>
        )}
      </div>
    </div>
  );
};
