import React, { useState, useMemo } from 'react';
import { Clock, Info, ShieldCheck, Flame, RefreshCw, Sparkles, AlertCircle } from 'lucide-react';
import { Account } from '../../../types/account';
import { useAccountStore } from '../../../stores/useAccountStore';

export interface WeeklyResetConvergenceRadarProps {
  accounts?: Account[];
  onSelectAccount?: (account: Account) => void;
  className?: string;
}

interface RadarNode {
  id: string;
  account: Account;
  email: string;
  dayIndex: number; // 0 = Mon, 6 = Sun
  dayName: string;
  hour: number;
  minute: number;
  theta: number; // radians
  r: number; // radial distance
  x: number;
  y: number;
  quotaFraction: number;
  resetDate: Date;
  diffMinutes: number;
  isImminent: boolean; // within 30 minutes of reset
  isOverdue: boolean;
}

const DAY_LABELS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

// Fallback accounts used if store accounts lack quota reset_time metadata
const FALLBACK_ACCOUNTS: Account[] = [
  {
    id: 'demo-acc-1',
    email: 'dev.primary@gmail.com',
    name: 'Dev Primary',
    created_at: Date.now() - 30 * 86400000,
    last_used: Date.now(),
    token: { access_token: '', refresh_token: '', expires_in: 3600, expiry_timestamp: Date.now() + 3600000, token_type: 'Bearer' },
    quota: {
      last_updated: Date.now(),
      models: [
        {
          name: 'gemini-2.5-flash',
          percentage: 95,
          reset_time: new Date(Date.now() + 18 * 60 * 1000).toISOString(), // 18m away (imminent)
        },
      ],
    },
  },
  {
    id: 'demo-acc-2',
    email: 'ops.worker1@gmail.com',
    name: 'Ops Worker 1',
    created_at: Date.now() - 25 * 86400000,
    last_used: Date.now(),
    token: { access_token: '', refresh_token: '', expires_in: 3600, expiry_timestamp: Date.now() + 3600000, token_type: 'Bearer' },
    quota: {
      last_updated: Date.now(),
      models: [
        {
          name: 'gemini-2.5-flash',
          percentage: 40,
          reset_time: new Date(Date.now() + 2 * 3600 * 1000).toISOString(), // 2h away
        },
      ],
    },
  },
  {
    id: 'demo-acc-3',
    email: 'team.frontend@gmail.com',
    name: 'Frontend Fleet',
    created_at: Date.now() - 20 * 86400000,
    last_used: Date.now(),
    token: { access_token: '', refresh_token: '', expires_in: 3600, expiry_timestamp: Date.now() + 3600000, token_type: 'Bearer' },
    quota: {
      last_updated: Date.now(),
      models: [
        {
          name: 'gemini-2.5-pro',
          percentage: 80,
          reset_time: new Date(Date.now() + 28 * 3600 * 1000).toISOString(), // ~1.1 days away
        },
      ],
    },
  },
  {
    id: 'demo-acc-4',
    email: 'qa.testing@gmail.com',
    name: 'QA Fleet',
    created_at: Date.now() - 15 * 86400000,
    last_used: Date.now(),
    token: { access_token: '', refresh_token: '', expires_in: 3600, expiry_timestamp: Date.now() + 3600000, token_type: 'Bearer' },
    quota: {
      last_updated: Date.now(),
      models: [
        {
          name: 'gemini-2.5-flash',
          percentage: 65,
          reset_time: new Date(Date.now() + 64 * 3600 * 1000).toISOString(), // ~2.6 days away
        },
      ],
    },
  },
  {
    id: 'demo-acc-5',
    email: 'prod.batch@gmail.com',
    name: 'Production Batch',
    created_at: Date.now() - 10 * 86400000,
    last_used: Date.now(),
    token: { access_token: '', refresh_token: '', expires_in: 3600, expiry_timestamp: Date.now() + 3600000, token_type: 'Bearer' },
    quota: {
      last_updated: Date.now(),
      models: [
        {
          name: 'gemini-2.5-flash',
          percentage: 30,
          reset_time: new Date(Date.now() + 110 * 3600 * 1000).toISOString(), // ~4.5 days away
        },
      ],
    },
  },
  {
    id: 'demo-acc-6',
    email: 'backup.reserve@gmail.com',
    name: 'Backup Reserve',
    created_at: Date.now() - 5 * 86400000,
    last_used: Date.now(),
    token: { access_token: '', refresh_token: '', expires_in: 3600, expiry_timestamp: Date.now() + 3600000, token_type: 'Bearer' },
    quota: {
      last_updated: Date.now(),
      models: [
        {
          name: 'gemini-2.5-pro',
          percentage: 90,
          reset_time: new Date(Date.now() + 148 * 3600 * 1000).toISOString(), // ~6.1 days away
        },
      ],
    },
  },
];

