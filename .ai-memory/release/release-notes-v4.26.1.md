## Quick Install v4.26.1

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.26.1/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.26.1/install.sh | bash
```

---

## What's Changed in v4.26.1

- **Compact Top Padding & Proper Spacing**: Reduced excessive top padding across Email & Alerts and Instances pages from 24-32px to 14-16px, removing redundant nested card containers and oversized headers.
- **Dedicated Scroll Containers**: Added `h-full w-full overflow-y-auto` to Email and Instances pages with `min-h-0` on layout main, ensuring the window never clips content and can be freely scrolled when more space is required.
- **Refined Card Hierarchy**: Compacted telemetry banners, badges, and card padding for a clean and balanced interface.
- **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
