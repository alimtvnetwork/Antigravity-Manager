import React from 'react';
import { User, Bot, Terminal, Wrench } from 'lucide-react';

export type PromptOriginTier =
  | 'USER_PROMPT'
  | 'SUBAGENT_INSTRUCTION'
  | 'SYSTEM_MESSAGE'
  | 'TOOL_OUTPUT';

interface OriginTierBadgePillProps {
  origin: PromptOriginTier | string;
  size?: 'xs' | 'sm';
  showLabel?: boolean;
  className?: string;
}

export const OriginTierBadgePill: React.FC<OriginTierBadgePillProps> = ({
  origin,
  size = 'xs',
  showLabel = true,
  className = '',
}) => {
  const normalized = origin.toUpperCase();

  let label = 'System';
  let badgeStyle = 'bg-zinc-100 text-zinc-800 border-zinc-300 dark:bg-zinc-800/80 dark:text-zinc-300 dark:border-zinc-700/60';
  let icon = <Terminal className="w-3 h-3 text-zinc-600 dark:text-zinc-400" />;

  switch (normalized) {
    case 'USER_PROMPT':
      label = 'User';
      badgeStyle = 'bg-sky-100 text-sky-800 border-sky-300 dark:bg-sky-950/70 dark:text-sky-300 dark:border-sky-600/50 shadow-xs shadow-sky-900/20';
      icon = <User className="w-3 h-3 text-sky-600 dark:text-sky-400" />;
      break;
    case 'SUBAGENT_INSTRUCTION':
      label = 'Subagent';
      badgeStyle = 'bg-purple-100 text-purple-800 border-purple-300 dark:bg-purple-950/70 dark:text-purple-300 dark:border-purple-600/50 shadow-xs shadow-purple-900/20';
      icon = <Bot className="w-3 h-3 text-purple-600 dark:text-purple-400" />;
      break;
    case 'TOOL_OUTPUT':
      label = 'Tool';
      badgeStyle = 'bg-amber-100 text-amber-800 border-amber-300 dark:bg-amber-950/70 dark:text-amber-300 dark:border-amber-600/50 shadow-xs shadow-amber-900/20';
      icon = <Wrench className="w-3 h-3 text-amber-600 dark:text-amber-400" />;
      break;
    case 'SYSTEM_MESSAGE':
    default:
      label = 'System';
      badgeStyle = 'bg-zinc-100 text-zinc-800 border-zinc-300 dark:bg-zinc-800/80 dark:text-zinc-300 dark:border-zinc-700/60';
      icon = <Terminal className="w-3 h-3 text-zinc-600 dark:text-zinc-400" />;
      break;
  }

  const sizeClasses = size === 'xs' ? 'text-[10px] px-1.5 py-0.5 gap-1' : 'text-xs px-2 py-0.5 gap-1.5';

  return (
    <span
      className={`inline-flex items-center font-mono font-medium rounded-full border backdrop-blur-md transition-all ${badgeStyle} ${sizeClasses} ${className}`}
      title={`Prompt Origin: ${normalized}`}
      role="status"
      aria-label={`Origin: ${label}`}
    >
      {icon}
      {showLabel && <span>{label}</span>}
    </span>
  );
};
