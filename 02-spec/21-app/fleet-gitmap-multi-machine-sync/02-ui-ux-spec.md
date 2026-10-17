# UI/UX Specification: Settings SSH & GitMap Section Overhaul

## 1. Top-Right Action Toolbar Redesign

### 1.1 Problem Statement
The current header buttons (`Import SSH from GitMap`, `Deploy Encrypted Credentials to Fleet`, `Check Cluster Balances`) render as disjoint, plain text links with thin vertical pipes, giving an unprofessional appearance that lacks tactile affordance and clear interactive states.

### 1.2 Segmented Pill Capsule Design
Wrap the three actions into a contiguous, dark-glass segmented pill capsule adhering to the project UI design conventions (`AGENTS.md`):

```tsx
<div className="inline-flex items-center rounded-full bg-slate-900/90 dark:bg-slate-900/90 border border-slate-700/80 p-0.5 shadow-sm divide-x divide-slate-700/60 backdrop-blur-sm">
  <button
    type="button"
    onClick={handleImportGitMapNodes}
    disabled={importingGitMap}
    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-l-full text-xs font-medium text-slate-300 hover:text-white hover:bg-slate-800/80 active:scale-95 transition-all disabled:opacity-50"
  >
    <ArrowDownToLine className="w-3.5 h-3.5 text-sky-400" />
    <span>Import SSH from GitMap</span>
  </button>
  
  <button
    type="button"
    onClick={handleDeployEncryptedCredentials}
    disabled={deployingCreds}
    className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-slate-300 hover:text-white hover:bg-slate-800/80 active:scale-95 transition-all disabled:opacity-50"
  >
    <CloudUpload className="w-3.5 h-3.5 text-emerald-400" />
    <span>Deploy Encrypted Credentials to Fleet</span>
  </button>
  
  <button
    type="button"
    onClick={handleRefreshClusterBalances}
    disabled={isRefreshingBalances}
    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-r-full text-xs font-medium text-sky-400 hover:text-sky-300 hover:bg-slate-800/80 active:scale-95 transition-all disabled:opacity-50"
  >
    <RefreshCw className={clsx("w-3.5 h-3.5", isRefreshingBalances && "animate-spin")} />
    <span>Check Cluster Balances</span>
  </button>
</div>
```

---

## 2. GitMap Status Banner & Install Box Modernization

### 2.1 Problem Statement
The GitMap status box renders raw un-sanitized paths (`GitMap binary path: C:\Users\... • Managed SSH Config: C:\Users\... • Nodes: 7`), creating visual clutter, while the action buttons on the right are styled as loose link labels.

### 2.2 Segmented Button Styling & Path Badges
1. **Action Buttons:**
   - Transform `Copy Install Command` and `Open PowerShell & Install GitMap` into high-contrast segmented pill buttons with explicit icons (`Copy`, `Terminal`), tactile active states, and feedback toasts (`Copied: agm ssh install-gitmap`).
2. **Metadata Path Cleaning:**
   - Format paths into compact, dark-glass badges with monospace font:
     - Badge 1: `GitMap v6.563.0` (Emerald badge with check icon)
     - Badge 2: `RSA-2048 Vault Active` (Purple badge with key icon)
     - Badge 3: `7 Nodes Configured` (Slate badge with server icon)
   - Store full binary and SSH config paths inside an expandable or hover tooltip (`title={binaryPath}`) instead of spilling long filesystem paths across the container.

---

## 3. Multi-Machine Matrix Polish

1. **Machine Alias & IP Column:**
   - Render machine alias alongside its genuine IP address: e.g., `main (192.168.1.20)`, `w2 (192.168.1.7)`.
   - Never render empty parentheses `()`. If IP is absent from connection, fallback to local lookup or placeholder `(DHCP)`.
2. **Account Email & Variable Resolution:**
   - Eliminate raw `${adminUser}` and `${linuxUser}` template strings.
   - Display the genuine target user (`Administrator@main`, `Administrator@w2`) with a subtle `SSH-NODE` or `VAULT` badge.
3. **Cluster Health & Error Visibility:**
   - When a node is reachable: green indicator dot with `Online` badge and actual 4h / Weekly credit percentages.
   - When a node is unreachable: amber indicator dot with `Unreachable` badge and an interactive popover or tooltip displaying the exact connection error (e.g. `Connection timed out after 1500ms`).
