import { showToast } from '../../common/ToastContainer';
import { EmailNotificationsState } from './useEmailCore';
import { telegramService, type TelegramConfig } from '../../../services/telegramService';
import { saveEmailSettings } from '../../../services/emailService';
import { useErrorStore } from '../../../stores/error-store';

export function useTelegramHandlers(s: EmailNotificationsState) {
    const {
        settings,
        setIsSaving,
        telegramConfig,
        setTelegramConfig,
        setTelegramStatus,
        setIsTestingTelegram,
        setIsDetectingChatId,
        setIsSavingTelegram,
        setIsSendingTelegramPing,
        setTelegramBotUsername,
    } = s;

    const handleSaveTelegram = async () => {
        if (!telegramConfig) return;
        setIsSavingTelegram(true);
        try {
            await telegramService.saveConfig(telegramConfig);
            showToast('Telegram bot settings saved successfully', 'success');
            const st = await telegramService.getStatus();
            setTelegramStatus(st);
        } catch (e: unknown) {
            const err = e instanceof Error ? e : new Error(String(e));
            showToast(`Failed to save Telegram settings: ${err.message}`, 'error');
            useErrorStore.getState().trackWarning(e, {
                source: 'useTelegramHandlers.handleSaveTelegram',
                triggerAction: 'save_telegram_config',
            });
        } finally {
            setIsSavingTelegram(false);
        }
    };

    const handleTestTelegram = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim()) {
            showToast('Please enter a Telegram Bot Token', 'warning');
            return;
        }
        setIsTestingTelegram(true);
        try {
            const username = await telegramService.testBot(telegramConfig.bot_token);
            setTelegramBotUsername(username);
            showToast(`Connected to Telegram bot: @${username}`, 'success');
        } catch (e: unknown) {
            setTelegramBotUsername(null);
            const err = e instanceof Error ? e : new Error(String(e));
            showToast(`Telegram connection failed: ${err.message}`, 'error');
            useErrorStore.getState().trackWarning(e, {
                source: 'useTelegramHandlers.handleTestTelegram',
                triggerAction: 'test_telegram_bot',
            });
        } finally {
            setIsTestingTelegram(false);
        }
    };

    const handleDetectTelegramChatId = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim()) {
            showToast('Please enter a Telegram Bot Token first, then send /ping to your bot in Telegram', 'warning');
            return;
        }
        setIsDetectingChatId(true);
        try {
            const res = await telegramService.detectChatId(telegramConfig.bot_token);
            setTelegramBotUsername(res.bot_username);
            const updatedConfig: TelegramConfig = {
                ...telegramConfig,
                allowed_chat_id: res.chat_id,
                is_enabled: true,
            };
            setTelegramConfig(updatedConfig);

            try {
                await telegramService.saveConfig(updatedConfig);
                const st = await telegramService.getStatus();
                setTelegramStatus(st);
                showToast('Chat ID detected & configuration saved! Telegram daemon activated.', 'success');
            } catch (saveErr: unknown) {
                useErrorStore.getState().trackWarning(saveErr, {
                    source: 'useTelegramHandlers.handleDetectTelegramChatId.autoSave',
                    triggerAction: 'save_telegram_config',
                });
                showToast(`Chat ID detected: ${res.chat_id}, but auto-save failed. Please click Save manually.`, 'warning');
            }
        } catch (e: unknown) {
            const err = e instanceof Error ? e : new Error(String(e));
            showToast(`${err.message}`, 'error');
            useErrorStore.getState().trackWarning(e, {
                source: 'useTelegramHandlers.handleDetectTelegramChatId',
                triggerAction: 'detect_telegram_chat_id',
            });
        } finally {
            setIsDetectingChatId(false);
        }
    };

    const handleSendTelegramPing = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim() || !telegramConfig.allowed_chat_id) {
            showToast('Please specify Bot Token and Allowed Chat ID first', 'warning');
            return;
        }
        setIsSendingTelegramPing(true);
        try {
            await telegramService.sendTestMessage(telegramConfig.bot_token, telegramConfig.allowed_chat_id);
            showToast('Test alert sent to your Telegram chat!', 'success');
        } catch (e: unknown) {
            const err = e instanceof Error ? e : new Error(String(e));
            showToast(`Failed to send test alert: ${err.message}`, 'error');
            useErrorStore.getState().trackWarning(e, {
                source: 'useTelegramHandlers.handleSendTelegramPing',
                triggerAction: 'send_telegram_test_message',
            });
        } finally {
            setIsSendingTelegramPing(false);
        }
    };

    const handleSaveSettings = async () => {
        setIsSaving(true);
        try {
            await saveEmailSettings(settings);
            showToast('Notification settings saved successfully', 'success');
        } catch (e: unknown) {
            const err = e instanceof Error ? e : new Error(String(e));
            showToast('Failed to save settings: ' + err.message, 'error');
            useErrorStore.getState().trackWarning(e, {
                source: 'useTelegramHandlers.handleSaveSettings',
                triggerAction: 'save_email_settings',
            });
        } finally {
            setIsSaving(false);
        }
    };

    return {
        handleSaveTelegram,
        handleTestTelegram,
        handleDetectTelegramChatId,
        handleDetectChatId: handleDetectTelegramChatId,
        handleSendTelegramPing,
        handleSaveSettings,
    };
}
