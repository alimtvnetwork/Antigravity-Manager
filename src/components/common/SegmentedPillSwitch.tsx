import React from 'react';

export interface SegmentedPillOption<T extends string | number> {
  value: T;
  label: string;
  icon?: React.ReactNode;
  badge?: string | number;
}

interface SegmentedPillSwitchProps<T extends string | number> {
  options: SegmentedPillOption<T>[];
  value: T;
  onChange: (val: T) => void;
  size?: 'sm' | 'md';
  className?: string;
  disabled?: boolean;
  'aria-label'?: string;
}

export function SegmentedPillSwitch<T extends string | number>({
  options,
  value,
  onChange,
  size = 'md',
  className = '',
  disabled = false,
  'aria-label': ariaLabel = 'Options',
}: SegmentedPillSwitchProps<T>): React.ReactElement {
  const handleKeyDown = (e: React.KeyboardEvent, currentIndex: number) => {
    if (disabled) return;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      const nextIndex = (currentIndex + 1) % options.length;
      onChange(options[nextIndex].value);
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      const prevIndex = (currentIndex - 1 + options.length) % options.length;
      onChange(options[prevIndex].value);
    }
  };

  const isSmall = size === 'sm';

  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={`inline-flex items-center p-0.5 rounded-full bg-slate-900/90 border border-slate-700/70 shadow-inner backdrop-blur-md ${
        disabled ? 'opacity-50 pointer-events-none' : ''
      } ${className}`}
    >
      {options.map((option, idx) => {
        const isSelected = option.value === value;

        return (
          <button
            key={String(option.value)}
            type="button"
            role="radio"
            aria-checked={isSelected}
            aria-label={option.label}
            disabled={disabled}
            tabIndex={isSelected ? 0 : -1}
            onClick={() => onChange(option.value)}
            onKeyDown={(e) => handleKeyDown(e, idx)}
            className={`relative flex items-center justify-center gap-1.5 rounded-full font-medium transition-all duration-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 ${
              isSmall ? 'px-2.5 py-1 text-xs' : 'px-3.5 py-1.5 text-xs'
            } ${
              isSelected
                ? 'bg-sky-500/20 text-sky-300 font-semibold shadow-sm border border-sky-400/40'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'
            }`}
          >
            {option.icon && <span className="opacity-90" aria-hidden="true">{option.icon}</span>}
            <span>{option.label}</span>
            {option.badge !== undefined && (
              <span
                aria-hidden="true"
                className={`ml-1 px-1.5 py-0.2 rounded-full text-[10px] font-bold ${
                  isSelected
                    ? 'bg-sky-400/30 text-sky-200'
                    : 'bg-slate-800 text-slate-400'
                }`}
              >
                {option.badge}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
