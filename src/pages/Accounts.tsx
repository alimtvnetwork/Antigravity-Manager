

import {
  Calendar,
  ChevronDown,
  Clock,
  Download,
  LayoutGrid,
  List,
  RefreshCw,
  Search,
  Sparkles,
  ToggleLeft,
  ToggleRight,
  Trash2,
  Upload,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import AccountDetailsDialog from "../components/accounts/AccountDetailsDialog";
import AccountGrid from "../components/accounts/AccountGrid";
import AccountTable from "../components/accounts/AccountTable";
import AddAccountDialog from "../components/accounts/AddAccountDialog";
import DeviceFingerprintDialog from "../components/accounts/DeviceFingerprintDialog";
import ModalDialog from "../components/common/ModalDialog";
import Pagination from "../components/common/Pagination";
import AccountErrorDialog from "../components/accounts/AccountErrorDialog";
import { UnifiedBackupModal } from "../components/modals/UnifiedBackupModal";
import { showToast } from "../components/common/ToastContainer";
import { exportAccounts } from "../services/accountService";
import { useAccountStore } from "../stores/useAccountStore";
import { useConfigStore } from "../stores/useConfigStore";
import { useUpdateStore } from "../stores/use-update-store";
import { Account } from "../types/account";
import { cn } from "../utils/cn";
import { isTauri } from "../utils/env";
import { request as invoke } from "../utils/request";
import { useTranslation } from "react-i18next";

type FilterType = "all" | "pro" | "ultra" | "free";
type ViewMode = "list" | "grid";
export type QuotaWindow = "5h" | "weekly";


function Accounts() {
  const { t } = useTranslation();
  const {
    accounts,
    currentAccount,
    fetchAccounts,
    fetchCurrentAccount,
    addAccount,
    deleteAccount,
    deleteAccounts,
    switchAccount,
    loading,
    refreshQuota,
    toggleProxyStatus,
    reorderAccounts,
    warmUpAccounts,
    warmUpAccount,
    updateAccountLabel,
  } = useAccountStore();
  const { config, showAllQuotas, toggleShowAllQuotas } = useConfigStore();
  const { updateInfo, setShowNotification } = useUpdateStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [filter, setFilter] = useState<FilterType>('all');
  const [isSearchExpanded, setIsSearchExpanded] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [viewMode, setViewMode] = useState<ViewMode>(() => {
    const saved = localStorage.getItem('accounts_view_mode');
    return (saved === 'list' || saved === 'grid') ? saved : 'list';
  });

  const [quotaWindow, setQuotaWindow] = useState<QuotaWindow>(() => {
    const saved = localStorage.getItem('accounts_quota_window');
    return (saved === '5h' || saved === 'weekly') ? saved : '5h';
  });

  // Save view mode preference
  useEffect(() => {
    localStorage.setItem('accounts_view_mode', viewMode);
  }, [viewMode]);

  // Save quota window preference
  useEffect(() => {
    localStorage.setItem('accounts_quota_window', quotaWindow);
  }, [quotaWindow]);

  // Fetch accounts and active current account on mount
  useEffect(() => {
    fetchAccounts();
    fetchCurrentAccount();
  }, [fetchAccounts, fetchCurrentAccount]);
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [deviceAccount, setDeviceAccount] = useState<Account | null>(null);
  const [detailsAccount, setDetailsAccount] = useState<Account | null>(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const [isBatchDelete, setIsBatchDelete] = useState(false);
  const [toggleProxyConfirm, setToggleProxyConfirm] = useState<{
    accountId: string;
    enable: boolean;
  } | null>(null);
  const [isWarmupConfirmOpen, setIsWarmupConfirmOpen] = useState(false);
  const [isWarmuping, setIsWarmuping] = useState(false);
  const [refreshingIds, setRefreshingIds] = useState<Set<string>>(new Set());
  const [errorAccountId, setErrorAccountId] = useState<string | null>(null);

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

  const [isBackupModalOpen, setIsBackupModalOpen] = useState(false);
  const [backupModalTab, setBackupModalTab] = useState<"export" | "import">("export");
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });

  useEffect(() => {
    if (!containerRef.current) return;
    const resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        setContainerSize({
          width: entry.contentRect.width,
          height: entry.contentRect.height,
        });
      }
    });
    resizeObserver.observe(containerRef.current);
    return () => resizeObserver.disconnect();
  }, []);

  // Pagination State
  const [currentPage, setCurrentPage] = useState(1);
  const [localPageSize, setLocalPageSize] = useState<number | null>(() => {
    const saved = localStorage.getItem("accounts_page_size");
    return saved ? parseInt(saved) : null;
  }); // Local pagination size preference

  // Save page size preference
  useEffect(() => {
    if (localPageSize !== null) {
      localStorage.setItem("accounts_page_size", localPageSize.toString());
    }
  }, [localPageSize]);

  // Dynamically calculate page size
  const ITEMS_PER_PAGE = useMemo(() => {
    // Prioritize local pagination size
    if (localPageSize && localPageSize > 0) {
      return localPageSize;
    }

    // Fallback to user configured fixed value
    if (config?.accounts_page_size && config.accounts_page_size > 0) {
      return config.accounts_page_size;
    }

    // Fallback to dynamic calculation logic
    if (!containerSize.height) return viewMode === "grid" ? 6 : 8;

    if (viewMode === "list") {
      const headerHeight = 36; // Compact header height
      const rowHeight = 72; // Row height with multi-model details
      // Calculate capacity, default min 10 rows
      const autoFitCount = Math.floor(
        (containerSize.height - headerHeight) / rowHeight,
      );
      return Math.max(10, autoFitCount);
    } else {
      const cardHeight = 180; // AccountCard height with padding
      const gap = 16; // gap-4

      // Match Tailwind breakpoint logic
      let cols = 1;
      if (containerSize.width >= 1200)
        cols = 4; // xl (~1280px)
      else if (containerSize.width >= 900)
        cols = 3; // lg (~1024px)
      else if (containerSize.width >= 600) cols = 2; // md (~768px)

      const rows = Math.max(
        1,
        Math.floor((containerSize.height + gap) / (cardHeight + gap)),
      );
      return cols * rows;
    }
  }, [localPageSize, config?.accounts_page_size, containerSize, viewMode]);

  useEffect(() => {
    fetchAccounts();
  }, []);

  // Reset pagination when view mode changes to avoid empty pages or confusion
  useEffect(() => {
    setCurrentPage(1);
  }, [viewMode]);

  // Search filter logic
  const searchedAccounts = useMemo(() => {
    if (!searchQuery) return accounts;
    const lowQuery = searchQuery.toLowerCase();
    return accounts.filter((a) => a.email.toLowerCase().includes(lowQuery));
  }, [accounts, searchQuery]);

  // Count filtered status numbers (based on search result)
  const filterCounts = useMemo(() => {
    return {
      all: searchedAccounts.length,
      pro: searchedAccounts.filter((a) =>
        a.quota?.subscription_tier?.toLowerCase().includes("pro"),
      ).length,
      ultra: searchedAccounts.filter((a) =>
        a.quota?.subscription_tier?.toLowerCase().includes("ultra"),
      ).length,
      free: searchedAccounts.filter((a) => {
        const tier = a.quota?.subscription_tier?.toLowerCase();
        return tier && !tier.includes("pro") && !tier.includes("ultra");
      }).length,
    };
  }, [searchedAccounts]);

  // Filter and search final results
  const filteredAccounts = useMemo(() => {
    let result = searchedAccounts;

    if (filter === "pro") {
      result = result.filter((a) =>
        a.quota?.subscription_tier?.toLowerCase().includes("pro"),
      );
    } else if (filter === "ultra") {
      result = result.filter((a) =>
        a.quota?.subscription_tier?.toLowerCase().includes("ultra"),
      );
    } else if (filter === "free") {
      result = result.filter((a) => {
        const tier = a.quota?.subscription_tier?.toLowerCase();
        return tier && !tier.includes("pro") && !tier.includes("ultra");
      });
    }

    return result;
  }, [searchedAccounts, filter]);

  // Pagination Logic
  const paginatedAccounts = useMemo(() => {
    const startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
    return filteredAccounts.slice(startIndex, startIndex + ITEMS_PER_PAGE);
  }, [filteredAccounts, currentPage, ITEMS_PER_PAGE]);

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  // Clear selection and reset page when filter changes
  useEffect(() => {
    setSelectedIds(new Set());
    setCurrentPage(1);
  }, [filter, searchQuery]);

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

  const [switchingAccountId, setSwitchingAccountId] = useState<string | null>(
    null,
  );

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
    setIsBatchDelete(false);
    try {
      const ids = Array.from(selectedIds);
      console.log("[Accounts] Batch deleting:", ids);
      await deleteAccounts(ids);
      setSelectedIds(new Set());
      console.log("[Accounts] Batch delete success");
      showToast(t("common.success"), "success");
    } catch (error) {
      console.error("[Accounts] Batch delete failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    }
  };

  const handleDelete = (accountId: string) => {
    console.log("[Accounts] Request to delete:", accountId);
    setDeleteConfirmId(accountId);
  };

  const executeDelete = async () => {
    if (!deleteConfirmId) return;

    try {
      console.log("[Accounts] Executing delete for:", deleteConfirmId);
      await deleteAccount(deleteConfirmId);
      console.log("[Accounts] Delete success");
      showToast(t("common.success"), "success");
    } catch (error) {
      console.error("[Accounts] Delete failed:", error);
      showToast(`${t("common.error")}: ${error}`, "error");
    } finally {
      setDeleteConfirmId(null);
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

  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isRefreshConfirmOpen, setIsRefreshConfirmOpen] = useState(false);

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

  return (
    <div className="h-full flex flex-col px-2.5 sm:px-4 pt-1.5 pb-4 gap-2.5 max-w-[1920px] mx-auto w-full min-w-0">
      {/* Top action bar: search, filters, and action buttons */}
      <div className="flex-none flex flex-wrap 2xl:flex-nowrap items-center justify-between gap-1.5 min-w-0 w-full">
        {/* Left controls: search, window, view mode, quota filter */}
        <div className="flex items-center gap-1 flex-wrap sm:flex-nowrap overflow-x-auto lg:overflow-x-visible scrollbar-none max-w-full py-0.5 min-w-0 shrink-0">
          {/* Search box - responsive */}
          <div className="hidden lg:block flex-none w-36 relative transition-all focus-within:w-44">
            <Search className="absolute left-2.5 top-1/2 transform -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
            <input
              type="text"
              placeholder={t('accounts.search_placeholder')}
              className="w-full h-8 pl-8 pr-3 bg-gray-100/50 dark:bg-white/[0.04] text-xs text-gray-900 dark:text-base-content border border-transparent hover:border-gray-200/50 dark:hover:border-white/5 rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500/50 placeholder:text-gray-400 dark:placeholder:text-gray-500 transition-all"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>

          {/* Search button - small screens */}
          <div className="lg:hidden relative">
            {!isSearchExpanded ? (
              <button
                onClick={() => {
                  setIsSearchExpanded(true);
                  setTimeout(() => searchInputRef.current?.focus(), 100);
                }}
                className="h-8 w-8 inline-flex items-center justify-center bg-gray-100/40 dark:bg-white/[0.04] hover:bg-gray-200/60 dark:hover:bg-white/[0.08] rounded-lg transition-colors"
                title={t('accounts.search_placeholder')}
              >
                <Search className="w-3.5 h-3.5 text-gray-600 dark:text-gray-300" />
              </button>
            ) : (
              <div className="absolute left-0 top-0 z-10 w-64 flex items-center gap-1">
                <div className="flex-1 relative">
                  <Search className="absolute left-2.5 top-1/2 transform -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
                  <input
                    ref={searchInputRef}
                    type="text"
                    placeholder={t('accounts.search_placeholder')}
                    className="w-full h-8 pl-8 pr-3 bg-white dark:bg-slate-900 text-xs text-gray-900 dark:text-base-content border border-transparent rounded-lg focus:outline-none focus:ring-1 focus:ring-blue-500/50 placeholder:text-gray-400 dark:placeholder:text-gray-500 shadow-lg"
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    onBlur={() => setIsSearchExpanded(false)}
                  />
                </div>
              </div>
            )}
          </div>

          {/* Quota window toggle (5H / Weekly) - Flat continuous grouping */}
          <div className="h-8 inline-flex items-center gap-0.5 p-0.5 rounded-lg bg-gray-100/40 dark:bg-white/[0.04] shrink-0">
            <button
              className={cn(
                "h-7 px-2.5 inline-flex items-center gap-1 rounded-md text-xs font-semibold transition-all",
                quotaWindow === "5h"
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content",
              )}
              onClick={() => setQuotaWindow("5h")}
              title={t("accounts.quota_window_5h", "5-hour rolling quota")}
            >
              <Clock className="w-3.5 h-3.5" />
              <span>5H</span>
            </button>
            <button
              className={cn(
                "h-7 px-2.5 inline-flex items-center gap-1 rounded-md text-xs font-semibold transition-all",
                quotaWindow === "weekly"
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content",
              )}
              onClick={() => setQuotaWindow("weekly")}
              title={t("accounts.quota_window_weekly", "7-day weekly quota")}
            >
              <Calendar className="w-3.5 h-3.5" />
              <span>{t("accounts.quota_window_weekly_short", "Weekly")}</span>
            </button>
          </div>

          {/* View mode switcher - Flat continuous grouping */}
          <div className="h-8 inline-flex items-center gap-0.5 p-0.5 rounded-lg bg-gray-100/40 dark:bg-white/[0.04] shrink-0">
            <button
              className={cn(
                "h-7 w-7 inline-flex items-center justify-center rounded-md transition-all",
                viewMode === "list"
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content",
              )}
              onClick={() => setViewMode("list")}
              title={t("accounts.views.list")}
            >
              <List className="w-4 h-4" />
            </button>
            <button
              className={cn(
                "h-7 w-7 inline-flex items-center justify-center rounded-md transition-all",
                viewMode === "grid"
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content",
              )}
              onClick={() => setViewMode("grid")}
              title={t("accounts.views.grid")}
            >
              <LayoutGrid className="w-4 h-4" />
            </button>
          </div>

          {/* Quota tier filter pills - Flat continuous grouping */}
          <div className="h-8 inline-flex items-center gap-0.5 p-0.5 rounded-lg bg-gray-100/40 dark:bg-white/[0.04] shrink-0">
            {/* All */}
            <button
              className={cn(
                "h-7 px-2 inline-flex items-center gap-1 rounded-md text-xs font-semibold transition-all whitespace-nowrap shrink-0",
                filter === 'all'
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs ring-1 ring-black/5"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content hover:bg-white/40"
              )}
              onClick={() => setFilter('all')}
              title={`${t('accounts.all')} (${filterCounts.all})`}
            >
              <span>{t('accounts.all')}</span>
              <span className={cn(
                "px-1.5 py-0.2 rounded-md text-[10px] font-bold transition-colors",
                filter === 'all'
                  ? "bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400"
                  : "bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400"
              )}>
                {filterCounts.all}
              </span>
            </button>

            {/* PRO */}
            <button
              className={cn(
                "h-7 px-2 inline-flex items-center gap-1 rounded-md text-xs font-semibold transition-all whitespace-nowrap shrink-0",
                filter === 'pro'
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs ring-1 ring-black/5"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content hover:bg-white/40"
              )}
              onClick={() => setFilter('pro')}
              title={`${t('accounts.pro')} (${filterCounts.pro})`}
            >
              <span>{t('accounts.pro')}</span>
              <span className={cn(
                "px-1.5 py-0.2 rounded-md text-[10px] font-bold transition-colors",
                filter === 'pro'
                  ? "bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400"
                  : "bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400"
              )}>
                {filterCounts.pro}
              </span>
            </button>

            {/* ULTRA */}
            <button
              className={cn(
                "h-7 px-2 inline-flex items-center gap-1 rounded-md text-xs font-semibold transition-all whitespace-nowrap shrink-0",
                filter === 'ultra'
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs ring-1 ring-black/5"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content hover:bg-white/40"
              )}
              onClick={() => setFilter('ultra')}
              title={`${t('accounts.ultra')} (${filterCounts.ultra})`}
            >
              <span>{t('accounts.ultra')}</span>
              <span className={cn(
                "px-1.5 py-0.2 rounded-md text-[10px] font-bold transition-colors",
                filter === 'ultra'
                  ? "bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400"
                  : "bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400"
              )}>
                {filterCounts.ultra}
              </span>
            </button>

            {/* FREE */}
            <button
              className={cn(
                "h-7 px-2 inline-flex items-center gap-1 rounded-md text-xs font-semibold transition-all whitespace-nowrap shrink-0",
                filter === 'free'
                  ? "bg-white dark:bg-base-100 text-blue-600 dark:text-blue-400 shadow-xs ring-1 ring-black/5"
                  : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-base-content hover:bg-white/40"
              )}
              onClick={() => setFilter('free')}
              title={`${t('accounts.free')} (${filterCounts.free})`}
            >
              <span>{t('accounts.free')}</span>
              <span className={cn(
                "px-1.5 py-0.2 rounded-md text-[10px] font-bold transition-colors",
                filter === 'free'
                  ? "bg-blue-100 dark:bg-blue-500/20 text-blue-600 dark:text-blue-400"
                  : "bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400"
              )}>
                {filterCounts.free}
              </span>
            </button>
          </div>
        </div>

        {/* Action buttons group */}
        <div className="flex items-center gap-1.5 shrink-0 ml-auto">
          <AddAccountDialog onAdd={handleAddAccount} showText={false} />

          {selectedIds.size > 0 && (
            <>
              <button
                className="px-2.5 py-2 bg-red-500 text-white text-xs font-medium rounded-lg hover:bg-red-600 transition-colors flex items-center gap-1.5 shadow-sm"
                onClick={handleBatchDelete}
                title={t("accounts.delete_selected", {
                  count: selectedIds.size,
                })}
              >
                <Trash2 className="w-3.5 h-3.5" />
                <span className="hidden xl:inline">
                  {t("accounts.delete_selected", { count: selectedIds.size })}
                </span>
              </button>
              <button
                className="px-2.5 py-2 bg-orange-500 text-white text-xs font-medium rounded-lg hover:bg-orange-600 transition-colors flex items-center gap-1.5 shadow-sm"
                onClick={() => handleBatchToggleProxy(false)}
                title={t("accounts.disable_proxy_selected", {
                  count: selectedIds.size,
                })}
              >
                <ToggleLeft className="w-3.5 h-3.5" />
                <span className="hidden xl:inline">
                  {t("accounts.disable_proxy_selected", {
                    count: selectedIds.size,
                  })}
                </span>
              </button>
              <button
                className="px-2.5 py-2 bg-green-500 text-white text-xs font-medium rounded-lg hover:bg-green-600 transition-colors flex items-center gap-1.5 shadow-sm"
                onClick={() => handleBatchToggleProxy(true)}
                title={t("accounts.enable_proxy_selected", {
                  count: selectedIds.size,
                })}
              >
                <ToggleRight className="w-3.5 h-3.5" />
                <span className="hidden xl:inline">
                  {t("accounts.enable_proxy_selected", {
                    count: selectedIds.size,
                  })}
                </span>
              </button>
            </>
          )}

          <button
            className={`px-2.5 py-2 bg-blue-500 text-white text-xs font-medium rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-1.5 shadow-sm ${isRefreshing ? "opacity-70 cursor-not-allowed" : ""}`}
            onClick={handleRefreshClick}
            disabled={isRefreshing}
            title={
              selectedIds.size > 0
                ? t("accounts.refresh_selected", { count: selectedIds.size })
                : t("accounts.refresh_all")
            }
          >
            <RefreshCw
              className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`}
            />
            <span className="hidden xl:inline">
              {isRefreshing
                ? t("common.loading")
                : selectedIds.size > 0
                  ? t("accounts.refresh_selected", { count: selectedIds.size })
                  : t("accounts.refresh_all")}
            </span>
          </button>

          <button
            className={`px-2.5 py-2 bg-orange-500 text-white text-xs font-medium rounded-lg hover:bg-orange-600 transition-colors flex items-center gap-1.5 shadow-sm ${isWarmuping ? "opacity-70 cursor-not-allowed" : ""}`}
            onClick={() => setIsWarmupConfirmOpen(true)}
            disabled={isWarmuping}
            title={
              selectedIds.size > 0
                ? t("accounts.warmup_selected", { count: selectedIds.size })
                : t("accounts.warmup_all", "Warm up all accounts")
            }
          >
            <Sparkles
              className={`w-3.5 h-3.5 ${isWarmuping ? "animate-pulse" : ""}`}
            />
            <span className="hidden xl:inline">
              {isWarmuping
                ? t("common.loading")
                : selectedIds.size > 0
                  ? t("accounts.warmup_selected", { count: selectedIds.size })
                  : t("accounts.warmup_all", "Warm up all")}
            </span>
          </button>

          <label className="flex items-center gap-2 cursor-pointer select-none px-2 py-2 border border-transparent hover:bg-gray-100 dark:hover:bg-base-200 rounded-lg transition-colors" title={t('accounts.show_all_quotas')}>
            <span className="text-xs font-medium text-gray-600 dark:text-gray-300 hidden xl:inline">
              {t('accounts.show_all_quotas')}
            </span>
            <input
              type="checkbox"
              className="toggle toggle-xs toggle-primary"
              checked={showAllQuotas}
              onChange={toggleShowAllQuotas}
            />
          </label>
          <div className="w-px h-4 bg-gray-200 dark:bg-gray-700 self-center mx-1 shrink-0"></div>

          <div className="relative group">
            <button
              type="button"
              className="px-2.5 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 text-xs font-medium rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors flex items-center gap-1.5 cursor-pointer"
              onClick={() => {
                setBackupModalTab("export");
                setIsBackupModalOpen(true);
              }}
              title={`${t("accounts.import_json")} / ${t("common.export")}`}
            >
              <Download className="w-3.5 h-3.5" />
              <span className="hidden lg:inline">Import / Export</span>
              <ChevronDown className="w-3 h-3 text-gray-400" />
            </button>
            <div className="invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-all duration-150 absolute right-0 top-full pt-1 w-44 z-[9999]">
              <div className="bg-white dark:bg-slate-900 border border-gray-200 dark:border-slate-800 rounded-xl shadow-xl py-1 text-xs">
                <button
                  type="button"
                  onClick={() => {
                    setBackupModalTab("import");
                    setIsBackupModalOpen(true);
                  }}
                  className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
                >
                  <Upload className="w-3.5 h-3.5 text-emerald-500" />
                  <span>{t("accounts.import_json")}</span>
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setBackupModalTab("export");
                    setIsBackupModalOpen(true);
                  }}
                  className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-slate-800 text-gray-700 dark:text-slate-300 cursor-pointer"
                >
                  <Download className="w-3.5 h-3.5 text-blue-500" />
                  <span>{t("common.export")}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Account list content area */}
      <div className="flex-1 min-h-0 relative" ref={containerRef}>
        {viewMode === "list" ? (
          <div className="h-full bg-white dark:bg-base-100 rounded-2xl shadow-sm border border-gray-100 dark:border-base-200 flex flex-col overflow-hidden">
            <div className="flex-1 overflow-y-auto">
              <AccountTable
                accounts={paginatedAccounts}
                selectedIds={selectedIds}
                refreshingIds={refreshingIds}
                onToggleSelect={handleToggleSelect}
                onToggleAll={handleToggleAll}
                currentAccountId={currentAccount?.id || null}
                currentAccountEmail={currentAccount?.email || null}
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
                    !!accounts.find((a) => a.id === id)?.proxy_disabled,
                  )
                }
                onReorder={reorderAccounts}
                onWarmup={handleWarmup}
                onUpdateLabel={handleUpdateLabel}
                onViewError={(id: string) => setErrorAccountId(id)}
                quotaWindow={quotaWindow}
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
              currentAccountId={currentAccount?.id || null}
              currentAccountEmail={currentAccount?.email || null}
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
                  !!accounts.find((a) => a.id === id)?.proxy_disabled,
                )
              }
              onWarmup={handleWarmup}
              onUpdateLabel={handleUpdateLabel}
              onViewError={(id: string) => setErrorAccountId(id)}
              quotaWindow={quotaWindow}
            />
          </div>
        )}
      </div>

      {/* Minimalist floating pagination */}
      {filteredAccounts.length > 0 && (
        <div className="flex-none">
          <Pagination
            currentPage={currentPage}
            totalPages={Math.ceil(filteredAccounts.length / ITEMS_PER_PAGE)}
            onPageChange={handlePageChange}
            totalItems={filteredAccounts.length}
            itemsPerPage={ITEMS_PER_PAGE}
            onPageSizeChange={(newSize) => {
              setLocalPageSize(newSize);
              setCurrentPage(1); // Reset to first page
            }}
            pageSizeOptions={[10, 20, 50, 100]}
            centerContent={
              updateInfo?.has_update ? (
                <button
                  type="button"
                  onClick={() => setShowNotification(true)}
                  className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-gradient-to-r from-blue-500/10 to-purple-500/10 hover:from-blue-500/20 hover:to-purple-500/20 text-blue-600 dark:text-blue-400 border border-blue-500/20 shadow-xs transition-all duration-200 cursor-pointer active:scale-95 group"
                  title="A newer version is available. Click to view and install."
                >
                  <Sparkles className="w-3.5 h-3.5 text-blue-500 group-hover:rotate-12 transition-transform" />
                  <span>Update Available: v{updateInfo.latest_version}</span>
                  <span className="text-[10px] bg-blue-600 hover:bg-blue-500 text-white px-1.5 py-0.5 rounded font-semibold ml-0.5">
                    Install Now
                  </span>
                </button>
              ) : null
            }
          />
        </div>
      )}

      <AccountDetailsDialog
        account={detailsAccount}
        onClose={() => setDetailsAccount(null)}
      />
      <DeviceFingerprintDialog
        account={deviceAccount}
        onClose={() => setDeviceAccount(null)}
      />

      <ModalDialog
        isOpen={!!deleteConfirmId || isBatchDelete}
        title={
          isBatchDelete
            ? t("accounts.dialog.batch_delete_title")
            : t("accounts.dialog.delete_title")
        }
        message={
          isBatchDelete
            ? t("accounts.dialog.batch_delete_msg", { count: selectedIds.size })
            : t("accounts.dialog.delete_msg")
        }
        type="confirm"
        confirmText={t("common.delete")}
        isDestructive={true}
        onConfirm={isBatchDelete ? executeBatchDelete : executeDelete}
        onCancel={() => {
          setDeleteConfirmId(null);
          setIsBatchDelete(false);
        }}
      />

      <ModalDialog
        isOpen={isRefreshConfirmOpen}
        title={
          selectedIds.size > 0
            ? t("accounts.dialog.batch_refresh_title")
            : t("accounts.dialog.refresh_title")
        }
        message={
          selectedIds.size > 0
            ? t("accounts.dialog.batch_refresh_msg", {
              count: selectedIds.size,
            })
            : t("accounts.dialog.refresh_msg")
        }
        type="confirm"
        confirmText={t("common.refresh")}
        isDestructive={false}
        onConfirm={executeRefresh}
        onCancel={() => setIsRefreshConfirmOpen(false)}
      />

      {toggleProxyConfirm && (
        <ModalDialog
          isOpen={!!toggleProxyConfirm}
          onCancel={() => setToggleProxyConfirm(null)}
          onConfirm={executeToggleProxy}
          title={
            toggleProxyConfirm.enable
              ? t("accounts.dialog.enable_proxy_title")
              : t("accounts.dialog.disable_proxy_title")
          }
          message={
            toggleProxyConfirm.enable
              ? t("accounts.dialog.enable_proxy_msg")
              : t("accounts.dialog.disable_proxy_msg")
          }
        />
      )}

      <ModalDialog
        isOpen={isWarmupConfirmOpen}
        title={
          selectedIds.size > 0
            ? t("accounts.dialog.batch_warmup_title", "Batch Manual Warmup")
            : t("accounts.dialog.warmup_all_title", "Warm Up All Accounts")
        }
        message={
          selectedIds.size > 0
            ? t(
              "accounts.dialog.batch_warmup_msg",
              "Are you sure you want to trigger warmup for the {{count}} selected accounts?",
              { count: selectedIds.size },
            )
            : t(
              "accounts.dialog.warmup_all_msg",
              "Are you sure you want to trigger warmup for all eligible accounts? This sends minimal traffic to Google services.",
            )
        }
        type="confirm"
        confirmText={t("accounts.warmup_now", "Warm Up Now")}
        isDestructive={false}
        onConfirm={handleWarmupAll}
        onCancel={() => setIsWarmupConfirmOpen(false)}
      />

      {/* Account details dialog */}
      <AccountDetailsDialog
        account={detailsAccount}
        onClose={() => setDetailsAccount(null)}
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
    </div>
  );
}

export default Accounts;
