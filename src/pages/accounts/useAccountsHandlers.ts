import { showToast } from "../../components/common/ToastContainer";
import { exportAccounts } from "../../services/accountService";
import { isTauri } from "../../utils/env";
import { request as invoke } from "../../utils/request";
import type { Account } from "../../types/account";
import type { useAccountsState } from "./useAccountsState";

type AccountsState = ReturnType<typeof useAccountsState>;

export function useAccountsHandlers(state: AccountsState) {
  const {
    t, config, showAllQuotas, toggleShowAllQuotas, updateInfo, installUpdate,
    isInstalling, searchQuery, setSearchQuery, filter, setFilter, currentPage,
    setCurrentPage, instances, activeInstanceId, boundInstanceAccountId, activeFocusTargetId, activeFocusTargetAccount,
    isSearchExpanded, setIsSearchExpanded, viewMode, setViewMode, selectedIds, setSelectedIds,
    deviceAccount, setDeviceAccount, detailsAccount, setDetailsAccount, deleteConfirmId, setDeleteConfirmId,
    isBatchDelete, setIsBatchDelete, isDeleting, setIsDeleting, toggleProxyConfirm, setToggleProxyConfirm,
    isWarmupConfirmOpen, setIsWarmupConfirmOpen, isWarmuping, setIsWarmuping, refreshingIds, setRefreshingIds,
    errorAccountId, setErrorAccountId, focusedAccountId, setFocusedAccountId, isBackupModalOpen, setIsBackupModalOpen,
    backupModalTab, setBackupModalTab, containerSize, setContainerSize, localPageSize, setLocalPageSize,
    ITEMS_PER_PAGE, searchedAccounts, filterCounts, filteredAccounts, paginatedAccounts, switchingAccountId,
    setSwitchingAccountId, isRefreshing, setIsRefreshing, isRefreshConfirmOpen, setIsRefreshConfirmOpen
  } = state;
  const handleWarmup = async (accountId: string) => {
    setRefreshingIds((prev) => {
      const next = new Set(prev);
      next.add(accountId);
      return next;
    });
    try {
      const msg = await warmUpAccount(accountId);
      showToast(msg, "success");
    } catch (error) {
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setRefreshingIds((prev) => {
        const next = new Set(prev);
        next.delete(accountId);
        return next;
      });
    }
  };

  const handleUpdateLabel = async (accountId: string, label: string) => {
    try {
      await updateAccountLabel(accountId, label);
      showToast(t('accounts.label_updated', 'Label updated'), 'success');
    } catch (error) {
      showToast(`${t('common.error')}: ${error}`, 'error');
    }
  };

  const handleWarmupAll = async () => {
    setIsWarmupConfirmOpen(false);
    setIsWarmuping(true);
    try {
      const isBatch = selectedIds.size > 0;
      if (isBatch) {
        const ids = Array.from(selectedIds);
        setRefreshingIds(new Set(ids));
        const results = await Promise.allSettled(
          ids.map((id) => warmUpAccount(id)),
        );
        let successCount = 0;
        results.forEach((r) => {
          if (r.status === "fulfilled") successCount++;
        });
        showToast(
          t("accounts.warmup_batch_triggered", { count: successCount }),
          "success",
        );
      } else {
        const msg = await warmUpAccounts();
        if (msg) {
          showToast(msg, "success");
        } else {
          showToast(
            t("accounts.warmup_all_triggered", "All accounts warmup task triggered"),
            "success",
          );
        }
      }
    } catch (error) {
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setIsWarmuping(false);
      setRefreshingIds(new Set());
    }
  };

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  const handleToggleSelect = (id: string) => {
    const newSet = new Set(selectedIds);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    setSelectedIds(newSet);
  };

  const handleToggleAll = () => {
    // Select all items on current page
    const currentIds = paginatedAccounts.map((a) => a.id);
    const allSelected = currentIds.every((id) => selectedIds.has(id));

    const newSet = new Set(selectedIds);
    if (allSelected) {
      currentIds.forEach((id) => newSet.delete(id));
    } else {
      currentIds.forEach((id) => newSet.add(id));
    }
    setSelectedIds(newSet);
  };

  const handleAddAccount = async (email: string, refreshToken: string) => {
    await addAccount(email, refreshToken);
  };

  const handleSwitch = async (accountId: string, targetIde?: string) => {
    if (loading || switchingAccountId) return;

    setSwitchingAccountId(accountId);
    console.log("[Accounts] handleSwitch called for:", accountId, "targetIde:", targetIde);
    try {
      await switchAccount(accountId, targetIde);
      showToast(t("common.success"), "success");
    } catch (error) {
      console.error("[Accounts] Switch failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      // Add a small delay for smoother UX
      setTimeout(() => {
        setSwitchingAccountId(null);
      }, 500);
    }
  };

  const handleRefresh = async (accountId: string) => {
    if (refreshingIds.has(accountId)) return;
    setRefreshingIds((prev) => {
      const next = new Set(prev);
      next.add(accountId);
      return next;
    });
    try {
      await refreshQuota(accountId);
      showToast(t("common.success"), "success");
    } catch (error) {
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setRefreshingIds((prev) => {
        const next = new Set(prev);
        next.delete(accountId);
        return next;
      });
    }
  };

  const handleBatchDelete = () => {
    if (selectedIds.size === 0) return;
    setIsBatchDelete(true);
  };

  const executeBatchDelete = async () => {
    if (selectedIds.size === 0 || isDeleting) return;

    setIsDeleting(true);
    try {
      const ids = Array.from(selectedIds);
      console.log("[Accounts] Batch deleting:", ids);
      await deleteAccounts(ids);
      setSelectedIds(new Set());
      console.log("[Accounts] Batch delete success");
      showToast(t("common.success"), "success");
      setIsBatchDelete(false);
    } catch (error) {
      console.error("[Accounts] Batch delete failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setIsDeleting(false);
    }
  };

  const handleDelete = (accountId: string) => {
    if (isDeleting) return;
    console.log("[Accounts] Request to delete:", accountId);
    setDeleteConfirmId(accountId);
  };

  const executeDelete = async () => {
    if (!deleteConfirmId || isDeleting) return;

    setIsDeleting(true);
    const idToDelete = deleteConfirmId;
    try {
      console.log("[Accounts] Executing delete for:", idToDelete);
      await deleteAccount(idToDelete);
      console.log("[Accounts] Delete success");
      showToast(t("common.success"), "success");
      setDeleteConfirmId(null);
    } catch (error) {
      console.error("[Accounts] Delete failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setIsDeleting(false);
    }
  };

  const handleToggleProxy = (accountId: string, currentlyDisabled: boolean) => {
    setToggleProxyConfirm({ accountId, enable: currentlyDisabled });
  };

  const executeToggleProxy = async () => {
    if (!toggleProxyConfirm) return;

    try {
      await toggleProxyStatus(
        toggleProxyConfirm.accountId,
        toggleProxyConfirm.enable,
        toggleProxyConfirm.enable
          ? undefined
          : t("accounts.proxy_disabled_reason_manual"),
      );
      showToast(t("common.success"), "success");
    } catch (error) {
      console.error("[Accounts] Toggle proxy status failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setToggleProxyConfirm(null);
    }
  };

  const handleBatchToggleProxy = async (enable: boolean) => {
    if (selectedIds.size === 0) return;

    try {
      const promises = Array.from(selectedIds).map((id) =>
        toggleProxyStatus(
          id,
          enable,
          enable ? undefined : t("accounts.proxy_disabled_reason_batch"),
        ),
      );
      await Promise.all(promises);
      showToast(
        enable
          ? t("accounts.toast.proxy_enabled", { count: selectedIds.size })
          : t("accounts.toast.proxy_disabled", { count: selectedIds.size }),
        "success",
      );
      setSelectedIds(new Set());
    } catch (error) {
      console.error("[Accounts] Batch toggle proxy status failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    }
  };

  const handleRefreshClick = () => {
    setIsRefreshConfirmOpen(true);
  };

  const executeRefresh = async () => {
    setIsRefreshConfirmOpen(false);
    setIsRefreshing(true);
    try {
      const isBatch = selectedIds.size > 0;
      let successCount = 0;
      let failedCount = 0;
      const details: string[] = [];

      if (isBatch) {
        // Batch refresh selected accounts
        const ids = Array.from(selectedIds);
        setRefreshingIds(new Set(ids));

        const results = await Promise.allSettled(
          ids.map((id) => refreshQuota(id)),
        );

        results.forEach((result, index) => {
          const id = ids[index];
          const email = accounts.find((a) => a.id === id)?.email || id;
          if (result.status === "fulfilled") {
            successCount++;
          } else {
            failedCount++;
            details.push(`${email}: ${result.reason}`);
          }
        });
      } else {
        // Refresh all accounts
        setRefreshingIds(new Set(accounts.map((a) => a.id)));
        const stats = await useAccountStore.getState().refreshAllQuotas();
        if (stats) {
          successCount = stats.success;
          failedCount = stats.failed;
          details.push(...stats.details);
        }
      }

      if (failedCount === 0) {
        showToast(
          t("accounts.refresh_selected", { count: successCount }),
          "success",
        );
      } else {
        showToast(
          `${t("common.success")}: ${successCount}, ${t("common.error")}: ${failedCount}`,
          "warning",
        );
        // You might want to show details in a different way, but for toast, keep it simple or use a "view details" action if supported.
        // For now, simpler toast is better than a huge alert.
        if (details.length > 0) {
          console.warn("Refresh failures:", details);
        }
      }
    } catch (error) {
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setIsRefreshing(false);
      setRefreshingIds(new Set());
    }
  };

  const handleFocusActiveAccount = () => {
    // 1. Identify target account ID: prioritized selected instance bound account, falling back to currentAccount
    const targetAccountId = activeFocusTargetId;
    if (!targetAccountId) {
      showToast(
        t("accounts.no_active_to_focus", "No active account found to focus"),
        "info"
      );
      return;
    }

    const targetAccount = accounts.find((a) => a.id === targetAccountId);
    if (!targetAccount) {
      showToast(
        t("accounts.no_active_to_focus", "No active account found to focus"),
        "info"
      );
      return;
    }

    // 2. Clear filters if targetAccount is hidden by active filter or search query
    if (filter !== "all" || searchQuery.trim().length > 0) {
      setFilter("all");
      setSearchQuery("");
    }

    // 3. Calculate target page number and navigate if necessary
    const targetIndex = accounts.findIndex((a) => a.id === targetAccountId);
    if (targetIndex >= 0) {
      const targetPage = Math.floor(targetIndex / ITEMS_PER_PAGE) + 1;
      if (targetPage !== currentPage) {
        setCurrentPage(targetPage);
      }
    }

    // 4. Trigger state-based auto-scroll and luminous high-contrast highlight
    setFocusedAccountId(targetAccountId);
  };

  const exportAccountsToJson = async (accountsToExport: Account[]) => {
    try {
      if (accountsToExport.length === 0) {
        showToast(t("dashboard.toast.export_no_accounts"), "warning");
        return;
      }

      // 1. Get export data from API (contains refresh_token)
      const accountIds = accountsToExport.map((acc) => acc.id);
      const response = await exportAccounts(accountIds);

      if (!response.accounts || response.accounts.length === 0) {
        showToast(t("dashboard.toast.export_no_accounts"), "warning");
        return;
      }

      const exportData = response.accounts;
      const content = JSON.stringify(exportData, null, 2);
      const fileName = `antigravity_accounts_${new Date().toISOString().split("T")[0]}.json`;

      // 2. Determine Path & Export
      if (isTauri()) {
        let path: string | null = null;
        const { join } = await import("@tauri-apps/api/path");

        if (config?.default_export_path) {
          // Use default path
          path = await join(config.default_export_path, fileName);
        } else {
          // Use Native Dialog
          const { save } = await import("@tauri-apps/plugin-dialog");
          path = await save({
            filters: [
              {
                name: "JSON",
                extensions: ["json"],
              },
            ],
            defaultPath: fileName,
          });
        }

        if (!path) return; // Cancelled

        // 3. Write File
        await invoke("save_text_file", { path, content });
        showToast(`${t("common.success")} ${path}`, "success");
      } else {
        // Web mode: download via browser
        const blob = new Blob([content], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = fileName;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        showToast(
          t("dashboard.toast.export_success", { path: fileName }),
          "success",
        );
      }
    } catch (error: any) {
      console.error("Export failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    }
  };

  const handleExportOne = (accountId: string) => {
    const account = accounts.find((a) => a.id === accountId);
    if (account) {
      exportAccountsToJson([account]);
    }
  };

  const handleViewDetails = (accountId: string) => {
    const account = accounts.find((a) => a.id === accountId);
    if (account) {
      setDetailsAccount(account);
    }
  };

  const handleViewDevice = (accountId: string) => {
    const account = accounts.find((a) => a.id === accountId);
    if (account) {
      setDeviceAccount(account);
    }
  };

  return {
    handleWarmup,
    handleUpdateLabel,
    handleWarmupAll,
    handlePageChange,
    handleToggleSelect,
    handleToggleAll,
    handleAddAccount,
    handleSwitch,
    handleRefresh,
    handleBatchDelete,
    executeBatchDelete,
    handleDelete,
    executeDelete,
    handleToggleProxy,
    executeToggleProxy,
    handleBatchToggleProxy,
    handleRefreshClick,
    executeRefresh,
    handleFocusActiveAccount,
    exportAccountsToJson,
    handleExportOne,
    handleViewDetails,
    handleViewDevice
  };
}