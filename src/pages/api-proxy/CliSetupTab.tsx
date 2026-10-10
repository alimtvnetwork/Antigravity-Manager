import type { AppConfig } from '../../types/config';
import type { ProxyStatus } from './types';
import { CliSyncCard } from '../../components/proxy/CliSyncCard';
import { DEFAULT_PROXY_PORT } from './constants';

interface CliSetupTabProps {
    config: AppConfig;
    status: ProxyStatus;
}

export function CliSetupTab({ config, status }: CliSetupTabProps) {
    return (
        <div className="p-4">
            <div className="bg-gray-50/60 dark:bg-base-200/50 rounded-xl p-3 border border-gray-200/60 dark:border-base-300">
                <CliSyncCard
                    proxyUrl={status.running ? status.base_url : `http://127.0.0.1:${config.proxy.port || DEFAULT_PROXY_PORT}`}
                    apiKey={config.proxy.api_key}
                />
            </div>
        </div>
    );
}
