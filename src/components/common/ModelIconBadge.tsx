import { Sparkles, Bot, Cpu, Layers } from 'lucide-react';
import { cn } from '../../utils/cn';

export type ModelFamily = 'gemini' | 'claude' | 'gpt' | 'deepseek' | 'unknown';

export interface ModelIconBadgeProps {
    modelId: string;
    size?: 'xs' | 'sm' | 'md';
    showLabel?: boolean;
    className?: string;
}

export function detectModelFamily(modelId: string): ModelFamily {
    const lower = modelId.trim().toLowerCase();
    if (lower.contains ? lower.includes('gemini') : lower.indexOf('gemini') !== -1) {
        return 'gemini';
    }
    if (lower.includes('claude') || lower.includes('sonnet') || lower.includes('opus')) {
        return 'claude';
    }
    if (lower.includes('gpt') || lower.includes('o1') || lower.includes('o3') || lower.includes('openai')) {
        return 'gpt';
    }
    if (lower.includes('deepseek')) {
        return 'deepseek';
    }
    return 'unknown';
}

const familyConfig: Record<
    ModelFamily,
    { label: string; bg: string; text: string; border: string; icon: typeof Sparkles }
> = {
    gemini: {
        label: 'Gemini',
        bg: 'bg-sky-500/15',
        text: 'text-sky-700 dark:text-sky-300',
        border: 'border-sky-500/30',
        icon: Sparkles,
    },
    claude: {
        label: 'Claude',
        bg: 'bg-amber-500/15',
        text: 'text-amber-700 dark:text-amber-300',
        border: 'border-amber-500/30',
        icon: Cpu,
    },
    gpt: {
        label: 'GPT',
        bg: 'bg-emerald-500/15',
        text: 'text-emerald-700 dark:text-emerald-300',
        border: 'border-emerald-500/30',
        icon: Bot,
    },
    deepseek: {
        label: 'DeepSeek',
        bg: 'bg-blue-500/15',
        text: 'text-blue-700 dark:text-blue-300',
        border: 'border-blue-500/30',
        icon: Layers,
    },
    unknown: {
        label: 'Model',
        bg: 'bg-slate-500/15',
        text: 'text-slate-700 dark:text-slate-300',
        border: 'border-slate-500/30',
        icon: Cpu,
    },
};

export function ModelIconBadge({
    modelId,
    size = 'sm',
    showLabel = true,
    className,
}: ModelIconBadgeProps) {
    const family = detectModelFamily(modelId);
    const cfg = familyConfig[family];
    const IconComponent = cfg.icon;

    const sizeClasses = {
        xs: 'text-[9px] px-1.5 py-0.2 gap-1 rounded',
        sm: 'text-[10px] px-2 py-0.5 gap-1.5 rounded-full font-semibold',
        md: 'text-xs px-2.5 py-1 gap-1.5 rounded-full font-semibold',
    }[size];

    return (
        <span
            title={modelId}
            className={cn(
                'inline-flex items-center border select-none transition-colors backdrop-blur-xs font-mono',
                cfg.bg,
                cfg.text,
                cfg.border,
                sizeClasses,
                className
            )}
        >
            <IconComponent className="w-3 h-3 shrink-0" />
            {showLabel && <span className="truncate">{cfg.label}</span>}
        </span>
    );
}

export default ModelIconBadge;
