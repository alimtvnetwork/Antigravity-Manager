import type { ReactElement } from 'react';
import { Snowflake } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { PortalTooltip, type TooltipTriggerProps } from '../common/PortalTooltip';

export interface ColdStartBadgeProps {
  className?: string;
  isCompact?: boolean;
}

function renderColdStartPill(
  label: string,
  className?: string,
  tooltipText?: string
): ReactElement<TooltipTriggerProps> {
  return (
    <span
      className={cn(
        'bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 text-[10px] px-2 py-0.5 rounded-full font-medium inline-flex items-center gap-1 select-none backdrop-blur-sm cursor-help',
        className
      )}
      title={tooltipText}
    >
      <Snowflake className="w-3 h-3 shrink-0 text-cyan-400" aria-hidden="true" />
      <span>{label}</span>
    </span>
  );
}

export function ColdStartBadge({ className, isCompact = false }: ColdStartBadgeProps): ReactElement {
  const { t } = useTranslation();
  const label = isCompact ? 'Cold' : 'Cold-Start';
  const tooltipText = t(
    'accounts.warmup.cold_start_tooltip',
    'This account has not yet initialized its 7-day rolling window. Smart Warmup will wake it to start the clock.'
  );

  return (
    <PortalTooltip content={tooltipText} placement="top">
      {renderColdStartPill(label, className, tooltipText)}
    </PortalTooltip>
  );
}

export default ColdStartBadge;
