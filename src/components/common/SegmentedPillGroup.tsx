import React from 'react';
import { cn } from '../../utils/cn';

export interface SegmentedPillOption {
    id: string;
    label: React.ReactNode;
    icon?: React.ReactNode;
    badge?: React.ReactNode;
    disabled?: boolean;
}

export interface SegmentedPillGroupProps {
    options: SegmentedPillOption[];
    value: string;
    onChange: (id: string) => void;
    size?: 'sm' | 'md';
    className?: string;
}

export function SegmentedPillGroup({
    options,
    value,
    onChange,
    size = 'sm',
    className,
}: SegmentedPillGroupProps) {
    const sizeClasses = {
        sm: 'text-xs px-2.5 py-1 gap-1.5',
        md: 'text-sm px-3.5 py-1.5 gap-2',
    }[size];

    return (
        <div
            role="tablist"
            className={cn(
                'inline-flex items-center rounded-full p-0.5 shadow-xs select-none backdrop-blur-md',
                'bg-slate-100 dark:bg-slate-900/90 border border-slate-200 dark:border-slate-800',
                className
            )}
        >
            {options.map((opt) => {
                const isActive = opt.id === value;
                return (
                    <button
                        key={opt.id}
                        type="button"
                        role="tab"
                        aria-selected={isActive}
                        disabled={opt.disabled}
                        onClick={() => !opt.disabled && onChange(opt.id)}
                        className={cn(
                            'inline-flex items-center justify-center font-medium rounded-full transition-all duration-150',
                            sizeClasses,
                            isActive
                                ? 'bg-white dark:bg-purple-600 text-slate-900 dark:text-white shadow-xs font-semibold'
                                : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 hover:bg-slate-200/50 dark:hover:bg-slate-800/50',
                            opt.disabled && 'opacity-40 cursor-not-allowed'
                        )}
                    >
                        {opt.icon && <span className="w-3.5 h-3.5 shrink-0 flex items-center">{opt.icon}</span>}
                        <span>{opt.label}</span>
                        {opt.badge && <span className="ml-0.5 shrink-0">{opt.badge}</span>}
                    </button>
                );
            })}
        </div>
    );
}

export default SegmentedPillGroup;
