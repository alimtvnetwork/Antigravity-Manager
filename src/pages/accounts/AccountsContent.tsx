import type { useAccountsState } from "./useAccountsState";
import type { useAccountsHandlers } from "./useAccountsHandlers";
import type { Account } from "../../types/account";
import AccountGrid from "../../components/accounts/AccountGrid";
import AccountTable from "../../components/accounts/AccountTable";

type AccountsState = ReturnType<typeof useAccountsState>;
type AccountsHandlers = ReturnType<typeof useAccountsHandlers>;

interface AccountsContentProps {
  state: AccountsState;
  handlers: AccountsHandlers;
}

export function AccountsContent({ state, handlers }: AccountsContentProps) {
  const {
    showAllQuotas, activeFocusTargetId, activeFocusTargetAccount,
    viewMode, selectedIds, refreshingIds, setErrorAccountId, focusedAccountId,
    paginatedAccounts, switchingAccountId, containerRef, currentAccount, accounts,
    reorderAccounts, updateAccountPriority,
  } = state;
  const {
    handleWarmup, handleUpdateLabel, handleToggleSelect, handleToggleAll,
    handleSwitch, handleRefresh, handleDelete,
    handleToggleProxy, handleExportOne, handleViewDetails, handleViewDevice
  } = handlers;
  return (
    <div className="flex-1 min-h-0 relative" ref={containerRef}>
      {viewMode === "list" ? (
        <div className="h-full bg-white dark:bg-base-100 rounded-2xl shadow-sm border border-gray-100 dark:border-base-200 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto [&_th:nth-child(4)]:w-1/2 [&_th:nth-child(5)]:w-1/2 [&_th:nth-child(4)]:min-w-[220px] [&_th:nth-child(5)]:min-w-[220px] [&_td:nth-child(4)]:min-w-[220px] [&_td:nth-child(5)]:min-w-[220px]">
            <AccountTable
              accounts={paginatedAccounts}
              selectedIds={selectedIds}
              refreshingIds={refreshingIds}
              onToggleSelect={handleToggleSelect}
              onToggleAll={handleToggleAll}
              currentAccountId={activeFocusTargetId || currentAccount?.id || null}
              currentAccountEmail={activeFocusTargetAccount?.email || currentAccount?.email || null}
              switchingAccountId={switchingAccountId}
              onSwitch={handleSwitch}
              onRefresh={handleRefresh}
              onViewDevice={handleViewDevice}
              onViewDetails={handleViewDetails}
              onExport={handleExportOne}
              onDelete={handleDelete}
              onToggleProxy={(id) =>
                handleToggleProxy(
                  id,
                  !!accounts.find((a: Account) => a.id === id)?.proxy_disabled,
                )
              }
              onReorder={reorderAccounts}
              onWarmup={handleWarmup}
              onUpdateLabel={handleUpdateLabel}
              onUpdatePriority={updateAccountPriority}
              onViewError={(id: string) => setErrorAccountId(id)}
              focusedAccountId={focusedAccountId}
              showAllEmails={showAllQuotas}
            />
          </div>
        </div>
      ) : (
        <div className="h-full overflow-y-auto">
          <AccountGrid
            accounts={paginatedAccounts}
            selectedIds={selectedIds}
            refreshingIds={refreshingIds}
            onToggleSelect={handleToggleSelect}
            currentAccountId={activeFocusTargetId || currentAccount?.id || null}
            currentAccountEmail={activeFocusTargetAccount?.email || currentAccount?.email || null}
            switchingAccountId={switchingAccountId}
            focusedAccountId={focusedAccountId}
            onSwitch={handleSwitch}
            onRefresh={handleRefresh}
            onViewDevice={handleViewDevice}
            onViewDetails={handleViewDetails}
            onExport={handleExportOne}
            onDelete={handleDelete}
            onToggleProxy={(id) =>
              handleToggleProxy(
                id,
                !!accounts.find((a: Account) => a.id === id)?.proxy_disabled,
              )
            }
            onWarmup={handleWarmup}
            onUpdateLabel={handleUpdateLabel}
            onUpdatePriority={updateAccountPriority}
            onViewError={(id: string) => setErrorAccountId(id)}
          />
        </div>
      )}
    </div>
  );
}
