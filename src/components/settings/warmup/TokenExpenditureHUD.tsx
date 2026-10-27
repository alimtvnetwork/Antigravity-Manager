import React, { useEffect } from 'react';
import { motion, useSpring, useTransform } from 'framer-motion';
import { Zap, Coins, Activity, RotateCcw, ShieldCheck } from 'lucide-react';
import { useWarmupStore } from '../../../stores/useWarmupStore';

interface AnimatedCounterProps {
  value: number;
  className?: string;
}

/**
 * Animated micro-counter using Framer Motion spring transition (stiffness: 100, damping: 15).
 */
export const AnimatedCounter: React.FC<AnimatedCounterProps> = ({ value, className = '' }) => {
  const spring = useSpring(value, { stiffness: 100, damping: 15 });
  const display = useTransform(spring, (current) => Math.round(current).toLocaleString());

  useEffect(() => {
    spring.set(value);
  }, [spring, value]);

  return <motion.span className={className}>{display}</motion.span>;
};

/**
 * Improvement 01: Live Token Expenditure HUD & Micro-Counter
 * Renders dark-glass card, animated token counters, and telemetry metric pills.
 */
export const TokenExpenditureHUD: React.FC = () => {
  const telemetry = useWarmupStore((state) => state.telemetry);
  const resetSessionTokens = useWarmupStore((state) => state.resetSessionTokens);

  const formattedCost =
    telemetry.estimatedCostUsd > 0
      ? `$${telemetry.estimatedCostUsd.toFixed(6)} USD`
      : '$0.000000 USD';

  return (
    <div className="bg-zinc-900/70 backdrop-blur-md border border-zinc-800/80 rounded-2xl p-4 shadow-xl text-zinc-100 transition-all duration-300">
      {/* Top Header Row */}
      <div className="flex items-center justify-between pb-3 border-b border-zinc-800/60">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
            <Coins size={16} />
          </div>
          <div>
            <div className="text-xs font-bold text-zinc-200 tracking-wide flex items-center gap-2">
              Token Expenditure HUD
              <span className="bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[10px] px-2 py-0.5 rounded-full font-semibold flex items-center gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                LIVE TELEMETRY
              </span>
            </div>
            <p className="text-[10px] text-zinc-400 mt-0.5">
              Micro-probes strictly consume ~1 token per renewal ping to start 7-day clocks.
            </p>
          </div>
        </div>

        <button
          type="button"
          onClick={resetSessionTokens}
          title="Reset session token counter"
          className="flex items-center gap-1 px-2.5 py-1 rounded-full bg-zinc-800/60 hover:bg-zinc-800 border border-zinc-700/60 text-zinc-400 hover:text-zinc-200 text-[10px] transition-colors"
        >
          <RotateCcw size={11} />
          <span>Reset Session</span>
        </button>
      </div>

      {/* Main Dual Counter Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 my-3.5">
        {/* Session Expenditure */}
        <div className="bg-zinc-950/40 border border-zinc-800/50 rounded-xl p-3 flex flex-col justify-between">
          <div className="flex items-center justify-between text-[11px] text-zinc-400 font-medium">
            <span className="flex items-center gap-1.5">
              <Zap size={13} className="text-amber-400" />
              Session Tokens Consumed
            </span>
            <span className="text-[10px] font-mono text-zinc-500">Current Session</span>
          </div>
          <div className="mt-1 font-mono text-2xl font-black tracking-tight text-emerald-400 dark:text-emerald-300 drop-shadow-[0_0_8px_rgba(52,211,153,0.3)]">
            <AnimatedCounter value={telemetry.sessionTokensConsumed} />
            <span className="text-xs font-normal text-zinc-500 ml-1.5">tokens</span>
          </div>
        </div>

        {/* Total Lifetime Expenditure */}
        <div className="bg-zinc-950/40 border border-zinc-800/50 rounded-xl p-3 flex flex-col justify-between">
          <div className="flex items-center justify-between text-[11px] text-zinc-400 font-medium">
            <span className="flex items-center gap-1.5">
              <ShieldCheck size={13} className="text-emerald-400" />
              Total Tokens Consumed
            </span>
            <span className="text-[10px] font-mono text-zinc-500">Lifetime Total</span>
          </div>
          <div className="mt-1 font-mono text-2xl font-black tracking-tight text-emerald-400 dark:text-emerald-300 drop-shadow-[0_0_8px_rgba(52,211,153,0.3)]">
            <AnimatedCounter value={telemetry.totalTokensConsumed} />
            <span className="text-xs font-normal text-zinc-500 ml-1.5">tokens</span>
          </div>
        </div>
      </div>

      {/* Segmented Metric Pills Footer */}
      <div className="flex flex-wrap items-center gap-2 pt-2 border-t border-zinc-800/50 text-[10px] font-mono">
        {/* Probes Count */}
        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-800/60 border border-zinc-700/50 text-zinc-300">
          <Activity size={11} className="text-orange-400" />
          <span>Probes:</span>
          <strong className="text-zinc-100">{telemetry.totalProbesExecuted}</strong>
        </div>

        {/* Avg Tokens */}
        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-800/60 border border-zinc-700/50 text-zinc-300">
          <span>Avg:</span>
          <strong className="text-zinc-100">{telemetry.avgTokensPerProbe.toFixed(2)} tok/probe</strong>
        </div>

        {/* Estimated Cost USD */}
        <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-800/60 border border-zinc-700/50 text-zinc-300">
          <span>Est. Cost:</span>
          <strong className="text-emerald-400">{formattedCost}</strong>
          <span className="text-zinc-500">(@ $0.00000015/tok)</span>
        </div>
      </div>
    </div>
  );
};

export default TokenExpenditureHUD;
