import React from 'react';

export interface MicroActionItem {
  id: string;
  icon: React.ReactNode;
  label: string;
  onClick: (e: React.MouseEvent) => void;
  variant?: 'default' | 'danger' | 'success' | 'amber';
  disabled?: boolean;
}

interface MicroActionButtonGroupProps {
  actions: MicroActionItem[];
  size?: 'xs' | 'sm';
  className?: string;
}

export const MicroActionButtonGroup: React.FC<MicroActionButtonGroupProps> = ({
  actions,
  size = 'xs',
  className = '',
}) => {
  const padClasses = size === 'xs' ? 'p-1' : 'p-1.5';

  return (
    <div
      role="toolbar"
      className={`inline-flex items-center rounded-full bg-slate-900/80 border border-slate-700/60 backdrop-blur-md shadow-sm divide-x divide-slate-700/50 overflow-hidden ${className}`}
    >
      {actions.map((action) => {
        let hoverColor = 'hover:bg-slate-800/80 text-slate-300 hover:text-white';
        if (action.variant === 'danger') {
          hoverColor = 'hover:bg-rose-950/80 text-rose-300 hover:text-rose-200';
        } else if (action.variant === 'success') {
          hoverColor = 'hover:bg-emerald-950/80 text-emerald-300 hover:text-emerald-200';
        } else if (action.variant === 'amber') {
          hoverColor = 'hover:bg-amber-950/80 text-amber-300 hover:text-amber-200';
        }

        return (
          <button
            key={action.id}
            type="button"
            title={action.label}
            aria-label={action.label}
            disabled={action.disabled}
            onClick={action.onClick}
            className={`transition-colors duration-150 focus:outline-none focus-visible:bg-slate-700 ${padClasses} ${hoverColor} ${
              action.disabled ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer'
            }`}
          >
            {action.icon}
          </button>
        );
      })}
    </div>
  );
};
