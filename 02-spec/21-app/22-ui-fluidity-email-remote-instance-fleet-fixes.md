# Architectural Specification: UI Fluidity, Plain-Text Email Remote Engine, and Fleet Multi-Instance Coordination

## User Request (Verbatim)
> is it done and release properly?
> https://prnt.sc/qquT8wIqXTAq
> https://prnt.sc/k6QrvPfpe20M
> https://prnt.sc/-kIggFZpVTf7
> https://prnt.sc/EhHRsBXqB3iY
> https://prnt.sc/6Ku9-BA10KBT
> https://prnt.sc/o-Y0pqTyhF_s
> https://prnt.sc/_l2BYnU9W7bO
> https://prnt.sc/K-N6_VTJG-gW
> https://prnt.sc/z-c1kxaTb1rS
> https://prnt.sc/JM2TO-8i0bmy
> https://prnt.sc/Cz9tf4v9V-AW
> 
> Error: button#btn-window-maximize "Restore" ...
> Another ERROR: Command plugin:opener|open_url not allowed by ACL
> 
> Screen fluidity: if we reduce the size of the screen, the screen is not fluid.
> The right-hand side items just goes out of the flow.
> Remove duplicate switch buttons. Remove UPSTREAM GENESIS / lbjlaq.
> Fix update notifier colliding with pagination.
> Combine import/export into one modal with password encryption and email export option.
> Auto-switch: quota < 20% checks every 1 min, baseline 8 mins, auto-switch at < 10%.
> Prevent focus stealing by background watchdog. Secondary database cap 450 MB.
> Quick clean: recycle icon, fix NaN undefined, clean conversations only.
> Email engine: strictly 100% plain text, zero HTML, 2-stage ACK + completion notifications, 10-min rate limit.
> Multi-instance sequence numbers (1, 2, ...) and distributed `agm ls`.
> Telegram bot setup script and documentation.

---

## 1. Domain Architecture & Invariants

### 1.1 UI Fluidity & Responsive Layout
- **Navbar & Action Bars**: All top action bars (`Accounts.tsx`, `Navbar.tsx`) implement `flex-wrap` and `min-w-0` to guarantee that buttons never push the viewport boundary when resizing or entering compact view.
- **Window Controls & ACL**: Tauri 2 permissions in `src-tauri/capabilities/default.json` explicitly allow `opener:default`, `opener:open_url`, `window:default`, and `window:start_dragging`.
- **Update Notifier Placement**: Positioned at `bottom-6 left-6` to eliminate any collision with the centered pagination controls at `bottom-4`.

### 1.2 Unified Encrypted Backup Vault
- **AES-GCM 256-bit Encryption**: Passwords run through PBKDF2 (100,000 iterations SHA-256) with 12-byte random IV and 16-byte salt.
- **Two Scopes**: `Accounts Only` (credentials and quota metadata) and `Full System` (accounts, proxy bindings, custom model rules, email watcher config, Supabase endpoints).
- **Outbound Dispatch**: If SMTP mailbox is configured, an option to send the encrypted backup directly to the administrative email is available.

### 1.3 Auto-Switcher & Focus Stability
- **Baseline Interval**: 480 seconds (8 minutes).
- **Caution Interval**: 60 seconds (1 minute) when any primary model quota falls below 20%.
- **Rotation Threshold**: Automatic failover triggered when quota < 10%.
- **Zero Focus Stealing**: `auto_focus_window` defaults to `false`. Routine background crash watchdog checks do not steal focus from other user applications.

### 1.4 Plain-Text Outbound Email Daemon
- **MIME Specification**: `Content-Type: text/plain; charset=UTF-8` with zero HTML markup or tags.
- **Two-Stage Delivery**: Stage 1 sends an instant ACK receipt (`[ACK: Received & Executing]`); Stage 2 sends the execution outcome and audit log.
- **SQLite 10-Minute Rate Limit**: Identical heavy operations from the same sender within 600 seconds are rejected with rate-limit notices.

### 1.5 Multi-Instance Sequence Numbering
- **Database & Model**: `seq_num: Option<u32>` stored on `InstanceConfig`.
- **Auto-Sequencing**: Newly provisioned instances automatically receive `max(seq_num) + 1`.
- **Distributed Listing**: `agm ls` formats `#seq`, ID, Name, Status, Bound Account, Node / IP, and Data Directory.

---

## 2. Quality & Verification Gates
- `npm run build` exits 0 (Vite client production build).
- `npx tsc --noEmit` exits 0.
- `cargo fmt -- --check` exits 0.
- `cargo check` exits 0 on `agm-alim`.
