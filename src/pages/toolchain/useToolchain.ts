import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useErrorStore } from '../../stores/error-store';
import { useTranslation } from 'react-i18next';

/** One installable toolchain item (from backend `toolchain_list_items`). */
import type { ToolchainItem, CheckEntry, Profile, ProgressPayload } from './types';
import { FALLBACK_ITEMS, PROFILES } from './data';

export function useToolchain() {
    const { t } = useTranslation();
    const [items, setItems] = useState<ToolchainItem[]>(FALLBACK_ITEMS);
    const [loadingItems, setLoadingItems] = useState(true);
    const [selectedItems, setSelectedItems] = useState<string[]>(['rust']);
    const [force, setForce] = useState(false);
    const [quiet, setQuiet] = useState(false);
    const [skipNativeDeps, setSkipNativeDeps] = useState(false);
    const [sshTarget, setSshTarget] = useState('');
    const [sshError, setSshError] = useState('');
    const [installing, setInstalling] = useState(false);
    const [log, setLog] = useState<string[]>([]);
    const [checkReport, setCheckReport] = useState<CheckEntry[] | null>(null);
    const [checking, setChecking] = useState(false);
    const [openTrouble, setOpenTrouble] = useState<number | null>(null);
    const logRef = useRef<HTMLDivElement>(null);

    const refreshItems = useCallback(async () => {
        setLoadingItems(true);
        try {
            const list = await invoke<ToolchainItem[]>('toolchain_list_items');
            if (Array.isArray(list) && list.length > 0) setItems(list);
        } catch (e) {
            /* backend not wired yet — keep fallback catalog */
            useErrorStore.getState().trackWarning(e, {
                source: 'Toolchain.loadItems',
                triggerAction: 'toolchain_list_items',
            });
        } finally {
            setLoadingItems(false);
        }
    }, []);

    useEffect(() => {
        void refreshItems();
    }, [refreshItems]);

    // Live progress events from the backend install run.
    useEffect(() => {
        let unlisten: UnlistenFn | undefined;
        void listen<ProgressPayload | string>('toolchain-progress', (event) => {
            const payload = event.payload;
            const line = typeof payload === 'string' ? payload : payload.line;
            setLog((prev) => [...prev.slice(-499), String(line)]);
        }).then((fn) => {
            unlisten = fn;
        });
        return () => {
            unlisten?.();
        };
    }, []);

    // Auto-scroll the terminal to the bottom on new output.
    useEffect(() => {
        const el = logRef.current;
        if (el) el.scrollTop = el.scrollHeight;
    }, [log]);

    const itemById = useCallback(
        (id: string) => items.find((i) => i.id === id),
        [items],
    );

    const toggleItem = (id: string) => {
        setSelectedItems((prev) =>
            prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id],
        );
    };

    const toggleProfile = (profile: Profile) => {
        const realItems = profile.items;
        const allSelected = realItems.every((id) => selectedItems.includes(id));
        setSelectedItems((prev) =>
            allSelected
                ? prev.filter((id) => !realItems.includes(id))
                : [...new Set([...prev, ...realItems])],
        );
    };

    const profileActive = (profile: Profile) => {
        const realItems = profile.items;
        return realItems.length > 0 && realItems.every((id) => selectedItems.includes(id));
    };

    const validateSsh = (value: string): boolean => {
        const v = value.trim();
        if (v === '') {
            setSshError('');
            return true;
        }
        if (/^[a-zA-Z0-9._-]+@[a-zA-Z0-9._-]+$/.test(v)) {
            setSshError('');
            return true;
        }
        setSshError('Must be user@host (key auth only, no passwords).');
        return false;
    };

    const runCheck = async () => {
        setChecking(true);
        setCheckReport(null);
        try {
            const report = await invoke<CheckEntry[]>('toolchain_check');
            setCheckReport(Array.isArray(report) ? report : []);
        } catch (e) {
            setCheckReport([
                {
                    id: 'backend',
                    label: 'Backend',
                    ok: false,
                    hint: e instanceof Error ? e.message : 'toolchain_check failed',
                },
            ]);
        } finally {
            setChecking(false);
        }
    };

    const runInstall = async () => {
        if (!validateSsh(sshTarget)) return;
        if (selectedItems.length === 0) return;
        setInstalling(true);
        setLog([]);
        try {
            await invoke('toolchain_install', {
                items: selectedItems,
                profiles: PROFILES.filter(profileActive).map((p: Profile) => p.id),
                force,
                quiet,
                skipNativeDeps,
                sshTarget: sshTarget.trim() === '' ? null : sshTarget.trim(),
            });
            setLog((prev) => [...prev, '── install finished ──']);
        } catch (e) {
            setLog((prev) => [
                ...prev,
                `[ERROR] ${e instanceof Error ? e.message : String(e)}`,
            ]);
        } finally {
            setInstalling(false);
            void refreshItems();
        }
    };

    return {
        t,
        items,
        setItems,
        loadingItems,
        setLoadingItems,
        selectedItems,
        setSelectedItems,
        force,
        setForce,
        quiet,
        setQuiet,
        skipNativeDeps,
        setSkipNativeDeps,
        sshTarget,
        setSshTarget,
        sshError,
        setSshError,
        installing,
        setInstalling,
        log,
        setLog,
        checkReport,
        setCheckReport,
        checking,
        setChecking,
        openTrouble,
        setOpenTrouble,
        refreshItems,
        itemById,
        toggleItem,
        toggleProfile,
        profileActive,
        runCheck,
        runInstall,
        validateSsh,
        logRef
    };
}