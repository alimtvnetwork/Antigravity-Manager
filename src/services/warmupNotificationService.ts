import { showToast } from '../components/common/ToastContainer';
import { useErrorStore } from '../stores/error-store';

export interface WarmupNotificationOptions {
    enableChime?: boolean;
    enableToast?: boolean;
}

interface TauriNotificationModule {
    sendNotification(options: string | { title?: string; body?: string }): void;
}

type AudioContextClass = typeof AudioContext;

function resolveAudioContext(): AudioContextClass | null {
    if (typeof window === 'undefined') return null;
    const win = window as unknown as {
        AudioContext?: AudioContextClass;
        webkitAudioContext?: AudioContextClass;
    };
    return win.AudioContext || win.webkitAudioContext || null;
}

function triggerAudioNode(ctx: AudioContext): void {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(440, ctx.currentTime);

    gain.gain.setValueAtTime(0.08, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 0.15);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.15);
    osc.onended = () => {
        ctx.close().catch((err: unknown) => {
            useErrorStore.getState().trackWarning(err, {
                source: 'warmupNotificationService.triggerAudioNode',
                triggerAction: 'closeAudioContext',
            });
        });
    };
}

export function playSynthesizedWarmupChime(): void {
    try {
        const AudioCtx = resolveAudioContext();
        if (!AudioCtx) return;

        const ctx = new AudioCtx();
        if (ctx.state === 'suspended') {
            ctx.resume().catch((err: unknown) => {
                useErrorStore.getState().trackWarning(err, {
                    source: 'warmupNotificationService.playSynthesizedWarmupChime',
                    triggerAction: 'resumeAudioContext',
                });
            });
        }
        triggerAudioNode(ctx);
    } catch (err: unknown) {
        useErrorStore.getState().trackWarning(err, {
            source: 'warmupNotificationService.playSynthesizedWarmupChime',
            triggerAction: 'synthesizeChime',
        });
    }
}

async function sendTauriNotification(title: string, body: string): Promise<boolean> {
    try {
        const tauriGlobal = typeof window !== 'undefined'
            ? (window as unknown as { __TAURI__?: { notification?: TauriNotificationModule } }).__TAURI__
            : undefined;

        if (tauriGlobal?.notification && typeof tauriGlobal.notification.sendNotification === 'function') {
            tauriGlobal.notification.sendNotification({ title, body });
            return true;
        }
    } catch (err: unknown) {
        useErrorStore.getState().trackWarning(err, {
            source: 'warmupNotificationService.sendTauriNotification',
            triggerAction: 'tauriNotification',
        });
    }
    return false;
}

async function sendWebNotification(title: string, body: string): Promise<boolean> {
    try {
        if (typeof window === 'undefined' || !('Notification' in window)) {
            return false;
        }
        if (Notification.permission === 'granted') {
            new Notification(title, { body });
            return true;
        }
        if (Notification.permission !== 'denied') {
            const permission = await Notification.requestPermission();
            if (permission === 'granted') {
                new Notification(title, { body });
                return true;
            }
        }
    } catch (err: unknown) {
        useErrorStore.getState().trackWarning(err, {
            source: 'warmupNotificationService.sendWebNotification',
            triggerAction: 'webNotification',
        });
    }
    return false;
}

export async function notifyWarmupCompletion(
    accountEmail: string,
    options: WarmupNotificationOptions = {}
): Promise<void> {
    const { enableChime = true, enableToast = true } = options;
    const title = 'Warmup Complete';
    const body = `Account ${accountEmail} weekly quota renewed and warmed!`;

    if (enableChime) {
        playSynthesizedWarmupChime();
    }
    if (enableToast) {
        showToast(`[Warmup Complete] ${body}`, 'success');
    }

    const isTauriDispatched = await sendTauriNotification(title, body);
    if (!isTauriDispatched) {
        await sendWebNotification(title, body);
    }
}
