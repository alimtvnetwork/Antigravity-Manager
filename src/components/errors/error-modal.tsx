import React, { useState } from 'react';
import { useErrorStore } from '../../stores/error-store';
import {
  generateCompactReport,
  generateComprehensiveAllDataReport,
  generateJsonReport,
} from '../../lib/error-report-generator';
import { showToast } from '../common/ToastContainer';
import { ModalHeader, TabNav, ModalFooter } from './error-modal/chrome';
import { OverviewTab, BackendTab, StackTab, ContextTab } from './error-modal/tabs';

export function ErrorModal(): React.ReactNode {
  const {
    selectedError,
    isModalOpen,
    closeErrorModal,
    errorQueue,
    currentQueueIndex,
    navigateQueue,
    activeTab,
    setActiveTab,
    removeError,
  } = useErrorStore();

  const [copiedAi, setCopiedAi] = useState(false);
  const [copiedAll, setCopiedAll] = useState(false);

  if (!isModalOpen) {
    return null;
  }
  if (!selectedError) {
    return null;
  }

  const handleCopyAi = () => {
    const text = generateCompactReport(selectedError);
    navigator.clipboard.writeText(text);
    setCopiedAi(true);
    setTimeout(() => setCopiedAi(false), 2000);
    showToast('Diagnostic AI report copied! Ready to share with AI.', 'success');
  };

  const handleCopyAllData = () => {
    const text = generateComprehensiveAllDataReport(selectedError);
    navigator.clipboard.writeText(text);
    setCopiedAll(true);
    setTimeout(() => setCopiedAll(false), 2000);
    showToast('All comprehensive error data copied to clipboard!', 'success');
  };

  const handleCopyJson = () => {
    const text = generateJsonReport(selectedError);
    navigator.clipboard.writeText(text);
    showToast('Raw error JSON copied to clipboard.', 'info');
  };

  const handleRemove = () => {
    removeError(selectedError.id);
    showToast(`Removed error [${selectedError.code}] from history`, 'info');
  };

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200">
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-2xl shadow-2xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden text-slate-900 dark:text-slate-100">
        <ModalHeader
          error={selectedError}
          queueLength={errorQueue.length}
          queueIndex={currentQueueIndex}
          onNavigate={navigateQueue}
          onClose={closeErrorModal}
          onCopyAllData={handleCopyAllData}
          onRemove={handleRemove}
          copiedAll={copiedAll}
        />

        <TabNav activeTab={activeTab} onSelect={setActiveTab} />

        <div className="flex-1 overflow-y-auto p-6 space-y-4">
          {activeTab === 'overview' && (
            <OverviewTab error={selectedError} onSelectTab={setActiveTab} />
          )}
          {activeTab === 'backend' && <BackendTab error={selectedError} />}
          {activeTab === 'stack' && <StackTab error={selectedError} />}
          {activeTab === 'context' && <ContextTab error={selectedError} />}
        </div>

        <ModalFooter
          copiedAi={copiedAi}
          copiedAll={copiedAll}
          onCopyAi={handleCopyAi}
          onCopyAllData={handleCopyAllData}
          onCopyJson={handleCopyJson}
          onRemove={handleRemove}
          onClose={closeErrorModal}
        />
      </div>
    </div>
  );
}
