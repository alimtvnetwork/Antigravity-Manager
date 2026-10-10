import type { useAccountsState } from "./useAccountsState";
import type { useAccountsHandlers } from "./useAccountsHandlers";
import AccountDetailsDialog from "../../components/accounts/AccountDetailsDialog";
import DeviceFingerprintDialog from "../../components/accounts/DeviceFingerprintDialog";
import ModalDialog from "../../components/common/ModalDialog";
import AccountErrorDialog from "../../components/accounts/AccountErrorDialog";
import { UnifiedBackupModal } from "../../components/modals/UnifiedBackupModal";

type AccountsState = ReturnType<typeof useAccountsState>;
type AccountsHandlers = ReturnType<typeof useAccountsHandlers>;

interface AccountsDialogsProps {
  state: AccountsState;
  handlers: AccountsHandlers;
}

export function AccountsDialogs({ state, handlers }: AccountsDialogsProps) {
  const {
    detailsAccount, setDetailsAccount, errorAccountId, setErrorAccountId, isBackupModalOpen, setIsBackupModalOpen,
    backupModalTab
  } = state;
  const {
    handleWarmup, handleUpdateLabel, handleWarmupAll, handlePageChange, handleToggleSelect, handleToggleAll,
    handleAddAccount, handleSwitch, handleRefresh, handleBatchDelete, executeBatchDelete, handleDelete,
    executeDelete, handleToggleProxy, executeToggleProxy, handleBatchToggleProxy, handleRefreshClick, executeRefresh,
    handleFocusActiveAccount, exportAccountsToJson, handleExportOne, handleViewDetails, handleViewDevice
  } = handlers;
  return (
    <>
    <AccountDetailsDialog
      account={detailsAccount}
      onClose={() => setDetailsAccount(null)}
      onUpdatePriority={updateAccountPriority}
    />

    {/* Account error dialog */}
    <AccountErrorDialog
      account={accounts.find(a => a.id === errorAccountId) || null}
      onClose={() => setErrorAccountId(null)}
    />

    {/* Unified Encrypted Backup Modal */}
    <UnifiedBackupModal
      isOpen={isBackupModalOpen}
      onClose={() => setIsBackupModalOpen(false)}
      initialTab={backupModalTab}
    />
    </>
  );
}
