import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';

interface TruncatedTextTooltipProps {
  text: string;
  maxWidth?: string; // CSS max-width, e.g. "200px" or "14rem"
  enableCopy?: boolean;
  className?: string;
  mono?: boolean;
}

export const TruncatedTextTooltip: React.FC<TruncatedTextTooltipProps> = ({
  text,
  maxWidth = '180px',
  enableCopy = true,
  className = '',
  mono = true,
}) => {
  const [copied, setCopied] = useState(false);
  const [isHovered, setIsHovered] = useState(false);

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div
      className={`relative inline-flex items-center group ${className}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <span
        style={{ maxWidth }}
        className={`truncate inline-block text-slate-300 select-all ${
          mono ? 'font-mono text-xs' : 'text-sm'
        }`}
        title={text}
      >
        {text}
      </span>

      {enableCopy && (
        <button
          type="button"
          onClick={handleCopy}
          aria-label="Copy full text"
          className="ml-1 p-0.5 opacity-0 group-hover:opacity-100 transition-opacity text-slate-400 hover:text-sky-300 focus:opacity-100 rounded focus:outline-none"
        >
          {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
        </button>
      )}

      {isHovered && text.length > 25 && (
        <div
          role="tooltip"
          className="absolute bottom-full left-0 mb-1 z-50 px-2.5 py-1.5 rounded-lg bg-slate-900/95 border border-slate-700/80 shadow-xl backdrop-blur-md text-slate-200 text-xs font-mono max-w-sm break-all pointer-events-none"
        >
          {text}
        </div>
      )}
    </div>
  );
};
