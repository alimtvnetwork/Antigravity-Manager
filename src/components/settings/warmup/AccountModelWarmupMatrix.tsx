import React, { useState, useMemo, useCallback } from 'react';
import { Clock, Search, SlidersHorizontal, AlertCircle } from 'lucide-react';
import { useAccountStore } from '../../../stores/useAccountStore';
import { Account } from '../../../types/account';
import { useErrorStore } from '../../../stores/error-store';
import { cn } from '../../../utils/cn';

export interface WarmupModelDef {
  id: string;
  label: string;
  shortLabel: string;
  family: 'gemini' | 'claude';
  tagColor: string;
}

export const MATRIX_WARMUP_MODELS: WarmupModelDef[] = [
  {
    id: 'gemini-2.5-pro',
    label: 'Gemini 2.5 Pro',
    shortLabel: '2.5 Pro',
    family: 'gemini',
    tagColor: 'text-sky-400 bg-sky-500/10 border-sky-500/20',
  },
  {
    id: 'gemini-2.5-flash',
    label: 'Gemini 2.5 Flash',
    shortLabel: '2.5 Flash',
    family: 'gemini',
    tagColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20',
  },
  {
    id: 'claude-3-7-sonnet',
    label: 'Claude 3.7 Sonnet',
    shortLabel: '3.7 Sonnet',
    family: 'claude',
    tagColor: 'text-purple-400 bg-purple-500/10 border-purple-500/20',
  },
  {
    id: 'gemini-2.0-flash',
    label: 'Gemini 2.0 Flash',
    shortLabel: '2.0 Flash',
    family: 'gemini',
    tagColor: 'text-blue-400 bg-blue-500/10 border-blue-500/20',
  },
];

export interface AccountModelWarmupMatrixProps {
  overrides?: Record<string, string[]>;
  onToggleOverride?: (accountId: string, modelId: string) => void;
  className?: string;
}

interface CooldownInfo {
  isCooldown: boolean;
  remainingHours: number;
  reason?: string;
}

/**
 * Calculates remaining rate limit cooldown hours for a given account and model.
 */
function getCooldownInfo(account: Account, modelId: string): CooldownInfo {
  const limit = account.live_limited_models?.[modelId];
  if (!limit || limit.until <= Date.now()) {
    return { isCooldown: false, remainingHours: 0 };
  }
  const remainingHours = Math.max(1, Math.ceil((limit.until - Date.now()) / (1000 * 60 * 60)));
  return {
    isCooldown: true,
    remainingHours,
    reason: limit.reason || 'Rate limit cooldown active',
  };
}

interface MatrixHeaderCellProps {
  model: WarmupModelDef;
}

/**
 * Header cell rendering model identity and family badge.
 */
const MatrixHeaderCell: React.FC<MatrixHeaderCellProps> = ({ model }) => {
  return (
    <th className="bg-zinc-900/90 p-2.5 font-semibold text-zinc-200 border-b border-zinc-800 text-center select-none">
      <div className="flex flex-col items-center gap-0.5">
        <span className="text-xs">{model.shortLabel}</span>
        <span className={cn('text-[10px] px-1.5 py-0.2 rounded border font-mono', model.tagColor)}>
          {model.family}
        </span>
      </div>
    </th>
  );
};

interface MatrixCheckboxCellProps {
  accountId: string;
  modelId: string;
  isIncluded: boolean;
  cooldown: CooldownInfo;
  onToggle: (accountId: string, modelId: string) => void;
}

/**
 * Interactive cell rendering tri-state warmup toggle and active cooldown indicator.
 */
