import React from 'react';
import { ProxyMonitor } from '../components/proxy/ProxyMonitor';

const Monitor: React.FC = () => {
    return (
        <div className="h-full flex flex-col px-4 sm:px-6 pt-2 pb-4 gap-4 max-w-7xl mx-auto w-full">
            <ProxyMonitor className="flex-1" />
        </div>
    );
};

export default Monitor;