export const WeeklyResetConvergenceRadar: React.FC<WeeklyResetConvergenceRadarProps> = ({
  accounts: propAccounts,
  onSelectAccount,
  className = '',
}) => {
  const storeAccounts = useAccountStore((state) => state.accounts);
  const rawAccounts = propAccounts || (storeAccounts.length > 0 ? storeAccounts : FALLBACK_ACCOUNTS);

  const [hoveredNode, setHoveredNode] = useState<RadarNode | null>(null);
  const [selectedNode, setSelectedNode] = useState<RadarNode | null>(null);

  // SVG Geometry Constants
  const size = 520;
  const cx = size / 2;
  const cy = size / 2;
  const maxRadius = 195;
  const minRadius = 45;

  const now = Date.now();
  const nowDate = new Date(now);
  const currentDayOfWeek = (nowDate.getUTCDay() + 6) % 7; // 0 = Mon, 6 = Sun
  const currentHour = nowDate.getUTCHours();
  const currentMinute = nowDate.getUTCMinutes();
  const nowTheta =
    ((currentDayOfWeek * 24 + currentHour + currentMinute / 60) / 168) * 2 * Math.PI - Math.PI / 2;

  // Process Accounts into Radar Nodes
  const nodes = useMemo<RadarNode[]>(() => {
    const list: RadarNode[] = [];

    rawAccounts.forEach((acc, index) => {
      // Find reset time from models or buckets
      let resetIso: string | undefined = undefined;
      let quotaPercent = 85;

      if (acc.quota?.models && acc.quota.models.length > 0) {
        for (const m of acc.quota.models) {
          if (m.reset_time) {
            resetIso = m.reset_time;
            quotaPercent = m.percentage ?? 85;
            break;
          }
        }
      }

      if (!resetIso && acc.quota?.quota_groups) {
        for (const g of acc.quota.quota_groups) {
          if (g.buckets) {
            for (const b of g.buckets) {
              if (b.reset_time) {
                resetIso = b.reset_time;
                quotaPercent = Math.round((b.remaining_fraction ?? 0.85) * 100);
                break;
              }
            }
          }
          if (resetIso) break;
        }
      }

      // If still none, distribute synthetic anchor for visualization
      let resetDate: Date;
      if (resetIso) {
        resetDate = new Date(resetIso);
      } else {
        // synthesize spread over week
        const offsetHours = (index * 26.5) % 168;
        resetDate = new Date(now + offsetHours * 3600 * 1000);
      }

      const d = (resetDate.getUTCDay() + 6) % 7; // 0 = Mon
      const h = resetDate.getUTCHours();
      const m = resetDate.getUTCMinutes();

      // Angle: theta = (DayOfWeek * 24 + Hour) / 168 * 2*pi - pi/2 (to start Monday at 12 o'clock)
      const weeklyHours = d * 24 + h + m / 60;
      const theta = (weeklyHours / 168) * 2 * Math.PI - Math.PI / 2;

      // Radial distance r based on remaining quota fraction
      const quotaFraction = Math.max(0.1, Math.min(1.0, quotaPercent / 100));
      const r = minRadius + quotaFraction * (maxRadius - minRadius);

      const x = cx + r * Math.cos(theta);
      const y = cy + r * Math.sin(theta);

      const diffMs = resetDate.getTime() - now;
      const diffMinutes = Math.round(diffMs / (60 * 1000));
      const isImminent = diffMinutes >= 0 && diffMinutes <= 30;
      const isOverdue = diffMinutes < 0 && diffMinutes > -120;

      list.push({
        id: acc.id || `acc-${index}`,
        account: acc,
        email: acc.email || `account-${index + 1}@proxy.local`,
        dayIndex: d,
        dayName: DAY_LABELS[d],
        hour: h,
        minute: m,
        theta,
        r,
        x,
        y,
        quotaFraction,
        resetDate,
        diffMinutes,
        isImminent,
        isOverdue,
      });
    });

    return list;
  }, [rawAccounts, now, cx, cy, minRadius, maxRadius]);

  const imminentCount = nodes.filter((n) => n.isImminent).length;

  const handleNodeClick = (node: RadarNode) => {
    setSelectedNode(node);
    if (onSelectAccount) {
      onSelectAccount(node.account);
    }
  };

  return (
    <div
      className={`relative rounded-2xl border border-zinc-800/80 bg-zinc-950/80 backdrop-blur-xl p-6 shadow-2xl flex flex-col items-center ${className}`}
    >
      {/* Radar Header */}
      <div className="w-full flex items-center justify-between mb-4 pb-3 border-b border-zinc-800/60">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-orange-500/15 border border-orange-500/30 flex items-center justify-center text-orange-400 shadow-[0_0_12px_rgba(249,115,22,0.2)]">
            <Flame className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-bold text-zinc-100 tracking-tight">
                Weekly Reset Convergence Radar
              </h3>
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-mono font-semibold bg-orange-500/15 text-orange-400 border border-orange-500/30">
                7-Day Polar
              </span>
            </div>
            <p className="text-xs text-zinc-400 mt-0.5">
              Polar coordinate mapping (θ = (Day × 24 + Hour) / 168 × 2π) with imminent reset telemetry
            </p>
          </div>
        </div>

        {/* Status capsules */}
        <div className="flex items-center gap-2">
          {imminentCount > 0 ? (
            <div className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-orange-500/15 border border-orange-500/40 text-orange-400 text-xs font-mono font-bold animate-pulse">
              <span className="w-2 h-2 rounded-full bg-orange-400" />
              <span>{imminentCount} Imminent (&lt;30m)</span>
            </div>
          ) : (
            <div className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 text-xs font-mono">
              <span className="w-2 h-2 rounded-full bg-emerald-400" />
              <span>Fleet Stable</span>
            </div>
          )}

          <div className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-zinc-900/80 border border-zinc-800 text-zinc-400 text-xs font-mono">
            <Clock className="w-3.5 h-3.5 text-zinc-500" />
            <span>
              UTC {DAY_LABELS[currentDayOfWeek]} {String(currentHour).padStart(2, '0')}:
              {String(currentMinute).padStart(2, '0')}
            </span>
          </div>
        </div>
      </div>

      {/* SVG Circular Radar Viewport */}
      <div className="relative flex items-center justify-center">
        <svg
          width={size}
          height={size}
          viewBox={`0 0 ${size} ${size}`}
          className="overflow-visible select-none"
        >
          <defs>
            {/* Radar Center Gradient */}
            <radialGradient id="radarCenterGlow" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stopColor="rgba(249, 115, 22, 0.15)" />
              <stop offset="100%" stopColor="rgba(249, 115, 22, 0.0)" />
            </radialGradient>

            {/* Sweep Ray Gradient */}
            <linearGradient id="sweepGradient" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="rgba(249, 115, 22, 0.4)" />
              <stop offset="100%" stopColor="rgba(249, 115, 22, 0.0)" />
            </linearGradient>

            {/* Node Pulsating Filter */}
            <filter id="blipGlow" x="-50%" y="-50%" width="200%" height="200%">
              <feGaussianBlur in="SourceGraphic" stdDeviation="4" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          {/* Center Glow Area */}
          <circle cx={cx} cy={cy} r={maxRadius} fill="url(#radarCenterGlow)" />

          {/* Concentric Dashed Circles (25%, 50%, 75%, 100%) */}
          {[0.25, 0.5, 0.75, 1.0].map((frac, idx) => {
            const r = minRadius + frac * (maxRadius - minRadius);
            return (
              <g key={`ring-${idx}`}>
                <circle
                  cx={cx}
                  cy={cy}
                  r={r}
                  fill="none"
                  stroke="rgba(113, 113, 122, 0.25)"
                  strokeDasharray="4 4"
                  strokeWidth="1"
                />
                <text
                  x={cx + 6}
                  y={cy - r + 12}
                  fill="rgba(113, 113, 122, 0.4)"
                  fontSize="9"
                  fontFamily="monospace"
                >
                  {Math.round(frac * 100)}%
                </text>
              </g>
            );
          })}

          {/* 7 Radial Spokes (Monday - Sunday) */}
          {DAY_LABELS.map((day, idx) => {
            const spokeAngle = (idx / 7) * 2 * Math.PI - Math.PI / 2;
            const x2 = cx + (maxRadius + 15) * Math.cos(spokeAngle);
            const y2 = cy + (maxRadius + 15) * Math.sin(spokeAngle);

            const labelR = maxRadius + 32;
            const lx = cx + labelR * Math.cos(spokeAngle);
            const ly = cy + labelR * Math.sin(spokeAngle);

            const isToday = idx === currentDayOfWeek;

            return (
              <g key={`spoke-${day}`}>
                <line
                  x1={cx}
                  y1={cy}
                  x2={x2}
                  y2={y2}
                  stroke={isToday ? 'rgba(249, 115, 22, 0.5)' : 'rgba(113, 113, 122, 0.2)'}
                  strokeWidth={isToday ? '1.5' : '1'}
                />
                <text
                  x={lx}
                  y={ly + 4}
                  textAnchor="middle"
                  fill={isToday ? '#fb923c' : '#a1a1aa'}
                  fontSize={isToday ? '12' : '11'}
                  fontWeight={isToday ? 'bold' : '500'}
                  fontFamily="monospace"
                  className="transition-colors"
                >
                  {day}
                </text>
              </g>
            );
          })}

          {/* Current UTC Time Sweep Beam Line */}
          <line
            x1={cx}
            y1={cy}
            x2={cx + maxRadius * Math.cos(nowTheta)}
            y2={cy + maxRadius * Math.sin(nowTheta)}
            stroke="#fb923c"
            strokeWidth="2"
            strokeLinecap="round"
            className="drop-shadow-[0_0_8px_rgba(249,115,22,0.8)]"
          />
          <circle
            cx={cx + maxRadius * Math.cos(nowTheta)}
            cy={cy + maxRadius * Math.sin(nowTheta)}
            r="4"
            fill="#fb923c"
            className="drop-shadow-[0_0_6px_rgba(249,115,22,1)]"
          />

          {/* Radar Center Pivot Hub */}
          <circle cx={cx} cy={cy} r="6" fill="#18181b" stroke="#71717a" strokeWidth="2" />
          <circle cx={cx} cy={cy} r="2.5" fill="#fb923c" />

          {/* Account Reset Nodes */}
          {nodes.map((node) => {
            const isHovered = hoveredNode?.id === node.id;
            const isSelected = selectedNode?.id === node.id;

            if (node.isImminent) {
              return (
                <g
                  key={node.id}
                  className="cursor-pointer"
                  onMouseEnter={() => setHoveredNode(node)}
                  onMouseLeave={() => setHoveredNode(null)}
                  onClick={() => handleNodeClick(node)}
                >
                  {/* Outer pulsating orange blip animation for accounts within 30 minutes of reset */}
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r="14"
                    fill="rgba(249, 115, 22, 0.35)"
                    className="animate-ping"
                  />
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r="8"
                    fill="rgba(249, 115, 22, 0.4)"
                    filter="url(#blipGlow)"
                  />
                  <circle
                    cx={node.x}
                    cy={node.y}
                    r={isSelected ? '6.5' : '5'}
                    fill="#f97316"
                    stroke="#fed7aa"
                    strokeWidth="2"
                    className="drop-shadow-[0_0_10px_rgba(249,115,22,0.9)]"
                  />
                </g>
              );
            }

            return (
              <g
                key={node.id}
                className="cursor-pointer"
                onMouseEnter={() => setHoveredNode(node)}
                onMouseLeave={() => setHoveredNode(null)}
                onClick={() => handleNodeClick(node)}
              >
                {/* Regular Node */}
                <circle
                  cx={node.x}
                  cy={node.y}
                  r={isHovered || isSelected ? '7' : '4.5'}
                  fill={isHovered ? '#38bdf8' : '#10b981'}
                  stroke={isHovered ? '#bae6fd' : '#047857'}
                  strokeWidth={isHovered || isSelected ? '2' : '1.5'}
                  className="transition-all duration-200"
                />
              </g>
            );
          })}
        </svg>

        {/* Floating Tooltip Card */}
        {hoveredNode && (
          <div
            className="absolute z-30 pointer-events-none rounded-xl bg-zinc-900/95 border border-zinc-700/80 p-3 shadow-2xl backdrop-blur-md text-zinc-100 font-mono text-xs w-64 transform -translate-x-1/2 -translate-y-full mb-3"
            style={{
              left: `${(hoveredNode.x / size) * 100}%`,
              top: `${(hoveredNode.y / size) * 100}%`,
            }}
          >
            <div className="flex items-center justify-between pb-1.5 border-b border-zinc-800">
              <span className="font-bold text-zinc-100 truncate pr-2">
                {hoveredNode.account.name || hoveredNode.email.split('@')[0]}
              </span>
              {hoveredNode.isImminent ? (
                <span className="px-1.5 py-0.2 rounded-full text-[9px] font-bold bg-orange-500/20 text-orange-400 border border-orange-500/40 animate-pulse">
                  &lt;30m DUE
                </span>
              ) : (
                <span className="px-1.5 py-0.2 rounded-full text-[9px] bg-emerald-500/20 text-emerald-400 border border-emerald-500/40">
                  {Math.round(hoveredNode.quotaFraction * 100)}% Quota
                </span>
              )}
            </div>

            <div className="mt-2 space-y-1 text-[11px] text-zinc-400">
              <div className="truncate text-zinc-300">
                <span className="text-zinc-500">Email:</span> {hoveredNode.email}
              </div>
              <div>
                <span className="text-zinc-500">Reset Day:</span> {hoveredNode.dayName}{' '}
                {String(hoveredNode.hour).padStart(2, '0')}:
                {String(hoveredNode.minute).padStart(2, '0')} UTC
              </div>
              <div className="flex items-center gap-1">
                <span className="text-zinc-500">Timeline:</span>
                {hoveredNode.diffMinutes > 0 ? (
                  <span className={hoveredNode.isImminent ? 'text-orange-400 font-bold' : 'text-zinc-300'}>
                    In {Math.floor(hoveredNode.diffMinutes / 60)}h {hoveredNode.diffMinutes % 60}m
                  </span>
                ) : (
                  <span className="text-rose-400 font-bold">Elapsed / Resetting</span>
                )}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Selected Account Detail Banner */}
      {selectedNode && (
        <div className="mt-4 w-full p-3 rounded-xl bg-zinc-900/60 border border-zinc-800/80 flex items-center justify-between text-xs font-mono">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-orange-500/15 border border-orange-500/30 flex items-center justify-center text-orange-400">
              <Sparkles className="w-4 h-4" />
            </div>
            <div>
              <div className="text-zinc-200 font-semibold">{selectedNode.email}</div>
              <div className="text-[11px] text-zinc-400">
                Reset: {selectedNode.dayName} at {String(selectedNode.hour).padStart(2, '0')}:
                {String(selectedNode.minute).padStart(2, '0')} UTC &bull;{' '}
                {selectedNode.diffMinutes > 0
                  ? `in ${Math.floor(selectedNode.diffMinutes / 60)}h ${selectedNode.diffMinutes % 60}m`
                  : 'Due now'}
              </div>
            </div>
          </div>
          <button
            onClick={() => setSelectedNode(null)}
            className="text-zinc-500 hover:text-zinc-300 text-[11px]"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Radar Legend Footer */}
      <div className="mt-5 w-full flex items-center justify-between text-[11px] font-mono text-zinc-400 pt-3 border-t border-zinc-800/60">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1.5">
            <span className="w-2.5 h-2.5 rounded-full bg-orange-400 shadow-[0_0_8px_rgba(249,115,22,0.6)] animate-pulse" />
            <span>&lt;30m Imminent Warmup</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-2.5 h-2.5 rounded-full bg-emerald-400" />
            <span>Normal Fleet Quota</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-3 h-0.5 bg-orange-400" />
            <span>Current UTC Ray</span>
          </div>
        </div>

        <div className="text-zinc-500">
          Showing {nodes.length} accounts across 168h cycle
        </div>
      </div>
    </div>
  );
};

export default WeeklyResetConvergenceRadar;
