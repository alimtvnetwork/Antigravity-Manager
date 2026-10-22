import React from 'react';
import { Minus, Plus } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface NumberStepperProps {
    value: number;
    onChange: (val: number) => void;
    min?: number;
    max?: number;
    step?: number;
    unit?: string;
    className?: string;
    disabled?: boolean;
}

export function NumberStepper({
    value,
    onChange,
    min = 0,
    max = 999999,
    step = 1,
    unit,
    className,
    disabled = false,
}: NumberStepperProps) {
    const handleIncrement = (e: React.MouseEvent) => {
        e.preventDefault();
        if (!disabled && value + step <= max) {
            onChange(value + step);
        }
    };

    const handleDecrement = (e: React.MouseEvent) => {
        e.preventDefault();
        if (!disabled && value - step >= min) {
            onChange(value - step);
        }
    };

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const parsed = parseFloat(e.target.value);
        if (!isNaN(parsed)) {
            const clamped = Math.max(min, Math.min(max, parsed));
            onChange(clamped);
        }
    };

    const canDecrement = !disabled && value - step >= min;
    const canIncrement = !disabled && value + step <= max;

    return (
        <div
            className={cn(
                'inline-flex items-center rounded-lg border p-0.5 shadow-xs transition-colors',
                'bg-slate-100 dark:bg-slate-900/80 border-slate-200 dark:border-slate-800',
                disabled && 'opacity-50 cursor-not-allowed',
                className
            )}
        >
            <button
                type="button"
                onClick={handleDecrement}
                disabled={!canDecrement}
                className={cn(
                    'flex items-center justify-center w-6 h-6 rounded-md transition-colors',
                    'hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-300',
                    'disabled:opacity-30 disabled:pointer-events-none'
                )}
            >
                <Minus className="w-3 h-3" />
            </button>

            <div className="flex items-center px-1.5 min-w-[36px] justify-center">
                <input
                    type="number"
                    value={value}
                    onChange={handleInputChange}
                    min={min}
                    max={max}
                    step={step}
                    disabled={disabled}
                    className="w-10 text-center font-mono text-xs bg-transparent border-0 focus:ring-0 focus:outline-none text-slate-800 dark:text-slate-100 p-0"
                />
                {unit && <span className="text-[10px] text-slate-400 ml-0.5">{unit}</span>}
            </div>

            <button
                type="button"
                onClick={handleIncrement}
                disabled={!canIncrement}
                className={cn(
                    'flex items-center justify-center w-6 h-6 rounded-md transition-colors',
                    'hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-300',
                    'disabled:opacity-30 disabled:pointer-events-none'
                )}
            >
                <Plus className="w-3 h-3" />
            </button>
        </div>
    );
}

export default NumberStepper;
