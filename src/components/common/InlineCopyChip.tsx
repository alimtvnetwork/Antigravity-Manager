import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';

interface InlineCopyChipProps {
  value: string;
  label?: string;
  truncateLength?: number;
  className?: string;
}

export const InlineCopyChip: React.FC<InlineCopyChipProps> = ({
  value,
  label,
  truncateLength,
  className = '',
}) => {
  const [copied, setCopied] = useState(false);

  const displayValue = truncateLength && value.length > truncateLength
    ? `${value.slice(0, truncateLength)}...`
    : value;

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      title={`Click to copy: ${value}`}
      className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full font-mono text-[11px] bg-slate-800/80 border border-slate-700/60 text-slate-300 hover:text-white hover:bg-slate-750 hover:border-slate-600 transition-all cursor-pointer focus:outline-none focus-visible:ring-1 focus-visible:ring-sky-400 select-all ${className}`}
    >
      {label && <span className="text-[10px] text-slate-400 font-sans font-medium uppercase tracking-wider">{label}:</span>}
      <span className="font-semibold text-slate-200">{displayValue}</span>
      {copied ? (
        <Check className="w-3 h-3 text-emerald-400 ml-0.5 animate-in zoom-in-75 duration-100" />
      ) : (
        <Copy className="w-3 h-3 text-slate-400 opacity-60 group-hover:opacity-100 ml-0.5" />
      )}
    </button>
  );
};
