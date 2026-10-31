import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';

export interface AccordionSectionCardProps {
  title: string;
  subtitle?: string;
  icon?: React.ReactNode;
  badge?: React.ReactNode;
  headerAction?: React.ReactNode;
  defaultExpanded?: boolean;
  expanded?: boolean;
  onToggle?: (nextExpanded: boolean) => void;
  children: React.ReactNode;
  className?: string;
}

export const AccordionSectionCard: React.FC<AccordionSectionCardProps> = ({
  title,
  subtitle,
  icon,
  badge,
  headerAction,
  defaultExpanded = false,
  expanded: controlledExpanded,
  onToggle,
  children,
  className = '',
}) => {
  const [internalExpanded, setInternalExpanded] = useState(defaultExpanded);
  const isExpanded = controlledExpanded !== undefined ? controlledExpanded : internalExpanded;

  const handleToggle = () => {
    const next = !isExpanded;
    if (controlledExpanded === undefined) {
      setInternalExpanded(next);
    }
    onToggle?.(next);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleToggle();
    }
  };

  return (
    <div
      className={`rounded-xl border border-slate-700/60 bg-slate-900/60 backdrop-blur-md overflow-hidden transition-all duration-200 ${className}`}
    >
      <div
        role="button"
        tabIndex={0}
        aria-expanded={isExpanded}
        onClick={handleToggle}
        onKeyDown={handleKeyDown}
        className="flex items-center justify-between px-4 py-3 cursor-pointer select-none hover:bg-slate-800/40 transition-colors focus:outline-none focus-visible:ring-1 focus-visible:ring-sky-400"
      >
        <div className="flex items-center gap-3 min-w-0">
          {icon && <span className="text-slate-400 shrink-0">{icon}</span>}
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-medium text-sm text-slate-200 truncate">{title}</span>
              {badge}
            </div>
            {subtitle && <p className="text-xs text-slate-400 truncate mt-0.5">{subtitle}</p>}
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          {headerAction && (
            <div onClick={(e) => e.stopPropagation()} className="mr-1">
              {headerAction}
            </div>
          )}
          <ChevronDown
            className={`w-4 h-4 text-slate-400 transition-transform duration-200 ${
              isExpanded ? 'rotate-180 text-sky-400' : ''
            }`}
          />
        </div>
      </div>

      {isExpanded && (
        <div className="border-t border-slate-700/40 px-4 py-3 bg-slate-950/30 animate-in fade-in-50 duration-150">
          {children}
        </div>
      )}
    </div>
  );
};