const MatrixCheckboxCell: React.FC<MatrixCheckboxCellProps> = ({
  accountId,
  modelId,
  isIncluded,
  cooldown,
  onToggle,
}) => {
  const handleToggle = () => onToggle(accountId, modelId);

  return (
    <td
      onClick={handleToggle}
      className={cn(
        'p-2.5 text-center border-b border-zinc-800/60 cursor-pointer transition-colors select-none',
        isIncluded ? 'bg-orange-500/10 hover:bg-orange-500/20' : 'hover:bg-zinc-800/40',
        cooldown.isCooldown && 'bg-zinc-900/40'
      )}
    >
      <div className="flex flex-col items-center justify-center gap-1">
        <input
          type="checkbox"
          checked={isIncluded}
          onChange={handleToggle}
          onClick={(e) => e.stopPropagation()}
          className="h-3.5 w-3.5 rounded border-zinc-700 bg-zinc-900 text-orange-500 focus:ring-0 focus:ring-offset-0 cursor-pointer accent-orange-500"
        />
        {cooldown.isCooldown && (
          <span
            title={cooldown.reason}
            className="inline-flex items-center gap-0.5 px-1.5 py-0.5 text-[9px] font-mono bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded"
          >
            <Clock className="w-2.5 h-2.5" />
            {cooldown.remainingHours}h cd
          </span>
        )}
      </div>
    </td>
  );
};

interface AccountRowProps {
  account: Account;
  models: WarmupModelDef[];
  activeModels: string[];
  onToggle: (accountId: string, modelId: string) => void;
  onToggleAll: (accountId: string, isSelectAll: boolean) => void;
}

/**
 * Single account row across all model columns.
 */
const AccountRow: React.FC<AccountRowProps> = ({
  account,
  models,
  activeModels,
  onToggle,
  onToggleAll,
}) => {
  const isAllSelected = models.every((m) => activeModels.includes(m.id));
  const isAccountActive = !account.disabled && !account.proxy_disabled;

  return (
    <tr className="hover:bg-zinc-800/20 transition-colors">
      <td className="p-2.5 border-b border-zinc-800/60 text-left">
        <div className="flex items-center gap-2">
          <div className="flex flex-col min-w-0 max-w-[200px]">
            <span className="text-xs font-medium text-zinc-200 truncate">
              {account.custom_label || account.email}
            </span>
            {account.custom_label && (
              <span className="text-[10px] text-zinc-500 truncate">{account.email}</span>
            )}
          </div>
          {!isAccountActive && (
            <span className="text-[9px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 shrink-0">
              Disabled
            </span>
          )}
        </div>
      </td>
      {models.map((model) => (
        <MatrixCheckboxCell
          key={model.id}
          accountId={account.id}
          modelId={model.id}
          isIncluded={activeModels.includes(model.id)}
          cooldown={getCooldownInfo(account, model.id)}
          onToggle={onToggle}
        />
      ))}
      <td className="p-2.5 border-b border-zinc-800/60 text-center">
        <button
          type="button"
          onClick={() => onToggleAll(account.id, !isAllSelected)}
          className="text-[10px] px-2 py-0.5 rounded-full bg-zinc-800 hover:bg-zinc-700 text-zinc-300 transition-colors"
        >
          {isAllSelected ? 'Clear' : 'All'}
        </button>
      </td>
    </tr>
  );
};

/**
 * Interactive Account × Model Warmup Matrix Grid.
 * Allows operators to configure model warmup inclusions per individual account.
 */
