/**
 * Unified Backup Encryption and Decryption Utility
 * Uses native Web Crypto API (AES-GCM 256-bit with PBKDF2 key derivation)
 */

export interface EncryptedBackupEnvelope {
    version: 1;
    encrypted: true;
    kdf: 'PBKDF2-SHA256';
    iterations: number;
    salt: string; // base64
    iv: string; // base64
    ciphertext: string; // base64
    hint: string;
}

export interface PlainBackupEnvelope {
    version: 1;
    encrypted: false;
    created_at: string;
    backup_type: 'accounts' | 'full';
    data: any;
}

export type BackupEnvelope = EncryptedBackupEnvelope | PlainBackupEnvelope;

function bufferToBase64(buffer: ArrayBuffer | Uint8Array): string {
    const bytes = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
}

function base64ToBuffer(base64: string): Uint8Array {
    const binary = window.atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
}

async function deriveKey(password: string, salt: Uint8Array, iterations = 100000): Promise<CryptoKey> {
    const enc = new TextEncoder();
    const keyMaterial = await window.crypto.subtle.importKey(
        'raw',
        enc.encode(password),
        { name: 'PBKDF2' },
        false,
        ['deriveKey']
    );

    return window.crypto.subtle.deriveKey(
        {
            name: 'PBKDF2',
            salt: salt,
            iterations,
            hash: 'SHA-256',
        },
        keyMaterial,
        { name: 'AES-GCM', length: 256 },
        false,
        ['encrypt', 'decrypt']
    );
}

/**
 * Create a backup envelope (either plain JSON or password-encrypted)
 */
export async function createBackupEnvelope(
    data: any,
    backupType: 'accounts' | 'full',
    password?: string
): Promise<string> {
    if (!password || password.trim().length === 0) {
        const plainEnvelope: PlainBackupEnvelope = {
            version: 1,
            encrypted: false,
            created_at: new Date().toISOString(),
            backup_type: backupType,
            data,
        };
        return JSON.stringify(plainEnvelope, null, 2);
    }

    const salt = window.crypto.getRandomValues(new Uint8Array(16));
    const iv = window.crypto.getRandomValues(new Uint8Array(12));
    const key = await deriveKey(password, salt);

    const enc = new TextEncoder();
    const payloadBytes = enc.encode(
        JSON.stringify({
            created_at: new Date().toISOString(),
            backup_type: backupType,
            data,
        })
    );

    const ciphertextBuffer = await window.crypto.subtle.encrypt(
        { name: 'AES-GCM', iv },
        key,
        payloadBytes
    );

    const envelope: EncryptedBackupEnvelope = {
        version: 1,
        encrypted: true,
        kdf: 'PBKDF2-SHA256',
        iterations: 100000,
        salt: bufferToBase64(salt),
        iv: bufferToBase64(iv),
        ciphertext: bufferToBase64(ciphertextBuffer),
        hint: 'Encrypted Antigravity Manager Backup',
    };

    return JSON.stringify(envelope, null, 2);
}

/**
 * Inspect raw string to determine if it is an encrypted envelope
 */
export function isEncryptedBackup(rawContent: string): boolean {
    try {
        const parsed = JSON.parse(rawContent);
        return Boolean(parsed && parsed.encrypted === true);
    } catch {
        return false;
    }
}

/**
 * Decrypt or parse a backup envelope.
 * Returns the unencrypted inner data payload.
 */
export async function parseBackupEnvelope(
    rawContent: string,
    password?: string
): Promise<{ backupType: 'accounts' | 'full'; data: any }> {
    let parsed: any;
    try {
        parsed = JSON.parse(rawContent);
    } catch {
        throw new Error('Invalid JSON content. File may be corrupted.');
    }

    // Case 1: Plain legacy accounts array
    if (Array.isArray(parsed)) {
        return { backupType: 'accounts', data: { accounts: parsed } };
    }

    // Case 2: Unencrypted envelope
    if (parsed.version === 1 && !parsed.encrypted && parsed.data) {
        return {
            backupType: parsed.backup_type || 'accounts',
            data: parsed.data,
        };
    }

    // Case 3: Encrypted envelope
    if (parsed.encrypted === true) {
        if (!password || password.trim().length === 0) {
            throw new Error('This backup is password-protected. Please enter your decryption password.');
        }

        const salt = base64ToBuffer(parsed.salt);
        const iv = base64ToBuffer(parsed.iv);
        const ciphertext = base64ToBuffer(parsed.ciphertext);
        const iterations = parsed.iterations || 100000;

        let key: CryptoKey;
        try {
            key = await deriveKey(password, salt, iterations);
        } catch (e) {
            throw new Error(`Failed to derive decryption key: ${e}`);
        }

        let decryptedBuffer: ArrayBuffer;
        try {
            decryptedBuffer = await window.crypto.subtle.decrypt(
                { name: 'AES-GCM', iv },
                key,
                ciphertext
            );
        } catch {
            throw new Error('Incorrect password or corrupted ciphertext.');
        }

        const dec = new TextDecoder();
        const decryptedJson = dec.decode(decryptedBuffer);
        const inner = JSON.parse(decryptedJson);

        return {
            backupType: inner.backup_type || 'accounts',
            data: inner.data,
        };
    }

    // Case 4: Raw object with accounts
    if (parsed.accounts && Array.isArray(parsed.accounts)) {
        return { backupType: 'accounts', data: parsed };
    }

    throw new Error('Unrecognized backup format.');
}
