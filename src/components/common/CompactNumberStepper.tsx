import React from 'react';
import { Minus, Plus } from 'lucide-react';

export interface CompactNumberStepperProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  disabled?: boolean;
  className?: string;
}

export const CompactNumberStepper: React.FC<CompactNumberStepperProps> = ({
  value,
  onChange,
  min = 0,
  max = 100,
  step = 1,
  unit = '',
  disabled = false,
  className = '',
}) => {
  const handleDecrement = () => {
    if (disabled || value <= min) return;
    onChange(Math.max(min, value - step));
  };

  const handleIncrement = () => {
    if (disabled || value >= max) return;
    onChange(Math.min(max, value + step));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;
    if (e.key === 'ArrowUp' || e.key === 'ArrowRight') {
      e.preventDefault();
      handleIncrement();
    } else if (e.key === 'ArrowDown' || e.key === 'ArrowLeft') {
      e.preventDefault();
      handleDecrement();
    }
  };

  return (
    <div
      role="spinbutton"
      tabIndex={disabled ? -1 : 0}
      aria-valuenow={value}
      aria-valuemin={min}
      aria-valuemax={max}
      onKeyDown={handleKeyDown}
      className={`inline-flex items-center rounded-full bg-slate-900/80 border border-slate-700/60 p-0.5 text-xs select-none backdrop-blur-md focus:outline-none focus-visible:ring-1 focus-visible:ring-sky-400 ${
        disabled ? 'opacity-50 pointer-events-none' : ''
      } ${className}`}
    >
      <button
        type="button"
        onClick={handleDecrement}
        disabled={disabled || value <= min}
        className="p-1 rounded-full text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 disabled:opacity-30 disabled:hover:bg-transparent transition-colors focus:outline-none"
        aria-label="Decrease value"
      >
        <Minus className="w-3 h-3" />
      </button>

      <span className="px-2 font-mono font-medium text-slate-200 min-w-[2.5rem] text-center">
        {value}
        {unit && <span className="text-[10px] text-slate-400 ml-0.5">{unit}</span>}
      </span>

      <button
        type="button"
        onClick={handleIncrement}
        disabled={disabled || value >= max}
        className="p-1 rounded-full text-slate-400 hover:text-slate-100 hover:bg-slate-800/80 disabled:opacity-30 disabled:hover:bg-transparent transition-colors focus:outline-none"
        aria-label="Increase value"
      >
        <Plus className="w-3 h-3" />
      </button>
    </div>
  );
};
