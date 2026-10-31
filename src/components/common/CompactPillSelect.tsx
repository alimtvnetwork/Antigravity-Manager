import React, { ReactNode } from 'react';
import { ChevronDown } from 'lucide-react';

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface CompactPillSelectProps {
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  icon?: ReactNode;
  prefix?: string;
  size?: 'sm' | 'md';
  className?: string;
  disabled?: boolean;
}

export const CompactPillSelect: React.FC<CompactPillSelectProps> = ({
  options,
  value,
  onChange,
  icon,
  prefix,
  size = 'md',
  className = '',
  disabled = false,
}) => {
  const getSizeClasses = () => {
    switch (size) {
      case 'sm':
        return 'h-6 text-[11px] px-2 gap-1';
      case 'md':
      default:
        return 'h-7 text-xs px-2.5 gap-1.5';
    }
  };

  return (
    <div
      className={`relative inline-flex items-center rounded-full bg-slate-900/60 dark:bg-slate-950/70 border border-slate-700/60 shadow-inner backdrop-blur-sm transition-all focus-within:border-sky-500/70 ${getSizeClasses()} ${
        disabled ? 'opacity-50 pointer-events-none' : ''
      } ${className}`}
    >
      {icon && <span className="text-slate-400 shrink-0 pointer-events-none">{icon}</span>}
      {prefix && (
        <span className="text-slate-400 font-mono text-[10px] uppercase shrink-0 pointer-events-none">
          {prefix}:
        </span>
      )}

      {/* Borderless Transparent Native Select */}
      <select
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-transparent border-0 p-0 pr-4 text-slate-200 dark:text-slate-200 font-medium focus:ring-0 focus:outline-none appearance-none cursor-pointer select-none"
      >
        {options.map((opt) => (
          <option
            key={`opt-${opt.value}`}
            value={opt.value}
            disabled={opt.disabled}
            className="bg-slate-900 text-slate-200 py-1"
          >
            {opt.label}
          </option>
        ))}
      </select>

      {/* Styled Micro Chevron */}
      <ChevronDown className="absolute right-2 w-3 h-3 text-slate-400 pointer-events-none" />
    </div>
  );
};
