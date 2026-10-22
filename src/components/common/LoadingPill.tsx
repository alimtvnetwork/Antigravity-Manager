import { Loader2 } from 'lucide-react';
import { cn } from '../../utils/cn';

export interface LoadingPillProps {
    label?: string;
    size?: 'xs' | 'sm' | 'md';
    className?: string;
}

export function LoadingPill({
    label = 'Loading...',
    size = 'sm',
    className,
}: LoadingPillProps) {
    const sizeClasses = {
        xs: 'text-[9px] px-1.5 py-0.2 gap-1 rounded',
        sm: 'text-[10px] px-2 py-0.5 gap-1.5 rounded-full font-semibold',
        md: 'text-xs px-3 py-1 gap-2 rounded-full font-semibold',
    }[size];

    return (
        <span
            className={cn(
                'inline-flex items-center border select-none backdrop-blur-xs font-mono',
                'bg-purple-500/10 text-purple-600 dark:text-purple-300 border-purple-500/20',
                sizeClasses,
                className
            )}
        >
            <Loader2 className="w-3 h-3 animate-spin shrink-0" />
            <span className="truncate">{label}</span>
        </span>
    );
}

export default LoadingPill;
