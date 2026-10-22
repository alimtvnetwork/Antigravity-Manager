import React from 'react';

export interface CompactActionItem {
  key: string;
  icon?: React.ReactNode;
  label?: string;
  title?: string;
  onClick: () => void;
  disabled?: boolean;
  highlight?: 'emerald' | 'amber' | 'sky' | 'rose' | 'slate';
  badge?: string | number;
}

interface CompactActionBarProps {
  actions: CompactActionItem[];
  size?: 'xs' | 'sm' | 'md';
  className?: string;
}

const highlightStyles: Record<string, string> = {
  emerald: 'text-emerald-400 hover:bg-emerald-500/20 hover:text-emerald-300',
  amber: 'text-amber-400 hover:bg-amber-500/20 hover:text-amber-300',
  sky: 'text-sky-400 hover:bg-sky-500/20 hover:text-sky-300',
  rose: 'text-rose-400 hover:bg-rose-500/20 hover:text-rose-300',
  slate: 'text-slate-300 hover:bg-slate-700/50 hover:text-white',
};

export const CompactActionBar: React.FC<CompactActionBarProps> = ({
  actions,
  size = 'sm',
  className = '',
}) => {
  const sizeClasses =
    size === 'xs'
      ? 'px-2 py-1 text-[11px] gap-1'
      : size === 'md'
      ? 'px-3 py-1.5 text-xs gap-1.5'
      : 'px-2.5 py-1 text-xs gap-1.5';

  return (
    <div
      role="toolbar"
      aria-label="Action Toolbar"
      className={`inline-flex items-center rounded-full bg-slate-900/85 border border-slate-700/60 shadow-inner backdrop-blur-md overflow-hidden ${className}`}
    >
      {actions.map((act, index) => {
        const highlightClass = act.highlight
          ? highlightStyles[act.highlight]
          : highlightStyles.slate;

        return (
          <React.Fragment key={act.key}>
            {index > 0 && <span className="h-3.5 w-px bg-slate-700/50" />}
            <button
              type="button"
              title={act.title || act.label}
              onClick={act.onClick}
              disabled={act.disabled}
              className={`flex items-center justify-center font-medium transition-colors duration-150 focus:outline-none focus-visible:ring-1 focus-visible:ring-sky-400 disabled:opacity-40 disabled:pointer-events-none ${sizeClasses} ${highlightClass}`}
            >
              {act.icon && <span className="flex-shrink-0">{act.icon}</span>}
              {act.label && <span>{act.label}</span>}
              {act.badge !== undefined && (
                <span className="ml-0.5 px-1 py-0.2 rounded-full text-[9px] bg-slate-800 text-slate-300 font-semibold">
                  {act.badge}
                </span>
              )}
            </button>
          </React.Fragment>
        );
      })}
    </div>
  );
};
