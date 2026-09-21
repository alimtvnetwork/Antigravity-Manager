import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../stores/error-store';

export interface TelegramConfig {
    bot_token: string;
    allowed_chat_id: number | null;
    is_enabled: boolean;
    poll_interval_secs: number;
}

export interface TelegramWatcherStatus {
    is_running: boolean;
    last_poll_at: number;
    last_update_id: number;
    last_message_received: string | null;
    error_message: string | null;
}

export const telegramService = {
    async getConfig(): Promise<TelegramConfig> {
        try {
            return await invoke<TelegramConfig>('get_telegram_config');
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'TelegramService' });
            throw error;
        }
    },

    async saveConfig(config: TelegramConfig): Promise<void> {
        try {
            await invoke('save_telegram_config', { config });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'TelegramService' });
            throw error;
        }
    },

    async testBot(botToken: string): Promise<string> {
        try {
            return await invoke<string>('test_telegram_bot', { botToken });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'TelegramService' });
            throw error;
        }
    },

    async getStatus(): Promise<TelegramWatcherStatus> {
        try {
            return await invoke<TelegramWatcherStatus>('get_telegram_status');
        } catch (error) {
            return {
                is_running: false,
                last_poll_at: 0,
                last_update_id: 0,
                last_message_received: null,
                error_message: String(error),
            };
        }
    },

    async sendTestMessage(botToken: string, chatId: number): Promise<void> {
        try {
            await invoke('send_telegram_test_message', { botToken, chatId });
        } catch (error) {
            useErrorStore.getState().captureError(error, { source: 'TelegramService' });
            throw error;
        }
    },
};
