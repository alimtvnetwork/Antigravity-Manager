import type { ReactElement } from 'react';
import { Clock } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { useWarmupResetTimer, type TimerUrgency } from '../../hooks/useWarmupResetTimer';
import { PortalTooltip, type TooltipTriggerProps } from '../common/PortalTooltip';
import { ColdStartBadge } from './ColdStartBadge';

export interface WarmupCountdownClockProps {
  resetTime?: string | number | null;
  className?: string;
  showLabel?: boolean;
}

interface UrgencyStyles {
  container: string;
  text: string;
  icon: string;
}

function getUrgencyStyles(urgency: TimerUrgency): UrgencyStyles {
  if (urgency === 'imminent' || urgency === 'expired') {
    return {
      container: 'bg-orange-500/10 border-orange-500/30 shadow-[0_0_8px_rgba(249,115,22,0.2)]',
      text: 'text-orange-400 dark:text-orange-300 font-bold animate-pulse',
      icon: 'text-orange-400 animate-pulse',
    };
  }
  if (urgency === 'approaching') {
    return {
      container: 'bg-amber-500/10 border-amber-500/30',
      text: 'text-amber-400 dark:text-amber-300 font-semibold',
      icon: 'text-amber-400',
    };
  }
  return {
    container: 'bg-zinc-900/60 border-zinc-700/40 hover:border-zinc-600/60',
    text: 'text-zinc-300 dark:text-zinc-200 font-medium',
    icon: 'text-zinc-400',
  };
}

function renderClockCapsule(
  formatted: string,
  styles: UrgencyStyles,
  showLabel: boolean,
  label: string,
  className?: string
): ReactElement<TooltipTriggerProps> {
  return (
    <div
      className={cn(
        'inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full border text-[10px] select-none backdrop-blur-md transition-all cursor-default',
        styles.container,
        className
      )}
    >
      <Clock className={cn('w-3 h-3 shrink-0', styles.icon)} aria-hidden="true" />
      {showLabel && (
        <span className="text-zinc-400 dark:text-zinc-500 font-normal">
          {label}:
        </span>
      )}
      <span className={cn('font-mono tracking-tight tabular-nums', styles.text)}>
        {formatted}
      </span>
    </div>
  );
}

export function WarmupCountdownClock({
  resetTime,
  className,
  showLabel = false,
}: WarmupCountdownClockProps): ReactElement {
  const { t } = useTranslation();
  const timer = useWarmupResetTimer(resetTime);

  if (!resetTime) {
    return <ColdStartBadge className={className} />;
  }

  const styles = getUrgencyStyles(timer.urgency);
  const label = t('accounts.reset_time', 'Reset');
  const tooltipContent = (
    <div className="flex flex-col gap-0.5 text-[11px] font-mono leading-tight py-0.5 min-w-[160px]">
      <div className="flex items-center justify-between gap-3 text-zinc-300">
        <span className="text-zinc-400 font-sans">Target:</span>
        <span className="text-zinc-200 font-medium">{String(resetTime)}</span>
      </div>
      <div className="flex items-center justify-between gap-3 text-zinc-300">
        <span className="text-zinc-400 font-sans">Remaining:</span>
        <span className={styles.text}>{timer.formatted}</span>
      </div>
    </div>
  );

  return (
    <PortalTooltip content={tooltipContent} placement="top">
      {renderClockCapsule(timer.formatted, styles, showLabel, label, className)}
    </PortalTooltip>
  );
}

export default WarmupCountdownClock;
