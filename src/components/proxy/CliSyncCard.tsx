import type { ReactNode } from 'react';
import { Terminal, CodeXml, Bot, Sparkles } from 'lucide-react';
import { cn } from '../../utils/cn';
import { Claude, OpenAI, Gemini, Grok, OpenClaw } from '@lobehub/icons';
import { JeikCodeIcon } from '../common/JeikCodeIcon';
import { CliAppCard } from './cli-sync/CliAppCard';
import { CliSyncModals } from './cli-sync/CliSyncModals';
import { useCliSyncActions } from './cli-sync/useCliSyncActions';
import type { CliAppType, CliSyncCardProps } from './cli-sync/types';

interface CliAppDef {
    app: CliAppType;
    name: string;
    icon: ReactNode;
}

const CLI_APPS: CliAppDef[] = [
    { app: 'JeikCode', name: 'JeikCode', icon: <JeikCodeIcon size="100%" className="w-full h-full" /> },
    {
        app: 'Claude', name: 'Claude Code', icon: (
            <div className="w-full h-full bg-[#D97757] flex items-center justify-center text-white">
                <Claude size={22} className="text-white" />
            </div>
        ),
    },
    {
        app: 'Codex', name: 'Codex AI', icon: (
            <div className="w-full h-full bg-[#000000] dark:bg-[#121214] flex items-center justify-center text-white">
                <OpenAI size={22} className="text-white" />
            </div>
        ),
    },
    {
        app: 'OpenCode', name: 'OpenCode', icon: (
            <div className="w-full h-full bg-[#000000] dark:bg-[#121214] flex items-center justify-center text-white">
                <CodeXml size={22} className="text-cyan-400" />
            </div>
        ),
    },
    {
        app: 'GrokBuild', name: 'Grok Build', icon: (
            <div className="w-full h-full bg-[#000000] dark:bg-[#121214] flex items-center justify-center text-white">
                <Grok size={22} className="text-white" />
            </div>
        ),
    },
    {
        app: 'Hermes', name: 'Hermes Agent', icon: (
            <div className="w-full h-full bg-gradient-to-br from-purple-700 via-indigo-700 to-violet-800 flex items-center justify-center text-white">
                <Sparkles size={22} className="text-purple-200" />
            </div>
        ),
    },
    {
        app: 'OpenClaw', name: 'OpenClaw', icon: (
            <div className="w-full h-full bg-[#18181b] dark:bg-[#121214] flex items-center justify-center text-white">
                <OpenClaw.Color size={26} />
            </div>
        ),
    },
    {
        app: 'Gemini', name: 'Gemini CLI', icon: (
            <div className="w-full h-full bg-[#18181b] dark:bg-[#121214] flex items-center justify-center">
                <Gemini.Color size={24} />
            </div>
        ),
    },
    {
        app: 'Droid', name: 'Droid', icon: (
            <div className="w-full h-full bg-gradient-to-br from-orange-500 via-amber-500 to-orange-600 flex items-center justify-center text-white">
                <Bot size={22} className="text-white drop-shadow-sm" />
            </div>
        ),
    },
];

export const CliSyncCard = ({ proxyUrl, apiKey, className }: CliSyncCardProps) => {
    const a = useCliSyncActions(proxyUrl, apiKey);

    return (
        <div className={cn("space-y-4", className)}>
            <div className="px-1 flex items-center justify-between">
                <div className="flex items-center gap-2 text-gray-400">
                    <Terminal size={14} />
                    <span className="text-[10px] font-bold uppercase tracking-widest">
                        {a.t('proxy.cli_sync.title')}
                    </span>
                </div>
                <p className="text-[10px] text-gray-400 dark:text-gray-500 italic">
                    {a.t('proxy.cli_sync.subtitle')}
                </p>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {CLI_APPS.map(({ app, name, icon }) => (
                    <CliAppCard
                        key={app}
                        t={a.t}
                        app={app}
                        icon={icon}
                        name={name}
                        status={a.statuses[app]}
                        isAppLoading={a.loading[app]}
                        isAppSyncing={a.syncing[app]}
                        selectedModel={a.selectedModels[app]}
                        onSelectModel={(val) => a.setSelectedModels(prev => ({ ...prev, [app]: val }))}
                        modelOptions={a.modelOptions}
                        syncAccounts={a.syncAccounts}
                        onToggleSyncAccounts={a.setSyncAccounts}
                        onSync={() => a.handleSync(app)}
                        onRestore={() => a.handleRestore(app)}
                        onClear={() => a.handleClear(app)}
                        onViewConfig={() => void a.handleViewConfig(app)}
                        onOpenExternalUrl={(url) => void a.openExternalUrl(url)}
                    />
                ))}
            </div>

            <CliSyncModals
                t={a.t}
                proxyUrl={proxyUrl}
                apiKey={apiKey}
                statuses={a.statuses}
                viewingConfig={a.viewingConfig}
                onCloseViewingConfig={() => a.setViewingConfig(null)}
                onViewConfigFile={(app, file) => void a.handleViewConfig(app, file)}
                restoreConfirmApp={a.restoreConfirmApp}
                onCancelRestore={() => a.setRestoreConfirmApp(null)}
                onConfirmRestore={() => void a.executeRestore()}
                syncConfirmApp={a.syncConfirmApp}
                onCancelSync={() => a.setSyncConfirmApp(null)}
                onConfirmSync={() => void a.executeSync()}
                clearConfirmApp={a.clearConfirmApp}
                onCancelClear={() => a.setClearConfirmApp(null)}
                onConfirmClear={() => void a.executeClear()}
                droidSyncModal={a.droidSyncModal}
                onCloseDroidModal={() => a.setDroidSyncModal(false)}
                onDroidSyncDone={() => void a.checkStatus('Droid')}
                hermesSyncModal={a.hermesSyncModal}
                onCloseHermesModal={() => a.setHermesSyncModal(false)}
                onHermesSyncDone={() => void a.checkStatus('Hermes')}
                openClawSyncModal={a.openClawSyncModal}
                onCloseOpenClawModal={() => a.setOpenClawSyncModal(false)}
                onOpenClawSyncDone={() => void a.checkStatus('OpenClaw')}
                openCodeSyncModal={a.openCodeSyncModal}
                onCloseOpenCodeModal={() => a.setOpenCodeSyncModal(false)}
                onOpenCodeSyncDone={() => void a.checkStatus('OpenCode')}
                syncAccounts={a.syncAccounts}
                getFormattedProxyUrl={a.getFormattedProxyUrl}
            />
        </div>
    );
};

export default CliSyncCard;
