import { Outlet } from 'react-router-dom';
import Navbar from '../navbar/Navbar';
import TitleBar from './TitleBar';
import BackgroundTaskRunner from '../common/BackgroundTaskRunner';
import ToastContainer from '../common/ToastContainer';
import { useViewStore } from '../../stores/useViewStore';
import MiniView from './MiniView';
import { useEffect } from 'react';
import { isTauri } from '../../utils/env';
import { ensureFullViewState } from '../../utils/windowManager';

function Layout() {
    const { isMiniView } = useViewStore();

    // Ensure correct window state when in Full View (not Mini View)
    // This handles the case where the app was closed in Mini View (small size, no decorations)
    // and restarted (defaults to Full View state but keeps last window properties)
    useEffect(() => {
        if (isMiniView) return;
        if (isTauri()) {
            ensureFullViewState();
        }
    }, [isMiniView]);

    // Ensure WebView2 invalidates backbuffer and repaints after minimize/restore
    useEffect(() => {
        const handleRestore = () => {
            if (document.hidden) return;
            window.dispatchEvent(new Event('resize'));
        };

        window.addEventListener('focus', handleRestore);
        document.addEventListener('visibilitychange', handleRestore);

        let unlisten: (() => void) | undefined;
        if (isTauri()) {
            import('@tauri-apps/api/event').then(({ listen }) => {
                listen('window-restored', () => {
                    handleRestore();
                }).then(fn => {
                    unlisten = fn;
                });
            });
        }

        return () => {
            window.removeEventListener('focus', handleRestore);
            document.removeEventListener('visibilitychange', handleRestore);
            if (unlisten) unlisten();
        };
    }, []);

    if (isMiniView) {
        return (
            <>
                <BackgroundTaskRunner />
                <ToastContainer />
                <MiniView />
            </>
        );
    }

    return (
        <div className="h-screen flex flex-col bg-[#FAFBFC] dark:bg-base-300">
            <TitleBar />
            <BackgroundTaskRunner />
            <ToastContainer />
            <Navbar />
            <main className="flex-1 min-h-0 overflow-hidden flex flex-col relative">
                <Outlet />
            </main>
        </div>
    );
}

export default Layout;
