import React from 'react';
import { useTranslation } from 'react-i18next';
import { ShieldCheck, Zap, Lock } from 'lucide-react';
import { cn } from '../../../utils/cn';

export interface TokenShieldCalloutProps {
  safetyCeiling?: number;
  isEnforced?: boolean;
  className?: string;
}

export const TokenShieldCallout: React.FC<TokenShieldCalloutProps> = ({
  safetyCeiling = 5,
  isEnforced = true,
  className,
}) => {
  const { t } = useTranslation();

  return (
    <div
      className={cn(
        'rounded-xl p-3.5 bg-gradient-to-r from-emerald-950/20 via-zinc-900/40 to-emerald-950/10 border border-emerald-500/20 flex items-start gap-3 shadow-sm',
        className
      )}
    >
      <div className="p-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 shrink-0 mt-0.5">
        <ShieldCheck className="w-5 h-5" />
      </div>

      <div className="flex-1 space-y-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <span className="text-xs font-semibold text-emerald-300 tracking-wide">
              {t('settings.warmup.token_shield_title', 'Token Shield: Active (Zero-Cost Probing Guaranteed)')}
            </span>
          </div>

          <div className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 font-mono text-[10px] font-medium tracking-wide">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
            <span>{isEnforced ? t('settings.warmup.shield_enforced', 'SHIELD: ENFORCED') : t('settings.warmup.shield_bypassed', 'SHIELD: BYPASSED')}</span>
          </div>
        </div>

        <p className="text-[11px] text-zinc-400 leading-relaxed">
          {t(
            'settings.warmup.token_shield_desc',
            'Probes send max_tokens=1 or empty messages that validate project authorization and trip quota reset timestamps without generating generative completions.'
          )}
        </p>

        <div className="flex flex-wrap items-center gap-2 pt-1 border-t border-emerald-500/10 text-[10px] text-zinc-400 font-mono">
          <div className="flex items-center gap-1 px-2 py-0.5 rounded bg-zinc-900/60 border border-zinc-800 text-zinc-300">
            <Lock className="w-3 h-3 text-emerald-400" />
            <span>
              {t('settings.warmup.safety_ceiling', 'Safety Ceiling')}:{' '}
              <strong className="text-emerald-300 font-semibold">{safetyCeiling} tok/acct/week</strong>
            </span>
          </div>

          <div className="flex items-center gap-1 px-2 py-0.5 rounded bg-zinc-900/60 border border-zinc-800 text-zinc-300">
            <Zap className="w-3 h-3 text-amber-400" />
            <span>
              {t('settings.warmup.probe_burn', 'Total Burn')}:{' '}
              <strong className="text-emerald-300 font-semibold">0.00%</strong>
            </span>
          </div>

          <span className="text-[10px] text-zinc-500 italic">
            {t('settings.warmup.safety_circuit', 'Auto-aborts probe if token consumption > 5')}
          </span>
        </div>
      </div>
    </div>
  );
};

export default TokenShieldCallout;
