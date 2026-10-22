import React, { ReactNode } from 'react';

export interface CapsuleRadioOption<T extends string | number> {
  value: T;
  label: ReactNode;
  icon?: ReactNode;
  badge?: string | number;
  disabled?: boolean;
}

export interface SegmentedCapsuleRadioProps<T extends string | number> {
  options: CapsuleRadioOption<T>[];
  value: T;
  onChange: (value: T) => void;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
  disabled?: boolean;
  'aria-label'?: string;
}

export function SegmentedCapsuleRadio<T extends string | number>({
  options,
  value,
  onChange,
  size = 'md',
  className = '',
  disabled = false,
  'aria-label': ariaLabel = 'Options',
}: SegmentedCapsuleRadioProps<T>) {
  const handleKeyDown = (e: React.KeyboardEvent, index: number) => {
    if (disabled) return;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      const nextIndex = (index + 1) % options.length;
      if (!options[nextIndex].disabled) onChange(options[nextIndex].value);
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      const prevIndex = (index - 1 + options.length) % options.length;
      if (!options[prevIndex].disabled) onChange(options[prevIndex].value);
    } else if (e.key === 'Home') {
      e.preventDefault();
      const first = options.find((o) => !o.disabled);
      if (first) onChange(first.value);
    } else if (e.key === 'End') {
      e.preventDefault();
      const last = [...options].reverse().find((o) => !o.disabled);
      if (last) onChange(last.value);
    }
  };

  const getSizeClasses = () => {
    switch (size) {
      case 'sm':
        return 'h-7 text-xs px-2.5 gap-1';
      case 'lg':
        return 'h-10 text-sm px-4 gap-2';
      case 'md':
      default:
        return 'h-8 text-xs md:text-sm px-3 gap-1.5';
    }
  };

  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={`inline-flex items-center p-0.5 rounded-full bg-slate-900/80 dark:bg-slate-950/80 border border-slate-700/60 shadow-inner backdrop-blur-md ${
        disabled ? 'opacity-50 pointer-events-none' : ''
      } ${className}`}
    >
      {options.map((opt, idx) => {
        const isSelected = opt.value === value;
        const isOptionDisabled = disabled || opt.disabled;

        return (
          <button
            key={`capsule-radio-${idx}-${String(opt.value)}`}
            type="button"
            role="radio"
            aria-checked={isSelected}
            disabled={isOptionDisabled}
            tabIndex={isSelected ? 0 : -1}
            onClick={() => !isOptionDisabled && onChange(opt.value)}
            onKeyDown={(e) => handleKeyDown(e, idx)}
            className={`relative flex items-center justify-center font-medium rounded-full transition-all duration-200 select-none focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-400 ${getSizeClasses()} ${
              isSelected
                ? 'bg-sky-600/90 text-white shadow-[0_0_10px_rgba(2,132,199,0.35)]'
                : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/40'
            }`}
          >
            {opt.icon && <span className="shrink-0" aria-hidden="true">{opt.icon}</span>}
            <span>{opt.label}</span>
            {opt.badge !== undefined && (
              <span
                aria-hidden="true"
                className={`ml-1 px-1.5 py-0.2 text-[10px] font-mono rounded-full ${
                  isSelected
                    ? 'bg-sky-800 text-sky-100'
                    : 'bg-slate-800 text-slate-400'
                }`}
              >
                {opt.badge}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
