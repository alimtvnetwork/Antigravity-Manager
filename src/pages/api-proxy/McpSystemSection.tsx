import type { TFunction } from 'i18next';
import { Puzzle } from 'lucide-react';
import type { AppConfig, ZaiMcpConfig } from '../../types/config';
import type { ProxyStatus } from './types';
import type { ZaiDispatcher } from './useZaiDispatcher';
import { DEFAULT_PROXY_PORT } from './constants';
import { CollapsibleCard } from './CollapsibleCard';

interface McpSystemSectionProps {
    config: AppConfig;
    t: TFunction;
    status: ProxyStatus;
    zai: Pick<ZaiDispatcher, 'updateZaiGeneralConfig'>;
}

const MCP_FEATURES = ['web_search', 'web_reader', 'vision'] as const;

export function McpSystemSection({ config, t, status, zai }: McpSystemSectionProps) {
    const { updateZaiGeneralConfig } = zai;
    const mcp = config.proxy.zai?.mcp;
    const port = status.running ? status.port : (config.proxy.port || DEFAULT_PROXY_PORT);

    const setMcpFlags = (updates: Partial<ZaiMcpConfig>) => {
        updateZaiGeneralConfig({ mcp: { ...(mcp || {}), ...updates } as ZaiMcpConfig });
    };

    return (
        <CollapsibleCard
            title={t('proxy.config.zai.mcp.title')}
            icon={<Puzzle size={18} className="text-blue-500" />}
            enabled={!!mcp?.enabled}
            onToggle={(checked) => setMcpFlags({ enabled: checked })}
            rightElement={
                <div className="flex gap-2 text-[10px]">
                    {MCP_FEATURES.map(f =>
                        mcp?.[`${f}_enabled` as keyof ZaiMcpConfig] && (
                            <span key={f} className="bg-blue-500 dark:bg-blue-600 px-1.5 py-0.5 rounded text-white font-semibold shadow-sm">
                                {t(`proxy.config.zai.mcp.${f}`).split(' ')[0]}
                            </span>
                        )
                    )}
                </div>
            }
        >
            <div className="space-y-3">
                <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                    <label className="flex items-center gap-2 border border-gray-100 dark:border-base-200 p-2 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-base-200/50 transition-colors">
                        <input
                            type="checkbox"
                            className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                            checked={!!mcp?.web_search_enabled}
                            onChange={(e) => setMcpFlags({ web_search_enabled: e.target.checked })}
                        />
                        <span className="text-xs">{t('proxy.config.zai.mcp.web_search')}</span>
                    </label>
                    <label className="flex items-center gap-2 border border-gray-100 dark:border-base-200 p-2 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-base-200/50 transition-colors">
                        <input
                            type="checkbox"
                            className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                            checked={!!mcp?.web_reader_enabled}
                            onChange={(e) => setMcpFlags({ web_reader_enabled: e.target.checked })}
                        />
                        <span className="text-xs">{t('proxy.config.zai.mcp.web_reader')}</span>
                    </label>
                    <label className="flex items-center gap-2 border border-gray-100 dark:border-base-200 p-2 rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-base-200/50 transition-colors">
                        <input
                            type="checkbox"
                            className="checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                            checked={!!mcp?.vision_enabled}
                            onChange={(e) => setMcpFlags({ vision_enabled: e.target.checked })}
                        />
                        <span className="text-xs">{t('proxy.config.zai.mcp.vision')}</span>
                    </label>
                </div>

                {mcp?.enabled && (
                    <div className="bg-slate-100 dark:bg-slate-800/80 rounded-lg p-3 text-[10px] font-mono text-slate-600 dark:text-slate-400">
                        <div className="mb-1 font-bold text-gray-400 uppercase tracking-wider">{t('proxy.config.zai.mcp.local_endpoints')}</div>
                        <div className="space-y-0.5 select-all">
                            {mcp?.web_search_enabled && <div>http://127.0.0.1:{port}/mcp/web_search_prime/mcp</div>}
                            {mcp?.web_reader_enabled && <div>http://127.0.0.1:{port}/mcp/web_reader/mcp</div>}
                            {mcp?.vision_enabled && <div>http://127.0.0.1:{port}/mcp/zai-mcp-server/mcp</div>}
                        </div>
                    </div>
                )}
            </div>
        </CollapsibleCard>
    );
}
