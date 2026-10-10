import React from 'react';
import { useAutoSwitcher } from './AutoSwitcher/useAutoSwitcher';
import { SwitcherHeader } from './AutoSwitcher/SwitcherHeader';
import { PollingSettings } from './AutoSwitcher/PollingSettings';
import { ModelSettings } from './AutoSwitcher/ModelSettings';
import { TaskContinuitySettings } from './AutoSwitcher/TaskContinuitySettings';
import { ManualRotation } from './AutoSwitcher/ManualRotation';
import { ScoringAlgorithm } from './AutoSwitcher/ScoringAlgorithm';
import { HowItWorksGuide } from './AutoSwitcher/HowItWorksGuide';
import type { AutoSwitcherSettingsProps } from './AutoSwitcher/switcherTypes';

export const AutoSwitcherSettings: React.FC<AutoSwitcherSettingsProps> = ({ config, onChange }) => {
    const api = useAutoSwitcher(config, onChange);
    const { currentConfig } = api;

    return (
        <div className="space-y-4">
            <SwitcherHeader {...api} />
            {currentConfig.is_enabled && (
                <>
                    <PollingSettings {...api} />
                    <ModelSettings {...api} />
                    <TaskContinuitySettings {...api} />
                    <ManualRotation {...api} />
                </>
            )}
            <ScoringAlgorithm {...api} />
            <HowItWorksGuide {...api} />
        </div>
    );
};

export default AutoSwitcherSettings;
