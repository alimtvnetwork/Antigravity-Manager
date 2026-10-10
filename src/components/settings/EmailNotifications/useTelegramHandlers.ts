import { showToast } from '../common/ToastContainer';
import { EmailNotificationsState } from './useEmailCore';

export function useTelegramHandlers(s: EmailNotificationsState) {
    const { settings, setIsSaving, telegramConfig, setTelegramConfig, setTelegramStatus, setIsTestingTelegram, setIsDetectingChatId, setIsSavingTelegram, setIsSendingTelegramPing, setTelegramBotUsername } = s;

    const handleSaveTelegram = async () => {
        if (!telegramConfig) return;
        setIsSavingTelegram(true);
        try {
            await telegramService.saveConfig(telegramConfig);
            showToast('Telegram bot settings saved successfully', 'success');
            const st = await telegramService.getStatus();
            setTelegramStatus(st);
        } catch (e: any) {
            showToast(`Failed to save Telegram settings: ${e?.message || e}`, 'error');
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
        } catch (e: any) {
            setTelegramBotUsername(null);
            showToast(`Telegram connection failed: ${e?.message || e}`, 'error');
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
            setTelegramConfig((prev) =>
                prev
                    ? { ...prev, allowed_chat_id: res.chat_id, is_enabled: true }
                    : {
                          bot_token: telegramConfig.bot_token,
                          allowed_chat_id: res.chat_id,
                          is_enabled: true,
                          poll_interval_secs: 5,
                      }
            );
            showToast(`Detected Chat ID ${res.chat_id} (${res.chat_label}) for @${res.bot_username}!`, 'success');
        } catch (e: any) {
            showToast(`${e?.message || e}`, 'error');
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
        } catch (e: any) {
            showToast(`Failed to send test alert: ${e?.message || e}`, 'error');
        } finally {
            setIsSendingTelegramPing(false);
        }
    };

    const handleSaveSettings = async () => {
        setIsSaving(true);
        try {
            await saveEmailSettings(settings);
            showToast('Notification settings saved successfully', 'success');
        } catch (e: any) {
            showToast('Failed to save settings: ' + (e?.message || e), 'error');
        } finally {
            setIsSaving(false);
        }
    };



    return {
        handleSaveTelegram,
        st,
        handleTestTelegram,
        username,
        handleDetectTelegramChatId,
        res,
        handleSendTelegramPing,
        handleSaveSettings,
    };
}
