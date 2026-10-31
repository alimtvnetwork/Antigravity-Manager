import React, { ReactNode } from 'react';
import { Inbox } from 'lucide-react';

export interface EmptyStateAction {
  label: string;
  onClick: () => void;
  icon?: ReactNode;
  variant?: 'primary' | 'secondary' | 'outline';
}

export interface EmptyStateCardProps {
  title: string;
  description?: string;
  icon?: ReactNode;
  action?: EmptyStateAction;
  secondaryAction?: EmptyStateAction;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

export const EmptyStateCard: React.FC<EmptyStateCardProps> = ({
  title,
  description,
  icon,
  action,
  secondaryAction,
  className = '',
  size = 'md',
}) => {
  const getPadding = () => {
    switch (size) {
      case 'sm':
        return 'py-6 px-4';
      case 'lg':
        return 'py-16 px-8';
      case 'md':
      default:
        return 'py-10 px-6';
    }
  };

  const getActionBtnClass = (variant: 'primary' | 'secondary' | 'outline' = 'primary') => {
    switch (variant) {
      case 'secondary':
        return 'bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700/80';
      case 'outline':
        return 'bg-transparent hover:bg-slate-800/40 text-slate-300 border border-slate-700/80';
      case 'primary':
      default:
        return 'bg-sky-600 hover:bg-sky-500 text-white shadow-md shadow-sky-600/30';
    }
  };

  return (
    <div
      className={`flex flex-col items-center justify-center text-center w-full bg-slate-900/40 dark:bg-slate-950/40 border border-dashed border-slate-700/60 rounded-2xl backdrop-blur-sm ${getPadding()} ${className}`}
    >
      {/* Icon Capsule */}
      <div className="flex items-center justify-center w-12 h-12 mb-3 rounded-xl bg-slate-800/60 dark:bg-slate-900/60 border border-slate-700/60 text-slate-400">
        {icon ? icon : <Inbox className="w-6 h-6 text-slate-400" />}
      </div>

      {/* Title */}
      <h3 className="text-sm md:text-base font-semibold text-slate-200 dark:text-slate-200">
        {title}
      </h3>

      {/* Description */}
      {description && (
        <p className="mt-1 max-w-sm text-xs md:text-sm text-slate-400 dark:text-slate-400">
          {description}
        </p>
      )}

      {/* Action Buttons */}
      {(action || secondaryAction) && (
        <div className="flex flex-wrap items-center justify-center gap-2.5 mt-5">
          {action && (
            <button
              type="button"
              onClick={action.onClick}
              className={`inline-flex items-center gap-1.5 px-3.5 py-1.5 text-xs md:text-sm font-medium rounded-lg transition-all active:scale-[0.98] ${getActionBtnClass(
                action.variant
              )}`}
            >
              {action.icon}
              <span>{action.label}</span>
            </button>
          )}

          {secondaryAction && (
            <button
              type="button"
              onClick={secondaryAction.onClick}
              className={`inline-flex items-center gap-1.5 px-3.5 py-1.5 text-xs md:text-sm font-medium rounded-lg transition-all active:scale-[0.98] ${getActionBtnClass(
                secondaryAction.variant || 'secondary'
              )}`}
            >
              {secondaryAction.icon}
              <span>{secondaryAction.label}</span>
            </button>
          )}
        </div>
      )}
    </div>
  );
};
