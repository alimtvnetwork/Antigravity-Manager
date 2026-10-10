export interface ToolchainItem {
    id: string;
    name: string;
    desc: string;
    installed: boolean;
}

/** One row of the readiness report (from backend `toolchain_check`). */
export interface CheckEntry {
    id: string;
    label: string;
    ok: boolean;
    hint?: string;
}

export interface ProgressPayload {
    line: string;
}

export interface Profile {
    id: string;
    label: string;
    desc: string;
    items: string[];
    icon: React.ComponentType<{ className?: string }>;
}
