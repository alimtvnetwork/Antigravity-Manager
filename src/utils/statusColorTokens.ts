/**
 * Centralized Status & Tier Color Token Helper.
 * Maps operational states, instance modes, and 4-tier prompt origins to consistent Tailwind tokens.
 */

export type PromptOriginTier =
  | 'USER_PROMPT'
  | 'SUBAGENT_INSTRUCTION'
  | 'SYSTEM_MESSAGE'
  | 'TOOL_OUTPUT'
  | 'WARMUP_PROBE';

export interface TierStyleTokens {
  badgeBg: string;
  borderColor: string;
  textColor: string;
  dotColor: string;
  displayName: string;
}

/**
 * Returns 4-Tier origin badge styling according to project guidelines.
 */
export function getPromptOriginTokens(origin: PromptOriginTier | string): TierStyleTokens {
  switch (origin) {
    case 'USER_PROMPT':
      return {
        badgeBg: 'bg-sky-500/10 dark:bg-sky-950/40',
        borderColor: 'border-sky-500/30',
        textColor: 'text-sky-300 dark:text-sky-200',
        dotColor: 'bg-sky-400',
        displayName: 'User',
      };
    case 'SUBAGENT_INSTRUCTION':
      return {
        badgeBg: 'bg-purple-500/10 dark:bg-purple-950/40',
        borderColor: 'border-purple-500/30',
        textColor: 'text-purple-300 dark:text-purple-200',
        dotColor: 'bg-purple-400',
        displayName: 'Subagent',
      };
    case 'SYSTEM_MESSAGE':
      return {
        badgeBg: 'bg-zinc-500/10 dark:bg-zinc-900/40',
        borderColor: 'border-zinc-600/30',
        textColor: 'text-zinc-300 dark:text-zinc-200',
        dotColor: 'bg-zinc-400',
        displayName: 'System',
      };
    case 'TOOL_OUTPUT':
      return {
        badgeBg: 'bg-amber-500/10 dark:bg-amber-950/40',
        borderColor: 'border-amber-500/30',
        textColor: 'text-amber-300 dark:text-amber-200',
        dotColor: 'bg-amber-400',
        displayName: 'Tool Output',
      };
    case 'WARMUP_PROBE':
      return {
        badgeBg: 'bg-orange-500/15 dark:bg-orange-950/40',
        borderColor: 'border-orange-500/30 dark:border-orange-800/60',
        textColor: 'text-orange-400 dark:text-orange-300',
        dotColor: 'bg-orange-400',
        displayName: 'Warmup Probe',
      };
    default:
      return {
        badgeBg: 'bg-slate-800/40',
        borderColor: 'border-slate-700/40',
        textColor: 'text-slate-300',
        dotColor: 'bg-slate-400',
        displayName: origin,
      };
  }
}

/**
 * Returns quota health colors based on percentage (0-100).
 */
export function getQuotaColorTokens(percentage: number): {
  track: string;
  fill: string;
  glow: string;
  text: string;
} {
  if (percentage <= 20) {
    return {
      track: 'bg-rose-950/40 border-rose-800/50',
      fill: 'bg-rose-500',
      glow: 'shadow-[0_0_8px_rgba(244,63,94,0.4)]',
      text: 'text-rose-400',
    };
  }
  if (percentage <= 50) {
    return {
      track: 'bg-amber-950/40 border-amber-800/50',
      fill: 'bg-amber-500',
      glow: 'shadow-[0_0_8px_rgba(245,158,11,0.4)]',
      text: 'text-amber-400',
    };
  }
  return {
    track: 'bg-emerald-950/40 border-emerald-800/50',
    fill: 'bg-emerald-500',
    glow: 'shadow-[0_0_8px_rgba(16,185,129,0.4)]',
    text: 'text-emerald-400',
  };
}
