import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { AlertTriangle, RefreshCw, X, ShieldAlert, KeyRound, SlidersHorizontal, Server } from 'lucide-react';
import { useErrorStore, CapturedError } from '../../../stores/error-store';
import { useAccountStore } from '../../../stores/useAccountStore';
import { request as invoke } from '../../../utils/request';
import { cn } from '../../../utils/cn';

export type DiagnosticErrorType = 'oauth_expired' | 'rate_limited' | 'proxy_down' | 'generic';

export interface WarmupDiagnosticBannerProps {
  error?: CapturedError | { message: string; code?: string; id?: string } | null;
  onFixApplied?: (type: DiagnosticErrorType) => void;
  onDismiss?: () => void;
  className?: string;
}

export const WarmupDiagnosticBanner: React.FC<WarmupDiagnosticBannerProps> = ({
  error: explicitError,
  onFixApplied,
  onDismiss,
  className,
}) => {
  const { t } = useTranslation();
  const recentErrors = useErrorStore((state) => state.recentErrors);
  const removeError = useErrorStore((state) => state.removeError);
  const startOAuthLogin = useAccountStore((state) => state.startOAuthLogin);

  const [isApplyingFix, setIsApplyingFix] = useState(false);
  const [isDismissed, setIsDismissed] = useState(false);

  // Detect relevant warmup errors from recentErrors if no explicit error provided
  const targetError = React.useMemo(() => {
    if (explicitError) return explicitError;

    // Scan recentErrors for warmup-related signatures
    const matched = recentErrors.find((err) => {
      const msg = (err.message || '').toLowerCase();
      const code = (err.code || '').toLowerCase();
      const source = (err.context?.source || '').toLowerCase();

      return (
        source.includes('warmup') ||
        msg.includes('warmup') ||
        msg.includes('403') ||
        msg.includes('oauth') ||
        msg.includes('429') ||
        msg.includes('rate limit') ||
        msg.includes('8045') ||
        msg.includes('connection refused') ||
        code.includes('403') ||
        code.includes('429')
      );
    });

    return matched || null;
  }, [explicitError, recentErrors]);

  if (!targetError || isDismissed) {
    return null;
  }

  const rawMessage = targetError.message || '';
  const lowerMsg = rawMessage.toLowerCase();

  // Classify failure pattern
  let errorType: DiagnosticErrorType = 'generic';
  let title = t('settings.warmup.diagnostic_generic_title', 'Warmup Probe Anomaly Detected');
  let description = rawMessage;
  let fixButtonText = t('settings.warmup.fix_generic', 'Troubleshoot');
  let FixIcon = ShieldAlert;

  if (lowerMsg.includes('403') || lowerMsg.includes('oauth') || lowerMsg.includes('expired') || lowerMsg.includes('revoked')) {
    errorType = 'oauth_expired';
    title = t('settings.warmup.diagnostic_403_title', 'OAuth Token Revoked or Expired (403)');
    description = t(
      'settings.warmup.diagnostic_403_desc',
      'Account credentials expired or authorization revoked. Warmup probes cannot refresh quota reset clocks.'
    );
    fixButtonText = t('settings.warmup.fix_403_btn', 'Re-authenticate Account');
    FixIcon = KeyRound;
  } else if (lowerMsg.includes('429') || lowerMsg.includes('rate limit') || lowerMsg.includes('resource_exhausted')) {
    errorType = 'rate_limited';
    title = t('settings.warmup.diagnostic_429_title', 'Upstream Rate Limit Exceeded (429)');
    description = t(
      'settings.warmup.diagnostic_429_desc',
      'Probe cadence too aggressive for upstream Google Gemini quota. Introduce backoff jitter.'
    );
    fixButtonText = t('settings.warmup.fix_429_btn', 'Increase Jitter & Cooldown');
    FixIcon = SlidersHorizontal;
  } else if (lowerMsg.includes('8045') || lowerMsg.includes('connection refused') || lowerMsg.includes('proxy')) {
    errorType = 'proxy_down';
    title = t('settings.warmup.diagnostic_8045_title', 'Local AI Proxy Not Running (Port 8045)');
    description = t(
      'settings.warmup.diagnostic_8045_desc',
      'The internal reverse proxy service is unreachable. Requests cannot be mapped to upstream protocols.'
    );
    fixButtonText = t('settings.warmup.fix_8045_btn', 'Restart Internal Proxy');
    FixIcon = Server;
  }

  const handleApplyFix = async () => {
    setIsApplyingFix(true);
    try {
      switch (errorType) {
        case 'oauth_expired':
          await startOAuthLogin();
          break;

        case 'rate_limited':
          // Attempt to increase jitter or clear rate limits via invoke
          try {
            await invoke('clear_all_proxy_rate_limits');
          } catch (e) {
            useErrorStore.getState().trackWarning(e, {
              source: 'WarmupDiagnosticBanner.clear_all_proxy_rate_limits',
              triggerAction: 'clear_all_proxy_rate_limits',
            });
          }
          break;

        case 'proxy_down':
          try {
            await invoke('start_proxy_service');
            await invoke('check_proxy_health');
          } catch (e) {
            useErrorStore.getState().trackWarning(e, {
              source: 'WarmupDiagnosticBanner.restart_proxy',
              triggerAction: 'start_proxy_service',
            });
          }
          break;

        case 'generic':
        default:
          try {
            await invoke('check_proxy_health');
          } catch (e) {
            useErrorStore.getState().trackWarning(e, {
              source: 'WarmupDiagnosticBanner.check_health',
              triggerAction: 'check_proxy_health',
            });
          }
          break;
      }

      if (targetError.id) {
        removeError(targetError.id);
      }
      setIsDismissed(true);
      if (onFixApplied) onFixApplied(errorType);
    } catch (err) {
      useErrorStore.getState().trackWarning(err, {
        source: 'WarmupDiagnosticBanner.handleApplyFix',
        triggerAction: `apply_fix_${errorType}`,
      });
    } finally {
      setIsApplyingFix(false);
    }
  };

  const handleDismiss = () => {
    setIsDismissed(true);
    if (targetError.id) {
      removeError(targetError.id);
    }
    if (onDismiss) onDismiss();
  };

  return (
    <div
      className={cn(
        'bg-red-950/30 border border-red-500/30 rounded-xl p-3.5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 text-xs text-red-200 shadow-lg transition-all duration-300',
        className
      )}
      role="alert"
    >
      <div className="flex items-start gap-3">
        <div className="p-1.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 shrink-0 mt-0.5">
          <AlertTriangle className="w-4 h-4" />
        </div>

        <div className="space-y-0.5">
          <div className="font-semibold text-red-300 tracking-wide flex items-center gap-2">
            <span>{title}</span>
          </div>
          <p className="text-[11px] text-zinc-300 leading-relaxed max-w-xl">
            {description}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-2 self-end sm:self-center shrink-0">
        <button
          type="button"
          onClick={handleApplyFix}
          disabled={isApplyingFix}
          className={cn(
            'inline-flex items-center gap-1.5 bg-red-500/20 hover:bg-red-500/30 text-red-300 border border-red-500/40 px-3 py-1 rounded-full font-medium transition-colors focus:outline-none focus:ring-1 focus:ring-red-400',
            isApplyingFix && 'opacity-60 cursor-not-allowed'
          )}
        >
          {isApplyingFix ? (
            <RefreshCw className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <FixIcon className="w-3.5 h-3.5" />
          )}
          <span>{fixButtonText}</span>
        </button>

        <button
          type="button"
          onClick={handleDismiss}
          className="p-1 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60 rounded-full transition-colors focus:outline-none"
          title={t('common.dismiss', 'Dismiss')}
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
};

export default WarmupDiagnosticBanner;
