import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';

interface KeyValueDetailRowProps {
  label: string;
  value: React.ReactNode;
  copyableText?: string;
  tooltip?: string;
  className?: string;
}

export const KeyValueDetailRow: React.FC<KeyValueDetailRowProps> = ({
  label,
  value,
  copyableText,
  tooltip,
  className = '',
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    if (!copyableText) return;
    navigator.clipboard.writeText(copyableText);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div
      className={`flex items-center justify-between py-1.5 px-2.5 rounded-lg border border-slate-800/40 bg-slate-900/40 hover:bg-slate-800/40 transition-colors text-xs ${className}`}
      title={tooltip}
    >
      <span className="text-slate-400 font-medium">{label}</span>
      <div className="flex items-center gap-1.5 font-mono text-slate-200">
        <span>{value}</span>
        {copyableText && (
          <button
            type="button"
            onClick={handleCopy}
            className="p-1 rounded text-slate-400 hover:text-sky-300 hover:bg-slate-800 transition-colors focus:outline-none"
            title="Copy value"
          >
            {copied ? (
              <Check className="w-3 h-3 text-emerald-400" />
            ) : (
              <Copy className="w-3 h-3" />
            )}
          </button>
        )}
      </div>
    </div>
  );
};
