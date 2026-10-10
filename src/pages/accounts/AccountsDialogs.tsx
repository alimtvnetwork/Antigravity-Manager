import type { useAccountsState } from "./useAccountsState";
import type { useAccountsHandlers } from "./useAccountsHandlers";
import type { Account } from "../../types/account";
import AccountDetailsDialog from "../../components/accounts/AccountDetailsDialog";
import AccountErrorDialog from "../../components/accounts/AccountErrorDialog";
import { UnifiedBackupModal } from "../../components/modals/UnifiedBackupModal";

type AccountsState = ReturnType<typeof useAccountsState>;
type AccountsHandlers = ReturnType<typeof useAccountsHandlers>;

interface AccountsDialogsProps {
  state: AccountsState;
  handlers: AccountsHandlers;
}

export function AccountsDialogs({ state }: AccountsDialogsProps) {
  const {
    detailsAccount, setDetailsAccount, errorAccountId, setErrorAccountId, isBackupModalOpen, setIsBackupModalOpen,
    backupModalTab, accounts, updateAccountPriority
  } = state;
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
    </>
  );
}
