import React, { useState, ReactNode } from 'react';
import { ChevronDown } from 'lucide-react';

export interface ExpandableSectionProps {
  title: ReactNode;
  children: ReactNode;
  icon?: ReactNode;
  badge?: ReactNode;
  headerAction?: ReactNode;
  defaultExpanded?: boolean;
  isExpanded?: boolean;
  onToggle?: (expanded: boolean) => void;
  className?: string;
  headerClassName?: string;
}

export const ExpandableSection: React.FC<ExpandableSectionProps> = ({
  title,
  children,
  icon,
  badge,
  headerAction,
  defaultExpanded = false,
  isExpanded: controlledExpanded,
  onToggle,
  className = '',
  headerClassName = '',
}) => {
  const [uncontrolledExpanded, setUncontrolledExpanded] = useState(defaultExpanded);
  const isExpanded = controlledExpanded !== undefined ? controlledExpanded : uncontrolledExpanded;

  const handleToggle = () => {
    const next = !isExpanded;
    if (controlledExpanded === undefined) {
      setUncontrolledExpanded(next);
    }
    if (onToggle) {
      onToggle(next);
    }
  };

  return (
    <div
      className={`rounded-2xl border border-slate-700/60 bg-slate-900/50 dark:bg-slate-950/60 backdrop-blur-md overflow-hidden transition-all duration-200 ${className}`}
    >
      {/* Header bar */}
      <div
        onClick={handleToggle}
        className={`flex items-center justify-between px-4 py-3 cursor-pointer select-none hover:bg-slate-800/40 transition-colors ${headerClassName}`}
      >
        <div className="flex items-center gap-2.5 min-w-0">
          {icon && <span className="text-slate-400 shrink-0">{icon}</span>}
          <div className="font-semibold text-sm md:text-base text-slate-200 truncate">
            {title}
          </div>
          {badge && <div className="shrink-0">{badge}</div>}
        </div>

        <div className="flex items-center gap-2 shrink-0 ml-2">
          {headerAction && (
            <div
              onClick={(e) => e.stopPropagation()}
              className="flex items-center"
            >
              {headerAction}
            </div>
          )}

          <div
            className={`p-1 text-slate-400 rounded-lg hover:text-slate-200 transition-transform duration-200 ${
              isExpanded ? 'rotate-180' : 'rotate-0'
            }`}
          >
            <ChevronDown className="w-4 h-4" />
          </div>
        </div>
      </div>

      {/* Collapsible Content Body */}
      {isExpanded && (
        <div className="px-4 pb-4 pt-1 border-t border-slate-800/80 animate-in fade-in-50 duration-150">
          {children}
        </div>
      )}
    </div>
  );
};
