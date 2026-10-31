# Subtask Plan 03: Settings SSH & GitMap Section UI Polish

- **Worker:** Worker 03
- **Target Files:**
  * `src/components/settings/EmailNotifications/SshGitmapSection.tsx`

## Objectives
1. **Top-Right Toolbar Redesign:**
   - Replace loose text links with a unified segmented pill capsule:
     `inline-flex items-center rounded-full bg-slate-900/90 dark:bg-slate-900/90 border border-slate-700/80 p-0.5 shadow-sm divide-x divide-slate-700/60 backdrop-blur-sm`
   - Actions: `Import SSH from GitMap`, `Deploy Encrypted Credentials to Fleet`, `Check Cluster Balances`.
2. **GitMap Status & Install Box Modernization:**
   - Transform `Copy Install Command (agm ssh install-gitmap)` and `Open PowerShell & Install GitMap` into high-contrast segmented pill buttons with hover and active states.
   - Show copy toast notification on command copy.
   - Clean up raw path text in GitMap status banner: render clean metadata badges (`GitMap v6.563.0`, `RSA-2048 Vault Active`, `Nodes: N`) with paths in tooltips rather than raw strings.
3. **Multi-Machine Matrix Polish:**
   - Fix empty `()`: render `item.ip_address || item.machine_ip || conn?.ip_address`.
   - Fix `${adminUser}` and `${linuxUser}`: display resolved user or friendly badge.
   - Highlight unreachable nodes with descriptive amber/red badges and hover tooltips showing connection error.
   - Ensure "Check Cluster Balances" triggers an informative toast with counts of online/offline nodes.
