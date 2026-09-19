# Plan 28: Dedicated Email Setup Page, Dropdown Boundary Locks, and Installer Asset Resolution

> **Plan Path:** `.ai-memory/plans/completed/28-email-setup-page-and-installer-hardening.md`
> **Status:** Completed
> **Task Origin & Inception:** User highlighted missing Email setup UI ("Where is your email setup UI? I don't see the UI. It feels very stupid") and recurring menu overflow issues caused by downloading stale installer versions from GitHub's delayed `/releases/latest` endpoint.
> **Target Release:** v4.24.0

---

## 1. Problem Statement & Root Cause Analysis

### A. Missing Email Setup UI Surface
- **Root Cause:** While `EmailNotificationSettings.tsx` existed and contained 1,035 lines of production-grade logic (SMTP/IMAP testing, split vault SQLite storage, quota drop alerts, remote CLI execution), it was buried as an invisible sub-tab under `Settings.tsx` (`activeTab === 'email'`).
- **Impact:** Users could not locate or access email configuration directly from navigation.

### B. Menu Overflow & Stale Version Skew
- **Root Cause:** When new versions were tagged and built on GitHub Actions, GitHub's `/repos/.../releases/latest` endpoint did not immediately point to the newly created release until all matrix artifacts finished uploading and the release was published as "latest".
- `install.ps1` queried `/releases/latest`, causing users to download stale releases (e.g. `v4.21.0`), which had the sprawling 10-tab navbar that pushed `<InstanceSelector />` off-screen (`● Def...`).
- **Impact:** Even after UI fixes were merged into the codebase, users running the installer were served older builds that still had the overflow issue.

---

## 2. Key Implementations & Enhancements

### A. Dedicated Top-Level Email Page & Main Nav Route
1. **Standalone Page (`src/pages/Email.tsx`):**
   - Created clean top-level view with dedicated header, active status badge, and back-to-settings quick link.
   - Embeds `<EmailNotificationSettings />` within a responsive card container.
2. **Route Registration (`src/App.tsx`):**
   - Registered `/email` route with `<Email />` component inside main app router.
3. **Primary Navigation (`src/components/navbar/Navbar.tsx`):**
   - Added `Mail` icon from `lucide-react`.
   - Registered `/email` with high priority in `navItems`.
   - Updated i18n locales (`en.json`, `zh.json`, `zh-TW.json`) with `"email": "Email & Alerts"`.

### B. Window Boundary Lock on Dropdowns (`NavMenu.tsx`, `InstanceSelector.tsx`)
1. Added `max-w-[calc(100vw-32px)]` to both `<NavMenu />` and `<InstanceSelector />` dropdowns to ensure menus never clip beyond screen boundaries.
2. Verified `shrink-0` constraints on `<InstanceSelector />` and right-side controls, ensuring they are never pushed off-screen.

### C. Hardened Release Resolution in Installer (`install.ps1`)
1. Updated `install.ps1` to query `/releases` array endpoint first.
2. Added asset-checking filter: `$resp | Where-Object { $_.assets -and $_.assets.Count -gt 0 } | Select-Object -First 1`.
3. Guarantees the installer always downloads the newest release that has published binary assets, completely avoiding stale cached releases.

---

## 3. Verification & Deliverables

- [x] Dedicated `/email` route and standalone `src/pages/Email.tsx` page.
- [x] `Email & Alerts` item with `Mail` icon added to navigation menu and locale bundles.
- [x] Dropdown popovers constrained with `max-w-[calc(100vw-32px)]`.
- [x] `install.ps1` dynamic release candidate discovery verified via test queries.
- [x] All repository files normalized to UNIX LF line endings and UTF-8 without BOM.
