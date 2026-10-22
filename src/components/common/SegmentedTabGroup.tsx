import React, { useRef } from 'react';

export interface TabOption<T extends string = string> {
  id: T;
  label: string;
  icon?: React.ReactNode;
  count?: number;
}

interface SegmentedTabGroupProps<T extends string = string> {
  options: TabOption<T>[];
  activeId: T;
  onChange: (id: T) => void;
  size?: 'sm' | 'md';
  className?: string;
}

export function SegmentedTabGroup<T extends string = string>({
  options,
  activeId,
  onChange,
  size = 'sm',
  className = '',
}: SegmentedTabGroupProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null);

  const handleKeyDown = (e: React.KeyboardEvent, index: number) => {
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      const nextIndex = (index + 1) % options.length;
      onChange(options[nextIndex].id);
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      const prevIndex = (index - 1 + options.length) % options.length;
      onChange(options[prevIndex].id);
    }
  };

  const padClasses = size === 'sm' ? 'px-2.5 py-1 text-xs' : 'px-3.5 py-1.5 text-sm';

  return (
    <div
      ref={containerRef}
      role="tablist"
      className={`inline-flex items-center p-0.5 rounded-full bg-slate-900/80 border border-slate-700/60 backdrop-blur-md shadow-inner ${className}`}
    >
      {options.map((option, idx) => {
        const isActive = option.id === activeId;
        return (
          <button
            key={option.id}
            role="tab"
            aria-selected={isActive}
            tabIndex={isActive ? 0 : -1}
            onClick={() => onChange(option.id)}
            onKeyDown={(e) => handleKeyDown(e, idx)}
            className={`inline-flex items-center gap-1.5 font-medium rounded-full transition-all duration-150 focus:outline-none focus-visible:ring-1 focus-visible:ring-sky-400 ${padClasses} ${
              isActive
                ? 'bg-slate-700/90 text-white shadow-sm border border-slate-600/60 font-semibold'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'
            }`}
          >
            {option.icon && <span className="opacity-80">{option.icon}</span>}
            <span>{option.label}</span>
            {option.count !== undefined && (
              <span
                className={`ml-0.5 px-1.5 py-0.2 text-[10px] rounded-full font-mono ${
                  isActive
                    ? 'bg-sky-500/20 text-sky-300 border border-sky-400/30'
                    : 'bg-slate-800 text-slate-400'
                }`}
              >
                {option.count}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
