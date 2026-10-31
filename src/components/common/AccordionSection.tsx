import React, { useState, useCallback, useId } from 'react';
import { ChevronDown } from 'lucide-react';

export interface AccordionSectionProps {
    title: React.ReactNode;
    subtitle?: React.ReactNode;
    icon?: React.ReactNode;
    badge?: React.ReactNode;
    isOpen?: boolean;
    defaultOpen?: boolean;
    onToggle?: (isOpen: boolean) => void;
    children: React.ReactNode;
    className?: string;
    headerClassName?: string;
    contentClassName?: string;
}

export const AccordionSection: React.FC<AccordionSectionProps> = ({
    title,
    subtitle,
    icon,
    badge,
    isOpen: controlledOpen,
    defaultOpen = false,
    onToggle,
    children,
    className = '',
    headerClassName = '',
    contentClassName = '',
}) => {
    const [internalOpen, setInternalOpen] = useState(defaultOpen);
    const contentId = useId();
    const headerId = `${contentId}-header`;

    const isExpanded = controlledOpen !== undefined ? controlledOpen : internalOpen;

    const handleToggle = useCallback(() => {
        const next = !isExpanded;
        if (controlledOpen === undefined) {
            setInternalOpen(next);
        }
        onToggle?.(next);
    }, [isExpanded, controlledOpen, onToggle]);

    const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleToggle();
        }
    }, [handleToggle]);

    return (
        <div
            className={`rounded-xl border border-slate-200 dark:border-slate-800/80 bg-white/70 dark:bg-slate-900/70 backdrop-blur-md overflow-hidden transition-all duration-200 ${className}`}
        >
            <div
                id={headerId}
                role="button"
                tabIndex={0}
                aria-expanded={isExpanded}
                aria-controls={contentId}
                onClick={handleToggle}
                onKeyDown={handleKeyDown}
                className={`flex items-center justify-between px-4 py-3 cursor-pointer select-none transition-colors hover:bg-slate-100/60 dark:hover:bg-slate-800/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500/60 ${headerClassName}`}
            >
                <div className="flex items-center gap-3 min-w-0">
                    {icon && <span className="text-slate-400 shrink-0">{icon}</span>}
                    <div className="flex flex-col min-w-0">
                        <div className="flex items-center gap-2">
                            <span className="text-sm font-semibold text-slate-800 dark:text-slate-200 truncate">
                                {title}
                            </span>
                            {badge && <span className="shrink-0">{badge}</span>}
                        </div>
                        {subtitle && (
                            <span className="text-xs text-slate-500 dark:text-slate-400 truncate">
                                {subtitle}
                            </span>
                        )}
                    </div>
                </div>

                <ChevronDown
                    size={16}
                    className={`text-slate-400 transition-transform duration-200 shrink-0 ml-2 ${
                        isExpanded ? 'rotate-180' : 'rotate-0'
                    }`}
                />
            </div>

            {isExpanded && (
                <div
                    id={contentId}
                    role="region"
                    aria-labelledby={headerId}
                    className={`px-4 pb-4 pt-1 border-t border-slate-100 dark:border-slate-800/40 text-sm text-slate-600 dark:text-slate-300 animate-in fade-in-50 duration-150 ${contentClassName}`}
                >
                    {children}
                </div>
            )}
        </div>
    );
};

export default AccordionSection;
