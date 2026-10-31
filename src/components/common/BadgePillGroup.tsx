import React, { useState } from 'react';

export interface BadgeItem {
  id: string;
  label: string;
  variant?: 'slate' | 'sky' | 'emerald' | 'amber' | 'purple';
}

interface BadgePillGroupProps {
  badges: BadgeItem[];
  maxVisible?: number;
  className?: string;
}

const variantStyles: Record<string, string> = {
  slate: 'bg-slate-800/80 text-slate-300 border-slate-700/60',
  sky: 'bg-sky-500/15 text-sky-300 border-sky-400/30',
  emerald: 'bg-emerald-500/15 text-emerald-300 border-emerald-400/30',
  amber: 'bg-amber-500/15 text-amber-300 border-amber-400/30',
  purple: 'bg-purple-500/15 text-purple-300 border-purple-400/30',
};

export const BadgePillGroup: React.FC<BadgePillGroupProps> = ({
  badges,
  maxVisible = 3,
  className = '',
}) => {
  const [expanded, setExpanded] = useState(false);

  if (!badges || badges.length === 0) return null;

  const visibleBadges = expanded ? badges : badges.slice(0, maxVisible);
  const overflowCount = badges.length - maxVisible;

  return (
    <div className={`inline-flex flex-wrap items-center gap-1.5 ${className}`}>
      {visibleBadges.map((badge) => {
        const style = variantStyles[badge.variant || 'slate'];
        return (
          <span
            key={badge.id}
            className={`inline-flex items-center px-2 py-0.5 rounded-full text-[11px] font-medium border backdrop-blur-sm shadow-xs ${style}`}
          >
            {badge.label}
          </span>
        );
      })}

      {!expanded && overflowCount > 0 && (
        <button
          type="button"
          onClick={() => setExpanded(true)}
          className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-semibold bg-slate-800/90 text-slate-400 hover:text-slate-200 border border-slate-700/60 transition-colors"
        >
          +{overflowCount} more
        </button>
      )}

      {expanded && overflowCount > 0 && (
        <button
          type="button"
          onClick={() => setExpanded(false)}
          className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[10px] font-semibold bg-slate-800/90 text-slate-400 hover:text-slate-200 border border-slate-700/60 transition-colors"
        >
          show less
        </button>
      )}
    </div>
  );
};
