

import {
  ChevronDown,
  Download,
  LayoutGrid,
  List,
  Loader2,
  LocateFixed,
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
import { useInstanceStore } from "../stores/useInstanceStore";
import { useUpdateStore } from "../stores/use-update-store";
import { resolveFocusTarget } from "../lib/resolve-focus-target";
import { Account } from "../types/account";
import { cn } from "../utils/cn";
import { isTauri } from "../utils/env";
import { request as invoke } from "../utils/request";
import { useTranslation } from "react-i18next";

type FilterType = "all" | "pro" | "ultra" | "free";
type ViewMode = "list" | "grid";


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
    updateAccountPriority,
  } = useAccountStore();
  const { config, showAllQuotas, toggleShowAllQuotas } = useConfigStore();
  const { updateInfo, installUpdate, isInstalling } = useUpdateStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [filter, setFilter] = useState<FilterType>('all');
  const [currentPage, setCurrentPage] = useState(1);
  const { instances, activeInstanceId } = useInstanceStore();

  const boundInstanceAccountId = useMemo(() => {
    const selectedInstance = instances.find((inst) => inst.id === activeInstanceId);
    if (!selectedInstance) return null;
    return (
      selectedInstance.config.bound_account_id ||
      accounts.find((a) => a.email === selectedInstance.config.bound_email)?.id ||
      null
    );
  }, [instances, activeInstanceId, accounts]);

  const activeFocusTargetId = useMemo(() => {
    return resolveFocusTarget(boundInstanceAccountId, currentAccount?.id || null);
  }, [boundInstanceAccountId, currentAccount?.id]);

  const activeFocusTargetAccount = useMemo(() => {
    if (!activeFocusTargetId) return null;
    return accounts.find((a) => a.id === activeFocusTargetId) || null;
  }, [accounts, activeFocusTargetId]);
  const [isSearchExpanded, setIsSearchExpanded] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [viewMode, setViewMode] = useState<ViewMode>(() => {
    const saved = localStorage.getItem('accounts_view_mode');
    return (saved === 'list' || saved === 'grid') ? saved : 'list';
  });

  // Save view mode preference
  useEffect(() => {
    localStorage.setItem('accounts_view_mode', viewMode);
  }, [viewMode]);

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
  const [isDeleting, setIsDeleting] = useState(false);
  const [toggleProxyConfirm, setToggleProxyConfirm] = useState<{
    accountId: string;
    enable: boolean;
  } | null>(null);
  const [isWarmupConfirmOpen, setIsWarmupConfirmOpen] = useState(false);
  const [isWarmuping, setIsWarmuping] = useState(false);
  const [refreshingIds, setRefreshingIds] = useState<Set<string>>(new Set());
  const [errorAccountId, setErrorAccountId] = useState<string | null>(null);
  const [focusedAccountId, setFocusedAccountId] = useState<string | null>(null);

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

  // Single auto-scroll and highlight pulse effect for focused account
  useEffect(() => {
    if (!focusedAccountId) return;
    let timer: ReturnType<typeof setTimeout> | null = null;
    const scrollTarget = () => {
      const cardEl = document.getElementById(`account-card-${focusedAccountId}`);
      const rowEl = document.getElementById(`account-row-${focusedAccountId}`);
      const targetEl = cardEl || rowEl;
      if (targetEl) {
        targetEl.scrollIntoView({ behavior: "smooth", block: "center" });
        timer = setTimeout(() => {
          setFocusedAccountId(null);
        }, 2000);
      }
    };

    const rafId = requestAnimationFrame(scrollTarget);
    return () => {
      cancelAnimationFrame(rafId);
      if (timer) {
        clearTimeout(timer);
      }
    };
  }, [focusedAccountId, currentPage, filter, searchQuery, paginatedAccounts]);
  const [localPageSize, setLocalPageSize] = useState<number | null>(() => {
    const saved = localStorage.getItem("accounts_page_size");
    const parsed = saved ? parseInt(saved, 10) : 150;
    return Number.isFinite(parsed) && parsed > 0 ? parsed : 150;
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

  // Listen for auto-switch events from backend to animate the specific account row and refresh state
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen<{ account_id: string; email: string }>('account://auto-switched', async (event) => {
        setSwitchingAccountId(event.payload.account_id);
        try {
          await fetchAccounts();
          await fetchCurrentAccount();
        } finally {
          setTimeout(() => {
            setSwitchingAccountId(null);
          }, 800);
        }
      }).then((fn) => {
        unlisten = fn;
      });
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [fetchAccounts, fetchCurrentAccount]);

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

  // Focus onto the active instance's bound account (falling back to currentAccount)
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

          <label className="h-8 inline-flex items-center shrink-0">
            <select
              className="h-8 rounded-lg border border-gray-200 bg-white px-2 text-xs font-semibold text-slate-950 dark:border-slate-700 dark:bg-[#070b10] dark:text-[#f5d76e]"
              value={filter}
              onChange={(event) => setFilter(event.target.value as FilterType)}
              aria-label="Account tier"
            >
              <option value="all">{t('accounts.all')} ({filterCounts.all})</option>
              <option value="pro">{t('accounts.pro')} ({filterCounts.pro})</option>
              <option value="ultra">{t('accounts.ultra')} ({filterCounts.ultra})</option>
              <option value="free">{t('accounts.free')} ({filterCounts.free})</option>
            </select>
          </label>
        </div>

        {/* Action buttons group */}
        <div className="flex items-center gap-2 shrink-0 ml-auto">
          {/* Batch operations if items selected */}
          {selectedIds.size > 0 && (
            <div className="flex items-center rounded-md bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
              <button
                className="px-2.5 py-1 text-red-600 dark:text-red-400 text-xs font-semibold rounded-l-md hover:bg-red-50 dark:hover:bg-red-950/30 transition-colors flex items-center gap-1.5 cursor-pointer"
                onClick={handleBatchDelete}
                title={t("accounts.delete_selected", { count: selectedIds.size })}
              >
                <Trash2 className="w-3.5 h-3.5" />
                <span className="hidden xl:inline">{t("accounts.delete_selected", { count: selectedIds.size })}</span>
              </button>
              <button
                className="px-2.5 py-1 text-orange-600 dark:text-orange-400 text-xs font-semibold hover:bg-orange-50 dark:hover:bg-orange-950/30 transition-colors flex items-center gap-1.5 cursor-pointer"
                onClick={() => handleBatchToggleProxy(false)}
                title={t("accounts.disable_proxy_selected", { count: selectedIds.size })}
              >
                <ToggleLeft className="w-3.5 h-3.5" />
                <span className="hidden xl:inline">{t("accounts.disable_proxy_selected", { count: selectedIds.size })}</span>
              </button>
              <button
                className="px-2.5 py-1 text-cyan-600 dark:text-cyan-400 text-xs font-semibold rounded-r-md hover:bg-cyan-50 dark:hover:bg-cyan-950/30 transition-colors flex items-center gap-1.5 cursor-pointer"
                onClick={() => handleBatchToggleProxy(true)}
                title={t("accounts.enable_proxy_selected", { count: selectedIds.size })}
              >
                <ToggleRight className="w-3.5 h-3.5" />
                <span className="hidden xl:inline">{t("accounts.enable_proxy_selected", { count: selectedIds.size })}</span>
              </button>
            </div>
          )}

          {/* Segmented Pill Capsule: Focus, +, Refresh, Warm, Show All Quotas */}
          <div className="flex items-center rounded-md bg-slate-100 dark:bg-[#0c2438] border border-slate-200/80 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
            {/* Focus Active/Picked Account Button */}
            <button
              type="button"
              className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-l-md text-amber-700 dark:text-amber-300 hover:bg-amber-100/70 dark:hover:bg-amber-950/40 transition-colors cursor-pointer"
              onClick={handleFocusActiveAccount}
              title={
                activeFocusTargetAccount
                  ? `Focus onto active account: ${activeFocusTargetAccount.email}`
                  : currentAccount
                  ? `Focus onto active account: ${currentAccount.email}`
                  : "Focus active/picked account"
              }
            >
              <LocateFixed className="w-3.5 h-3.5 text-amber-600 dark:text-amber-400" />
              <span className="hidden sm:inline">Focus</span>
            </button>

            {/* Add Account Dialog */}
            <AddAccountDialog
              onAdd={handleAddAccount}
              showText={false}
              triggerClassName="flex items-center justify-center px-2.5 py-1 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
            />

            {/* Refresh */}
            <button
              className={cn(
                "flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-blue-600 dark:text-cyan-300 hover:bg-blue-50 dark:hover:bg-[#15334d] transition-colors cursor-pointer",
                isRefreshing && "opacity-70 cursor-not-allowed"
              )}
              onClick={handleRefreshClick}
              disabled={isRefreshing}
              title={
                selectedIds.size > 0
                  ? t("accounts.refresh_selected", { count: selectedIds.size })
                  : t("accounts.refresh_all")
              }
            >
              <RefreshCw
                className={cn("w-3.5 h-3.5", isRefreshing && "animate-spin")}
              />
              <span className="hidden xl:inline">
                {isRefreshing
                  ? t("common.loading")
                  : selectedIds.size > 0
                    ? t("accounts.refresh_selected", { count: selectedIds.size })
                    : t("accounts.refresh_all")}
              </span>
            </button>

            {/* Warmup */}
            <button
              className={cn(
                "flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-orange-600 dark:text-orange-400 hover:bg-orange-50 dark:hover:bg-[#15334d] transition-colors cursor-pointer",
                isWarmuping && "opacity-70 cursor-not-allowed"
              )}
              onClick={() => setIsWarmupConfirmOpen(true)}
              disabled={isWarmuping}
              title={
                selectedIds.size > 0
                  ? t("accounts.warmup_selected", { count: selectedIds.size })
                  : t("accounts.warmup_all", "Warm up all accounts")
              }
            >
              <Sparkles
                className={cn("w-3.5 h-3.5", isWarmuping && "animate-pulse")}
              />
              <span className="hidden xl:inline">
                {isWarmuping
                  ? t("common.loading")
                  : selectedIds.size > 0
                    ? t("accounts.warmup_selected", { count: selectedIds.size })
                    : t("accounts.warmup_all", "Warm up all")}
              </span>
            </button>

            {/* Show All Emails */}
            <label
              className="flex items-center gap-2 cursor-pointer select-none px-3 py-1 rounded-r-md text-slate-600 dark:text-slate-300 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors"
              title={t("accounts.show_all_emails")}
            >
              <span className="text-xs font-semibold hidden xl:inline">
                {t("accounts.show_all_emails")}
              </span>
              <input
                type="checkbox"
                className="toggle toggle-xs toggle-primary"
                checked={showAllQuotas}
                onChange={toggleShowAllQuotas}
              />
            </label>
          </div>
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
                  <Upload className="w-3.5 h-3.5 text-cyan-600 dark:text-cyan-400" />
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
                    !!accounts.find((a) => a.id === id)?.proxy_disabled,
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
                  !!accounts.find((a) => a.id === id)?.proxy_disabled,
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
            pageSizeOptions={[50, 100, 150, 200]}
            centerContent={
              updateInfo?.has_update ? (
                <button
                  type="button"
                  onClick={async () => {
                    showToast(t("update_notification.installing_desc", "Launching official updater in background..."), "info");
                    await installUpdate();
                  }}
                  disabled={isInstalling}
                  className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-gradient-to-r from-blue-500/10 to-purple-500/10 hover:from-blue-500/20 hover:to-purple-500/20 text-blue-600 dark:text-blue-400 border border-blue-500/20 shadow-xs transition-all duration-200 cursor-pointer active:scale-95 group disabled:opacity-75 disabled:cursor-not-allowed"
                  title="A newer version is available. Click to launch installer."
                >
                  {isInstalling ? (
                    <Loader2 className="w-3.5 h-3.5 text-blue-500 animate-spin" />
                  ) : (
                    <Sparkles className="w-3.5 h-3.5 text-blue-500 group-hover:rotate-12 transition-transform" />
                  )}
                  <span>Update Available: v{updateInfo.latest_version}</span>
                  <span className="text-[10px] bg-blue-600 hover:bg-blue-500 text-white px-1.5 py-0.5 rounded font-semibold ml-0.5 flex items-center gap-1">
                    {isInstalling ? "Installing..." : "Install Now"}
                  </span>
                </button>
              ) : null
            }
          />
        </div>
      )}

      <AccountDetailsDialog
        account={accounts.find(a => a.id === detailsAccount?.id) || null}
        onClose={() => setDetailsAccount(null)}
        onUpdatePriority={updateAccountPriority}
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
        isLoading={isDeleting}
        onConfirm={isBatchDelete ? executeBatchDelete : executeDelete}
        onCancel={() => {
          if (isDeleting) return;
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
    </div>
  );
}

export default Accounts;
