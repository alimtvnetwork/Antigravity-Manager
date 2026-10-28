import type { ReactElement, MouseEvent } from 'react';
import { Ban, Zap } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { PortalTooltip } from '../common/PortalTooltip';

export interface AccountWarmupOptOutProps {
  isExempt?: boolean;
  onToggle?: (exempt: boolean) => void;
  className?: string;
  isCompact?: boolean;
}

function resolveLabel(isExempt: boolean, isCompact: boolean): string {
  if (isExempt) {
    return isCompact ? 'Exempt' : 'Warmup Exempt';
  }
  return isCompact ? 'Active' : 'Warmup Active';
}

function resolveTooltip(isExempt: boolean, t: (key: string, defaultValue: string) => string): string {
  if (isExempt) {
    return t(
      'accounts.warmup.opt_out_exempt_tooltip',
      'This account is exempt from automated warmup probes.'
    );
  }
  return t(
    'accounts.warmup.opt_out_active_tooltip',
    'Automated warmup probes run on this account.'
  );
}

function renderPillIcon(isExempt: boolean): ReactElement {
  if (isExempt) {
    return <Ban className="w-3 h-3 text-amber-400 shrink-0" aria-hidden="true" />;
  }
  return <Zap className="w-3 h-3 text-emerald-400 shrink-0" aria-hidden="true" />;
}

export function AccountWarmupOptOut({
  isExempt = false,
  onToggle,
  className,
  isCompact = false,
}: AccountWarmupOptOutProps): ReactElement {
  const { t } = useTranslation();
  const label = resolveLabel(isExempt, isCompact);
  const tooltip = resolveTooltip(isExempt, t);

  const handleClick = (event: MouseEvent<HTMLButtonElement>): void => {
    event.stopPropagation();
    onToggle?.(!isExempt);
  };

  return (
    <PortalTooltip content={tooltip} placement="top">
      <button
        type="button"
        role="switch"
        aria-checked={isExempt}
        aria-label={label}
        onClick={handleClick}
        className={cn(
          'bg-zinc-800 text-zinc-400 border border-zinc-700 text-[10px] px-2 py-0.5 rounded-full inline-flex items-center gap-1.5 font-medium transition-colors hover:border-zinc-500 hover:text-zinc-200 cursor-pointer select-none',
          isExempt && 'bg-zinc-800/95 text-amber-300 border-amber-500/40',
          className
        )}
      >
        {renderPillIcon(isExempt)}
        <span>{label}</span>
      </button>
    </PortalTooltip>
  );
}

export default AccountWarmupOptOut;