export const AccountModelWarmupMatrix: React.FC<AccountModelWarmupMatrixProps> = ({
  overrides: propOverrides,
  onToggleOverride,
  className,
}) => {
  const accounts = useAccountStore((state) => state.accounts);
  const [internalOverrides, setInternalOverrides] = useState<Record<string, string[]>>({});
  const [searchQuery, setSearchQuery] = useState('');

  const overrides = propOverrides ?? internalOverrides;

  const handleToggle = useCallback(
    (accountId: string, modelId: string) => {
      try {
        if (onToggleOverride) {
          onToggleOverride(accountId, modelId);
          return;
        }
        setInternalOverrides((prev) => {
          const current = prev[accountId] ?? MATRIX_WARMUP_MODELS.map((m) => m.id);
          const hasModel = current.includes(modelId);
          const updated = hasModel
            ? current.filter((m) => m !== modelId)
            : [...current, modelId];
          return { ...prev, [accountId]: updated };
        });
      } catch (err) {
        useErrorStore.getState().trackWarning(err, {
          source: 'AccountModelWarmupMatrix',
          triggerAction: 'toggle_override',
        });
      }
    },
    [onToggleOverride]
  );

  const handleToggleAll = useCallback(
    (accountId: string, isSelectAll: boolean) => {
      try {
        const nextModels = isSelectAll ? MATRIX_WARMUP_MODELS.map((m) => m.id) : [];
        if (onToggleOverride) {
          MATRIX_WARMUP_MODELS.forEach((m) => {
            const current = overrides[accountId] ?? MATRIX_WARMUP_MODELS.map((x) => x.id);
            const isCurrentlySelected = current.includes(m.id);
            if (isCurrentlySelected !== isSelectAll) {
              onToggleOverride(accountId, m.id);
            }
          });
          return;
        }
        setInternalOverrides((prev) => ({ ...prev, [accountId]: nextModels }));
      } catch (err) {
        useErrorStore.getState().trackWarning(err, {
          source: 'AccountModelWarmupMatrix',
          triggerAction: 'toggle_all_override',
        });
      }
    },
    [onToggleOverride, overrides]
  );

  const filteredAccounts = useMemo(() => {
    const trimmed = searchQuery.trim().toLowerCase();
    if (!trimmed) {
      return accounts;
    }
    return accounts.filter(
      (a) =>
        a.email.toLowerCase().includes(trimmed) ||
        (a.custom_label && a.custom_label.toLowerCase().includes(trimmed))
    );
  }, [accounts, searchQuery]);

  return (
    <div
      className={cn(
        'border border-zinc-800 rounded-xl bg-zinc-950/70 backdrop-blur-md p-4 flex flex-col gap-3',
        className
      )}
    >
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <SlidersHorizontal className="w-4 h-4 text-orange-400" />
          <h4 className="text-xs font-semibold text-zinc-100 uppercase tracking-wider">
            Account × Model Warmup Matrix
          </h4>
          <span className="text-[10px] text-zinc-500 font-mono">
            ({accounts.length} Accounts)
          </span>
        </div>
        <div className="relative">
          <Search className="w-3.5 h-3.5 text-zinc-500 absolute left-2.5 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            placeholder="Filter accounts..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-44 pl-8 pr-2.5 py-1 text-xs bg-zinc-900/90 border border-zinc-800 rounded-lg text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-orange-500/60"
          />
        </div>
      </div>

      <div className="overflow-x-auto rounded-xl border border-zinc-800">
        <table className="w-full border-collapse text-xs text-zinc-300">
          <thead>
            <tr>
              <th className="bg-zinc-900/90 p-2.5 font-semibold text-zinc-200 border-b border-zinc-800 text-left">
                Account
              </th>
              {MATRIX_WARMUP_MODELS.map((model) => (
                <MatrixHeaderCell key={model.id} model={model} />
              ))}
              <th className="bg-zinc-900/90 p-2.5 font-semibold text-zinc-200 border-b border-zinc-800 text-center w-20">
                Action
              </th>
            </tr>
          </thead>
          <tbody>
            {filteredAccounts.length === 0 ? (
              <tr>
                <td
                  colSpan={MATRIX_WARMUP_MODELS.length + 2}
                  className="p-6 text-center text-zinc-500 text-xs"
                >
                  <div className="flex flex-col items-center justify-center gap-1.5">
                    <AlertCircle className="w-4 h-4 text-zinc-600" />
                    <span>No accounts found matching filter</span>
                  </div>
                </td>
              </tr>
            ) : (
              filteredAccounts.map((account) => {
                const activeModels =
                  overrides[account.id] ?? MATRIX_WARMUP_MODELS.map((m) => m.id);
                return (
                  <AccountRow
                    key={account.id}
                    account={account}
                    models={MATRIX_WARMUP_MODELS}
                    activeModels={activeModels}
                    onToggle={handleToggle}
                    onToggleAll={handleToggleAll}
                  />
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
