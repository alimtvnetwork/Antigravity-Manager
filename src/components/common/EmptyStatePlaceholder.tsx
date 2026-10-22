import React from 'react';

interface EmptyStatePlaceholderProps {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  actionLabel?: string;
  onAction?: () => void;
  className?: string;
}

export const EmptyStatePlaceholder: React.FC<EmptyStatePlaceholderProps> = ({
  icon,
  title,
  description,
  actionLabel,
  onAction,
  className = '',
}) => {
  return (
    <div
      className={`flex flex-col items-center justify-center p-8 text-center rounded-2xl bg-slate-900/60 border border-slate-800/80 backdrop-blur-md ${className}`}
    >
      {icon && (
        <div className="flex items-center justify-center w-12 h-12 mb-3.5 rounded-full bg-slate-800/80 border border-slate-700/60 text-slate-400">
          {icon}
        </div>
      )}
      <h3 className="text-sm font-semibold text-slate-200 mb-1">{title}</h3>
      {description && (
        <p className="text-xs text-slate-400 max-w-sm mb-4 leading-relaxed">
          {description}
        </p>
      )}
      {actionLabel && onAction && (
        <button
          type="button"
          onClick={onAction}
          className="px-4 py-1.5 text-xs font-medium rounded-full bg-sky-500/20 text-sky-300 border border-sky-400/30 hover:bg-sky-500/30 hover:border-sky-400/50 transition-colors shadow-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
        >
          {actionLabel}
        </button>
      )}
    </div>
  );
};
