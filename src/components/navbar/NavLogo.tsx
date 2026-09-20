import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import LogoIcon from '../../../src-tauri/icons/icon.png';
import versionData from '../../../version.json';
import { isTauri } from '../../utils/env';

export function NavLogo() {
    const { t } = useTranslation();
    const [appVersion, setAppVersion] = useState<string>(
        versionData.version || versionData.Version || '4.29.0'
    );

    useEffect(() => {
        const isDesktop = isTauri();
        if (isDesktop) {
            import('@tauri-apps/api/app').then(({ getVersion }) => {
                getVersion().then((v) => {
                    const hasVersion = Boolean(v);
                    if (hasVersion) {
                        setAppVersion(v);
                    }
                }).catch(() => {});
            });
        }
    }, []);

    const hasVPrefix = appVersion.startsWith('v');
    const displayVersion = hasVPrefix ? appVersion : `v${appVersion}`;

    return (
        <Link to="/" draggable="false" title="Antigravity Manager Tools By Alim" className="flex w-full min-w-0 items-center gap-2 text-xl font-semibold text-gray-900 dark:text-slate-100">
            <div className="relative flex items-center justify-center">
                <img
                    src={LogoIcon}
                    alt="Logo"
                    className="w-8 h-8 cursor-pointer transition-opacity hover:opacity-90 relative z-10"
                    draggable="false"
                />
            </div>

            <span className="font-bold text-sm sm:text-base text-gray-900 dark:text-slate-100 whitespace-nowrap">
                <span className="inline sm:hidden">AGM</span>
                <span className="hidden sm:inline">{t('common.app_name', 'Agm Tool By Alim')}</span>
            </span>

            <span className="px-1.5 py-0.5 text-[10px] font-mono font-bold rounded-md bg-blue-50 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400 border border-blue-200/60 dark:border-blue-800/60 shrink-0 leading-none">
                {displayVersion}
            </span>
        </Link>
    );
}
