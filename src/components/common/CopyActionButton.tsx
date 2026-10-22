import React, { useState } from 'react';
import { Copy, Check } from 'lucide-react';
import { copyToClipboard } from '../../utils/clipboard';

export interface CopyActionButtonProps {
  text: string;
  label?: string;
  successLabel?: string;
  variant?: 'icon' | 'capsule' | 'minimal';
  size?: 'sm' | 'md';
  className?: string;
  onCopySuccess?: () => void;
}

export const CopyActionButton: React.FC<CopyActionButtonProps> = ({
  text,
  label = 'Copy',
  successLabel = 'Copied!',
  variant = 'capsule',
  size = 'md',
  className = '',
  onCopySuccess,
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation();
    const success = await copyToClipboard(text);
    if (success) {
      setCopied(true);
      if (onCopySuccess) onCopySuccess();
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const getSizeClasses = () => {
    switch (size) {
      case 'sm':
        return 'h-7 text-xs px-2 gap-1';
      case 'md':
      default:
        return 'h-8 text-xs md:text-sm px-2.5 gap-1.5';
    }
  };

  if (variant === 'icon') {
    return (
      <button
        type="button"
        onClick={handleCopy}
        title={copied ? successLabel : label}
        className={`p-1.5 rounded-lg border border-slate-700/60 transition-all ${
          copied
            ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/50'
            : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 bg-slate-900/60'
        } ${className}`}
      >
        {copied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
      </button>
    );
  }

  if (variant === 'minimal') {
    return (
      <button
        type="button"
        onClick={handleCopy}
        className={`inline-flex items-center text-xs font-mono transition-colors ${
          copied ? 'text-emerald-400' : 'text-slate-400 hover:text-slate-200'
        } ${className}`}
      >
        {copied ? (
          <Check className="w-3.5 h-3.5 mr-1 text-emerald-400" />
        ) : (
          <Copy className="w-3.5 h-3.5 mr-1" />
        )}
        <span>{copied ? successLabel : label}</span>
      </button>
    );
  }

  // Capsule default variant
  return (
    <button
      type="button"
      onClick={handleCopy}
      className={`inline-flex items-center justify-center font-medium rounded-full border transition-all duration-200 active:scale-95 ${getSizeClasses()} ${
        copied
          ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/50 shadow-[0_0_8px_rgba(16,185,129,0.3)]'
          : 'bg-slate-900/60 hover:bg-slate-800/80 text-slate-300 border-slate-700/60'
      } ${className}`}
    >
      {copied ? (
        <Check className="w-3.5 h-3.5 text-emerald-400" />
      ) : (
        <Copy className="w-3.5 h-3.5 text-slate-400" />
      )}
      <span>{copied ? successLabel : label}</span>
    </button>
  );
};
