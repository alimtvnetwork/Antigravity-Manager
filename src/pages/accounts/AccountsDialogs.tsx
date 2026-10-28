import type { useAccountsState } from "./useAccountsState";
import type { useAccountsHandlers } from "./useAccountsHandlers";
import type { Account } from "../../types/account";
import { useAccountStore } from "../../stores/useAccountStore";
import AccountDetailsDialog from "../../components/accounts/AccountDetailsDialog";
import AccountErrorDialog from "../../components/accounts/AccountErrorDialog";
import { UnifiedBackupModal } from "../../components/modals/UnifiedBackupModal";
import ModalDialog from "../../components/common/ModalDialog";
import { WarmupConfirmModal } from "../../components/accounts/WarmupConfirmModal";

type AccountsState = ReturnType<typeof useAccountsState>;
type AccountsHandlers = ReturnType<typeof useAccountsHandlers>;

interface AccountsDialogsProps {
  state: AccountsState;
  handlers: AccountsHandlers;
}

export function AccountsDialogs({ state, handlers }: AccountsDialogsProps) {
  const {
    t,
    detailsAccount, setDetailsAccount, errorAccountId, setErrorAccountId, isBackupModalOpen, setIsBackupModalOpen,
    backupModalTab, accounts, updateAccountPriority,
    deleteConfirmId, setDeleteConfirmId,
    isBatchDelete, setIsBatchDelete,
    isDeleting,
    selectedIds,
    isWarmupConfirmOpen, setIsWarmupConfirmOpen,
    isWarmuping,
  } = state;

  const handleWarmupConfirm = async (): Promise<void> => {
    setIsWarmupConfirmOpen(false);
    const customWarmup = (handlers as { handleWarmupAccounts?: () => Promise<void> }).handleWarmupAccounts;
    if (typeof customWarmup === 'function') {
      await customWarmup();
      return;
    }
    if (typeof handlers.handleWarmupAll === 'function') {
      await handlers.handleWarmupAll();
      return;
    }
    await useAccountStore.getState().warmUpAccounts();
  };
  return (
    <>
      <AccountDetailsDialog
        account={detailsAccount}
        onClose={() => setDetailsAccount(null)}
        onUpdatePriority={updateAccountPriority}
      />

      {/* Account error dialog */}
      <AccountErrorDialog
        account={accounts.find((a: Account) => a.id === errorAccountId) || null}
        onClose={() => setErrorAccountId(null)}
      />

      {/* Unified Encrypted Backup Modal */}
      <UnifiedBackupModal
        isOpen={isBackupModalOpen}
        onClose={() => setIsBackupModalOpen(false)}
        initialTab={backupModalTab}
      />

      {/* Single Account Deletion Confirmation Modal */}
      <ModalDialog
        isOpen={Boolean(deleteConfirmId)}
        title={t('accounts.dialog.delete_title', 'Delete Account')}
        message={t('accounts.dialog.delete_message', 'Are you sure you want to delete account {{email}}? This action cannot be undone.', {
          email: accounts.find((a: Account) => a.id === deleteConfirmId)?.email || ''
        })}
        type="confirm"
        isDestructive={true}
        isLoading={isDeleting}
        confirmText={t('common.delete', 'Delete')}
        cancelText={t('common.cancel', 'Cancel')}
        onConfirm={handlers.executeDelete}
        onCancel={() => setDeleteConfirmId(null)}
        onClose={() => setDeleteConfirmId(null)}
      />

      {/* Batch Account Deletion Confirmation Modal */}
      <ModalDialog
        isOpen={isBatchDelete}
        title={t('accounts.dialog.batch_delete_title', 'Delete Selected Accounts')}
        message={t('accounts.dialog.batch_delete_message', 'Are you sure you want to delete {{count}} selected accounts? This action cannot be undone.', {
          count: selectedIds.size
        })}
        type="confirm"
        isDestructive={true}
        isLoading={isDeleting}
        confirmText={t('common.delete', 'Delete')}
        cancelText={t('common.cancel', 'Cancel')}
        onConfirm={handlers.executeBatchDelete}
        onCancel={() => setIsBatchDelete(false)}
        onClose={() => setIsBatchDelete(false)}
      />

      {/* Account Warmup Confirmation Modal */}
      <WarmupConfirmModal
        isOpen={isWarmupConfirmOpen}
        onClose={() => setIsWarmupConfirmOpen(false)}
        onConfirm={handleWarmupConfirm}
        selectedCount={selectedIds.size}
        isWarmuping={isWarmuping}
      />
    </>
  );
}
