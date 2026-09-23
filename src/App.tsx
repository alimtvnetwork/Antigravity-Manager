import { createBrowserRouter, RouterProvider, Navigate } from 'react-router-dom';

import Layout from './components/layout/Layout';
import Dashboard from './pages/Dashboard';
import Accounts from './pages/Accounts';
import Instances from './pages/Instances';
import Settings from './pages/Settings';
import ApiProxy from './pages/ApiProxy';

import Monitor from './pages/Monitor';
import TokenStats from './pages/TokenStats';
import Security from './pages/Security';
import Email from './pages/Email';
import ThemeManager from './components/common/ThemeManager';
import UserToken from './pages/UserToken';
import { ApiKeyFun } from './pages/ApiKeyFun';
import { UpdateNotification } from './components/UpdateNotification';
import DebugConsole from './components/debug/DebugConsole';
import { useEffect } from 'react';
import { useConfigStore } from './stores/useConfigStore';
import { useAccountStore } from './stores/useAccountStore';
import { useUpdateStore } from './stores/use-update-store';
import { useTranslation } from 'react-i18next';
import { listen } from '@tauri-apps/api/event';
import { isTauri } from './utils/env';
import { request as invoke } from './utils/request';
import { AdminAuthGuard } from './components/common/AdminAuthGuard';
import { ErrorModal } from './components/errors/error-modal';
import { initGlobalErrorListeners } from './lib/error-listener';

const router = createBrowserRouter([
  {
    path: '/',
    element: <Layout />,
    children: [
      {
        index: true,
        element: <Navigate to="/accounts" replace />,
      },
      {
        path: 'dashboard',
        element: <Dashboard />,
      },
      {
        path: 'accounts',
        element: <Accounts />,
      },
      {
        path: 'instances',
        element: <Instances />,
      },
      {
        path: 'api-proxy',

        element: <ApiProxy />,
      },
      {
        path: 'monitor',
        element: <Monitor />,
      },
      {
        path: 'token-stats',
        element: <TokenStats />,
      },
      {
        path: 'user-token',
        element: <UserToken />,
      },
      {
        path: 'apikey-fun',
        element: <ApiKeyFun />,
      },
      {
        path: 'security',
        element: <Security />,
      },
      {
        path: 'email',
        element: <Email />,
      },
      {
        path: 'settings',
        element: <Settings />,
      },
    ],
  },
]);

function App() {
  const { config, loadConfig } = useConfigStore();
  const { fetchCurrentAccount, fetchAccounts } = useAccountStore();
  const { i18n } = useTranslation();

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  // Sync language from config
  useEffect(() => {
    if (config?.language) {
      i18n.changeLanguage(config.language);
      // Support RTL
      if (config.language === 'ar') {
        document.documentElement.dir = 'rtl';
      } else {
        document.documentElement.dir = 'ltr';
      }
    }
  }, [config?.language, i18n]);

  // Listen for tray events
  useEffect(() => {
    if (!isTauri()) return;
    const unlistenPromises: Promise<() => void>[] = [];

    // 监听托盘切换账号事件
    unlistenPromises.push(
      listen('tray://account-switched', () => {
        console.log('[App] Tray account switched, refreshing...');
        fetchCurrentAccount();
        fetchAccounts();
      })
    );

    // 监听托盘刷新事件
    unlistenPromises.push(
      listen('tray://refresh-current', () => {
        console.log('[App] Tray refresh triggered, refreshing...');
        fetchCurrentAccount();
        fetchAccounts();
      })
    );

    // 监听后端全量刷新事件 (Command / Scheduler)
    unlistenPromises.push(
      listen('accounts://refreshed', () => {
        console.log('[App] Backend triggered quota refresh, syncing UI...');
        fetchCurrentAccount();
        fetchAccounts();
      })
    );

    // Cleanup
    return () => {
      Promise.all(unlistenPromises).then(unlisteners => {
        unlisteners.forEach(unlisten => unlisten());
      });
    };
  }, [fetchCurrentAccount, fetchAccounts]);

  // Update notification state from central store
  const { showNotification, setShowNotification, checkForUpdates } = useUpdateStore();

  // Check for updates on startup
  useEffect(() => {
    const checkUpdates = async () => {
      try {
        const shouldCheck = await invoke<boolean>('should_check_updates_on_startup').catch(() => true);
        if (shouldCheck) {
          await checkForUpdates();
          await invoke('update_last_check_time').catch(() => {});
        }
      } catch (error) {
        console.error('Failed to check update settings:', error);
      }
    };

    // Delay check to avoid blocking initial render
    const timer = setTimeout(checkUpdates, 2000);
    return () => clearTimeout(timer);
  }, [checkForUpdates]);

  // Initialize global error, unhandled rejection, and click-trail listeners
  useEffect(() => {
    const cleanup = initGlobalErrorListeners();
    return cleanup;
  }, []);

  return (
    <AdminAuthGuard>
      <ThemeManager />
      <DebugConsole />
      <ErrorModal />
      {showNotification && (
        <UpdateNotification onClose={() => setShowNotification(false)} />
      )}
      <RouterProvider router={router} />
    </AdminAuthGuard>
  );
}

export default App;
