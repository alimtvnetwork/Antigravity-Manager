import { Cpu, Zap, Globe, Package } from 'lucide-react';
import type { ToolchainItem, Profile } from './types';

export const PROFILES: Profile[] = [
    {
        id: 'minimal',
        label: 'Minimal',
        desc: 'Rust toolchain only — rustc, cargo, clippy, rustfmt.',
        items: ['rust'],
        icon: Cpu,
    },
    {
        id: 'rust-dev',
        label: 'Rust Dev',
        desc: 'Minimal + sccache (compile cache) + cargo-watch (auto rebuild).',
        items: ['rust', 'sccache', 'cargo-watch'],
        icon: Zap,
    },
    {
        id: 'frontend',
        label: 'Frontend',
        desc: 'Node.js LTS (fnm) + pnpm + Tauri CLI for the web UI.',
        items: ['nodejs', 'pnpm', 'tauri-cli'],
        icon: Globe,
    },
    {
        id: 'full',
        label: 'Full',
        desc: 'Everything: Rust Dev + Frontend + GitHub CLI.',
        items: ['rust', 'sccache', 'cargo-watch', 'nodejs', 'pnpm', 'tauri-cli', 'gh'],
        icon: Package,
    },
];

/** Fallback item catalog if the backend is unreachable (ids must match backend). */
export const FALLBACK_ITEMS: ToolchainItem[] = [
    { id: 'rust', name: 'Rust', desc: 'rustup + stable toolchain (rustc, cargo, clippy, rustfmt)', installed: false },
    { id: 'nodejs', name: 'Node.js', desc: 'Node.js LTS via fnm (fast node manager)', installed: false },
    { id: 'sccache', name: 'sccache', desc: 'Shared compilation cache — much faster Rust rebuilds', installed: false },
    { id: 'cargo-watch', name: 'cargo-watch', desc: 'Auto-rebuild on file change (dev loop)', installed: false },
    { id: 'tauri-cli', name: 'Tauri CLI', desc: 'tauri dev / tauri build commands', installed: false },
    { id: 'gh', name: 'GitHub CLI', desc: 'gh — repo automation from the terminal', installed: false },
    { id: 'pnpm', name: 'pnpm', desc: 'Fast, disk-efficient npm alternative', installed: false },
];

export const TROUBLESHOOTING: Array<{ issue: string; fix: string }> = [
    {
        issue: 'cargo: command not found',
        fix: 'Restart your shell, or run: source ~/.cargo/env. The installer adds ~/.cargo/bin to PATH in ~/.profile — new terminals pick it up automatically.',
    },
    {
        issue: 'gdk-sys build failed (gdk-3.0 not found)',
        fix: 'The Tauri GTK system libraries are missing. Re-run the installer WITHOUT --skip-native-deps so the apt/brew native packages get installed.',
    },
    {
        issue: 'rustup update failed (offline)',
        fix: 'Check network connectivity and proxy settings. rustup needs https://sh.rustup.rs and static.rust-lang.org reachable.',
    },
    {
        issue: 'permission denied during apt install',
        fix: 'Re-run with sudo, or as root. The script auto-uses sudo when available; on locked-down machines install the listed packages manually.',
    },
    {
        issue: 'toolchain mismatch (rust-toolchain.toml)',
        fix: 'Run: rustup default <version>, or pass --toolchain <version> to the installer to match the pinned channel.',
    },
    {
        issue: 'SSH remote install hangs',
        fix: 'SSH keys must be pre-configured (key auth only, BatchMode). Test first: ssh -o BatchMode=yes user@host "echo ok".',
    },
];

