import React from 'react';
import { User, Bot, Terminal, Wrench, Flame } from 'lucide-react';
import { PromptOriginTier } from '../../types/prompt';

export interface PromptOriginBadgeProps {
  origin: PromptOriginTier | string;
  size?: 'xs' | 'sm' | 'md';
  showLabel?: boolean;
  className?: string;
  onClick?: () => void;
}

export const PromptOriginBadge: React.FC<PromptOriginBadgeProps> = ({
  origin,
  size = 'xs',
  showLabel = true,
  className = '',
  onClick,
}) => {
  const normalized = origin.toUpperCase();

  let label = 'System';
  let badgeStyle =
    'bg-zinc-500/15 text-zinc-400 border border-zinc-500/30 dark:bg-zinc-900/40 dark:text-zinc-300 dark:border-zinc-700/60';
  let iconComponent = <Terminal className="w-3 h-3 text-zinc-400" />;

  switch (normalized) {
    case 'USER_PROMPT':
      label = 'User';
      badgeStyle =
        'bg-sky-500/15 text-sky-400 border border-sky-500/30 dark:bg-sky-950/40 dark:text-sky-300 dark:border-sky-800/60';
      iconComponent = <User className="w-3 h-3 text-sky-400" />;
      break;

    case 'SUBAGENT_INSTRUCTION':
      label = 'Subagent';
      badgeStyle =
        'bg-purple-500/15 text-purple-400 border border-purple-500/30 dark:bg-purple-950/40 dark:text-purple-300 dark:border-purple-800/60';
      iconComponent = <Bot className="w-3 h-3 text-purple-400" />;
      break;

    case 'SYSTEM_MESSAGE':
      label = 'System';
      badgeStyle =
        'bg-zinc-500/15 text-zinc-400 border border-zinc-500/30 dark:bg-zinc-900/40 dark:text-zinc-300 dark:border-zinc-700/60';
      iconComponent = <Terminal className="w-3 h-3 text-zinc-400" />;
      break;

    case 'TOOL_OUTPUT':
      label = 'Tool Output';
      badgeStyle =
        'bg-amber-500/15 text-amber-400 border border-amber-500/30 dark:bg-amber-950/40 dark:text-amber-300 dark:border-amber-800/60';
      iconComponent = <Wrench className="w-3 h-3 text-amber-400" />;
      break;

    case 'WARMUP_PROBE':
      label = 'Warmup Probe';
      badgeStyle =
        'bg-orange-500/15 text-orange-400 border border-orange-500/30 shadow-[0_0_8px_rgba(249,115,22,0.15)] dark:bg-orange-950/40 dark:text-orange-300 dark:border-orange-800/60';
      iconComponent = <Flame className="w-3 h-3 text-orange-400" />;
      break;

    default:
      label = origin;
      badgeStyle =
        'bg-zinc-800/50 text-zinc-300 border border-zinc-700/50 dark:bg-zinc-900/50 dark:text-zinc-300 dark:border-zinc-700/50';
      iconComponent = <Terminal className="w-3 h-3 text-zinc-400" />;
      break;
  }

  const sizeClasses =
    size === 'xs'
      ? 'text-[10px] px-1.5 py-0.5 gap-1'
      : size === 'sm'
      ? 'text-xs px-2 py-0.5 gap-1.5'
      : 'text-xs px-2.5 py-1 gap-2';

  const interactiveClasses = onClick
    ? 'cursor-pointer hover:brightness-110 active:scale-95'
    : '';

  return (
    <span
      className={`inline-flex items-center font-mono font-medium rounded-full border backdrop-blur-md transition-all uppercase tracking-wider ${badgeStyle} ${sizeClasses} ${interactiveClasses} ${className}`}
      title={`Prompt Origin: ${normalized}`}
      role="status"
      aria-label={`Origin: ${label}`}
      onClick={onClick}
    >
      {iconComponent}
      {showLabel && <span>{label}</span>}
    </span>
  );
};

export default PromptOriginBadge;
