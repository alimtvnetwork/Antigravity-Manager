import React, { useState, useRef, useEffect } from 'react';
import { cn } from '../../utils/cn';

export interface DropdownMenuItem {
    id: string;
    label: React.ReactNode;
    icon?: React.ReactNode;
    danger?: boolean;
    disabled?: boolean;
    onClick: () => void;
}

export interface DropdownMenuProps {
    trigger: React.ReactNode;
    items: DropdownMenuItem[];
    align?: 'left' | 'right';
    className?: string;
    menuClassName?: string;
}

export function DropdownMenu({
    trigger,
    items,
    align = 'right',
    className,
    menuClassName,
}: DropdownMenuProps) {
    const [isOpen, setIsOpen] = useState(false);
    const containerRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
                setIsOpen(false);
            }
        };

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setIsOpen(false);
            }
        };

        if (isOpen) {
            document.addEventListener('mousedown', handleClickOutside);
            document.addEventListener('keydown', handleKeyDown);
        }

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            document.removeEventListener('keydown', handleKeyDown);
        };
    }, [isOpen]);

    return (
        <div ref={containerRef} className={cn('relative inline-block text-left', className)}>
            <div onClick={() => setIsOpen(!isOpen)} role="button" tabIndex={0}>
                {trigger}
            </div>

            {isOpen && (
                <div
                    role="menu"
                    className={cn(
                        'absolute z-50 mt-1 min-w-[160px] py-1 rounded-xl shadow-xl',
                        'bg-white/95 dark:bg-slate-900/95 backdrop-blur-md',
                        'border border-slate-200 dark:border-slate-800 text-slate-800 dark:text-slate-100',
                        'animate-in fade-in zoom-in-95 duration-100',
                        align === 'right' ? 'right-0' : 'left-0',
                        menuClassName
                    )}
                >
                    {items.map((item) => (
                        <button
                            key={item.id}
                            type="button"
                            role="menuitem"
                            disabled={item.disabled}
                            onClick={() => {
                                if (!item.disabled) {
                                    item.onClick();
                                    setIsOpen(false);
                                }
                            }}
                            className={cn(
                                'w-full flex items-center gap-2 px-3 py-1.5 text-xs text-left transition-colors duration-150',
                                item.disabled && 'opacity-40 cursor-not-allowed',
                                !item.disabled && item.danger && 'text-rose-600 dark:text-rose-400 hover:bg-rose-500/10',
                                !item.disabled &&
                                    !item.danger &&
                                    'hover:bg-slate-100 dark:hover:bg-slate-800/80 text-slate-700 dark:text-slate-200'
                            )}
                        >
                            {item.icon && <span className="w-3.5 h-3.5 shrink-0 flex items-center">{item.icon}</span>}
                            <span className="truncate">{item.label}</span>
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}

export default DropdownMenu;
