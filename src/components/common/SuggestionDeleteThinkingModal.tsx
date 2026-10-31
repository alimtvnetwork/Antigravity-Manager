import React, { useEffect, useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import ModalDialog from './ModalDialog';
import { useConfigStore } from '../../stores/useConfigStore';
import { request } from '../../utils/request';
import { showToast } from './ToastContainer';
import { useErrorStore } from '../../stores/error-store';

// ============================================================================
// Developer Release Control Configuration
// ============================================================================
// When you need to release a new version and control whether to remind users to delete the think block cache, modify the following two configuration items:
//
// 1. SUGGESTION_DELETE_THINKING_STORE:
//    - true: Enable this version recommendation (if it is detected that an old user has upgraded and has an old cache, a pop-up window will pop up to remind you)
//    - false: Turn off suggestions (any user upgrade will not be disturbed by pop-ups)
export const SUGGESTION_DELETE_THINKING_STORE = false;

// 2. SUGGESTION_TARGET_VERSION:
//    - Suggested target version number/batch number for cleanup (e.g. "4.8.2").
//    - As long as you update this version number to the currently released new version, old users will get a pop-up suggestion after upgrading.
//    - Regardless of whether the user clicks [Delete Now] or [Don’t Delete Now], the confirmed version will be recorded locally and pop-ups will never pop up again.
export const SUGGESTION_TARGET_VERSION = '4.8.1';

export const SuggestionDeleteThinkingModal: React.FC = () => {
  const { t } = useTranslation();
  const { config, saveConfig } = useConfigStore();
  const [isOpen, setIsOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const checkedRef = useRef(false);

  useEffect(() => {
    if (!config || checkedRef.current) return;

    const checkSuggestion = async () => {
      checkedRef.current = true;

      // 1. If the developer has not enabled this version, it is recommended to delete it and exit directly.
      if (!SUGGESTION_DELETE_THINKING_STORE) {
        return;
      }

      // 2. If the user has already made a choice in this version (cleaned or canceled), there will be no more pop-up harassment permanently.
      if (config.dismissed_thinking_cleanup_version === SUGGESTION_TARGET_VERSION) {
        return;
      }

      // 3. Check whether there is a historical thinking block record in the current SQLite database
      try {
        const res = await request<number | { count?: number }>('get_thinking_store_count');
        const count = typeof res === 'number' ? res : res?.count ?? 0;

        if (count === 0) {
          // For new installation users or if there is currently no old thinking block record, the current version is silently marked as aligned to avoid pop-ups disturbing new users.
          await saveConfig(
            {
              ...config,
              suggestion_delete_thinking_store: true,
              thinking_cleanup_dismissed: true,
              dismissed_thinking_cleanup_version: SUGGESTION_TARGET_VERSION,
            },
            true
          );
          return;
        }

        // For old users and old historical data exists locally, the pop-up window will be displayed with delay.
        setTimeout(() => {
          setIsOpen(true);
        }, 1200);
      } catch (err) {
        console.warn('[SuggestionDeleteThinking] Failed to query thinking count:', err);
        // Tracked in the error module; suggestion prompt simply won't appear, safe to continue.
        useErrorStore.getState().trackWarning(err, {
          source: 'SuggestionDeleteThinkingModal.checkSuggestion',
          triggerAction: 'query_thinking_count',
        });
      }
    };

    checkSuggestion();
  }, [config, saveConfig]);

  const handleConfirm = async () => {
    if (!config) return;
    setIsDeleting(true);
    try {
      await request('clear_thinking_store');
      await saveConfig(
        {
          ...config,
          suggestion_delete_thinking_store: true,
          thinking_cleanup_dismissed: true,
          dismissed_thinking_cleanup_version: SUGGESTION_TARGET_VERSION,
        },
        true
      );
      showToast(t('suggestion_delete_thinking.success'), 'success');
      setIsOpen(false);
    } catch (err: any) {
      console.error('[SuggestionDeleteThinking] Clear error:', err);
      // Tracked in the error module; user already notified via error toast.
      useErrorStore.getState().trackWarning(err, {
        source: 'SuggestionDeleteThinkingModal.handleClear',
        triggerAction: 'clear_thinking_store',
      });
      showToast(String(err), 'error');
    } finally {
      setIsDeleting(false);
    }
  };

  const handleCancel = async () => {
    if (!config) return;
    try {
      // Write the suggestion as false, and record that the user has confirmed this version, and the window will never pop up next time it is started.
      await saveConfig(
        {
          ...config,
          suggestion_delete_thinking_store: false,
          thinking_cleanup_dismissed: true,
          dismissed_thinking_cleanup_version: SUGGESTION_TARGET_VERSION,
        },
        true
      );
    } catch (err) {
      console.warn('[SuggestionDeleteThinking] Save dismissed state error:', err);
      // Tracked in the error module; dismissal simply won't persist, safe to continue.
      useErrorStore.getState().trackWarning(err, {
        source: 'SuggestionDeleteThinkingModal.handleDismiss',
        triggerAction: 'save_dismissed_state',
      });
    }
    setIsOpen(false);
  };

  if (!isOpen) return null;

  return (
    <ModalDialog
      isOpen={isOpen}
      title={t('suggestion_delete_thinking.title')}
      message={t('suggestion_delete_thinking.message')}
      type="confirm"
      isDestructive={true}
      isLoading={isDeleting}
      confirmText={t('suggestion_delete_thinking.confirm')}
      cancelText={t('suggestion_delete_thinking.cancel')}
      onConfirm={handleConfirm}
      onCancel={handleCancel}
    />
  );
};

export default SuggestionDeleteThinkingModal;
