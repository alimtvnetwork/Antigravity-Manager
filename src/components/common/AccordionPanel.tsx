import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface AccordionPanelProps {
    title: React.ReactNode;
    subtitle?: string;
    badge?: React.ReactNode;
    isOpen?: boolean;
    defaultOpen?: boolean;
    onToggle?: (isOpen: boolean) => void;
    children: React.ReactNode;
    className?: string;
    headerClassName?: string;
    contentClassName?: string;
}

export function AccordionPanel({
    title,
    subtitle,
    badge,
    isOpen: controlledIsOpen,
    defaultOpen = false,
    onToggle,
    children,
    className,
    headerClassName,
    contentClassName,
}: AccordionPanelProps) {
    const [internalOpen, setInternalOpen] = useState(defaultOpen);
    const isControlled = controlledIsOpen !== undefined;
    const open = isControlled ? controlledIsOpen : internalOpen;

    const handleToggle = () => {
        const nextState = !open;
        if (!isControlled) {
            setInternalOpen(nextState);
        }
        onToggle?.(nextState);
    };

    return (
        <div
            className={cn(
                'rounded-xl border border-slate-200/80 dark:border-slate-800/80 bg-white/70 dark:bg-slate-900/60 backdrop-blur-md shadow-xs transition-all duration-200 overflow-hidden',
                open && 'border-slate-300 dark:border-slate-700/80 shadow-md',
                className
            )}
        >
            <button
                type="button"
                onClick={handleToggle}
                aria-expanded={open}
                className={cn(
                    'w-full flex items-center justify-between px-4 py-3 text-left transition-colors duration-150',
                    'hover:bg-slate-500/5 dark:hover:bg-slate-800/40 focus:outline-none focus-visible:ring-2 focus-visible:ring-purple-500',
                    headerClassName
                )}
            >
                <div className="flex items-center gap-3 min-w-0">
                    <div className="flex flex-col min-w-0">
                        <div className="text-sm font-semibold text-slate-800 dark:text-slate-100 truncate">
                            {title}
                        </div>
                        {subtitle && (
                            <div className="text-xs text-slate-500 dark:text-slate-400 truncate">
                                {subtitle}
                            </div>
                        )}
                    </div>
                    {badge && <div className="shrink-0">{badge}</div>}
                </div>

                <div className="shrink-0 ml-3 text-slate-400 dark:text-slate-500 transition-transform duration-200">
                    <ChevronDown
                        className={cn(
                            'w-4 h-4 transition-transform duration-200',
                            open && 'rotate-180 text-purple-600 dark:text-purple-400'
                        )}
                    />
                </div>
            </button>

            <div
                className={cn(
                    'grid transition-all duration-200 ease-in-out',
                    open ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0 pointer-events-none'
                )}
            >
                <div className="overflow-hidden">
                    <div
                        className={cn(
                            'px-4 pb-4 pt-1 border-t border-slate-100 dark:border-slate-800/60 text-sm text-slate-600 dark:text-slate-300',
                            contentClassName
                        )}
                    >
                        {children}
                    </div>
                </div>
            </div>
        </div>
    );
}

export default AccordionPanel;
