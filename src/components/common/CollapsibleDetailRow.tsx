import React, { useState } from 'react';
import { ChevronRight } from 'lucide-react';

export interface CollapsibleDetailRowProps {
  label: React.ReactNode;
  value?: React.ReactNode;
  children: React.ReactNode;
  defaultOpen?: boolean;
  isOpen?: boolean;
  onToggle?: (open: boolean) => void;
  className?: string;
}

export const CollapsibleDetailRow: React.FC<CollapsibleDetailRowProps> = ({
  label,
  value,
  children,
  defaultOpen = false,
  isOpen: controlledOpen,
  onToggle,
  className = '',
}) => {
  const [internalOpen, setInternalOpen] = useState(defaultOpen);
  const open = controlledOpen !== undefined ? controlledOpen : internalOpen;

  const toggle = () => {
    const next = !open;
    if (controlledOpen === undefined) {
      setInternalOpen(next);
    }
    onToggle?.(next);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      toggle();
    }
  };

  return (
    <div className={`border-b border-slate-800/60 last:border-0 ${className}`}>
      <div
        role="button"
        tabIndex={0}
        aria-expanded={open}
        onClick={toggle}
        onKeyDown={handleKeyDown}
        className="flex items-center justify-between py-2 px-2.5 rounded-lg cursor-pointer select-none hover:bg-slate-800/40 transition-colors focus:outline-none focus-visible:ring-1 focus-visible:ring-sky-400"
      >
        <div className="flex items-center gap-2 min-w-0">
          <ChevronRight
            className={`w-3.5 h-3.5 text-slate-400 transition-transform duration-200 shrink-0 ${
              open ? 'rotate-90 text-sky-400' : ''
            }`}
          />
          <span className="text-xs font-medium text-slate-300 truncate">{label}</span>
        </div>
        {value && <div className="text-xs text-slate-400 font-mono shrink-0 ml-2">{value}</div>}
      </div>

      {open && (
        <div className="pl-6 pr-2.5 pb-2.5 pt-1 text-xs text-slate-400 animate-in fade-in-50 duration-150">
          {children}
        </div>
      )}
    </div>
  );
};
