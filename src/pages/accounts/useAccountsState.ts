import { useEffect, useMemo, useRef, useState } from "react";
import { useAccountStore } from "../../stores/useAccountStore";
import { useConfigStore } from "../../stores/useConfigStore";
import { useInstanceStore } from "../../stores/useInstanceStore";
import type { InstanceStatus } from "../../services/instanceService";
import { useUpdateStore } from "../../stores/use-update-store";
import { resolveFocusTarget } from "../../lib/resolve-focus-target";
import { Account } from "../../types/account";
import { useTranslation } from "react-i18next";
import type { FilterType, ViewMode } from "./types";

export function useAccountsState() {
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
    const selectedInstance = instances.find((inst: InstanceStatus) => inst.config.id === activeInstanceId);
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


  // Clear selection and reset page when filter changes
  useEffect(() => {
    setSelectedIds(new Set());
    setCurrentPage(1);
  }, [filter, searchQuery]);




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










  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isRefreshConfirmOpen, setIsRefreshConfirmOpen] = useState(false);



  // Focus onto the active instance's bound account (falling back to currentAccount)




  return {
    t,
    config,
    showAllQuotas,
    toggleShowAllQuotas,
    updateInfo,
    installUpdate,
    isInstalling,
    searchQuery,
    setSearchQuery,
    filter,
    setFilter,
    currentPage,
    setCurrentPage,
    instances,
    activeInstanceId,
    boundInstanceAccountId,
    activeFocusTargetId,
    activeFocusTargetAccount,
    isSearchExpanded,
    setIsSearchExpanded,
    viewMode,
    setViewMode,
    selectedIds,
    setSelectedIds,
    deviceAccount,
    setDeviceAccount,
    detailsAccount,
    setDetailsAccount,
    deleteConfirmId,
    setDeleteConfirmId,
    isBatchDelete,
    setIsBatchDelete,
    isDeleting,
    setIsDeleting,
    toggleProxyConfirm,
    setToggleProxyConfirm,
    isWarmupConfirmOpen,
    setIsWarmupConfirmOpen,
    isWarmuping,
    setIsWarmuping,
    refreshingIds,
    setRefreshingIds,
    errorAccountId,
    setErrorAccountId,
    focusedAccountId,
    setFocusedAccountId,
    isBackupModalOpen,
    setIsBackupModalOpen,
    backupModalTab,
    setBackupModalTab,
    containerSize,
    setContainerSize,
    localPageSize,
    setLocalPageSize,
    ITEMS_PER_PAGE,
    searchedAccounts,
    filterCounts,
    filteredAccounts,
    paginatedAccounts,
    switchingAccountId,
    setSwitchingAccountId,
    isRefreshing,
    setIsRefreshing,
    isRefreshConfirmOpen,
    setIsRefreshConfirmOpen,
    searchInputRef,
    containerRef,
    currentAccount,
    accounts,
    reorderAccounts,
    updateAccountPriority,
    fetchAccounts,
    loading,
    warmUpAccount,
    warmUpAccounts,
    updateAccountLabel,
    addAccount,
    switchAccount,
    deleteAccount,
    deleteAccounts,
    refreshQuota,
    toggleProxyStatus,
    fetchCurrentAccount
  };
}